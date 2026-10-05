//! Host side of the Mentor execution plane.
//!
//! **Prototype.** It proves the chain kernel → read-only rootfs → guest agent → vsock on one machine. It does
//! not yet use the jailer, cgroups, a per-session disk, snapshots or networking, and must not run untrusted
//! workloads as is. See `doc/architecture.md` §4.1 for the target design.

pub mod microvm;
