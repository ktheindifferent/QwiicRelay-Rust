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

//! This provides a Rust API for SparkFun I2C relays like the [SparkFun Qwiic Single Relay].
//!
//! [SparkFun Qwiic Single Relay]: https://www.sparkfun.com/sparkfun-qwiic-single-relay.html

#![forbid(unsafe_code)]
#![deny(warnings)]
//#![forbid(missing_docs)] // TODO: add docs for everything
#![forbid(missing_debug_implementations)]
#![deny(unused)]
#![no_std]

use embedded_hal_async::i2c::{self, I2c, SevenBitAddress};

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

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RelayState {
    Off = 0x00,
    On = 0x01,
    SingleFirmwareVersion = 0x04,
    SingleStatusVersion = 0x05,
}

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Status {
    Off = 0,
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct QwiicRelay<T> {
    i2c: T,
    i2c_addr: SevenBitAddress,
}

impl<T: I2c<Error = E>, E: i2c::Error> QwiicRelay<T> {
    pub fn new(
        i2c: T,
        i2c_addr: SevenBitAddress,
    ) -> Result<QwiicRelay<T>, E> {
        Ok(QwiicRelay {
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

// TODO: add tests based on `embedded-hal-mock`
