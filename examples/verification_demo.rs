use qwiic_relay_rs::{QwiicRelay, QwiicRelayConfig, VerificationConfig};
use std::thread;
use std::time::Duration;

fn main() {
    println!("Qwiic Relay State Verification Demo");
    println!("====================================\n");

    // Example 1: Strict verification mode (default)
    println!("1. Testing with STRICT verification mode:");
    println!("   - Verifies state after each operation");
    println!("   - Retries up to 3 times on failure");
    println!("   - 1 second timeout\n");
    
    let strict_config = QwiicRelayConfig::with_verification(
        4,
        VerificationConfig::strict()
    );
    
    match QwiicRelay::new(strict_config, "/dev/i2c-1", 0x08) {
        Ok(mut relay) => {
            println!("   Turning relay 1 ON with verification...");
            match relay.set_relay_on(Some(1)) {
                Ok(_) => println!("   ✓ Relay 1 is verified ON"),
                Err(e) => println!("   ✗ Failed: {}", e),
            }
            
            thread::sleep(Duration::from_millis(500));
            
            println!("   Turning relay 1 OFF with verification...");
            match relay.set_relay_off(Some(1)) {
                Ok(_) => println!("   ✓ Relay 1 is verified OFF"),
                Err(e) => println!("   ✗ Failed: {}", e),
            }
        }
        Err(e) => println!("   Could not connect to relay board: {}", e),
    }
    
    println!();

    // Example 2: Lenient verification mode
    println!("2. Testing with LENIENT verification mode:");
    println!("   - More retries (5) and longer delays");
    println!("   - 2 second timeout");
    println!("   - Better for noisy environments\n");
    
    let lenient_config = QwiicRelayConfig::with_verification(
        4,
        VerificationConfig::lenient()
    );
    
    match QwiicRelay::new(lenient_config, "/dev/i2c-1", 0x08) {
        Ok(mut relay) => {
            for i in 1..=4 {
                println!("   Testing relay {}...", i);
                
                match relay.set_relay_on(Some(i)) {
                    Ok(_) => print!("     ON ✓"),
                    Err(e) => print!("     ON ✗: {}", e),
                }
                
                thread::sleep(Duration::from_millis(200));
                
                match relay.set_relay_off(Some(i)) {
                    Ok(_) => println!(" -> OFF ✓"),
                    Err(e) => println!(" -> OFF ✗: {}", e),
                }
            }
        }
        Err(e) => println!("   Could not connect to relay board: {}", e),
    }
    
    println!();

    // Example 3: Custom verification configuration
    println!("3. Testing with CUSTOM verification settings:");
    println!("   - 2 retries, 100ms retry delay");
    println!("   - 50ms verification delay");
    println!("   - 500ms timeout\n");
    
    let custom_verification = VerificationConfig::default()
        .with_max_retries(2)
        .with_retry_delay(100)
        .with_verification_delay(50)
        .with_timeout(500);
    
    let custom_config = QwiicRelayConfig::with_verification(4, custom_verification);
    
    match QwiicRelay::new(custom_config, "/dev/i2c-1", 0x08) {
        Ok(mut relay) => {
            println!("   Rapid relay switching with custom timing...");
            
            for _ in 0..3 {
                relay.set_relay_on(Some(1)).ok();
                thread::sleep(Duration::from_millis(100));
                relay.set_relay_off(Some(1)).ok();
                thread::sleep(Duration::from_millis(100));
            }
            
            println!("   ✓ Rapid switching completed");
        }
        Err(e) => println!("   Could not connect to relay board: {}", e),
    }
    
    println!();

    // Example 4: Disabled verification (fastest, no checking)
    println!("4. Testing with DISABLED verification:");
    println!("   - No state verification");
    println!("   - Fastest operation");
    println!("   - Use when verification is not needed\n");
    
    let disabled_config = QwiicRelayConfig::with_verification(
        4,
        VerificationConfig::disabled()
    );
    
    match QwiicRelay::new(disabled_config, "/dev/i2c-1", 0x08) {
        Ok(mut relay) => {
            println!("   Ultra-fast relay operations without verification...");
            
            let start = std::time::Instant::now();
            
            for _ in 0..10 {
                relay.set_relay_on(Some(1)).ok();
                relay.set_relay_off(Some(1)).ok();
            }
            
            let elapsed = start.elapsed();
            println!("   ✓ 20 operations completed in {:?}", elapsed);
        }
        Err(e) => println!("   Could not connect to relay board: {}", e),
    }
    
    println!("\nDemo completed!");
}