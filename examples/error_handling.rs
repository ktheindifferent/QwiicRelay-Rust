use qwiic_relay_rs::{QwiicRelay, QwiicRelayConfig, RelayError, VerificationConfig};

fn main() {
    println!("Qwiic Relay Error Handling Demo");
    println!("================================\n");

    // Configure strict verification with minimal retries for demo
    let verification = VerificationConfig::default()
        .with_max_retries(1)
        .with_timeout(500);
    
    let config = QwiicRelayConfig::with_verification(4, verification);
    
    // Try to connect to relay board
    match QwiicRelay::new(config, "/dev/i2c-1", 0x08) {
        Ok(mut relay) => {
            println!("Connected to relay board successfully!\n");
            
            // Example 1: Handling state verification failures
            println!("1. Handling state verification failures:");
            match relay.set_relay_on(Some(1)) {
                Ok(_) => {
                    println!("   ✓ Relay 1 turned on successfully");
                }
                Err(RelayError::StateVerificationFailed { 
                    relay_num, 
                    expected, 
                    actual, 
                    attempts 
                }) => {
                    println!("   ✗ State verification failed!");
                    println!("     - Relay: {:?}", relay_num);
                    println!("     - Expected: {}", if expected { "ON" } else { "OFF" });
                    println!("     - Actual: {}", if actual { "ON" } else { "OFF" });
                    println!("     - Attempts made: {}", attempts);
                    
                    // You could implement fallback logic here
                    println!("     Implementing fallback logic...");
                }
                Err(RelayError::Timeout { 
                    relay_num, 
                    operation, 
                    duration_ms 
                }) => {
                    println!("   ✗ Operation timed out!");
                    println!("     - Relay: {:?}", relay_num);
                    println!("     - Operation: {}", operation);
                    println!("     - Timeout: {}ms", duration_ms);
                }
                Err(RelayError::I2C(e)) => {
                    println!("   ✗ I2C communication error: {}", e);
                    println!("     Check your wiring and I2C connection");
                }
                Err(RelayError::InvalidConfiguration(msg)) => {
                    println!("   ✗ Configuration error: {}", msg);
                }
            }
            
            println!();
            
            // Example 2: Checking relay state with error handling
            println!("2. Checking relay state with error handling:");
            match relay.get_relay_state(Some(1)) {
                Ok(state) => {
                    println!("   ✓ Relay 1 is currently: {}", 
                        if state { "ON" } else { "OFF" });
                }
                Err(e) => {
                    println!("   ✗ Failed to read relay state: {}", e);
                    
                    // Pattern match on specific error types
                    match e {
                        RelayError::I2C(_) => {
                            println!("     This is an I2C error - check connection");
                        }
                        _ => {
                            println!("     Unexpected error type");
                        }
                    }
                }
            }
            
            println!();
            
            // Example 3: Batch operations with individual error handling
            println!("3. Batch operations with error handling:");
            for relay_num in 1..=4 {
                print!("   Relay {}: ", relay_num);
                
                // Try to turn on
                match relay.set_relay_on(Some(relay_num)) {
                    Ok(_) => print!("ON ✓"),
                    Err(_) => print!("ON ✗"),
                }
                
                print!(" -> ");
                
                // Try to turn off
                match relay.set_relay_off(Some(relay_num)) {
                    Ok(_) => println!("OFF ✓"),
                    Err(_) => println!("OFF ✗"),
                }
            }
            
            println!();
            
            // Example 4: Invalid address error handling
            println!("4. Testing invalid I2C address change:");
            match relay.change_i2c_address(0x80) {  // Invalid address (> 0x78)
                Ok(_) => {
                    println!("   ✓ Address changed (unexpected!)");
                }
                Err(RelayError::InvalidConfiguration(msg)) => {
                    println!("   ✓ Correctly caught invalid configuration: {}", msg);
                }
                Err(e) => {
                    println!("   ✗ Unexpected error: {}", e);
                }
            }
        }
        Err(e) => {
            println!("Failed to connect to relay board: {}", e);
            println!("\nTroubleshooting steps:");
            println!("1. Check that the relay board is connected to I2C pins");
            println!("2. Verify the I2C address (default is 0x08)");
            println!("3. Ensure I2C is enabled on your system");
            println!("4. Check that you have permission to access /dev/i2c-1");
            
            // Pattern match on the error type for specific guidance
            match e {
                RelayError::I2C(i2c_err) => {
                    println!("\nSpecific I2C error: {}", i2c_err);
                }
                _ => {
                    println!("\nError details: {}", e);
                }
            }
        }
    }
    
    println!("\nError handling demo completed!");
}