//! Wire protocol between the host agent and the guest agent.
//!
//! The host opens one vsock connection per request and sends one JSON line. `exec`, `resume` and `shutdown` answer
//! with one JSON line. `terminal` answers nothing: after the request line the connection carries the raw bytes of
//! a pseudo-terminal, in both directions, until the shell exits.

use serde::{Deserialize, Serialize};

/// vsock port the guest agent listens on.
pub const PORT: u32 = 52;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "lowercase")]
pub enum Request {
    /// Runs a command to completion. `argv` is never passed through a shell by the agent.
    Exec {
        argv: Vec<String>,
        #[serde(default)]
        workdir: Option<String>,
    },
    /// Opens an interactive shell on a pseudo-terminal.
    Terminal { cols: u16, rows: u16 },
    /// Sent once after the microVM was restored from a snapshot: its clock stopped when the snapshot was taken.
    Resume {
        /// Current time, in milliseconds since the Unix epoch.
        unix_millis: u64,
    },
    /// Stops the microVM.
    Shutdown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecResult {
    /// Exit code, or `None` when the command was killed by a signal.
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    /// Set when the command could not be started.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
