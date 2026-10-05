//! `mentor-host` prototype command line.
//!
//! ```text
//! mentor-host exec  [options] -- <command> [args…]   boot a microVM, run one command, print its output, stop
//! mentor-host shell [options]                        boot a microVM and attach an interactive shell
//! mentor-host snapshot <directory> [options]         boot a microVM and save it as a snapshot
//!
//! exec and shell accept `--from <directory>` to start from a snapshot instead of booting the kernel.
//!
//! options: --firecracker <path> --kernel <path> --rootfs <path> --vcpus <n> --memory <MiB> --disk <MiB>
//! defaults: .dev/firecracker, .dev/vmlinux, .dev/rootfs.ext4, 1 vCPU, 256 MiB of memory, 256 MiB of session disk
//! ```

use std::io::{IsTerminal, Read, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use mentor_host::microvm::{Config, MicroVm, Snapshot};

fn usage() -> ExitCode {
    eprintln!(
        "usage: mentor-host <exec|shell|snapshot DIR> [--from DIR] [--firecracker P] [--kernel P] [--rootfs P] [--vcpus N] [--memory MiB] [--disk MiB] [-- command…]"
    );
    ExitCode::from(2)
}

/// Puts the local terminal in raw mode for the duration of a shell session.
struct RawMode(Option<libc::termios>);

impl RawMode {
    fn enable() -> Self {
        if !std::io::stdin().is_terminal() {
            return Self(None);
        }
        // SAFETY: termios calls on stdin with a zero-initialised structure filled by `tcgetattr`.
        unsafe {
            let mut saved: libc::termios = std::mem::zeroed();
            if libc::tcgetattr(0, &mut saved) != 0 {
                return Self(None);
            }
            let mut raw = saved;
            libc::cfmakeraw(&mut raw);
            libc::tcsetattr(0, libc::TCSANOW, &raw);
            Self(Some(saved))
        }
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        if let Some(saved) = self.0 {
            // SAFETY: restores the attributes read by `tcgetattr`.
            unsafe { libc::tcsetattr(0, libc::TCSANOW, &saved) };
        }
    }
}

fn terminal_size() -> (u16, u16) {
    // SAFETY: `TIOCGWINSZ` fills a `winsize`; on failure the zeroed value is replaced by the default below.
    let size = unsafe {
        let mut size: libc::winsize = std::mem::zeroed();
        libc::ioctl(1, libc::TIOCGWINSZ, &mut size);
        size
    };
    if size.ws_col == 0 {
        (80, 24)
    } else {
        (size.ws_col, size.ws_row)
    }
}

fn run() -> std::io::Result<ExitCode> {
    let mut args = std::env::args().skip(1);
    let Some(mode) = args.next() else { return Ok(usage()) };
    let mut config = Config {
        firecracker: PathBuf::from(".dev/firecracker"),
        kernel: PathBuf::from(".dev/vmlinux"),
        rootfs: PathBuf::from(".dev/rootfs.ext4"),
        vcpus: 1,
        memory_mib: 256,
        session_disk_mib: Some(256),
    };
    let mut command: Vec<String> = Vec::new();
    let mut from: Option<PathBuf> = None;
    let snapshot_dir: Option<PathBuf> = if mode == "snapshot" { args.next().map(PathBuf::from) } else { None };
    while let Some(arg) = args.next() {
        let mut value = || args.next().ok_or_else(|| std::io::Error::other(format!("{arg} needs a value")));
        match arg.as_str() {
            "--from" => from = Some(value()?.into()),
            "--firecracker" => config.firecracker = value()?.into(),
            "--kernel" => config.kernel = value()?.into(),
            "--rootfs" => config.rootfs = value()?.into(),
            "--vcpus" => config.vcpus = value()?.parse().map_err(std::io::Error::other)?,
            "--memory" => config.memory_mib = value()?.parse().map_err(std::io::Error::other)?,
            "--disk" => config.session_disk_mib = Some(value()?.parse().map_err(std::io::Error::other)?),
            "--" => {
                command = args.by_ref().collect();
            }
            _ => return Ok(usage()),
        }
    }

    let start = || -> std::io::Result<MicroVm> {
        let vm = match &from {
            Some(directory) => MicroVm::restore(&Snapshot::open(directory, &config)?, &config)?,
            None => MicroVm::start(&config)?,
        };
        let how = if from.is_some() { "restored from a snapshot" } else { "cold boot" };
        eprintln!("mentor-host: guest agent ready in {:.1} ms ({how})\r", vm.boot_time.as_secs_f64() * 1000.0);
        Ok(vm)
    };
    match mode.as_str() {
        "snapshot" => {
            let Some(directory) = snapshot_dir else { return Ok(usage()) };
            let snapshot = start()?.snapshot(&directory)?;
            eprintln!("mentor-host: snapshot written to {}", snapshot.directory.display());
            Ok(ExitCode::SUCCESS)
        }
        "exec" if !command.is_empty() => {
            let vm = start()?;
            let result = vm.exec(&command, None)?;
            print!("{}", result.stdout);
            eprint!("{}", result.stderr);
            if let Some(error) = &result.error {
                eprintln!("mentor-host: {error}");
            }
            vm.shutdown()?;
            Ok(ExitCode::from(result.exit_code.map_or(1, |code| code as u8)))
        }
        "shell" => {
            let vm = start()?;
            let (cols, rows) = terminal_size();
            let mut output = vm.terminal(cols, rows)?;
            let mut input = output.try_clone()?;
            {
                let _raw = RawMode::enable();
                std::thread::spawn(move || {
                    let _ = std::io::copy(&mut std::io::stdin(), &mut input);
                });
                let mut stdout = std::io::stdout();
                let mut buffer = [0u8; 4096];
                loop {
                    match output.read(&mut buffer) {
                        Ok(0) | Err(_) => break,
                        Ok(n) => {
                            stdout.write_all(&buffer[..n])?;
                            stdout.flush()?;
                        }
                    }
                }
            }
            vm.shutdown()?;
            Ok(ExitCode::SUCCESS)
        }
        _ => Ok(usage()),
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(err) => {
            eprintln!("mentor-host: {err}");
            ExitCode::FAILURE
        }
    }
}
