//! One Firecracker microVM and the connection to its guest agent.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use mentor_guest::protocol::{ExecResult, Request, PORT};

/// How long the guest agent may take to answer after the VM process starts.
const BOOT_TIMEOUT: Duration = Duration::from_secs(15);
const VSOCK_SOCKET: &str = "vsock.sock";
const CONSOLE_LOG: &str = "console.log";

/// What a microVM is made of.
#[derive(Debug, Clone)]
pub struct Config {
    pub firecracker: PathBuf,
    pub kernel: PathBuf,
    /// ext4 image mounted read-only as the root file system; its init is `mentor-guest`.
    pub rootfs: PathBuf,
    pub vcpus: u8,
    pub memory_mib: u32,
}

/// A running microVM. Dropping it kills the VM and removes its run directory.
pub struct MicroVm {
    process: Child,
    run_dir: PathBuf,
    /// Time between starting Firecracker and the guest agent accepting a connection.
    pub boot_time: Duration,
}

fn error(message: impl Into<String>) -> std::io::Error {
    std::io::Error::other(message.into())
}

impl MicroVm {
    /// Boots a microVM and waits until its guest agent answers.
    pub fn start(config: &Config) -> std::io::Result<Self> {
        let run_dir = std::env::temp_dir().join(format!(
            "mentor-vm-{}-{:x}",
            std::process::id(),
            Instant::now().elapsed().as_nanos() ^ rand_suffix()
        ));
        std::fs::create_dir_all(&run_dir)?;
        let machine = serde_json::json!({
            "boot-source": {
                "kernel_image_path": absolute(&config.kernel)?,
                // `reboot=k` makes a guest reboot exit Firecracker; `panic=1` does the same on a kernel panic.
                "boot_args": "console=ttyS0 reboot=k panic=1 pci=off quiet init=/sbin/mentor-guest",
            },
            "drives": [{ "drive_id": "rootfs", "path_on_host": absolute(&config.rootfs)?, "is_root_device": true, "is_read_only": true }],
            "machine-config": { "vcpu_count": config.vcpus, "mem_size_mib": config.memory_mib },
            "vsock": { "guest_cid": 3, "uds_path": VSOCK_SOCKET },
        });
        std::fs::write(run_dir.join("machine.json"), serde_json::to_vec_pretty(&machine)?)?;
        let console = std::fs::File::create(run_dir.join(CONSOLE_LOG))?;
        let started = Instant::now();
        let mut command = Command::new(absolute(&config.firecracker)?);
        // SAFETY: `prctl` is async-signal-safe. The VM must not outlive this process, even if it is killed.
        unsafe {
            command.pre_exec(|| {
                libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL);
                Ok(())
            });
        }
        let process = command
            .args(["--no-api", "--config-file", "machine.json"])
            .current_dir(&run_dir)
            .stdin(Stdio::null())
            .stdout(console.try_clone()?)
            .stderr(console)
            .spawn()
            .map_err(|err| error(format!("cannot start {}: {err}", config.firecracker.display())))?;
        let mut vm = Self { process, run_dir, boot_time: Duration::ZERO };
        loop {
            if let Some(status) = vm.process.try_wait()? {
                return Err(error(format!("Firecracker exited during boot ({status}):\n{}", vm.console())));
            }
            if vm.connect().is_ok() {
                vm.boot_time = started.elapsed();
                return Ok(vm);
            }
            if started.elapsed() > BOOT_TIMEOUT {
                return Err(error(format!("the guest agent did not answer within {BOOT_TIMEOUT:?}:\n{}", vm.console())));
            }
            std::thread::sleep(Duration::from_millis(10));
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
        while std::io::Read::read(&mut stream, &mut byte)? == 1 && byte[0] != b'\n' {
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

fn absolute(path: &Path) -> std::io::Result<PathBuf> {
    path.canonicalize().map_err(|err| error(format!("{}: {err}", path.display())))
}

/// A few unpredictable bits for the run directory name; not a security boundary (the directory is private).
fn rand_suffix() -> u128 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_nanos())
}
