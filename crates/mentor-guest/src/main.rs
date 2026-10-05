//! `mentor-guest`: init process and agent of a Mentor microVM.
//!
//! It mounts the pseudo file systems, then serves host requests on a vsock port (see [`mentor_guest::protocol`]).
//! The control channel is vsock only: the microVM needs no network interface and no SSH server.
//!
//! Randomness after a snapshot restore is handled by the kernel (VMGenID reseeds its generator); the clock is
//! set by the host with a `resume` request.
//!
//! Prototype limits: commands run as root, orphaned processes are not reaped, output is not size-limited.

use std::ffi::CString;
use std::fs::File;
use std::io::{Read, Write};
use std::os::fd::{AsRawFd, FromRawFd, RawFd};
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};

use mentor_guest::protocol::{ExecResult, Request, PORT};

const WORKDIR: &str = "/workspace";
/// Second virtio block device: the writable disk of the session.
const SESSION_DISK: &str = "/dev/vdb";
const MAX_REQUEST_BYTES: usize = 64 * 1024;

fn check(result: libc::c_int, what: &str) -> std::io::Result<libc::c_int> {
    if result < 0 {
        let err = std::io::Error::last_os_error();
        Err(std::io::Error::new(err.kind(), format!("{what}: {err}")))
    } else {
        Ok(result)
    }
}

fn mount(source: &str, target: &str, fstype: &str, data: &str) {
    let (source, target_c, fstype, data) =
        (CString::new(source).unwrap(), CString::new(target).unwrap(), CString::new(fstype).unwrap(), CString::new(data).unwrap());
    // SAFETY: all pointers are valid NUL-terminated strings that outlive the call.
    let result = unsafe { libc::mount(source.as_ptr(), target_c.as_ptr(), fstype.as_ptr(), 0, data.as_ptr().cast()) };
    if result < 0 {
        let err = std::io::Error::last_os_error();
        // The kernel may already have mounted devtmpfs (EBUSY): that is fine.
        if err.raw_os_error() != Some(libc::EBUSY) {
            eprintln!("mentor-guest: mount {target}: {err}");
        }
    }
}

/// What an init process must do before anything else can run.
fn init_system() {
    mount("proc", "/proc", "proc", "");
    mount("sysfs", "/sys", "sysfs", "");
    mount("devtmpfs", "/dev", "devtmpfs", "");
    let _ = std::fs::create_dir_all("/dev/pts");
    mount("devpts", "/dev/pts", "devpts", "ptmxmode=0666");
    // The root file system is read-only: everything writable is a tmpfs for now (a per-session disk later).
    mount("tmpfs", "/tmp", "tmpfs", "size=64m,mode=1777");
    mount("tmpfs", "/run", "tmpfs", "size=16m,mode=0755");
    // The session disk, when the host attached one, is the learner's workspace: its size is a hard quota.
    if std::path::Path::new(SESSION_DISK).exists() {
        mount(SESSION_DISK, WORKDIR, "ext4", "");
    } else {
        mount("tmpfs", WORKDIR, "tmpfs", "size=128m,mode=0755");
    }
    std::env::set_var("PATH", "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin");
    std::env::set_var("HOME", "/root");
    std::env::set_var("TERM", "xterm-256color");
}

fn listen(port: u32) -> std::io::Result<RawFd> {
    // SAFETY: plain socket calls on a zero-initialised, correctly sized `sockaddr_vm`.
    unsafe {
        let fd = check(libc::socket(libc::AF_VSOCK, libc::SOCK_STREAM | libc::SOCK_CLOEXEC, 0), "socket(AF_VSOCK)")?;
        let mut addr: libc::sockaddr_vm = std::mem::zeroed();
        addr.svm_family = libc::AF_VSOCK as libc::sa_family_t;
        addr.svm_port = port;
        addr.svm_cid = libc::VMADDR_CID_ANY;
        check(libc::bind(fd, (&raw const addr).cast(), std::mem::size_of::<libc::sockaddr_vm>() as libc::socklen_t), "bind")?;
        check(libc::listen(fd, 16), "listen")?;
        Ok(fd)
    }
}

fn exec(argv: &[String], workdir: Option<&str>) -> ExecResult {
    let failed = |error: String| ExecResult { exit_code: None, stdout: String::new(), stderr: String::new(), error: Some(error) };
    let Some((program, args)) = argv.split_first() else { return failed("empty argv".to_string()) };
    match Command::new(program).args(args).current_dir(workdir.unwrap_or(WORKDIR)).stdin(Stdio::null()).output() {
        Ok(output) => ExecResult {
            exit_code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            error: None,
        },
        Err(err) => failed(err.to_string()),
    }
}

/// Runs a login shell on a new pseudo-terminal and relays its bytes over `connection` until it exits.
fn terminal(mut connection: File, cols: u16, rows: u16) -> std::io::Result<()> {
    let (mut master, mut slave) = (0, 0);
    let size = libc::winsize { ws_row: rows, ws_col: cols, ws_xpixel: 0, ws_ypixel: 0 };
    // SAFETY: `openpty` fills the two descriptors; the name and termios pointers may be null.
    check(unsafe { libc::openpty(&mut master, &mut slave, std::ptr::null_mut(), std::ptr::null(), &size) }, "openpty")?;
    // SAFETY: both descriptors were just returned by `openpty` and are owned here.
    let (mut master, slave) = unsafe { (File::from_raw_fd(master), File::from_raw_fd(slave)) };

    let mut command = Command::new("/bin/sh");
    command.arg("-l").current_dir(WORKDIR).stdin(slave.try_clone()?).stdout(slave.try_clone()?).stderr(slave.try_clone()?);
    // SAFETY: only async-signal-safe calls between fork and exec; they make the pty the controlling terminal.
    unsafe {
        command.pre_exec(|| {
            check(libc::setsid(), "setsid")?;
            check(libc::ioctl(0, libc::TIOCSCTTY as _, 0), "TIOCSCTTY")?;
            Ok(())
        });
    }
    let mut child = command.spawn()?;
    // The command keeps its own copies of the slave end: while one is open here, reading the master never
    // reports that the shell has exited.
    drop(command);
    drop(slave);

    let (mut input, mut pty_in) = (connection.try_clone()?, master.try_clone()?);
    std::thread::spawn(move || {
        let _ = std::io::copy(&mut input, &mut pty_in);
    });
    // Reading the master fails with EIO once the shell has exited: that is the normal end of the session.
    let _ = std::io::copy(&mut master, &mut connection);
    child.wait()?;
    // SAFETY: shutting down a valid socket descriptor; unblocks the input thread on the host side.
    unsafe { libc::shutdown(connection.as_raw_fd(), libc::SHUT_RDWR) };
    Ok(())
}

fn shutdown() -> ! {
    // SAFETY: `sync` and `reboot` take no pointers. With `reboot=k` on the kernel command line, Firecracker
    // exits when the guest reboots.
    unsafe {
        libc::sync();
        libc::reboot(libc::RB_AUTOBOOT);
    }
    std::process::exit(0)
}

/// Reads the request line one byte at a time: a buffered reader could swallow the first bytes typed in a
/// terminal session, which follow the request on the same connection.
fn read_request_line(connection: &mut File) -> std::io::Result<Vec<u8>> {
    let (mut line, mut byte) = (Vec::new(), [0u8; 1]);
    while line.len() < MAX_REQUEST_BYTES && connection.read(&mut byte)? == 1 && byte[0] != b'\n' {
        line.push(byte[0]);
    }
    Ok(line)
}

fn handle(connection: File) -> std::io::Result<()> {
    let mut connection = connection;
    let line = read_request_line(&mut connection)?;
    if line.is_empty() {
        // The host probes readiness by connecting and hanging up.
        return Ok(());
    }
    match serde_json::from_slice::<Request>(&line) {
        Ok(Request::Exec { argv, workdir }) => {
            writeln!(connection, "{}", serde_json::to_string(&exec(&argv, workdir.as_deref()))?)?;
        }
        Ok(Request::Terminal { cols, rows }) => terminal(connection, cols, rows)?,
        Ok(Request::Resume { unix_millis }) => {
            let time =
                libc::timespec { tv_sec: (unix_millis / 1000) as libc::time_t, tv_nsec: (unix_millis % 1000 * 1_000_000) as libc::c_long };
            // SAFETY: `time` is a valid `timespec` for the duration of the call.
            let ok = unsafe { libc::clock_settime(libc::CLOCK_REALTIME, &time) } == 0;
            writeln!(connection, "{}", serde_json::json!({ "ok": ok }))?;
        }
        Ok(Request::Shutdown) => {
            writeln!(connection, "{{\"ok\":true}}")?;
            drop(connection);
            shutdown();
        }
        Err(err) => writeln!(connection, "{}", serde_json::json!({ "error": format!("invalid request: {err}") }))?,
    }
    Ok(())
}

fn main() {
    if std::process::id() == 1 {
        init_system();
    }
    let listener = match listen(PORT) {
        Ok(fd) => fd,
        Err(err) => {
            eprintln!("mentor-guest: cannot listen on vsock port {PORT}: {err}");
            std::process::exit(1);
        }
    };
    println!("mentor-guest: ready on vsock port {PORT}");
    loop {
        // SAFETY: `accept` on a listening socket; null address pointers are allowed.
        let fd = unsafe { libc::accept4(listener, std::ptr::null_mut(), std::ptr::null_mut(), libc::SOCK_CLOEXEC) };
        if fd < 0 {
            continue;
        }
        // SAFETY: `fd` is a fresh descriptor owned by this `File`.
        let connection = unsafe { File::from_raw_fd(fd) };
        std::thread::spawn(move || {
            if let Err(err) = handle(connection) {
                eprintln!("mentor-guest: request failed: {err}");
            }
        });
    }
}
