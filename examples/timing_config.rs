/// Example demonstrating configurable I2C timing features
/// 
/// This example shows how to:
/// - Use different timing configurations for different relay types
/// - Adjust timing at runtime
/// - Auto-detect optimal timing settings

use qwiic_relay_rs::{QwiicRelay, QwiicRelayConfig};
use std::thread;
use std::time::Duration;

fn main() {
    println!("Qwiic Relay Timing Configuration Example");
    println!("=========================================\n");

    // Example 1: Using default timing
    println!("1. Default Configuration:");
    let default_config = QwiicRelayConfig::default();
    println!("   Relay count: {}", default_config.relay_count);
    println!("   Write delay: {}μs", default_config.write_delay_us);
    println!("   State change delay: {}ms", default_config.state_change_delay_ms);
    println!("   Init delay: {}ms\n", default_config.init_delay_ms);

    // Example 2: Configuration for solid state relays
    println!("2. Solid State Relay Configuration:");
    let solid_state_config = QwiicRelayConfig::for_solid_state(4);
    println!("   Relay count: {}", solid_state_config.relay_count);
    println!("   Write delay: {}μs (faster switching)", solid_state_config.write_delay_us);
    println!("   State change delay: {}ms (no mechanical delay)", solid_state_config.state_change_delay_ms);
    println!("   Init delay: {}ms\n", solid_state_config.init_delay_ms);

    // Example 3: Configuration for mechanical relays
    println!("3. Mechanical Relay Configuration:");
    let mechanical_config = QwiicRelayConfig::for_mechanical(4);
    println!("   Relay count: {}", mechanical_config.relay_count);
    println!("   Write delay: {}μs (conservative timing)", mechanical_config.write_delay_us);
    println!("   State change delay: {}ms (accounts for mechanical switching)", mechanical_config.state_change_delay_ms);
    println!("   Init delay: {}ms\n", mechanical_config.init_delay_ms);

    // Example 4: Custom timing configuration
    println!("4. Custom Timing Configuration:");
    let custom_config = QwiicRelayConfig::with_timing(2, 8, 12, 150);
    println!("   Relay count: {}", custom_config.relay_count);
    println!("   Write delay: {}μs", custom_config.write_delay_us);
    println!("   State change delay: {}ms", custom_config.state_change_delay_ms);
    println!("   Init delay: {}ms\n", custom_config.init_delay_ms);

    // Example 5: Runtime timing adjustment (requires hardware)
    println!("5. Runtime Timing Adjustment (requires hardware):");
    
    // Uncomment the following code if you have hardware connected
    /*
    let config = QwiicRelayConfig::default();
    let mut relay = match QwiicRelay::new(config, "/dev/i2c-1", 0x08) {
        Ok(r) => r,
        Err(e) => {
            println!("   Failed to initialize relay: {:?}", e);
            return;
        }
    };

    // Initialize the relay
    if let Err(e) = relay.init() {
        println!("   Failed to initialize: {:?}", e);
        return;
    }

    println!("   Initial write delay: {}μs", relay.config.write_delay_us);
    
    // Adjust timing at runtime
    relay.set_write_delay(15);
    println!("   Adjusted write delay: {}μs", relay.config.write_delay_us);
    
    relay.set_state_change_delay(20);
    println!("   Adjusted state change delay: {}ms", relay.config.state_change_delay_ms);

    // Test relay operations with new timing
    println!("   Testing relay operations with adjusted timing...");
    
    for i in 1..=3 {
        println!("   Cycle {}/3", i);
        
        if let Err(e) = relay.set_relay_on(Some(1)) {
            println!("   Failed to turn relay on: {:?}", e);
            break;
        }
        thread::sleep(Duration::from_millis(500));
        
        if let Err(e) = relay.set_relay_off(Some(1)) {
            println!("   Failed to turn relay off: {:?}", e);
            break;
        }
        thread::sleep(Duration::from_millis(500));
    }
    */
    println!("   (Hardware test skipped - uncomment code to test with hardware)\n");

    // Example 6: Auto-detect timing (requires hardware)
    println!("6. Auto-Detect Optimal Timing (requires hardware):");
    
    // Uncomment the following code if you have hardware connected
    /*
    let config = QwiicRelayConfig::default();
    let mut relay = match QwiicRelay::new(config, "/dev/i2c-1", 0x08) {
        Ok(r) => r,
        Err(e) => {
            println!("   Failed to initialize relay: {:?}", e);
            return;
        }
    };

    if let Err(e) = relay.init() {
        println!("   Failed to initialize: {:?}", e);
        return;
    }

    println!("   Starting auto-detection...");
    println!("   Initial timing: write={}μs, state_change={}ms", 
        relay.config.write_delay_us, 
        relay.config.state_change_delay_ms);
    
    match relay.auto_detect_timing() {
        Ok(true) => {
            println!("   ✓ Timing optimized successfully!");
            println!("   Optimized timing: write={}μs, state_change={}ms", 
                relay.config.write_delay_us, 
                relay.config.state_change_delay_ms);
        }
        Ok(false) => {
            println!("   Could not optimize timing, using defaults");
        }
        Err(e) => {
            println!("   Auto-detection failed: {:?}", e);
        }
    }
    */
    println!("   (Hardware test skipped - uncomment code to test with hardware)\n");

    println!("Example complete!");
    println!("\nTiming configuration allows you to:");
    println!("• Optimize performance for different relay types");
    println!("• Adjust timing for different I2C bus speeds");
    println!("• Handle long I2C buses with increased capacitance");
    println!("• Auto-detect optimal settings for your specific hardware");
}