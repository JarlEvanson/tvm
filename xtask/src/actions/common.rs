//! Common code for [`Action`][action] handling.
//!
//! [action]: crate::cli::Action

use core::error;
use std::{fmt, io};

// Runs a [`Command`][c], handling non-zero exit codes and other failures.
///
/// [c]: std::process::Command
pub fn run_cmd(mut cmd: std::process::Command) -> Result<(), RunCommandError> {
    println!("Running command: {cmd:?}");

    let status = cmd.status()?;
    if !status.success() {
        return Err(RunCommandError::CommandFailed {
            code: status.code(),
        });
    }

    Ok(())
}

/// Various errors that can occur while running a command.
#[derive(Debug)]
pub enum RunCommandError {
    /// An error occurred while launching the command.
    ProcessError(io::Error),
    /// The command exited with a non-zero exit code.
    CommandFailed {
        /// The exit of code of the command.
        code: Option<i32>,
    },
}

impl From<io::Error> for RunCommandError {
    fn from(value: io::Error) -> Self {
        Self::ProcessError(value)
    }
}

impl fmt::Display for RunCommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProcessError(error) => write!(f, "error launching command: {error}"),
            Self::CommandFailed { code: Some(code) } => {
                write!(f, "command failed with exit status {code}")
            }
            Self::CommandFailed { code: None } => write!(f, "command terminated by signal"),
        }
    }
}

impl error::Error for RunCommandError {}
