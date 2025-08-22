// Copyright 2021 Caleb Mitchell Smith-Woolrich (PixelCoda)
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! A Rust library for controlling SparkFun Qwiic Relay boards via I2C.
//!
//! This library provides a simple interface for controlling various types of Qwiic Relay boards
//! including single relays, dual solid state relays, quad relays, and quad solid state relays.
//!
//! # Example
//! ```no_run
//! use qwiic_relay_rs::{QwiicRelay, QwiicRelayConfig};
//!
//! let config = QwiicRelayConfig::default();
//! let mut relay = QwiicRelay::new(config, "/dev/i2c-1", 0x08).unwrap();
//! relay.set_relay_on(Some(1)).unwrap();
//! ```

extern crate i2cdev;

mod error;
mod verification;

use std::thread;
use std::time::{Duration, Instant};

use i2cdev::core::*;
use i2cdev::linux::LinuxI2CDevice;

pub use error::{RelayError, RelayResult};
pub use verification::{VerificationConfig, VerificationMode};

/// I2C addresses for different Qwiic Relay board configurations.
#[derive(Copy, Clone)]
pub enum Addresses {
    SingleRelayDefault = 0x18,
    SingleRelayJumperClosed = 0x19,
    QuadRelayDefault = 0x6D,
    QuadRelayJumperClosed = 0x6C,
    DualSolidState = 0x0A,
    DualSolidStateJumperClosed = 0x0B,
    QuadSolidState = 0x08,
    QuadSolidStateJumperClosed = 0x09,
}

/// Commands that can be sent to the Qwiic Relay boards.
#[derive(Copy, Clone)]
pub enum Command {
    DualQuadToggleBase = 0x00,
    ToggleRelayOne = 0x01,
    ToggleRelayTwo = 0x02,
    ToggleRelayThree = 0x03,
    ToggleRelayFour = 0x04,
    RelayOneStatus = 0x05,
    RelayTwoStatus = 0x06,
    RelayThreeStatus = 0x07,
    RelayFourStatus = 0x08,
    TurnAllOff = 0x0A,
    TurnAllOn = 0x0B,
    ToggleAll = 0x0C,
}

/// Relay state and control values.
#[derive(Copy, Clone)]
pub enum RelayState {
    Off = 0x00,
    On = 0x01,
    SingleFirmwareVersion = 0x04,
    SingleStatusVersion = 0x05,
}

/// Status values returned by the relay board.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum RelayStatus {
    Off = 0,
    On = 1,
}

impl From<bool> for RelayStatus {
    fn from(value: bool) -> Self {
        if value { RelayStatus::On } else { RelayStatus::Off }
    }
}

impl From<RelayStatus> for bool {
    fn from(status: RelayStatus) -> Self {
        status == RelayStatus::On
    }
}

impl From<u8> for RelayStatus {
    fn from(value: u8) -> Self {
        if value != 0 { RelayStatus::On } else { RelayStatus::Off }
    }
}

impl From<RelayStatus> for u8 {
    fn from(status: RelayStatus) -> Self {
        status as u8
    }
}

/// Configuration for a Qwiic Relay board.
#[derive(Clone, Copy)]
pub struct QwiicRelayConfig {
    /// Number of relays on the board (1, 2, or 4).
    pub relay_count: u8,
    /// Configuration for state verification after relay operations.
    pub verification: VerificationConfig,
    /// Microseconds delay after write operations (default: 10).
    pub write_delay_us: u32,
    /// Milliseconds to wait for state change (default: 10).
    pub state_change_delay_ms: u32,
    /// Milliseconds to wait during initialization (default: 200).
    pub init_delay_ms: u32,
}

impl QwiicRelayConfig {
    /// Creates a new configuration with the specified number of relays and default timing.
    ///
    /// # Arguments
    /// * `relay_count` - Number of relays on the board (typically 1, 2, or 4)
    pub fn new(relay_count: u8) -> QwiicRelayConfig {
        QwiicRelayConfig {
            relay_count,
            verification: VerificationConfig::default(),
            write_delay_us: 10,
            state_change_delay_ms: 10,
            init_delay_ms: 200,
        }
    }

    /// Creates a new configuration with custom verification settings.
    ///
    /// # Arguments
    /// * `relay_count` - Number of relays on the board
    /// * `verification` - Verification configuration
    pub fn with_verification(relay_count: u8, verification: VerificationConfig) -> QwiicRelayConfig {
        QwiicRelayConfig {
            relay_count,
            verification,
            write_delay_us: 10,
            state_change_delay_ms: 10,
            init_delay_ms: 200,
        }
    }

    /// Creates a configuration with custom timing parameters.
    ///
    /// # Arguments
    /// * `relay_count` - Number of relays on the board
    /// * `write_delay_us` - Microseconds delay after write operations
    /// * `state_change_delay_ms` - Milliseconds to wait for state change
    /// * `init_delay_ms` - Milliseconds to wait during initialization
    pub fn with_timing(
        relay_count: u8,
        write_delay_us: u32,
        state_change_delay_ms: u32,
        init_delay_ms: u32,
    ) -> QwiicRelayConfig {
        QwiicRelayConfig {
            relay_count,
            verification: VerificationConfig::default(),
            write_delay_us,
            state_change_delay_ms,
            init_delay_ms,
        }
    }

    /// Creates a configuration with both custom verification and timing settings.
    ///
    /// # Arguments
    /// * `relay_count` - Number of relays on the board
    /// * `verification` - Verification configuration
    /// * `write_delay_us` - Microseconds delay after write operations
    /// * `state_change_delay_ms` - Milliseconds to wait for state change
    /// * `init_delay_ms` - Milliseconds to wait during initialization
    pub fn with_verification_and_timing(
        relay_count: u8,
        verification: VerificationConfig,
        write_delay_us: u32,
        state_change_delay_ms: u32,
        init_delay_ms: u32,
    ) -> QwiicRelayConfig {
        QwiicRelayConfig {
            relay_count,
            verification,
            write_delay_us,
            state_change_delay_ms,
            init_delay_ms,
        }
    }

    /// Creates a configuration optimized for solid state relays.
    /// Solid state relays typically switch faster than mechanical relays.
    pub fn for_solid_state(relay_count: u8) -> QwiicRelayConfig {
        QwiicRelayConfig {
            relay_count,
            verification: VerificationConfig::default(),
            write_delay_us: 5,      // Faster switching
            state_change_delay_ms: 5,  // No mechanical delay
            init_delay_ms: 100,     // Faster initialization
        }
    }

    /// Creates a configuration optimized for mechanical relays.
    /// Mechanical relays need more time for physical switching.
    pub fn for_mechanical(relay_count: u8) -> QwiicRelayConfig {
        QwiicRelayConfig {
            relay_count,
            verification: VerificationConfig::default(),
            write_delay_us: 15,     // More conservative timing
            state_change_delay_ms: 20,  // Account for mechanical switching
            init_delay_ms: 250,     // Longer initialization
        }
    }

    /// Sets the write delay in microseconds.
    pub fn set_write_delay_us(&mut self, delay_us: u32) {
        self.write_delay_us = delay_us;
    }

    /// Sets the state change delay in milliseconds.
    pub fn set_state_change_delay_ms(&mut self, delay_ms: u32) {
        self.state_change_delay_ms = delay_ms;
    }

    /// Sets the initialization delay in milliseconds.
    pub fn set_init_delay_ms(&mut self, delay_ms: u32) {
        self.init_delay_ms = delay_ms;
    }
}

impl Default for QwiicRelayConfig {
    /// Creates a default configuration for a quad relay board (4 relays) with standard timing.
    fn default() -> Self {
        QwiicRelayConfig::new(4)
    }
}

/// Main interface for controlling a Qwiic Relay board.
pub struct QwiicRelay {
    dev: LinuxI2CDevice,
    /// The configuration for this relay board.
    pub config: QwiicRelayConfig,
}

type RelayDeviceStatus = Result<bool, RelayError>;
type VersionResult = Result<u8, RelayError>;

impl QwiicRelay {
    /// Creates a new QwiicRelay instance.
    ///
    /// # Arguments
    /// * `config` - Configuration for the relay board
    /// * `bus` - I2C bus path (e.g., "/dev/i2c-1")
    /// * `i2c_addr` - I2C address of the relay board
    ///
    /// # Returns
    /// A Result containing the new QwiicRelay instance or an I2C error.
    pub fn new(
        config: QwiicRelayConfig,
        bus: &str,
        i2c_addr: u16,
    ) -> Result<QwiicRelay, RelayError> {
        let dev = LinuxI2CDevice::new(bus, i2c_addr)?;
        Ok(QwiicRelay { dev, config })
    }

    /// Initializes the relay board.
    ///
    /// Waits for the relay board to set up using the configured initialization delay.
    pub fn init(&mut self) -> RelayResult<()> {
        // Wait for the QwiicRelay to set up
        thread::sleep(Duration::from_millis(self.config.init_delay_ms as u64));
        Ok(())
    }

    /// Turns on a specific relay.
    ///
    /// # Arguments
    /// * `relay_num` - Optional relay number (1-4). If None, operates on single relay boards.
    ///
    /// # Returns
    /// A Result indicating success or I2C error.
    pub fn set_relay_on(&mut self, relay_num: Option<u8>) -> RelayResult<()> {
        match self.config.verification.mode {
            VerificationMode::Disabled => self.set_relay_on_unverified(relay_num),
            _ => self.set_relay_on_verified(relay_num),
        }
    }

    /// Internal method to turn on a relay without verification.
    fn set_relay_on_unverified(&mut self, relay_num: Option<u8>) -> RelayResult<()> {
        match relay_num {
            Some(num) => {
                let read_command = 0x04 + num;
                let temp = self.dev.smbus_read_byte_data(read_command)?;

                if RelayStatus::from(temp) == RelayStatus::Off {
                    self.write_byte((Command::DualQuadToggleBase as u8) + num)?;
                }
                Ok(())
            }
            None => self.write_byte(RelayState::On as u8),
        }
    }

    /// Internal method to turn on a relay with state verification and retry logic.
    fn set_relay_on_verified(&mut self, relay_num: Option<u8>) -> RelayResult<()> {
        let start_time = Instant::now();
        let timeout = self.config.verification.timeout();
        let max_retries = self.config.verification.max_retries;
        let expected_state = true;

        for attempt in 0..=max_retries {
            // Check if timeout exceeded
            if start_time.elapsed() > timeout {
                return Err(RelayError::Timeout {
                    relay_num,
                    operation: "set_relay_on".to_string(),
                    duration_ms: timeout.as_millis() as u64,
                });
            }

            // Try to set the relay on
            self.set_relay_on_unverified(relay_num)?;

            // Wait for state to stabilize
            thread::sleep(self.config.verification.verification_delay());

            // Verify the state
            match self.get_relay_state(relay_num) {
                Ok(actual_state) if actual_state == expected_state => {
                    return Ok(());
                }
                Ok(actual_state) => {
                    // State mismatch
                    if attempt == max_retries {
                        // Final attempt failed
                        let error = RelayError::StateVerificationFailed {
                            relay_num,
                            expected: expected_state,
                            actual: actual_state,
                            attempts: attempt + 1,
                        };

                        // In lenient mode, we might allow the operation to succeed with a warning
                        if matches!(self.config.verification.mode, VerificationMode::Lenient) {
                            // In a real implementation, you might want to log this
                            // For now, we'll still return the error in lenient mode
                            // but you could modify this behavior
                            return Err(error);
                        } else {
                            return Err(error);
                        }
                    }
                    // Retry after delay
                    thread::sleep(self.config.verification.retry_delay());
                }
                Err(e) if attempt == max_retries => {
                    // I2C error on final attempt
                    return Err(e);
                }
                Err(_) => {
                    // I2C error, retry after delay
                    thread::sleep(self.config.verification.retry_delay());
                }
            }
        }

        // This should never be reached due to the loop structure
        unreachable!("Verification loop completed without returning")
    }

    /// Turns off a specific relay.
    ///
    /// # Arguments
    /// * `relay_num` - Optional relay number (1-4). If None, operates on single relay boards.
    ///
    /// # Returns
    /// A Result indicating success or I2C error.
    pub fn set_relay_off(&mut self, relay_num: Option<u8>) -> RelayResult<()> {
        match self.config.verification.mode {
            VerificationMode::Disabled => self.set_relay_off_unverified(relay_num),
            _ => self.set_relay_off_verified(relay_num),
        }
    }

    /// Internal method to turn off a relay without verification.
    fn set_relay_off_unverified(&mut self, relay_num: Option<u8>) -> RelayResult<()> {
        match relay_num {
            Some(num) => {
                let read_command = 0x04 + num;
                let temp = self.dev.smbus_read_byte_data(read_command)?;

                if RelayStatus::from(temp) == RelayStatus::On {
                    self.write_byte((Command::DualQuadToggleBase as u8) + num)?;
                }
                Ok(())
            }
            None => self.write_byte(RelayState::Off as u8),
        }
    }

    /// Internal method to turn off a relay with state verification and retry logic.
    fn set_relay_off_verified(&mut self, relay_num: Option<u8>) -> RelayResult<()> {
        let start_time = Instant::now();
        let timeout = self.config.verification.timeout();
        let max_retries = self.config.verification.max_retries;
        let expected_state = false;

        for attempt in 0..=max_retries {
            // Check if timeout exceeded
            if start_time.elapsed() > timeout {
                return Err(RelayError::Timeout {
                    relay_num,
                    operation: "set_relay_off".to_string(),
                    duration_ms: timeout.as_millis() as u64,
                });
            }

            // Try to set the relay off
            self.set_relay_off_unverified(relay_num)?;

            // Wait for state to stabilize
            thread::sleep(self.config.verification.verification_delay());

            // Verify the state
            match self.get_relay_state(relay_num) {
                Ok(actual_state) if actual_state == expected_state => {
                    return Ok(());
                }
                Ok(actual_state) => {
                    // State mismatch
                    if attempt == max_retries {
                        // Final attempt failed
                        let error = RelayError::StateVerificationFailed {
                            relay_num,
                            expected: expected_state,
                            actual: actual_state,
                            attempts: attempt + 1,
                        };

                        // In lenient mode, we might allow the operation to succeed with a warning
                        if matches!(self.config.verification.mode, VerificationMode::Lenient) {
                            // In a real implementation, you might want to log this
                            // For now, we'll still return the error in lenient mode
                            // but you could modify this behavior
                            return Err(error);
                        } else {
                            return Err(error);
                        }
                    }
                    // Retry after delay
                    thread::sleep(self.config.verification.retry_delay());
                }
                Err(e) if attempt == max_retries => {
                    // I2C error on final attempt
                    return Err(e);
                }
                Err(_) => {
                    // I2C error, retry after delay
                    thread::sleep(self.config.verification.retry_delay());
                }
            }
        }

        // This should never be reached due to the loop structure
        unreachable!("Verification loop completed without returning")
    }

    /// Gets the current state of a specific relay.
    ///
    /// # Arguments
    /// * `relay_num` - Optional relay number (1-4). If None, checks the first relay.
    ///
    /// # Returns
    /// A Result containing true if the relay is on, false if off, or an I2C error.
    pub fn get_relay_state(&mut self, relay_num: Option<u8>) -> RelayDeviceStatus {
        let read_command = match relay_num {
            Some(num) => 0x04 + num,
            None => 0x04,
        };
        
        let temp = self.dev.smbus_read_byte_data(read_command)?;
        Ok(RelayStatus::from(temp).into())
    }

    /// Turns on all relays on the board.
    pub fn set_all_relays_on(&mut self) -> RelayResult<()> {
        self.write_byte(Command::TurnAllOn as u8)
    }

    /// Turns off all relays on the board.
    pub fn set_all_relays_off(&mut self) -> RelayResult<()> {
        self.write_byte(Command::TurnAllOff as u8)
    }

    /// Gets the firmware version of the relay board.
    ///
    /// # Returns
    /// A Result containing the firmware version number or an I2C error.
    pub fn get_version(&mut self) -> VersionResult {
        let version = self
            .dev
            .smbus_read_byte_data(RelayState::SingleFirmwareVersion as u8)?;
        Ok(version)
    }

    /// Writes a single byte command to the relay board.
    ///
    /// # Arguments
    /// * `command` - The command byte to send
    ///
    /// # Returns
    /// A Result indicating success or I2C error.
    pub fn write_byte(&mut self, command: u8) -> RelayResult<()> {
        self.dev.smbus_write_byte(command)?;
        thread::sleep(Duration::new(0, self.config.write_delay_us * 1000));
        Ok(())
    }

    /// Updates the timing configuration at runtime.
    ///
    /// # Arguments
    /// * `config` - The new configuration with updated timing parameters
    pub fn update_config(&mut self, config: QwiicRelayConfig) {
        self.config = config;
    }

    /// Adjusts the write delay at runtime.
    ///
    /// # Arguments
    /// * `delay_us` - The new delay in microseconds
    pub fn set_write_delay(&mut self, delay_us: u32) {
        self.config.write_delay_us = delay_us;
    }

    /// Adjusts the state change delay at runtime.
    ///
    /// # Arguments
    /// * `delay_ms` - The new delay in milliseconds
    pub fn set_state_change_delay(&mut self, delay_ms: u32) {
        self.config.state_change_delay_ms = delay_ms;
    }

    /// Attempts to auto-detect optimal timing for the relay board.
    /// 
    /// This method tests different timing configurations and finds the fastest
    /// reliable settings. Returns true if optimization was successful.
    ///
    /// # Returns
    /// A Result containing true if timing was optimized, or an I2C error.
    pub fn auto_detect_timing(&mut self) -> Result<bool, LinuxI2CError> {
        // Save original config
        let original_config = self.config;
        
        // Test configurations from fastest to slowest
        let test_configs = [
            (5, 5),    // Very fast (solid state optimal)
            (10, 10),  // Standard
            (15, 15),  // Conservative
            (20, 20),  // Very conservative
        ];
        
        for (write_us, state_ms) in test_configs.iter() {
            self.config.write_delay_us = *write_us;
            self.config.state_change_delay_ms = *state_ms;
            
            // Test relay operations with current timing
            let mut success = true;
            
            // Test turning relay 1 on and off multiple times
            for _ in 0..3 {
                if let Err(_) = self.set_relay_on(Some(1)) {
                    success = false;
                    break;
                }
                
                thread::sleep(Duration::from_millis(self.config.state_change_delay_ms as u64));
                
                // Verify the relay is actually on
                match self.get_relay_state(Some(1)) {
                    Ok(state) if !state => {
                        success = false;
                        break;
                    }
                    Err(_) => {
                        success = false;
                        break;
                    }
                    _ => {}
                }
                
                if let Err(_) = self.set_relay_off(Some(1)) {
                    success = false;
                    break;
                }
                
                thread::sleep(Duration::from_millis(self.config.state_change_delay_ms as u64));
                
                // Verify the relay is actually off
                match self.get_relay_state(Some(1)) {
                    Ok(state) if state => {
                        success = false;
                        break;
                    }
                    Err(_) => {
                        success = false;
                        break;
                    }
                    _ => {}
                }
            }
            
            if success {
                // Found working configuration, add small safety margin
                self.config.write_delay_us = write_us + 2;
                self.config.state_change_delay_ms = state_ms + 2;
                
                // Ensure relay is off after testing
                let _ = self.set_relay_off(Some(1));
                
                return Ok(true);
            }
        }
        
        // Restore original config if all tests failed
        self.config = original_config;
        Ok(false)
    }

    /// Changes the I2C address of the relay board.
    /// 
    /// Note: This will permanently change the I2C address of the device.
    /// After changing the address, you'll need to create a new QwiicRelay instance
    /// with the new address.
    ///
    /// # Arguments
    /// * `new_address` - The new I2C address to set (must be between 0x07 and 0x78)
    ///
    /// # Returns
    /// A Result indicating success or I2C error.
    pub fn change_i2c_address(&mut self, new_address: u8) -> RelayResult<()> {
        // Validate address range (7-bit I2C addresses)
        if !(0x07..=0x78).contains(&new_address) {
            return Err(RelayError::InvalidConfiguration(
                format!("I2C address must be between 0x07 and 0x78, got 0x{:02X}", new_address)
            ));
        }

        // Command to change address: 0xC7 followed by new address
        const CHANGE_ADDRESS_COMMAND: u8 = 0xC7;
        
        // Send the change address command
        self.dev.smbus_write_byte_data(CHANGE_ADDRESS_COMMAND, new_address)?;
        
        // Wait for the device to process the address change
        thread::sleep(Duration::from_millis(100));
        
        Ok(())
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod basic_tests {
    use super::*;

    #[test]
    #[ignore] // Requires actual hardware to run
    fn test_relay_operations() {
        let config = QwiicRelayConfig::default();
        let mut qwiic_relay =
            QwiicRelay::new(config, "/dev/i2c-1", 0x08).expect("Could not init device");

        // Test firmware version
        let version = qwiic_relay
            .get_version()
            .expect("Failed to get firmware version");
        println!("Firmware Version: {}", version);

        // Test all relays on/off
        qwiic_relay
            .set_all_relays_off()
            .expect("Failed to turn all relays off");
        thread::sleep(Duration::from_millis(500));

        qwiic_relay
            .set_all_relays_on()
            .expect("Failed to turn all relays on");
        thread::sleep(Duration::from_millis(500));

        qwiic_relay
            .set_all_relays_off()
            .expect("Failed to turn all relays off");
        thread::sleep(Duration::from_millis(500));

        // Test individual relays
        for relay_num in 1..=4 {
            // Turn on
            qwiic_relay
                .set_relay_on(Some(relay_num))
                .expect(&format!("Failed to turn on relay {}", relay_num));
            thread::sleep(Duration::from_millis(250));

            // Verify state
            let state = qwiic_relay
                .get_relay_state(Some(relay_num))
                .expect(&format!("Failed to get state of relay {}", relay_num));
            assert!(state, "Relay {} should be on", relay_num);

            // Turn off
            qwiic_relay
                .set_relay_off(Some(relay_num))
                .expect(&format!("Failed to turn off relay {}", relay_num));
            thread::sleep(Duration::from_millis(250));

            // Verify state
            let state = qwiic_relay
                .get_relay_state(Some(relay_num))
                .expect(&format!("Failed to get state of relay {}", relay_num));
            assert!(!state, "Relay {} should be off", relay_num);
        }
    }

    #[test]
    fn test_config_creation() {
        let config = QwiicRelayConfig::new(2);
        assert_eq!(config.relay_count, 2);
        assert_eq!(config.write_delay_us, 10);
        assert_eq!(config.state_change_delay_ms, 10);
        assert_eq!(config.init_delay_ms, 200);

        let default_config = QwiicRelayConfig::default();
        assert_eq!(default_config.relay_count, 4);
        assert_eq!(default_config.write_delay_us, 10);
        assert_eq!(default_config.state_change_delay_ms, 10);
        assert_eq!(default_config.init_delay_ms, 200);
    }

    #[test]
    fn test_config_with_different_relay_counts() {
        let single = QwiicRelayConfig::new(1);
        assert_eq!(single.relay_count, 1);
        
        let dual = QwiicRelayConfig::new(2);
        assert_eq!(dual.relay_count, 2);
        
        let quad = QwiicRelayConfig::new(4);
        assert_eq!(quad.relay_count, 4);
    }

    #[test]
    fn test_addresses_enum_values() {
        assert_eq!(Addresses::SingleRelayDefault as u16, 0x18);
        assert_eq!(Addresses::SingleRelayJumperClosed as u16, 0x19);
        assert_eq!(Addresses::QuadRelayDefault as u16, 0x6D);
        assert_eq!(Addresses::QuadRelayJumperClosed as u16, 0x6C);
        assert_eq!(Addresses::DualSolidState as u16, 0x0A);
        assert_eq!(Addresses::DualSolidStateJumperClosed as u16, 0x0B);
        assert_eq!(Addresses::QuadSolidState as u16, 0x08);
        assert_eq!(Addresses::QuadSolidStateJumperClosed as u16, 0x09);
    }

    #[test]
    fn test_command_enum_values() {
        assert_eq!(Command::DualQuadToggleBase as u8, 0x00);
        assert_eq!(Command::ToggleRelayOne as u8, 0x01);
        assert_eq!(Command::ToggleRelayTwo as u8, 0x02);
        assert_eq!(Command::ToggleRelayThree as u8, 0x03);
        assert_eq!(Command::ToggleRelayFour as u8, 0x04);
        assert_eq!(Command::RelayOneStatus as u8, 0x05);
        assert_eq!(Command::RelayTwoStatus as u8, 0x06);
        assert_eq!(Command::RelayThreeStatus as u8, 0x07);
        assert_eq!(Command::RelayFourStatus as u8, 0x08);
        assert_eq!(Command::TurnAllOff as u8, 0x0A);
        assert_eq!(Command::TurnAllOn as u8, 0x0B);
        assert_eq!(Command::ToggleAll as u8, 0x0C);
    }

    #[test]
    fn test_relay_state_enum_values() {
        assert_eq!(RelayState::Off as u8, 0x00);
        assert_eq!(RelayState::On as u8, 0x01);
        assert_eq!(RelayState::SingleFirmwareVersion as u8, 0x04);
        assert_eq!(RelayState::SingleStatusVersion as u8, 0x05);
    }

    #[test]
    fn test_relay_status_enum_values() {
        assert_eq!(RelayStatus::Off as u8, 0);
        assert_eq!(RelayStatus::On as u8, 1);
    }

    #[test]
    fn test_relay_status_from_bool() {
        assert_eq!(RelayStatus::from(false), RelayStatus::Off);
        assert_eq!(RelayStatus::from(true), RelayStatus::On);
    }

    #[test]
    fn test_relay_status_to_bool() {
        assert_eq!(bool::from(RelayStatus::Off), false);
        assert_eq!(bool::from(RelayStatus::On), true);
    }

    #[test]
    fn test_relay_status_from_u8() {
        assert_eq!(RelayStatus::from(0u8), RelayStatus::Off);
        assert_eq!(RelayStatus::from(1u8), RelayStatus::On);
        assert_eq!(RelayStatus::from(255u8), RelayStatus::On);
        assert_eq!(RelayStatus::from(10u8), RelayStatus::On);
    }

    #[test]
    fn test_relay_status_to_u8() {
        assert_eq!(u8::from(RelayStatus::Off), 0);
        assert_eq!(u8::from(RelayStatus::On), 1);
    }

    #[test]
    fn test_relay_status_equality() {
        assert_eq!(RelayStatus::Off, RelayStatus::Off);
        assert_eq!(RelayStatus::On, RelayStatus::On);
        assert_ne!(RelayStatus::Off, RelayStatus::On);
    }

    #[test]
    fn test_config_clone() {
        let original = QwiicRelayConfig::new(3);
        let cloned = original.clone();
        assert_eq!(original.relay_count, cloned.relay_count);
    }

    #[test]
    fn test_config_copy() {
        let original = QwiicRelayConfig::new(2);
        let copied = original;
        assert_eq!(copied.relay_count, 2);
    }

    #[test]
    #[ignore] // Requires actual hardware and permanently changes device address
    fn test_change_i2c_address() {
        let config = QwiicRelayConfig::default();
        let mut qwiic_relay =
            QwiicRelay::new(config, "/dev/i2c-1", 0x08).expect("Could not init device");

        // Test changing to a new address
        let new_address = 0x09;
        qwiic_relay
            .change_i2c_address(new_address)
            .expect("Failed to change I2C address");
        
        // Note: After this, you would need to create a new QwiicRelay instance
        // with the new address to continue communicating with the device
        println!("Address changed to 0x{:02X}", new_address);
    }

    #[test]
    fn test_config_with_timing() {
        let config = QwiicRelayConfig::with_timing(2, 15, 25, 300);
        assert_eq!(config.relay_count, 2);
        assert_eq!(config.write_delay_us, 15);
        assert_eq!(config.state_change_delay_ms, 25);
        assert_eq!(config.init_delay_ms, 300);
    }

    #[test]
    fn test_config_for_solid_state() {
        let config = QwiicRelayConfig::for_solid_state(4);
        assert_eq!(config.relay_count, 4);
        assert_eq!(config.write_delay_us, 5);
        assert_eq!(config.state_change_delay_ms, 5);
        assert_eq!(config.init_delay_ms, 100);
    }

    #[test]
    fn test_config_for_mechanical() {
        let config = QwiicRelayConfig::for_mechanical(2);
        assert_eq!(config.relay_count, 2);
        assert_eq!(config.write_delay_us, 15);
        assert_eq!(config.state_change_delay_ms, 20);
        assert_eq!(config.init_delay_ms, 250);
    }

    #[test]
    fn test_config_setters() {
        let mut config = QwiicRelayConfig::new(1);
        
        config.set_write_delay_us(25);
        assert_eq!(config.write_delay_us, 25);
        
        config.set_state_change_delay_ms(30);
        assert_eq!(config.state_change_delay_ms, 30);
        
        config.set_init_delay_ms(500);
        assert_eq!(config.init_delay_ms, 500);
    }

    #[test]
    #[ignore] // Requires actual hardware
    fn test_runtime_timing_adjustment() {
        let mut config = QwiicRelayConfig::default();
        let mut qwiic_relay =
            QwiicRelay::new(config, "/dev/i2c-1", 0x08).expect("Could not init device");
        
        // Test setting different delays at runtime
        qwiic_relay.set_write_delay(20);
        assert_eq!(qwiic_relay.config.write_delay_us, 20);
        
        qwiic_relay.set_state_change_delay(15);
        assert_eq!(qwiic_relay.config.state_change_delay_ms, 15);
        
        // Test updating entire config
        config.write_delay_us = 30;
        config.state_change_delay_ms = 25;
        qwiic_relay.update_config(config);
        assert_eq!(qwiic_relay.config.write_delay_us, 30);
        assert_eq!(qwiic_relay.config.state_change_delay_ms, 25);
    }

    #[test]
    #[ignore] // Requires actual hardware
    fn test_auto_detect_timing() {
        let config = QwiicRelayConfig::default();
        let mut qwiic_relay =
            QwiicRelay::new(config, "/dev/i2c-1", 0x08).expect("Could not init device");
        
        // Initialize the relay board
        qwiic_relay.init().expect("Failed to initialize");
        
        // Try to auto-detect optimal timing
        match qwiic_relay.auto_detect_timing() {
            Ok(optimized) => {
                if optimized {
                    println!("Timing optimized: write_delay={}μs, state_change_delay={}ms",
                        qwiic_relay.config.write_delay_us,
                        qwiic_relay.config.state_change_delay_ms);
                } else {
                    println!("Could not optimize timing, using defaults");
                }
            }
            Err(e) => {
                println!("Auto-detect timing failed: {:?}", e);
            }
        }
    }
}
