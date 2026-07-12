//! Exit-code error type (SPEC-EXIT-001).
//!
//! Codes: 1 General, 2 Usage, 3 Auth, 4 RateLimit, 5 API, 10 FileIO.
//! `main` prints the message to stderr and exits with `code` (SPEC-EXIT-002).

use std::fmt;

/// Exit codes per SPEC-EXIT-001. Values are the process exit codes.
pub mod exit {
    pub const GENERAL: i32 = 1;
    pub const USAGE: i32 = 2;
    pub const AUTH: i32 = 3;
    pub const RATE_LIMIT: i32 = 4;
    pub const API: i32 = 5;
    pub const FILE_IO: i32 = 10;
}

/// An error carrying a process exit code and a human message.
#[derive(Debug)]
pub struct AppError {
    pub code: i32,
    pub message: String,
}

impl AppError {
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    /// Exit 1 — general failure (also the fallback for parse errors, SPEC-EXIT-003).
    pub fn general(message: impl Into<String>) -> Self {
        Self::new(exit::GENERAL, message)
    }

    /// Exit 2 — explicit in-code usage error (e.g. `steps must be between 2 and 8`).
    pub fn usage(message: impl Into<String>) -> Self {
        Self::new(exit::USAGE, message)
    }

    /// Exit 3 — authentication failure.
    pub fn auth(message: impl Into<String>) -> Self {
        Self::new(exit::AUTH, message)
    }

    /// Exit 4 — rate limited.
    pub fn rate_limit(message: impl Into<String>) -> Self {
        Self::new(exit::RATE_LIMIT, message)
    }

    /// Exit 5 — provider/API error.
    pub fn api(message: impl Into<String>) -> Self {
        Self::new(exit::API, message)
    }

    /// Exit 10 — filesystem/IO error.
    pub fn file_io(message: impl Into<String>) -> Self {
        Self::new(exit::FILE_IO, message)
    }

    /// A placeholder for command bodies not yet ported (exit 1).
    pub fn unimplemented(what: &str) -> Self {
        Self::general(format!("{what}: not implemented yet"))
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for AppError {}

pub type AppResult<T> = Result<T, AppError>;
