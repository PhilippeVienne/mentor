//! One Firecracker microVM and the connection to its guest agent.
//!
//! Each microVM owns a private run directory, which is Firecracker's working directory. Everything the VM
//! refers to lives there under a fixed relative name (`rootfs.ext4`, `session.ext4`, `vsock.sock`): a snapshot
//! records these names, so it can be restored in any other run directory, and the same layout will fit the
//! jailer's chroot.

use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use mentor_guest::protocol::{ExecResult, Request, PORT};

use crate::api;

/// How long the guest agent may take to answer after the VM process starts.
const BOOT_TIMEOUT: Duration = Duration::from_secs(15);
const API_SOCKET: &str = "api.sock";
const VSOCK_SOCKET: &str = "vsock.sock";
const CONSOLE_LOG: &str = "console.log";
const ROOTFS: &str = "rootfs.ext4";
const SESSION_DISK: &str = "session.ext4";
const SNAPSHOT_STATE: &str = "vm.state";
const SNAPSHOT_MEMORY: &str = "vm.memory";

/// What a microVM is made of.
#[derive(Debug, Clone)]
pub struct Config {
    pub firecracker: PathBuf,
    pub kernel: PathBuf,
    /// ext4 image mounted read-only as the root file system; its init is `mentor-guest`.
    pub rootfs: PathBuf,
    pub vcpus: u8,
    pub memory_mib: u32,
    /// Size of the writable disk mounted on `/workspace`. It is a hard quota: the guest cannot write more.
    /// `None` gives a tmpfs workspace instead, taken from the VM's memory.
    pub session_disk_mib: Option<u32>,
}

/// A paused microVM saved on disk: starting from it skips the kernel boot and the guest's start-up.
#[derive(Debug, Clone)]
pub struct Snapshot {
    /// Directory holding the VM state, its memory, and the session disk as it was when the snapshot was taken.
    pub directory: PathBuf,
    firecracker: PathBuf,
    rootfs: PathBuf,
    has_session_disk: bool,
}

impl Snapshot {
    /// Opens a snapshot previously written in `directory` by [`MicroVm::snapshot`] with the same `config`.
    pub fn open(directory: &Path, config: &Config) -> std::io::Result<Self> {
        let directory = absolute(directory)?;
        for file in [SNAPSHOT_STATE, SNAPSHOT_MEMORY] {
            if !directory.join(file).is_file() {
                return Err(error(format!("{} is not a snapshot: {file} is missing", directory.display())));
            }
        }
        let has_session_disk = directory.join(SESSION_DISK).is_file();
        Ok(Self { directory, firecracker: config.firecracker.clone(), rootfs: absolute(&config.rootfs)?, has_session_disk })
    }
}

/// A running microVM. Dropping it kills the VM and removes its run directory.
pub struct MicroVm {
    process: Child,
    run_dir: PathBuf,
    config: Config,
    /// Time between starting Firecracker and the guest agent accepting a connection.
    pub boot_time: Duration,
}

fn error(message: impl Into<String>) -> std::io::Error {
    std::io::Error::other(message.into())
}

fn absolute(path: &Path) -> std::io::Result<PathBuf> {
    path.canonicalize().map_err(|err| error(format!("{}: {err}", path.display())))
}

/// A private directory for one microVM.
fn new_run_dir() -> std::io::Result<PathBuf> {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |elapsed| elapsed.as_nanos());
    let name = format!("mentor-vm-{}-{}-{nanos:x}", std::process::id(), COUNTER.fetch_add(1, Ordering::Relaxed));
    let run_dir = std::env::temp_dir().join(name);
    std::fs::create_dir(&run_dir)?;
    Ok(run_dir)
}

/// Creates an empty ext4 file system of exactly `size_mib` in a sparse file: it takes almost no space until used.
fn format_disk(path: &Path, size_mib: u32) -> std::io::Result<()> {
    std::fs::File::create(path)?.set_len(u64::from(size_mib) * 1024 * 1024)?;
    let output = Command::new("mke2fs")
        .args(["-q", "-t", "ext4", "-F", "-m", "0", "-E", "root_owner=0:0,lazy_itable_init=1,lazy_journal_init=1"])
        .arg(path)
        .output()
        .map_err(|err| error(format!("cannot run mke2fs: {err}")))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(error(format!("mke2fs failed: {}", String::from_utf8_lossy(&output.stderr))))
    }
}

/// Copies a disk image without filling its holes.
fn copy_sparse(from: &Path, to: &Path) -> std::io::Result<()> {
    let status = Command::new("cp").args(["--sparse=always", "--reflink=auto"]).arg(from).arg(to).status()?;
    if status.success() {
        Ok(())
    } else {
        Err(error(format!("cannot copy {}", from.display())))
    }
}

fn spawn_firecracker(binary: &Path, run_dir: &Path, extra_args: &[&str]) -> std::io::Result<Child> {
    let console = std::fs::File::create(run_dir.join(CONSOLE_LOG))?;
    let mut command = Command::new(absolute(binary)?);
    // SAFETY: `prctl` is async-signal-safe. The VM must not outlive this process, even if it is killed.
    unsafe {
        command.pre_exec(|| {
            libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL);
            Ok(())
        });
    }
    command
        .args(["--api-sock", API_SOCKET])
        .args(extra_args)
        .current_dir(run_dir)
        .stdin(Stdio::null())
        .stdout(console.try_clone()?)
        .stderr(console)
        .spawn()
        .map_err(|err| error(format!("cannot start {}: {err}", binary.display())))
}

impl MicroVm {
    /// Cold boot: starts the kernel and waits until the guest agent answers.
    pub fn start(config: &Config) -> std::io::Result<Self> {
        let run_dir = new_run_dir()?;
        std::os::unix::fs::symlink(absolute(&config.rootfs)?, run_dir.join(ROOTFS))?;
        let mut drives =
            vec![serde_json::json!({ "drive_id": "rootfs", "path_on_host": ROOTFS, "is_root_device": true, "is_read_only": true })];
        if let Some(size_mib) = config.session_disk_mib {
            format_disk(&run_dir.join(SESSION_DISK), size_mib)?;
            drives.push(
                serde_json::json!({ "drive_id": "session", "path_on_host": SESSION_DISK, "is_root_device": false, "is_read_only": false }),
            );
        }
        let machine = serde_json::json!({
            "boot-source": {
                "kernel_image_path": absolute(&config.kernel)?,
                // `reboot=k` makes a guest reboot exit Firecracker; `panic=1` does the same on a kernel panic.
                "boot_args": "console=ttyS0 reboot=k panic=1 pci=off quiet init=/sbin/mentor-guest",
            },
            "drives": drives,
            "machine-config": { "vcpu_count": config.vcpus, "mem_size_mib": config.memory_mib },
            "vsock": { "guest_cid": 3, "uds_path": VSOCK_SOCKET },
        });
        std::fs::write(run_dir.join("machine.json"), serde_json::to_vec_pretty(&machine)?)?;
        let started = Instant::now();
        let process = spawn_firecracker(&config.firecracker, &run_dir, &["--config-file", "machine.json"])?;
        let mut vm = Self { process, run_dir, config: config.clone(), boot_time: Duration::ZERO };
        vm.wait_for_guest(started)?;
        Ok(vm)
    }

    /// Starts a microVM from a snapshot: no kernel boot, the guest agent is already running.
    ///
    /// The restored VM gets its own copy of the session disk as it was when the snapshot was taken.
    pub fn restore(snapshot: &Snapshot, config: &Config) -> std::io::Result<Self> {
        let run_dir = new_run_dir()?;
        std::os::unix::fs::symlink(absolute(&snapshot.rootfs)?, run_dir.join(ROOTFS))?;
        if snapshot.has_session_disk {
            copy_sparse(&snapshot.directory.join(SESSION_DISK), &run_dir.join(SESSION_DISK))?;
        }
        let started = Instant::now();
        let process = spawn_firecracker(&snapshot.firecracker, &run_dir, &[])?;
        let mut vm = Self { process, run_dir, config: config.clone(), boot_time: Duration::ZERO };
        vm.wait_for_api(started)?;
        let load = serde_json::json!({
            "snapshot_path": snapshot.directory.join(SNAPSHOT_STATE),
            "mem_backend": { "backend_type": "File", "backend_path": snapshot.directory.join(SNAPSHOT_MEMORY) },
            "resume_vm": true,
        });
        vm.api("PUT", "/snapshot/load", &load)?;
        vm.wait_for_guest(started)?;
        // The guest's clock stopped at the snapshot: give it the current time.
        let unix_millis = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |elapsed| elapsed.as_millis() as u64);
        let mut answer = String::new();
        BufReader::new(vm.request(&Request::Resume { unix_millis })?).read_line(&mut answer)?;
        Ok(vm)
    }

    /// Pauses the microVM, saves it in `directory` and stops it. The snapshot can be restored any number of times.
    ///
    /// Take it once the guest is idle and the workspace is in the state every session should start from.
    pub fn snapshot(mut self, directory: &Path) -> std::io::Result<Snapshot> {
        std::fs::create_dir_all(directory)?;
        let directory = absolute(directory)?;
        // Flush the guest's file systems so that the copied session disk is consistent with its memory.
        self.exec(&["sync".to_string()], None)?;
        self.api("PATCH", "/vm", &serde_json::json!({ "state": "Paused" }))?;
        let create = serde_json::json!({
            "snapshot_type": "Full",
            "snapshot_path": directory.join(SNAPSHOT_STATE),
            "mem_file_path": directory.join(SNAPSHOT_MEMORY),
        });
        self.api("PUT", "/snapshot/create", &create)?;
        let has_session_disk = self.config.session_disk_mib.is_some();
        if has_session_disk {
            copy_sparse(&self.run_dir.join(SESSION_DISK), &directory.join(SESSION_DISK))?;
        }
        self.process.kill()?;
        Ok(Snapshot { directory, firecracker: self.config.firecracker.clone(), rootfs: absolute(&self.config.rootfs)?, has_session_disk })
    }

    fn api(&self, method: &str, path: &str, body: &serde_json::Value) -> std::io::Result<String> {
        api::call(&self.run_dir.join(API_SOCKET), method, path, body)
    }

    fn fail_if_exited(&mut self, during: &str) -> std::io::Result<()> {
        match self.process.try_wait()? {
            Some(status) => Err(error(format!("Firecracker exited during {during} ({status}):\n{}", self.console()))),
            None => Ok(()),
        }
    }

    fn wait_for_api(&mut self, started: Instant) -> std::io::Result<()> {
        while UnixStream::connect(self.run_dir.join(API_SOCKET)).is_err() {
            self.fail_if_exited("start-up")?;
            if started.elapsed() > BOOT_TIMEOUT {
                return Err(error("the Firecracker API socket did not appear"));
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        Ok(())
    }

    fn wait_for_guest(&mut self, started: Instant) -> std::io::Result<()> {
        loop {
            self.fail_if_exited("boot")?;
            if self.connect().is_ok() {
                self.boot_time = started.elapsed();
                return Ok(());
            }
            if started.elapsed() > BOOT_TIMEOUT {
                return Err(error(format!("the guest agent did not answer within {BOOT_TIMEOUT:?}:\n{}", self.console())));
            }
            std::thread::sleep(Duration::from_millis(2));
        }
    }

    /// Everything the guest wrote on its serial console so far.
    pub fn console(&self) -> String {
        std::fs::read_to_string(self.run_dir.join(CONSOLE_LOG)).unwrap_or_default()
    }

    /// Opens a connection to the guest agent, through Firecracker's vsock Unix socket.
    fn connect(&self) -> std::io::Result<UnixStream> {
        let mut stream = UnixStream::connect(self.run_dir.join(VSOCK_SOCKET))?;
        stream.set_read_timeout(Some(Duration::from_secs(2)))?;
        writeln!(stream, "CONNECT {PORT}")?;
        let mut answer = Vec::new();
        // Read byte by byte: nothing past the handshake line may be consumed.
        let mut byte = [0u8; 1];
        while stream.read(&mut byte)? == 1 && byte[0] != b'\n' {
            answer.push(byte[0]);
        }
        if !answer.starts_with(b"OK ") {
            return Err(error(format!("vsock handshake refused: {}", String::from_utf8_lossy(&answer))));
        }
        stream.set_read_timeout(None)?;
        Ok(stream)
    }

    fn request(&self, request: &Request) -> std::io::Result<UnixStream> {
        let mut stream = self.connect()?;
        writeln!(stream, "{}", serde_json::to_string(request)?)?;
        Ok(stream)
    }

    /// Runs a command in the guest and waits for it to finish.
    pub fn exec(&self, argv: &[String], workdir: Option<&str>) -> std::io::Result<ExecResult> {
        let stream = self.request(&Request::Exec { argv: argv.to_vec(), workdir: workdir.map(str::to_string) })?;
        let mut line = String::new();
        BufReader::new(stream).read_line(&mut line)?;
        serde_json::from_str(&line).map_err(|err| error(format!("unexpected answer from the guest ({err}): {line}")))
    }

    /// Opens an interactive shell; the returned stream carries the raw bytes of its pseudo-terminal.
    pub fn terminal(&self, cols: u16, rows: u16) -> std::io::Result<UnixStream> {
        self.request(&Request::Terminal { cols, rows })
    }

    /// Asks the guest to stop, then waits for Firecracker to exit (killing it after 5 seconds).
    pub fn shutdown(mut self) -> std::io::Result<()> {
        // Wait for the acknowledgement: hanging up first would abort the request in the guest.
        if let Ok(stream) = self.request(&Request::Shutdown) {
            stream.set_read_timeout(Some(Duration::from_secs(2)))?;
            let _ = BufReader::new(stream).read_line(&mut String::new());
        }
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if self.process.try_wait()?.is_some() {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        self.process.kill()
    }
}

impl Drop for MicroVm {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
        let _ = std::fs::remove_dir_all(&self.run_dir);
    }
}
