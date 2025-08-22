# Changelog

## [Unreleased] - State Verification and Error Recovery

### Added
- **State Verification System**: Automatic verification after relay toggle operations
  - Verifies relay actually changed to expected state
  - Configurable retry logic with attempts counter
  - Timeout handling to prevent indefinite waiting
  - Three built-in verification modes: Strict, Lenient, and Disabled

- **Enhanced Error Types** (`src/error.rs`):
  - `RelayError::StateVerificationFailed`: Indicates relay didn't reach expected state
  - `RelayError::Timeout`: Operation exceeded configured timeout
  - `RelayError::InvalidConfiguration`: Configuration parameter validation errors
  - Improved error messages with detailed context

- **Verification Configuration** (`src/verification.rs`):
  - `VerificationConfig` struct with customizable parameters:
    - `max_retries`: Number of retry attempts (default: 3)
    - `retry_delay_ms`: Delay between retries (default: 50ms)
    - `verification_delay_ms`: Delay before checking state (default: 20ms)
    - `timeout_ms`: Total operation timeout (default: 1000ms)
  - Pre-configured modes:
    - `VerificationConfig::strict()`: Default, balanced settings
    - `VerificationConfig::lenient()`: More retries, longer delays for noisy environments
    - `VerificationConfig::disabled()`: No verification for maximum speed
  - Builder pattern for custom configurations

- **New Public APIs**:
  - `QwiicRelayConfig::with_verification()`: Create config with custom verification
  - `VerificationConfig` builder methods for fine-tuning parameters
  - Enhanced error types exposed in public API

- **Comprehensive Test Suite** (`src/tests.rs`):
  - Unit tests for verification configurations
  - Mock tests for retry logic and timeout scenarios
  - Integration tests for hardware verification (requires device)
  - Error handling and display tests

- **Example Programs**:
  - `examples/verification_demo.rs`: Demonstrates all verification modes
  - `examples/error_handling.rs`: Shows proper error handling patterns

### Changed
- `QwiicRelayConfig` now includes `verification` field
- `set_relay_on()` and `set_relay_off()` now perform verification by default
- Return types updated from `Result<(), LinuxI2CError>` to `Result<(), RelayError>`
- `change_i2c_address()` now returns proper `InvalidConfiguration` errors

### Implementation Details
- Internal methods `set_relay_on_unverified()` and `set_relay_off_unverified()` for raw operations
- Verified methods `set_relay_on_verified()` and `set_relay_off_verified()` implement retry loop
- Timeout tracking using `Instant::now()` for accurate timing
- State verification performed after configurable delay to allow hardware settling

### Backward Compatibility
- Default configuration maintains existing behavior with added verification
- `VerificationConfig::disabled()` provides original non-verified behavior
- All existing APIs remain functional with enhanced error types