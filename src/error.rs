use std::error::Error;
use std::fmt;
use i2cdev::linux::LinuxI2CError;
use crate::RelayStatus;

#[derive(Debug)]
pub enum RelayError {
    I2C(LinuxI2CError),
    StateVerificationFailed {
        relay_num: Option<u8>,
        expected: bool,
        actual: bool,
        attempts: u8,
    },
    VerificationFailed {
        relay_num: Option<u8>,
        expected: RelayStatus,
        attempts: u8,
    },
    VerificationTimeout {
        relay_num: Option<u8>,
        expected: RelayStatus,
        timeout_ms: u64,
    },
    Timeout {
        relay_num: Option<u8>,
        operation: String,
        duration_ms: u64,
    },
    InvalidConfiguration(String),
    InvalidRelayNumber {
        relay_num: u8,
        max_relays: u8,
    },
    InvalidI2CAddress(u8),
}

impl fmt::Display for RelayError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RelayError::I2C(err) => write!(f, "I2C error: {}", err),
            RelayError::StateVerificationFailed {
                relay_num,
                expected,
                actual,
                attempts,
            } => {
                let relay_desc = relay_num
                    .map(|n| format!("relay {}", n))
                    .unwrap_or_else(|| "relay".to_string());
                write!(
                    f,
                    "State verification failed for {}: expected {}, got {} after {} attempts",
                    relay_desc,
                    if *expected { "ON" } else { "OFF" },
                    if *actual { "ON" } else { "OFF" },
                    attempts
                )
            }
            RelayError::VerificationFailed {
                relay_num,
                expected,
                attempts,
            } => {
                let relay_desc = relay_num
                    .map(|n| format!("relay {}", n))
                    .unwrap_or_else(|| "relay".to_string());
                write!(
                    f,
                    "Verification failed for {}: expected {:?} after {} attempts",
                    relay_desc, expected, attempts
                )
            }
            RelayError::VerificationTimeout {
                relay_num,
                expected,
                timeout_ms,
            } => {
                let relay_desc = relay_num
                    .map(|n| format!("relay {}", n))
                    .unwrap_or_else(|| "relay".to_string());
                write!(
                    f,
                    "Verification timeout for {}: expected {:?} after {}ms",
                    relay_desc, expected, timeout_ms
                )
            }
            RelayError::Timeout {
                relay_num,
                operation,
                duration_ms,
            } => {
                let relay_desc = relay_num
                    .map(|n| format!("relay {}", n))
                    .unwrap_or_else(|| "relay".to_string());
                write!(
                    f,
                    "Timeout during {} for {}: exceeded {}ms",
                    operation, relay_desc, duration_ms
                )
            }
            RelayError::InvalidConfiguration(msg) => {
                write!(f, "Invalid configuration: {}", msg)
            }
            RelayError::InvalidRelayNumber {
                relay_num,
                max_relays,
            } => {
                write!(
                    f,
                    "Invalid relay number {}: valid range is 1-{}",
                    relay_num, max_relays
                )
            }
            RelayError::InvalidI2CAddress(addr) => {
                write!(f, "Invalid I2C address 0x{:02X}: valid range is 0x08-0x77", addr)
            }
        }
    }
}

impl Error for RelayError {}

impl From<LinuxI2CError> for RelayError {
    fn from(err: LinuxI2CError) -> Self {
        RelayError::I2C(err)
    }
}

pub type RelayResult<T> = Result<T, RelayError>;