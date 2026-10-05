//! Guest side of the Mentor execution plane. The binary is the microVM's init process; the library exposes
//! the [`protocol`] shared with the host agent.

pub mod protocol;
