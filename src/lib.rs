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

extern crate i2cdev;

use std::thread;
use std::time::Duration;

use i2cdev::core::*;
use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};

#[derive(Copy, Clone)]
pub enum Addresses {
    SingleRelayDefault = 0x18,
    SingleRelayJumperClosed = 0x19,
    QuadRelayDefault = 0x6D,
    QuadRelayJumperClosed = 0x6C,
    DualSolidState = 0x0A,
    DualSolidStateJumperClosed = 0x0B,
    QuadSolidState = 0x08,
    QuadSolidStateJumperClosed = 0x09
}

#[derive(Copy, Clone)]
pub enum Command {
    DualQuadToggleBase = 0x00,
    ToggleRelayOne = 0x01,
    ToggleRelayTwo = 0x02,
    ToggleRelayThree = 0x03,
    ToggleRelayFour = 0x04,
    RelayOneStaus = 0x05,
    RelayTwoStaus = 0x06,
    RelayThreeStaus = 0x07,
    RelayFourStaus = 0x08,
    TurnAllOff = 0x0A,
    TurnAllOn = 0x0B,
    ToggleAll = 0x0C
}

#[derive(Copy, Clone)]
pub enum RelayState {
    Off = 0x00,
    On = 0x01,
    SingleFirmwareVersion = 0x04,
    SingleStatusVersion = 0x05,
}

#[derive(Copy, Clone)]
pub enum Status {
    Off = 0
}

pub struct QwiicRelayConfig {
    relay_count: u8
}

impl QwiicRelayConfig {
    pub fn new(relay_count: u8) -> QwiicRelayConfig {
        QwiicRelayConfig {
            relay_count: 4,
        }
    }

    pub fn default() -> QwiicRelayConfig {
        QwiicRelayConfig::new(4)
    }
}

// QwiicRelay
pub struct QwiicRelay {
    dev: LinuxI2CDevice,
    config: QwiicRelayConfig,
}

type RelayDeviceStatus = Result<bool, LinuxI2CError>;
type RelayResult = Result<(), LinuxI2CError>;
type VersionResult = Result<u8, LinuxI2CError>;

impl QwiicRelay {
    pub fn new(config: QwiicRelayConfig, bus: &str, i2c_addr: u16) -> Result<QwiicRelay, LinuxI2CError> {
        let dev = LinuxI2CDevice::new(bus, i2c_addr)?;
        Ok(QwiicRelay {
               dev,
               config,
           })
    }

    pub fn init(&mut self) -> RelayResult {





  
        // Wait for the QwiicRelay to set up
        thread::sleep(Duration::from_millis(200));

        Ok(())
    }

    pub fn set_relay_on(&mut self, relay_num: Option<u8>) -> RelayResult {
        match relay_num {
            Some(num) => {
                let read_command = 0x04 + num as u8;
                let temp = self.dev.smbus_read_byte_data(read_command)?;

                if temp == (Status::Off as u8) {
                    self.write_byte((Command::DualQuadToggleBase as u8) + num);
                    return Ok(());
                } else {
                    return Ok(());
                }                
            },
            None => {
                self.write_byte(RelayState::On as u8);
                return Ok(());
            }
        }
    }

    pub fn set_relay_off(&mut self, relay_num: Option<u8>) -> RelayResult {
        match relay_num {
            Some(num) => {
                let read_command = 0x04 + num as u8;
                let temp = self.dev.smbus_read_byte_data(read_command)?;

                if temp != (Status::Off as u8) {
                    self.write_byte((Command::DualQuadToggleBase as u8) + num);
                    return Ok(());
                } else {
                    return Ok(());
                }                
            },
            None => {
                self.write_byte(RelayState::Off as u8);
                return Ok(());
            }
        }
    }

    pub fn get_relay_state(&mut self, relay_num: Option<u8>) -> RelayDeviceStatus {
        match relay_num {
            Some(num) => {
                let read_command = 0x04 + num as u8;
                let temp = self.dev.smbus_read_byte_data(read_command)?;

                if temp != (Status::Off as u8) {
                    return Ok(true);
                } else {
                    return Ok(false);
                }                
            },
            None => {
                let read_command = 0x04;
                let temp = self.dev.smbus_read_byte_data(read_command)?;

                if temp != (Status::Off as u8) {
                    return Ok(true);
                } else {
                    return Ok(false);
                }         
            }
        }
    }

    pub fn set_all_relays_on(&mut self) -> RelayResult {
        self.write_byte(Command::TurnAllOn as u8)
    }

    pub fn set_all_relays_off(&mut self) -> RelayResult {
        self.write_byte(Command::TurnAllOff as u8)
    }

    pub fn get_version(&mut self) -> VersionResult {
        let version = self.dev.smbus_read_byte_data(RelayState::SingleFirmwareVersion as u8)?;
        Ok(version)
    }

    pub fn write_byte(&mut self, command: u8) -> RelayResult {
        self.dev.smbus_write_byte(command)?;
        thread::sleep(Duration::new(0, 10_000));
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {

        let config = QwiicRelayConfig::default();
        let mut qwiic_relay = QwiicRelay::new(config, "/dev/i2c-1", 0x08).expect("Could not init device");
        let version = qwiic_relay.get_version();
        match version {
            Ok(v) => {
                println!("Firmware Version: {}", v);
    
    
                println!("all off");
                qwiic_relay.set_all_relays_off().unwrap();
                thread::sleep(Duration::from_secs(2));
            
                println!("all on");
                qwiic_relay.set_all_relays_on().unwrap();
                thread::sleep(Duration::from_secs(2));
            
                println!("all off");
                qwiic_relay.set_all_relays_off().unwrap();
                thread::sleep(Duration::from_secs(2));
    
                println!("set_relay_on: 1");
                qwiic_relay.set_relay_on(Some(1)).unwrap();
                thread::sleep(Duration::from_secs(2));
    
                println!("get_relay_state: 1");
                let relay_one_state = qwiic_relay.get_relay_state(Some(1)).unwrap();
                if relay_one_state {
                    println!("relay 1 is on!");
                }
                thread::sleep(Duration::from_secs(2));
                
    
                println!("set_relay_off: 1");
                qwiic_relay.set_relay_off(Some(1)).unwrap();
                thread::sleep(Duration::from_secs(2));
    
                println!("set_relay_on: 2");
                qwiic_relay.set_relay_on(Some(2)).unwrap();
                thread::sleep(Duration::from_secs(2));
    
                println!("get_relay_state: 2");
                let relay_one_state = qwiic_relay.get_relay_state(Some(2)).unwrap();
                if relay_one_state {
                    println!("relay 2 is on!");
                }
                thread::sleep(Duration::from_secs(2));
    
                println!("set_relay_off: 2");
                qwiic_relay.set_relay_off(Some(2)).unwrap();
                thread::sleep(Duration::from_secs(2));
    
                println!("set_relay_on: 3");
                qwiic_relay.set_relay_on(Some(3)).unwrap();
                thread::sleep(Duration::from_secs(2));
    
                println!("get_relay_state: 3");
                let relay_one_state = qwiic_relay.get_relay_state(Some(3)).unwrap();
                if relay_one_state {
                    println!("relay 3 is on!");
                }
                thread::sleep(Duration::from_secs(2));
    
                println!("set_relay_off: 3");
                qwiic_relay.set_relay_off(Some(3)).unwrap();
                thread::sleep(Duration::from_secs(2));
    
                println!("set_relay_on: 4");
                qwiic_relay.set_relay_on(Some(4)).unwrap();
                thread::sleep(Duration::from_secs(2));
    
                println!("get_relay_state: 4");
                let relay_one_state = qwiic_relay.get_relay_state(Some(4)).unwrap();
                if relay_one_state {
                    println!("relay 4 is on!");
                }
                thread::sleep(Duration::from_secs(2));
        
                println!("set_relay_off: 4");
                qwiic_relay.set_relay_off(Some(4)).unwrap();
                thread::sleep(Duration::from_secs(2));
            },
            Err(e) => println!("{:?}", e)
        }
    }
}


