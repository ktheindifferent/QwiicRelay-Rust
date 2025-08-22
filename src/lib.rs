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

use std::thread;
use std::time::Duration;

use i2cdev::core::*;
use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};

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
#[derive(Copy, Clone)]
pub enum Status {
    Off = 0,
}

/// Configuration for a Qwiic Relay board.
#[derive(Clone, Copy)]
pub struct QwiicRelayConfig {
    /// Number of relays on the board (1, 2, or 4).
    pub relay_count: u8,
}

impl QwiicRelayConfig {
    /// Creates a new configuration with the specified number of relays.
    ///
    /// # Arguments
    /// * `relay_count` - Number of relays on the board (typically 1, 2, or 4)
    pub fn new(relay_count: u8) -> QwiicRelayConfig {
        QwiicRelayConfig { relay_count }
    }
}

impl Default for QwiicRelayConfig {
    /// Creates a default configuration for a quad relay board (4 relays).
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

type RelayDeviceStatus = Result<bool, LinuxI2CError>;
type RelayResult = Result<(), LinuxI2CError>;
type VersionResult = Result<u8, LinuxI2CError>;

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
    ) -> Result<QwiicRelay, LinuxI2CError> {
        let dev = LinuxI2CDevice::new(bus, i2c_addr)?;
        Ok(QwiicRelay { dev, config })
    }

    /// Initializes the relay board.
    ///
    /// Waits 200ms for the relay board to set up.
    pub fn init(&mut self) -> RelayResult {
        // Wait for the QwiicRelay to set up
        thread::sleep(Duration::from_millis(200));
        Ok(())
    }

    /// Turns on a specific relay.
    ///
    /// # Arguments
    /// * `relay_num` - Optional relay number (1-4). If None, operates on single relay boards.
    ///
    /// # Returns
    /// A Result indicating success or I2C error.
    pub fn set_relay_on(&mut self, relay_num: Option<u8>) -> RelayResult {
        match relay_num {
            Some(num) => {
                let read_command = 0x04 + num;
                let temp = self.dev.smbus_read_byte_data(read_command)?;

                if temp == (Status::Off as u8) {
                    self.write_byte((Command::DualQuadToggleBase as u8) + num)?;
                }
                Ok(())
            }
            None => self.write_byte(RelayState::On as u8),
        }
    }

    /// Turns off a specific relay.
    ///
    /// # Arguments
    /// * `relay_num` - Optional relay number (1-4). If None, operates on single relay boards.
    ///
    /// # Returns
    /// A Result indicating success or I2C error.
    pub fn set_relay_off(&mut self, relay_num: Option<u8>) -> RelayResult {
        match relay_num {
            Some(num) => {
                let read_command = 0x04 + num;
                let temp = self.dev.smbus_read_byte_data(read_command)?;

                if temp != (Status::Off as u8) {
                    self.write_byte((Command::DualQuadToggleBase as u8) + num)?;
                }
                Ok(())
            }
            None => self.write_byte(RelayState::Off as u8),
        }
    }

    /// Reads the status of a relay at the specified register offset.
    ///
    /// # Arguments
    /// * `relay_offset` - The relay register offset (0 for relay 1, 1 for relay 2, etc.)
    ///
    /// # Returns
    /// A Result containing true if the relay is on, false if off, or an I2C error.
    fn read_relay_status(&mut self, relay_offset: u8) -> RelayDeviceStatus {
        let read_command = 0x04 + relay_offset;
        let temp = self.dev.smbus_read_byte_data(read_command)?;
        Ok(temp != Status::Off as u8)
    }

    /// Gets the current state of a specific relay.
    ///
    /// # Arguments
    /// * `relay_num` - Optional relay number (1-4). If None, checks the first relay (relay 1).
    ///
    /// # Relay Numbering Scheme:
    /// - Relay 1: Register 0x04 (offset 0)
    /// - Relay 2: Register 0x05 (offset 1)  
    /// - Relay 3: Register 0x06 (offset 2)
    /// - Relay 4: Register 0x07 (offset 3)
    ///
    /// # Returns
    /// A Result containing true if the relay is on, false if off, or an I2C error.
    pub fn get_relay_state(&mut self, relay_num: Option<u8>) -> RelayDeviceStatus {
        self.read_relay_status(relay_num.unwrap_or(0))
    }

    /// Turns on all relays on the board.
    pub fn set_all_relays_on(&mut self) -> RelayResult {
        self.write_byte(Command::TurnAllOn as u8)
    }

    /// Turns off all relays on the board.
    pub fn set_all_relays_off(&mut self) -> RelayResult {
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
    pub fn write_byte(&mut self, command: u8) -> RelayResult {
        self.dev.smbus_write_byte(command)?;
        thread::sleep(Duration::new(0, 10_000));
        Ok(())
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
    pub fn change_i2c_address(&mut self, new_address: u8) -> RelayResult {
        // Validate address range (7-bit I2C addresses)
        if !(0x07..=0x78).contains(&new_address) {
            return Err(LinuxI2CError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "I2C address must be between 0x07 and 0x78",
            )));
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
mod tests {
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

        let default_config = QwiicRelayConfig::default();
        assert_eq!(default_config.relay_count, 4);
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
    fn test_status_enum_values() {
        assert_eq!(Status::Off as u8, 0);
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
    #[ignore] // Requires actual hardware to run
    fn test_get_relay_state_with_none() {
        let config = QwiicRelayConfig::default();
        let mut qwiic_relay =
            QwiicRelay::new(config, "/dev/i2c-1", 0x08).expect("Could not init device");

        // Test getting state with None (should check relay 1 at offset 0)
        let state = qwiic_relay.get_relay_state(None);
        assert!(state.is_ok(), "Should successfully read relay state");
    }

    #[test]
    #[ignore] // Requires actual hardware to run
    fn test_get_relay_state_with_specific_relay() {
        let config = QwiicRelayConfig::default();
        let mut qwiic_relay =
            QwiicRelay::new(config, "/dev/i2c-1", 0x08).expect("Could not init device");

        // Test getting state for each relay (0-3 offset)
        for relay_num in 0..4 {
            let state = qwiic_relay.get_relay_state(Some(relay_num));
            assert!(state.is_ok(), "Should successfully read relay {} state", relay_num + 1);
        }
    }

    #[test]
    #[ignore] // Requires actual hardware to run
    fn test_relay_state_consistency() {
        let config = QwiicRelayConfig::default();
        let mut qwiic_relay =
            QwiicRelay::new(config, "/dev/i2c-1", 0x08).expect("Could not init device");

        // Turn off all relays first
        qwiic_relay.set_all_relays_off().expect("Failed to turn off all relays");
        
        // Check that None parameter reads relay 1 (offset 0)
        let state_none = qwiic_relay.get_relay_state(None).expect("Failed to get relay state");
        let state_zero = qwiic_relay.get_relay_state(Some(0)).expect("Failed to get relay state");
        assert_eq!(state_none, state_zero, "None should default to relay 1 (offset 0)");
        
        // Both should be off
        assert_eq!(state_none, false, "Relay should be off after turning all off");
    }

    #[test]
    #[ignore] // Requires actual hardware to run
    fn test_relay_state_after_on_off() {
        let config = QwiicRelayConfig::default();
        let mut qwiic_relay =
            QwiicRelay::new(config, "/dev/i2c-1", 0x08).expect("Could not init device");

        // Turn off relay 1 first
        qwiic_relay.set_relay_off(Some(0)).expect("Failed to turn off relay");
        
        // Get initial state
        let initial_state = qwiic_relay.get_relay_state(Some(0)).expect("Failed to get relay state");
        assert_eq!(initial_state, false, "Relay should be off initially");
        
        // Turn on relay 1
        qwiic_relay.set_relay_on(Some(0)).expect("Failed to turn on relay");
        
        // Check new state
        let new_state = qwiic_relay.get_relay_state(Some(0)).expect("Failed to get relay state");
        assert_eq!(new_state, true, "Relay should be on after turning on");
        
        // Turn off again
        qwiic_relay.set_relay_off(Some(0)).expect("Failed to turn off relay");
        
        // Check final state
        let final_state = qwiic_relay.get_relay_state(Some(0)).expect("Failed to get relay state");
        assert_eq!(final_state, false, "Relay should be off after turning off");
    }
}
