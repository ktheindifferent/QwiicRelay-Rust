# Qwiic Relay Rust Library - Technical Overview

## Project Structure
```
qwiic-relay-rs/
├── Cargo.toml          # Package configuration
├── LICENSE             # License file
├── README.md           # User documentation and examples
└── src/
    └── lib.rs          # Main library implementation
```

## Core Components

### 1. I2C Communication Layer
- Uses `i2cdev` crate for Linux I2C interface
- Implements SMBus protocol for relay control
- 10µs delay after each write operation for stability

### 2. Relay Control API
The library provides a clean API for relay operations:

#### Basic Operations
- `set_relay_on(relay_num)` - Turn specific relay on
- `set_relay_off(relay_num)` - Turn specific relay off
- `get_relay_state(relay_num)` - Query relay state
- `set_all_relays_on()` - Turn all relays on
- `set_all_relays_off()` - Turn all relays off

#### Device Management
- `get_version()` - Get firmware version
- `init()` - Initialize relay board (200ms startup delay)
- `change_i2c_address(new_address)` - Change the I2C address of the relay board

### 3. Data Structures

#### Enums
- `Addresses` - I2C addresses for different relay board models
- `Command` - Control commands for relay operations
- `RelayState` - On/Off states and version registers
- `Status` - Status values from relay board

#### Structs
- `QwiicRelayConfig` - Configuration with relay count
- `QwiicRelay` - Main controller with I2C device handle

### 4. Relay Toggle Logic
The library uses a toggle-based approach for multi-relay boards:
1. Read current relay state
2. If state differs from desired, send toggle command
3. For single relay boards, directly write On/Off state

## Supported Hardware
- Single Relay (0x18, 0x19)
- Dual Solid State Relay (0x0A, 0x0B)
- Quad Relay (0x6D, 0x6C)
- Quad Solid State Relay (0x08, 0x09)

## Testing Strategy
- Unit tests for configuration creation
- Integration tests for relay operations (marked as ignored, require hardware)
- Tests verify state changes and proper command execution

## Error Handling
- All I2C operations return `Result` types
- Errors are properly propagated using `?` operator
- Clear error messages for debugging

## Performance Considerations
- Minimal delays (10µs for writes, 200ms for init)
- Efficient state checking before toggle operations
- Batch operations for controlling all relays