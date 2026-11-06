//! Adapter-independent OBD-II building blocks.
//!
//! - [`Transport`] is the trait any adapter backend implements.
//! - [`pid`] decodes Mode 01 (current data) PIDs into physical values.
//! - [`response`] extracts payload bytes from raw adapter text.

pub mod error;
pub mod pid;
pub mod response;
pub mod transport;

pub use error::{Error, Result};
pub use pid::{PidDef, Reading};
pub use transport::Transport;
