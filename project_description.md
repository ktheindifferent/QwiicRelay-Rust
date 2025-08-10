# Qwiic Relay Rust Library

## Project Description

A Rust library for controlling SparkFun Qwiic Relay boards via I2C on Linux systems, primarily targeting ARM devices like Raspberry Pi.

## Key Features
- Support for various Qwiic Relay board types (single, dual, quad relays)
- I2C communication interface
- Individual relay control (on/off)
- Bulk relay operations (all on/off)
- Relay state querying
- Firmware version detection

## Recent Work Completed
- Fixed typos in enum names (RelayStaus → RelayStatus)
- Fixed bug in config constructor (was ignoring relay_count parameter)
- Cleaned up unused code and removed unnecessary blank lines
- Added comprehensive documentation for all public APIs
- Fixed variable naming issues in tests (all relay_one_state → appropriate names)
- Simplified example code in README to be more concise and readable
- Implemented Default trait properly instead of custom default method
- Fixed all clippy warnings (unnecessary casts, needless returns)
- Added Rust 2021 edition to Cargo.toml
- Removed promotional content from README
- Improved error handling to propagate errors properly
- Made config fields public for better API usability
- Added proper test structure with hardware and unit tests

## Architecture
- Main struct: `QwiicRelay` - handles I2C communication with relay board
- Configuration: `QwiicRelayConfig` - stores relay board settings
- Enums for addresses, commands, states, and status values
- Uses linux-i2c library for low-level I2C operations