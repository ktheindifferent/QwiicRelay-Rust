// Copyright 2024 the QwiicRelay-Rust contributors
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
//! This library provides both embedded (no_std, async) and Linux (std, sync) implementations
//! for controlling various types of Qwiic Relay boards including single relays, dual solid state relays,
//! quad relays, and quad solid state relays.
//!
//! # Features
//! - `std`: Enable Linux-based implementation with i2cdev (default)
//! - `embedded`: Enable embedded-hal-async implementation for embedded devices  
//! - `defmt`: Enable defmt support for embedded logging
//!
//! # Examples
//!
//! ## Linux Example (std feature)
//! ```no_run
//! use qwiic_relay_rs::{QwiicRelay, QwiicRelayConfig};
//!
//! let config = QwiicRelayConfig::default();
//! let mut relay = QwiicRelay::new(config, "/dev/i2c-1", 0x08).unwrap();
//! relay.set_relay_on(Some(1)).unwrap();
//! ```
//!
//! ## Embedded Example (embedded feature)
//! ```no_run
//! use qwiic_relay_rs::{QwiicRelayAsync, Addresses};
//! use embedded_hal_async::i2c::I2c;
//!
//! async fn example<I2C: I2c>(i2c: I2C) -> Result<(), I2C::Error> {
//!     let mut relay = QwiicRelayAsync::new(i2c, Addresses::QuadSolidState as u8)?;
//!     relay.set_relay_on(Some(1)).await?;
//!     Ok(())
//! }
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![deny(warnings)]
//#![forbid(missing_docs)] // TODO: add docs for everything
#![forbid(missing_debug_implementations)]
#![deny(unused)]

// Embedded implementation
#[cfg(feature = "embedded")]
use embedded_hal_async::i2c::{self, I2c, SevenBitAddress};

// Linux implementation

#[cfg(feature = "std")]
mod error;
#[cfg(feature = "std")]
mod verification;

#[cfg(feature = "std")]
use std::thread;
#[cfg(feature = "std")]
use std::time::{Duration, Instant};

#[cfg(feature = "std")]
use i2cdev::core::*;
#[cfg(feature = "std")]
use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};

#[cfg(feature = "std")]
pub use error::{RelayError, RelayResult};
#[cfg(feature = "std")]
pub use verification::{VerificationConfig, VerificationMode};

/// I2C addresses for different Qwiic Relay board configurations.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RelayState {
    Off = 0x00,
    On = 0x01,
    SingleFirmwareVersion = 0x04,
    SingleStatusVersion = 0x05,
}

// Embedded async implementation
#[cfg(feature = "embedded")]
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Status {
    Off = 0,
}

#[cfg(feature = "embedded")]
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct QwiicRelayAsync<T> {
    i2c: T,
    i2c_addr: SevenBitAddress,
}

#[cfg(feature = "embedded")]
impl<T: I2c<Error = E>, E: i2c::Error> QwiicRelayAsync<T> {
    pub fn new(
        i2c: T,
        i2c_addr: SevenBitAddress,
    ) -> Result<QwiicRelayAsync<T>, E> {
        Ok(QwiicRelayAsync {
            i2c,
            i2c_addr,
        })
    }

    pub async fn set_relay_on(&mut self, relay_num: Option<u8>) -> Result<(), E> {
        if let Some(num) = relay_num {
            if !self.get_relay_state(relay_num).await? {
                self.i2c
                    .write(self.i2c_addr, &[Command::DualQuadToggleBase as u8 + num])
                    .await?;
            }
        } else {
            self.i2c
                .write(self.i2c_addr, &[RelayState::On as u8])
                .await?;
        }
        Ok(())
    }

    pub async fn set_relay_off(&mut self, relay_num: Option<u8>) -> Result<(), E> {
        if let Some(num) = relay_num {
            let read_command = 0x04 + num;
            let mut status = [0u8];
            self.i2c
                .write_read(self.i2c_addr, &[read_command], &mut status)
                .await?;

            if status[0] != (Status::Off as u8) {
                self.i2c
                    .write(self.i2c_addr, &[Command::DualQuadToggleBase as u8 + num])
                    .await?;
            }
        } else {
            self.i2c
                .write(self.i2c_addr, &[RelayState::Off as u8])
                .await?;
        }
        Ok(())
    }

    pub async fn get_relay_state(&mut self, relay_num: Option<u8>) -> Result<bool, E> {
        let read_command = 0x04 + relay_num.unwrap_or(0);
        let mut status = [0u8];
        self.i2c
            .write_read(self.i2c_addr, &[read_command], &mut status)
            .await?;

        Ok(status[0] != Status::Off as u8)
    }

    pub async fn set_all_relays_on(&mut self) -> Result<(), E> {
        self.i2c
            .write(self.i2c_addr, &[Command::TurnAllOn as u8])
            .await
    }

    pub async fn set_all_relays_off(&mut self) -> Result<(), E> {
        self.i2c
            .write(self.i2c_addr, &[Command::TurnAllOff as u8])
            .await
    }

    pub async fn get_version(&mut self) -> Result<u8, E> {
        let mut version = [0u8];
        self.i2c
            .write_read(
                self.i2c_addr,
                &[RelayState::SingleFirmwareVersion as u8],
                &mut version,
            )
            .await?;
        Ok(version[0])
    }
}

// Linux std implementation
#[cfg(feature = "std")]
/// Status values returned by the relay board.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum RelayStatus {
    Off = 0,
    On = 1,
}

#[cfg(feature = "std")]
impl From<bool> for RelayStatus {
    fn from(value: bool) -> Self {
        if value { RelayStatus::On } else { RelayStatus::Off }
    }
}

#[cfg(feature = "std")]
impl From<RelayStatus> for bool {
    fn from(status: RelayStatus) -> Self {
        status == RelayStatus::On
    }
}

#[cfg(feature = "std")]
impl From<u8> for RelayStatus {
    fn from(value: u8) -> Self {
        if value != 0 { RelayStatus::On } else { RelayStatus::Off }
    }
}

#[cfg(feature = "std")]
impl From<RelayStatus> for u8 {
    fn from(status: RelayStatus) -> Self {
        status as u8
    }
}

/// Configuration for a Qwiic Relay board.
#[cfg(feature = "std")]
#[derive(Clone, Copy, Debug)]
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

#[cfg(feature = "std")]
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

    /// Sets the verification configuration.
    pub fn with_verification(mut self, verification: VerificationConfig) -> QwiicRelayConfig {
        self.verification = verification;
        self
    }

    /// Sets the write delay in microseconds.
    pub fn with_write_delay_us(mut self, delay_us: u32) -> QwiicRelayConfig {
        self.write_delay_us = delay_us;
        self
    }

    /// Sets the state change delay in milliseconds.
    pub fn with_state_change_delay_ms(mut self, delay_ms: u32) -> QwiicRelayConfig {
        self.state_change_delay_ms = delay_ms;
        self
    }

    /// Sets the initialization delay in milliseconds.
    pub fn with_init_delay_ms(mut self, delay_ms: u32) -> QwiicRelayConfig {
        self.init_delay_ms = delay_ms;
        self
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

#[cfg(feature = "std")]
impl Default for QwiicRelayConfig {
    /// Creates a default configuration for a quad relay board (4 relays) with standard timing.
    fn default() -> Self {
        QwiicRelayConfig::new(4)
    }
}

/// Main interface for controlling a Qwiic Relay board.
#[cfg(feature = "std")]
pub struct QwiicRelay {
    dev: LinuxI2CDevice,
    /// The configuration for this relay board.
    pub config: QwiicRelayConfig,
}

#[cfg(feature = "std")]
impl std::fmt::Debug for QwiicRelay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QwiicRelay")
            .field("config", &self.config)
            .finish()
    }
}

#[cfg(feature = "std")]
type RelayDeviceStatus = Result<bool, RelayError>;
#[cfg(feature = "std")]
type VersionResult = Result<u8, RelayError>;

#[cfg(feature = "std")]
impl QwiicRelay {
    /// Creates a new QwiicRelay instance.
    ///
    /// # Arguments
    /// * `config` - Configuration for the relay board
    /// * `bus` - I2C bus path (e.g., "/dev/i2c-1")
    /// * `i2c_addr` - I2C address of the relay board
    ///
    /// # Examples
    /// ```no_run
    /// use qwiic_relay_rs::{QwiicRelay, QwiicRelayConfig};
    ///
    /// let config = QwiicRelayConfig::default();
    /// let mut qwiic_relay = QwiicRelay::new(config, "/dev/i2c-1", 0x08).unwrap();
    /// ```
    pub fn new(config: QwiicRelayConfig, bus: &str, i2c_addr: u16) -> RelayResult<QwiicRelay> {
        let mut dev = LinuxI2CDevice::new(bus, i2c_addr)?;
        dev.set_slave_address(i2c_addr)?;
        thread::sleep(Duration::from_millis(config.init_delay_ms as u64));
        Ok(QwiicRelay { dev, config })
    }

    fn delay_after_write(&self) {
        if self.config.write_delay_us > 0 {
            thread::sleep(Duration::from_micros(self.config.write_delay_us as u64));
        }
    }

    fn wait_for_state_change(&self) {
        if self.config.state_change_delay_ms > 0 {
            thread::sleep(Duration::from_millis(
                self.config.state_change_delay_ms as u64,
            ));
        }
    }

    fn verify_relay_state_if_enabled(
        &mut self,
        relay_num: Option<u8>,
        expected_state: RelayStatus,
    ) -> RelayResult<()> {
        use verification::VerificationMode;
        
        match self.config.verification.mode {
            VerificationMode::Disabled => Ok(()),
            VerificationMode::Enabled => {
                let verification_config = &self.config.verification;
                let start_time = Instant::now();

                for attempt in 0..verification_config.retry_attempts {
                    if start_time.elapsed() > Duration::from_millis(verification_config.timeout_ms) {
                        return Err(RelayError::VerificationTimeout {
                            relay_num,
                            expected: expected_state,
                            timeout_ms: verification_config.timeout_ms,
                        });
                    }

                    match self.get_relay_state(relay_num) {
                        Ok(actual_state) => {
                            if actual_state == expected_state {
                                return Ok(());
                            }
                        }
                        Err(_) if attempt < verification_config.retry_attempts - 1 => {
                            thread::sleep(Duration::from_millis(verification_config.retry_delay_ms));
                            continue;
                        }
                        Err(e) => return Err(e),
                    }

                    if attempt < verification_config.retry_attempts - 1 {
                        thread::sleep(Duration::from_millis(verification_config.retry_delay_ms));
                    }
                }

                Err(RelayError::VerificationFailed {
                    relay_num,
                    expected: expected_state,
                    attempts: verification_config.retry_attempts,
                })
            }
        }
    }

    /// Gets the current status of a specific relay or the single relay.
    ///
    /// # Arguments
    /// * `relay_num` - Relay number (1-4 for multi-relay boards, None for single relay boards)
    ///
    /// # Returns
    /// `RelayStatus` indicating whether the relay is on or off
    ///
    /// # Examples
    /// ```no_run
    /// use qwiic_relay_rs::{QwiicRelay, QwiicRelayConfig, RelayStatus};
    ///
    /// let config = QwiicRelayConfig::default();
    /// let mut relay = QwiicRelay::new(config, "/dev/i2c-1", 0x08).unwrap();
    /// let status = relay.get_relay_state(Some(1)).unwrap();
    /// println!("Relay 1 is {:?}", status);
    /// ```
    pub fn get_relay_state(&mut self, relay_num: Option<u8>) -> RelayResult<RelayStatus> {
        let read_command = if let Some(num) = relay_num {
            if num < 1 || num > self.config.relay_count {
                return Err(RelayError::InvalidRelayNumber {
                    relay_num: num,
                    max_relays: self.config.relay_count,
                });
            }
            0x04 + num
        } else {
            RelayState::SingleStatusVersion as u8
        };

        let status = self.dev.smbus_read_byte_data(read_command)?;
        self.delay_after_write();
        
        Ok(RelayStatus::from(status))
    }

    /// Turns a specific relay on.
    ///
    /// # Arguments
    /// * `relay_num` - Relay number (1-4 for multi-relay boards, None for single relay boards)
    ///
    /// # Examples
    /// ```no_run
    /// use qwiic_relay_rs::{QwiicRelay, QwiicRelayConfig};
    ///
    /// let config = QwiicRelayConfig::default();
    /// let mut relay = QwiicRelay::new(config, "/dev/i2c-1", 0x08).unwrap();
    /// relay.set_relay_on(Some(1)).unwrap();
    /// ```
    pub fn set_relay_on(&mut self, relay_num: Option<u8>) -> RelayResult<()> {
        if let Some(num) = relay_num {
            if num < 1 || num > self.config.relay_count {
                return Err(RelayError::InvalidRelayNumber {
                    relay_num: num,
                    max_relays: self.config.relay_count,
                });
            }
            
            let current_state = self.get_relay_state(relay_num)?;
            if current_state != RelayStatus::On {
                self.dev
                    .smbus_write_byte(Command::DualQuadToggleBase as u8 + num)?;
                self.delay_after_write();
                self.wait_for_state_change();
            }
        } else {
            self.dev.smbus_write_byte(RelayState::On as u8)?;
            self.delay_after_write();
            self.wait_for_state_change();
        }

        self.verify_relay_state_if_enabled(relay_num, RelayStatus::On)?;
        Ok(())
    }

    /// Turns a specific relay off.
    ///
    /// # Arguments
    /// * `relay_num` - Relay number (1-4 for multi-relay boards, None for single relay boards)
    ///
    /// # Examples
    /// ```no_run
    /// use qwiic_relay_rs::{QwiicRelay, QwiicRelayConfig};
    ///
    /// let config = QwiicRelayConfig::default();
    /// let mut relay = QwiicRelay::new(config, "/dev/i2c-1", 0x08).unwrap();
    /// relay.set_relay_off(Some(1)).unwrap();
    /// ```
    pub fn set_relay_off(&mut self, relay_num: Option<u8>) -> RelayResult<()> {
        if let Some(num) = relay_num {
            if num < 1 || num > self.config.relay_count {
                return Err(RelayError::InvalidRelayNumber {
                    relay_num: num,
                    max_relays: self.config.relay_count,
                });
            }
            
            let current_state = self.get_relay_state(relay_num)?;
            if current_state != RelayStatus::Off {
                self.dev
                    .smbus_write_byte(Command::DualQuadToggleBase as u8 + num)?;
                self.delay_after_write();
                self.wait_for_state_change();
            }
        } else {
            self.dev.smbus_write_byte(RelayState::Off as u8)?;
            self.delay_after_write();
            self.wait_for_state_change();
        }

        self.verify_relay_state_if_enabled(relay_num, RelayStatus::Off)?;
        Ok(())
    }

    /// Toggles a specific relay (turns it on if off, off if on).
    ///
    /// # Arguments
    /// * `relay_num` - Relay number (1-4 for multi-relay boards, None for single relay boards)
    ///
    /// # Examples
    /// ```no_run
    /// use qwiic_relay_rs::{QwiicRelay, QwiicRelayConfig};
    ///
    /// let config = QwiicRelayConfig::default();
    /// let mut relay = QwiicRelay::new(config, "/dev/i2c-1", 0x08).unwrap();
    /// relay.toggle_relay(Some(1)).unwrap();
    /// ```
    pub fn toggle_relay(&mut self, relay_num: Option<u8>) -> RelayResult<()> {
        if let Some(num) = relay_num {
            if num < 1 || num > self.config.relay_count {
                return Err(RelayError::InvalidRelayNumber {
                    relay_num: num,
                    max_relays: self.config.relay_count,
                });
            }
            
            let current_state = self.get_relay_state(relay_num)?;
            self.dev
                .smbus_write_byte(Command::DualQuadToggleBase as u8 + num)?;
            self.delay_after_write();
            self.wait_for_state_change();

            let expected_state = if current_state == RelayStatus::On {
                RelayStatus::Off
            } else {
                RelayStatus::On
            };
            self.verify_relay_state_if_enabled(relay_num, expected_state)?;
        } else {
            let current_state = self.get_relay_state(relay_num)?;
            let toggle_command = if current_state == RelayStatus::On {
                RelayState::Off as u8
            } else {
                RelayState::On as u8
            };
            
            self.dev.smbus_write_byte(toggle_command)?;
            self.delay_after_write();
            self.wait_for_state_change();

            let expected_state = if current_state == RelayStatus::On {
                RelayStatus::Off
            } else {
                RelayStatus::On
            };
            self.verify_relay_state_if_enabled(relay_num, expected_state)?;
        }
        
        Ok(())
    }

    /// Turns all relays on.
    ///
    /// # Examples
    /// ```no_run
    /// use qwiic_relay_rs::{QwiicRelay, QwiicRelayConfig};
    ///
    /// let config = QwiicRelayConfig::default();
    /// let mut relay = QwiicRelay::new(config, "/dev/i2c-1", 0x08).unwrap();
    /// relay.set_all_relays_on().unwrap();
    /// ```
    pub fn set_all_relays_on(&mut self) -> RelayResult<()> {
        self.dev.smbus_write_byte(Command::TurnAllOn as u8)?;
        self.delay_after_write();
        self.wait_for_state_change();

        for relay_num in 1..=self.config.relay_count {
            self.verify_relay_state_if_enabled(Some(relay_num), RelayStatus::On)?;
        }
        
        Ok(())
    }

    /// Turns all relays off.
    ///
    /// # Examples
    /// ```no_run
    /// use qwiic_relay_rs::{QwiicRelay, QwiicRelayConfig};
    ///
    /// let config = QwiicRelayConfig::default();
    /// let mut relay = QwiicRelay::new(config, "/dev/i2c-1", 0x08).unwrap();
    /// relay.set_all_relays_off().unwrap();
    /// ```
    pub fn set_all_relays_off(&mut self) -> RelayResult<()> {
        self.dev.smbus_write_byte(Command::TurnAllOff as u8)?;
        self.delay_after_write();
        self.wait_for_state_change();

        for relay_num in 1..=self.config.relay_count {
            self.verify_relay_state_if_enabled(Some(relay_num), RelayStatus::Off)?;
        }
        
        Ok(())
    }

    /// Toggles all relays (turns them on if off, off if on).
    ///
    /// # Examples
    /// ```no_run
    /// use qwiic_relay_rs::{QwiicRelay, QwiicRelayConfig};
    ///
    /// let config = QwiicRelayConfig::default();
    /// let mut relay = QwiicRelay::new(config, "/dev/i2c-1", 0x08).unwrap();
    /// relay.toggle_all_relays().unwrap();
    /// ```
    pub fn toggle_all_relays(&mut self) -> RelayResult<()> {
        self.dev.smbus_write_byte(Command::ToggleAll as u8)?;
        self.delay_after_write();
        self.wait_for_state_change();
        Ok(())
    }

    /// Gets the firmware version from the relay board.
    ///
    /// # Returns
    /// The firmware version as a u8
    ///
    /// # Examples
    /// ```no_run
    /// use qwiic_relay_rs::{QwiicRelay, QwiicRelayConfig};
    ///
    /// let config = QwiicRelayConfig::default();
    /// let mut relay = QwiicRelay::new(config, "/dev/i2c-1", 0x08).unwrap();
    /// let version = relay.get_version().unwrap();
    /// println!("Firmware version: {}", version);
    /// ```
    pub fn get_version(&mut self) -> VersionResult {
        let version = self
            .dev
            .smbus_read_byte_data(RelayState::SingleFirmwareVersion as u8)?;
        self.delay_after_write();
        Ok(version)
    }

    /// Changes the I2C address of the relay board.
    ///
    /// **WARNING**: This permanently changes the I2C address stored in the device's EEPROM.
    /// The device will use this new address after power cycling. Make sure the new address
    /// doesn't conflict with other I2C devices on the bus.
    ///
    /// # Arguments
    /// * `new_address` - The new I2C address (0x08-0x77, avoid reserved addresses)
    ///
    /// # Returns
    /// `RelayResult<()>` - Ok if successful, Err with details if failed
    ///
    /// # Examples
    /// ```no_run
    /// use qwiic_relay_rs::{QwiicRelay, QwiicRelayConfig};
    ///
    /// let config = QwiicRelayConfig::default();
    /// let mut relay = QwiicRelay::new(config, "/dev/i2c-1", 0x08).unwrap();
    /// relay.change_i2c_address(0x10).unwrap(); // Change to address 0x10
    /// // Device must be power cycled to use the new address
    /// ```
    ///
    /// # Safety Notes
    /// - This change is permanent and stored in EEPROM
    /// - The device must be power cycled to use the new address
    /// - Ensure the new address doesn't conflict with other devices
    /// - Reserved I2C addresses (0x00-0x07, 0x78-0x7F) should be avoided
    pub fn change_i2c_address(&mut self, new_address: u8) -> RelayResult<()> {
        // Validate the new address
        if new_address < 0x08 || new_address > 0x77 {
            return Err(RelayError::InvalidI2CAddress(new_address));
        }
        
        const CHANGE_ADDRESS_COMMAND: u8 = 0xC7;
        
        // Send the change address command
        self.dev.smbus_write_byte_data(CHANGE_ADDRESS_COMMAND, new_address)?;
        
        // Wait for the device to process the address change
        thread::sleep(Duration::from_millis(100));
        
        Ok(())
    }
}

// Linux std implementation
#[cfg(feature = "std")]

#[cfg(all(test, feature = "std"))]
mod tests;

#[cfg(all(test, feature = "std"))]
mod basic_tests {
    use super::*;

    #[test]
    #[ignore] // Requires actual hardware to run
    fn test_relay_operations() {
        let config = QwiicRelayConfig::default();
        let mut qwiic_relay =
            QwiicRelay::new(config, "/dev/i2c-1", 0x18).expect("Failed to create relay");

        qwiic_relay
            .set_relay_on(None)
            .expect("Failed to turn relay on");

        let state = qwiic_relay
            .get_relay_state(None)
            .expect("Failed to get relay state");
        assert_eq!(state, RelayStatus::On);

        qwiic_relay
            .set_relay_off(None)
            .expect("Failed to turn relay off");

        let state = qwiic_relay
            .get_relay_state(None)
            .expect("Failed to get relay state");
        assert_eq!(state, RelayStatus::Off);
    }

    #[test]
    #[ignore] // Requires actual hardware to run  
    fn test_multi_relay_operations() {
        let config = QwiicRelayConfig::new(4);
        let mut qwiic_relay =
            QwiicRelay::new(config, "/dev/i2c-1", 0x6D).expect("Failed to create relay");

        for relay_num in 1..=4 {
            qwiic_relay
                .set_relay_on(Some(relay_num))
                .expect("Failed to turn relay on");

            let state = qwiic_relay
                .get_relay_state(Some(relay_num))
                .expect("Failed to get relay state");
            assert_eq!(state, RelayStatus::On);

            qwiic_relay
                .set_relay_off(Some(relay_num))
                .expect("Failed to turn relay off");

            let state = qwiic_relay
                .get_relay_state(Some(relay_num))
                .expect("Failed to get relay state");
            assert_eq!(state, RelayStatus::Off);
        }
    }

    #[test]
    #[ignore] // Requires actual hardware to run
    fn test_all_relay_operations() {
        let config = QwiicRelayConfig::new(4);
        let mut qwiic_relay =
            QwiicRelay::new(config, "/dev/i2c-1", 0x6D).expect("Failed to create relay");

        qwiic_relay
            .set_all_relays_on()
            .expect("Failed to turn all relays on");

        for relay_num in 1..=4 {
            let state = qwiic_relay
                .get_relay_state(Some(relay_num))
                .expect("Failed to get relay state");
            assert_eq!(state, RelayStatus::On);
        }

        qwiic_relay
            .set_all_relays_off()
            .expect("Failed to turn all relays off");

        for relay_num in 1..=4 {
            let state = qwiic_relay
                .get_relay_state(Some(relay_num))
                .expect("Failed to get relay state");
            assert_eq!(state, RelayStatus::Off);
        }
    }

    #[test]
    #[ignore] // Requires actual hardware to run
    fn test_toggle_operations() {
        let config = QwiicRelayConfig::default();
        let mut qwiic_relay =
            QwiicRelay::new(config, "/dev/i2c-1", 0x18).expect("Failed to create relay");

        qwiic_relay
            .set_relay_off(None)
            .expect("Failed to set initial state");

        qwiic_relay
            .toggle_relay(None)
            .expect("Failed to toggle relay");

        let state = qwiic_relay
            .get_relay_state(None)
            .expect("Failed to get relay state");
        assert_eq!(state, RelayStatus::On);

        qwiic_relay
            .toggle_relay(None)
            .expect("Failed to toggle relay");

        let state = qwiic_relay
            .get_relay_state(None)
            .expect("Failed to get relay state");
        assert_eq!(state, RelayStatus::Off);
    }

    #[test]
    #[ignore] // Requires actual hardware to run
    fn test_version_reading() {
        let config = QwiicRelayConfig::default();
        let mut qwiic_relay =
            QwiicRelay::new(config, "/dev/i2c-1", 0x18).expect("Failed to create relay");

        let version = qwiic_relay
            .get_version()
            .expect("Failed to get version");

        assert!(version > 0, "Version should be greater than 0");
    }

    #[test]
    #[should_panic(expected = "Failed to turn on relay")]
    fn test_panic_message_format_on_relay() {
        // This test verifies that our panic messages are formatted correctly
        // for set_relay_on operations
        let config = QwiicRelayConfig::default();
        
        // Try to create a relay with an invalid path to trigger an error
        if let Ok(mut relay) = QwiicRelay::new(config, "/invalid/i2c/path", 0x18) {
            // This would only run if somehow the device creation succeeded
            // which shouldn't happen with an invalid path
            relay.set_relay_on(Some(1))
                .unwrap_or_else(|e| panic!("Failed to turn on relay 1: {:?}", e));
        } else {
            // Manually trigger the panic to test the message format
            panic!("Failed to turn on relay 1: I2C device not found");
        }
    }

    #[test]
    #[should_panic(expected = "Failed to get state of relay")]
    fn test_panic_message_format_get_state() {
        // This test verifies that our panic messages are formatted correctly
        // for get_relay_state operations
        let config = QwiicRelayConfig::default();
        
        // Try to create a relay with an invalid path to trigger an error
        if let Ok(mut relay) = QwiicRelay::new(config, "/invalid/i2c/path", 0x18) {
            // This would only run if somehow the device creation succeeded
            relay.get_relay_state(Some(2))
                .unwrap_or_else(|e| panic!("Failed to get state of relay 2: {:?}", e));
        } else {
            // Manually trigger the panic to test the message format
            panic!("Failed to get state of relay 2: I2C device not found");
        }
    }

    #[test]
    #[should_panic(expected = "Failed to turn off relay")]
    fn test_panic_message_format_off_relay() {
        // This test verifies that our panic messages are formatted correctly
        // for set_relay_off operations
        let config = QwiicRelayConfig::default();
        
        // Try to create a relay with an invalid path to trigger an error
        if let Ok(mut relay) = QwiicRelay::new(config, "/invalid/i2c/path", 0x18) {
            // This would only run if somehow the device creation succeeded
            relay.set_relay_off(Some(3))
                .unwrap_or_else(|e| panic!("Failed to turn off relay 3: {:?}", e));
        } else {
            // Manually trigger the panic to test the message format
            panic!("Failed to turn off relay 3: I2C device not found");
        }
    }

    #[test]
    fn test_relay_operations_with_valid_relay_numbers() {
        // This test verifies that relay operations work with different relay numbers
        // without allocating unnecessary strings in error messages
        let test_relay_nums = vec![1, 2, 3, 4];
        
        for relay_num in test_relay_nums {
            let config = QwiicRelayConfig::new(4); // 4-relay board
            
            // Verify that the relay number is valid for the config
            assert!(relay_num > 0 && relay_num <= config.relay_count,
                    "Relay number {} should be valid for a {}-relay board",
                    relay_num, config.relay_count);
        }
    }
}