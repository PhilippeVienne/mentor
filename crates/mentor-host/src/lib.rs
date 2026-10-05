//! Host side of the Mentor execution plane.
//!
//! **Prototype.** It proves the chain kernel → read-only rootfs → guest agent → vsock on one machine, with a
//! per-session disk and snapshots. It does not yet use the jailer, cgroups or networking, and must not run
//! untrusted workloads as is. See `doc/architecture.md` §4.1 for the target design.

mod api;
pub mod microvm;
