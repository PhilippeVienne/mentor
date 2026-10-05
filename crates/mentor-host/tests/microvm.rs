//! End-to-end tests of the prototype execution plane: they boot real Firecracker microVMs.
//!
//! They need KVM and the development artefacts of `tools/setup-dev.sh` and `tools/build-rootfs.sh`
//! (`.dev/firecracker`, `.dev/vmlinux`, `.dev/rootfs.ext4`). Without them every test is skipped, with a
//! message, so that `cargo test` stays green on machines without KVM.

use std::io::{Read, Write};
use std::path::Path;
use std::time::Duration;

use mentor_host::microvm::{Config, MicroVm};
use std::time::Instant;

fn config() -> Option<Config> {
    let dev = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.dev");
    let config = Config {
        firecracker: dev.join("firecracker"),
        kernel: dev.join("vmlinux"),
        rootfs: dev.join("rootfs.ext4"),
        vcpus: 1,
        memory_mib: 128,
        session_disk_mib: Some(32),
    };
    let ready = Path::new("/dev/kvm").exists() && [&config.firecracker, &config.kernel, &config.rootfs].iter().all(|path| path.exists());
    if !ready {
        eprintln!("skipped: KVM or the .dev artefacts are missing (run tools/setup-dev.sh and tools/build-rootfs.sh)");
    }
    ready.then_some(config)
}

fn sh(vm: &MicroVm, script: &str) -> mentor_guest::protocol::ExecResult {
    vm.exec(&["sh".to_string(), "-c".to_string(), script.to_string()], None).expect("exec")
}

#[test]
fn boots_and_runs_a_command() {
    let Some(config) = config() else { return };
    let vm = MicroVm::start(&config).expect("boot");
    assert!(vm.boot_time < Duration::from_secs(5), "boot took {:?}", vm.boot_time);
    let result = sh(&vm, "echo out; echo err >&2; exit 3");
    assert_eq!((result.exit_code, result.stdout.as_str(), result.stderr.as_str()), (Some(3), "out\n", "err\n"));
    // The guest agent is the init process and the working directory is /workspace.
    assert_eq!(sh(&vm, "cat /proc/1/comm; pwd").stdout, "mentor-guest\n/workspace\n");
    vm.shutdown().expect("shutdown");
}

#[test]
fn a_missing_program_is_reported_not_fatal() {
    let Some(config) = config() else { return };
    let vm = MicroVm::start(&config).expect("boot");
    let result = vm.exec(&["/no/such/program".to_string()], None).expect("exec");
    assert!(result.exit_code.is_none() && result.error.is_some());
    assert_eq!(sh(&vm, "echo still-alive").stdout, "still-alive\n");
}

#[test]
fn root_is_read_only_and_there_is_no_network() {
    let Some(config) = config() else { return };
    let vm = MicroVm::start(&config).expect("boot");
    assert_ne!(sh(&vm, "touch /etc/written").exit_code, Some(0));
    assert_eq!(sh(&vm, "echo data > /workspace/file && cat /workspace/file").stdout, "data\n");
    // Only the loopback interface exists: the control channel is vsock.
    assert_eq!(sh(&vm, "ls /sys/class/net").stdout, "lo\n");
}

#[test]
fn two_microvms_share_nothing() {
    let Some(config) = config() else { return };
    let (first, second) = (MicroVm::start(&config).expect("boot"), MicroVm::start(&config).expect("boot"));
    sh(&first, "echo secret > /workspace/only-here");
    assert_eq!(sh(&first, "cat /workspace/only-here").stdout, "secret\n");
    assert_ne!(sh(&second, "cat /workspace/only-here").exit_code, Some(0));
}

#[test]
fn terminal_is_a_real_pty() {
    let Some(config) = config() else { return };
    let vm = MicroVm::start(&config).expect("boot");
    let mut stream = vm.terminal(100, 30).expect("terminal");
    stream.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
    stream.write_all(b"tty; stty size; echo done-$((6*7)); exit\n").unwrap();
    let mut output = String::new();
    // The stream ends when the shell exits.
    let _ = stream.read_to_string(&mut output);
    assert!(output.contains("/dev/pts/0"), "{output}");
    assert!(output.contains("30 100"), "{output}");
    assert!(output.contains("done-42"), "{output}");
}

#[test]
fn session_disk_is_a_hard_quota() {
    let Some(config) = config() else { return };
    let vm = MicroVm::start(&config).expect("boot");
    assert_eq!(sh(&vm, "awk '$2 == \"/workspace\" { print $1, $3 }' /proc/mounts").stdout, "/dev/vdb ext4\n");
    // 32 MiB of disk: writing 64 MiB must fail, and what was written stays within the quota.
    let overflow = sh(&vm, "dd if=/dev/zero of=/workspace/big bs=1M count=64 2>&1; du -m /workspace/big | cut -f1");
    assert!(overflow.stdout.contains("No space left on device"), "{}", overflow.stdout);
    let written: u32 = overflow.stdout.lines().last().unwrap().trim().parse().unwrap();
    assert!(written <= 32, "{written} MiB written on a 32 MiB disk");
    // The VM keeps working once the disk is full.
    assert_eq!(sh(&vm, "rm /workspace/big && echo ok > /workspace/f && cat /workspace/f").stdout, "ok\n");
}

#[test]
fn snapshot_restores_fast_and_each_copy_is_independent() {
    let Some(config) = config() else { return };
    let directory = std::env::temp_dir().join(format!("mentor-snapshot-test-{}", std::process::id()));
    let cold = MicroVm::start(&config).expect("boot");
    let cold_boot = cold.boot_time;
    sh(&cold, "echo prepared > /workspace/template; echo in-memory > /tmp/marker");
    let snapshot = cold.snapshot(&directory).expect("snapshot");

    let started = Instant::now();
    let (first, second) = (MicroVm::restore(&snapshot, &config).expect("restore"), MicroVm::restore(&snapshot, &config).expect("restore"));
    eprintln!("cold boot {cold_boot:?}, restore {:?} and {:?} (both in {:?})", first.boot_time, second.boot_time, started.elapsed());
    assert!(first.boot_time < cold_boot, "restore ({:?}) should beat a cold boot ({cold_boot:?})", first.boot_time);

    // Disk and memory state both come back.
    assert_eq!(sh(&first, "cat /workspace/template /tmp/marker").stdout, "prepared\nin-memory\n");
    // Each restored VM has its own disk.
    sh(&first, "echo mine > /workspace/private");
    assert_ne!(sh(&second, "cat /workspace/private").exit_code, Some(0));
    // The clock was set on resume: within a minute of the host's.
    let guest_time: i64 = sh(&second, "date +%s").stdout.trim().parse().unwrap();
    let host_time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
    assert!((guest_time - host_time).abs() < 60, "guest clock {guest_time} vs host {host_time}");
    // The two copies do not share their random generator state.
    let random = |vm: &MicroVm| sh(vm, "head -c 16 /dev/urandom | od -An -tx1").stdout;
    assert_ne!(random(&first), random(&second));

    drop((first, second));
    let _ = std::fs::remove_dir_all(&directory);
}
