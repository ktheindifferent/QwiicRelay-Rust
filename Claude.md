# Qwiic Relay Rust Library - Claude Documentation

## Project Overview
This is a Rust library for controlling SparkFun Qwiic Relay boards via I2C communication on Linux systems. The library provides a comprehensive interface for managing various types of Qwiic Relay boards including single relays, dual solid state relays, quad relays, and quad solid state relays.

## Repository Structure
```
qwiic-relay-rs/
├── Cargo.toml           # Project manifest with dependencies and metadata
├── Cargo.lock          # Dependency lock file
├── LICENSE             # Dual license: MIT OR Apache-2.0
├── README.md           # Main project documentation
├── Claude.md           # This file - AI assistant documentation
├── overview.md         # Project overview documentation
├── project_description.md # Detailed project description
├── todo.md             # Project TODO list
└── src/
    └── lib.rs          # Main library implementation
```

## Codebase Description

### Main Library (`src/lib.rs`)
The library is implemented as a single module that provides:

1. **Core Types and Enums:**
   - `Addresses`: Predefined I2C addresses for different relay board configurations
   - `Command`: Commands that can be sent to relay boards
   - `RelayState`: States and control values for relays
   - `Status`: Status values returned by relay boards

2. **Configuration:**
   - `QwiicRelayConfig`: Configuration struct for specifying relay board parameters (number of relays)

3. **Main Interface:**
   - `QwiicRelay`: Primary struct for controlling relay boards
     - Connection management via I2C
     - Individual relay control (on/off)
     - Bulk relay control (all on/off)
     - State querying
     - Firmware version retrieval
     - I2C address modification capability

## Dependencies
- **i2cdev** (v0.4.4): Linux I2C device communication
- **enum_primitive** (v0.1.1): Enum primitive type conversions

## Key Features
- Support for multiple relay board types (1, 2, or 4 relays)
- Both standard and solid-state relay support
- Individual and bulk relay control
- State querying for verification
- Firmware version checking
- Dynamic I2C address modification
- Comprehensive error handling using Result types
- Well-documented API with rustdoc comments

## Testing
The codebase includes comprehensive unit tests and hardware integration tests:
- Unit tests for configuration, enums, and type validations
- Hardware tests (marked with `#[ignore]`) for actual relay operations
- Test coverage for all major functionality

## Build Commands
```bash
# Build the library
cargo build

# Run tests (excluding hardware tests)
cargo test

# Run all tests including hardware tests
cargo test -- --ignored

# Generate documentation
cargo doc --open

# Check code without building
cargo check

# Format code
cargo fmt

# Run linter
cargo clippy
```

## API Usage Example
```rust
use qwiic_relay_rs::{QwiicRelay, QwiicRelayConfig};

// Create configuration for a quad relay board
let config = QwiicRelayConfig::default(); // defaults to 4 relays

// Initialize relay controller
let mut relay = QwiicRelay::new(config, "/dev/i2c-1", 0x08)?;

// Control individual relays
relay.set_relay_on(Some(1))?;  // Turn on relay 1
relay.set_relay_off(Some(1))?; // Turn off relay 1

// Control all relays
relay.set_all_relays_on()?;
relay.set_all_relays_off()?;

// Check relay state
let is_on = relay.get_relay_state(Some(1))?;

// Get firmware version
let version = relay.get_version()?;
```

## Recent Updates
- Added `change_i2c_address` method for dynamically modifying relay board I2C addresses
- Comprehensive test suite with unit and integration tests
- Full rustdoc documentation for all public APIs
- Support for various Qwiic Relay board configurations

## License
Dual licensed under:
- MIT License
- Apache License 2.0

## Author
Caleb Mitchell Smith-Woolrich (PixelCoda) <calebsmithwoolrich@gmail.com>

## Repository
https://github.com/PixelCoda/QwiicRelay-Rust

## Documentation
https://docs.rs/qwiic-relay-rs