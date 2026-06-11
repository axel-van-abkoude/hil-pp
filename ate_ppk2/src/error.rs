//! Errors used by the HILpp crate

use std::error::Error;
use std::fmt;

#[allow(missing_docs)]
pub type Result<T> = std::result::Result<T, HILppError>;

#[derive(Debug)]
#[allow(missing_docs)]
pub enum HILppError {
    Ppk2(ppk2::Error),
    Libc(std::io::Error),
    TimeError(String),
    Custom(String),
}

impl fmt::Display for HILppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HILppError::Ppk2(error) => write!(f, "[PPK2 ERROR] {}", error),
            HILppError::Libc(s) => write!(f, "[LIBC ERROR] {}", s),
            HILppError::Custom(s) => write!(f, "[CUSTOM ERROR] {}", s),
            HILppError::TimeError(s) => write!(f, "[TIMER ERROR], {}", s),
        }
    }
}

impl Error for HILppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            HILppError::Ppk2(error) => Some(error),
            HILppError::Libc(error) => Some(error),
            HILppError::Custom(_) => None,
            HILppError::TimeError(_) => None,
        }
    }
}
