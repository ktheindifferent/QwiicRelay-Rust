# Qwiic Relay I2C library for Rust (WIP)

## Description

This library aims at controlling Qwiic Relays using I2C from Linux. Its
primary target is ARM devices such as Raspberry Pi or FriendlyARM's NanoPi Neo.
It should nonetheless work on other Linux distributions with access to an I2C
bus.

Currently I only have access to the Quad Solid State Relay for testing purposes. If you have issues with other Qwiic Relays please submit an issue or a pull request.

Roadmap:
* Map relay commands and addresses to structs (DONE)
* Ability to toggle all relays on/off (DONE)
* Ability to toggle individual relays on/off (DONE)
* Ability to read relay status (DONE)
* Ability to check firmware version (DONE)
* Ability to change relay hardware address (DONE)
* Configurable I2C communication timing (DONE)
* Auto-detect optimal timing settings (DONE)

## How to use library

Add the following line to your cargo.toml:
```
qwiic-relay-rs = "0.1.11"
```

Or for the most recent commit on the master branch use:
```
qwiic-relay-rs = { git = "https://github.com/PixelCoda/QwiicRelay-Rust.git", version = "*" }
```

Example:
```rust


use qwiic_relay_rs::{QwiicRelay, QwiicRelayConfig};
use std::thread;
use std::time::Duration;

fn main() {
    let config = QwiicRelayConfig::default();
    let mut qwiic_relay = QwiicRelay::new(config, "/dev/i2c-1", 0x08)
        .expect("Could not init device");
    
    // Get and display firmware version
    match qwiic_relay.get_version() {
        Ok(v) => println!("Firmware Version: {}", v),
        Err(e) => {
            println!("Error getting version: {:?}", e);
            return;
        }
    }

    // Test all relays on/off
    println!("Testing all relays...");
    qwiic_relay.set_all_relays_off().unwrap();
    thread::sleep(Duration::from_secs(1));
    
    qwiic_relay.set_all_relays_on().unwrap();
    thread::sleep(Duration::from_secs(1));
    
    qwiic_relay.set_all_relays_off().unwrap();
    thread::sleep(Duration::from_secs(1));

    // Test individual relays
    for relay_num in 1..=4 {
        println!("Testing relay {}", relay_num);
        
        // Turn on relay
        qwiic_relay.set_relay_on(Some(relay_num)).unwrap();
        thread::sleep(Duration::from_millis(500));
        
        // Check state
        if qwiic_relay.get_relay_state(Some(relay_num)).unwrap() {
            println!("  Relay {} is ON", relay_num);
        }
        
        // Turn off relay
        qwiic_relay.set_relay_off(Some(relay_num)).unwrap();
        thread::sleep(Duration::from_millis(500));
    }
    
    println!("Test complete!");
}
```

## Timing Configuration

The library now supports configurable I2C communication timing to accommodate different relay board types and I2C bus speeds. 

### Default Timing
The default configuration uses:
- Write delay: 10μs (after each I2C write operation)
- State change delay: 10ms (for relay state transitions)
- Initialization delay: 200ms (board startup time)

### Board-Specific Configurations

**Solid State Relays** (faster switching):
```rust
let config = QwiicRelayConfig::for_solid_state(4);
// Uses: 5μs write delay, 5ms state change, 100ms init
```

**Mechanical Relays** (slower switching):
```rust
let config = QwiicRelayConfig::for_mechanical(4);
// Uses: 15μs write delay, 20ms state change, 250ms init
```

### Custom Timing
```rust
let config = QwiicRelayConfig::with_timing(
    4,    // relay count
    15,   // write delay in microseconds
    25,   // state change delay in milliseconds
    300   // init delay in milliseconds
);
```

### Runtime Adjustment
```rust
let mut relay = QwiicRelay::new(config, "/dev/i2c-1", 0x08)?;

// Adjust timing at runtime
relay.set_write_delay(20);           // 20μs write delay
relay.set_state_change_delay(30);    // 30ms state change delay

// Or update the entire configuration
let new_config = QwiicRelayConfig::for_solid_state(4);
relay.update_config(new_config);
```

### Auto-Detection
The library can attempt to find optimal timing automatically:
```rust
let mut relay = QwiicRelay::new(config, "/dev/i2c-1", 0x08)?;
relay.init()?;

match relay.auto_detect_timing() {
    Ok(true) => println!("Timing optimized successfully"),
    Ok(false) => println!("Could not optimize, using defaults"),
    Err(e) => println!("Auto-detection failed: {:?}", e),
}
```

### Timing Guidelines

| Board Type | Write Delay | State Change | Init Delay | Notes |
|------------|------------|--------------|------------|-------|
| Solid State | 5-10μs | 5-10ms | 100-150ms | Fast electronic switching |
| Mechanical | 10-20μs | 15-30ms | 200-300ms | Physical relay movement |
| Long I2C Bus | 15-30μs | 20-40ms | 250-400ms | Increased capacitance |
| High Speed I2C | 2-5μs | 5-10ms | 100ms | 400kHz+ bus speed |

### Benchmarking
Run benchmarks to test different timing configurations:
```bash
cargo bench
```

The benchmark will test various timing configurations and report performance differences.

## References

* https://github.com/sparkfun/Qwiic_Relay_Py/blob/main/qwiic_relay.py
* https://github.com/sparkfun/SparkFun_Qwiic_Relay_Arduino_Library/tree/master/src

## License

Licensed under either of:

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.