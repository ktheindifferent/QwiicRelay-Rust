use std::time::Duration;

#[derive(Clone, Copy, Debug)]
pub enum VerificationMode {
    Strict,
    Lenient,
    Disabled,
}

impl Default for VerificationMode {
    fn default() -> Self {
        VerificationMode::Strict
    }
}

#[derive(Clone, Copy, Debug)]
pub struct VerificationConfig {
    pub mode: VerificationMode,
    pub max_retries: u8,
    pub retry_delay_ms: u64,
    pub verification_delay_ms: u64,
    pub timeout_ms: u64,
}

impl Default for VerificationConfig {
    fn default() -> Self {
        VerificationConfig {
            mode: VerificationMode::Strict,
            max_retries: 3,
            retry_delay_ms: 50,
            verification_delay_ms: 20,
            timeout_ms: 1000,
        }
    }
}

impl VerificationConfig {
    pub fn strict() -> Self {
        Self::default()
    }

    pub fn lenient() -> Self {
        VerificationConfig {
            mode: VerificationMode::Lenient,
            max_retries: 5,
            retry_delay_ms: 100,
            verification_delay_ms: 50,
            timeout_ms: 2000,
        }
    }

    pub fn disabled() -> Self {
        VerificationConfig {
            mode: VerificationMode::Disabled,
            max_retries: 0,
            retry_delay_ms: 0,
            verification_delay_ms: 0,
            timeout_ms: 0,
        }
    }

    pub fn with_mode(mut self, mode: VerificationMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn with_max_retries(mut self, retries: u8) -> Self {
        self.max_retries = retries;
        self
    }

    pub fn with_retry_delay(mut self, delay_ms: u64) -> Self {
        self.retry_delay_ms = delay_ms;
        self
    }

    pub fn with_verification_delay(mut self, delay_ms: u64) -> Self {
        self.verification_delay_ms = delay_ms;
        self
    }

    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    pub fn retry_delay(&self) -> Duration {
        Duration::from_millis(self.retry_delay_ms)
    }

    pub fn verification_delay(&self) -> Duration {
        Duration::from_millis(self.verification_delay_ms)
    }

    pub fn timeout(&self) -> Duration {
        Duration::from_millis(self.timeout_ms)
    }
}