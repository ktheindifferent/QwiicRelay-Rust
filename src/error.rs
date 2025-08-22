use std::error::Error;
use std::fmt;
use i2cdev::linux::LinuxI2CError;

#[derive(Debug)]
pub enum RelayError {
    I2C(LinuxI2CError),
    StateVerificationFailed {
        relay_num: Option<u8>,
        expected: bool,
        actual: bool,
        attempts: u8,
    },
    Timeout {
        relay_num: Option<u8>,
        operation: String,
        duration_ms: u64,
    },
    InvalidConfiguration(String),
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