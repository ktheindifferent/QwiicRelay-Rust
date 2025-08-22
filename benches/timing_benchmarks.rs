// Benchmarks for testing different timing configurations
// Run with: cargo bench

use qwiic_relay_rs::{QwiicRelay, QwiicRelayConfig};
use std::time::{Duration, Instant};

/// Benchmark relay operations with different timing configurations
fn benchmark_timing_config(config: QwiicRelayConfig, iterations: u32) -> Duration {
    // Note: This requires actual hardware to run
    let mut relay = match QwiicRelay::new(config, "/dev/i2c-1", 0x08) {
        Ok(r) => r,
        Err(e) => {
            println!("Failed to initialize relay: {:?}", e);
            return Duration::from_secs(0);
        }
    };

    let start = Instant::now();
    
    for _ in 0..iterations {
        // Turn relay on
        if let Err(e) = relay.set_relay_on(Some(1)) {
            println!("Failed to turn relay on: {:?}", e);
            break;
        }
        
        // Turn relay off
        if let Err(e) = relay.set_relay_off(Some(1)) {
            println!("Failed to turn relay off: {:?}", e);
            break;
        }
    }
    
    start.elapsed()
}

/// Benchmark different relay board configurations
fn main() {
    println!("Qwiic Relay Timing Benchmarks");
    println!("==============================");
    
    let iterations = 100;
    
    // Test standard configuration
    println!("\nStandard Configuration (10μs write delay, 10ms state change):");
    let standard_config = QwiicRelayConfig::new(4);
    let standard_time = benchmark_timing_config(standard_config, iterations);
    println!("  Time for {} iterations: {:?}", iterations, standard_time);
    println!("  Average per operation: {:?}", standard_time / (iterations * 2));
    
    // Test solid state optimized configuration
    println!("\nSolid State Configuration (5μs write delay, 5ms state change):");
    let solid_state_config = QwiicRelayConfig::for_solid_state(4);
    let solid_state_time = benchmark_timing_config(solid_state_config, iterations);
    println!("  Time for {} iterations: {:?}", iterations, solid_state_time);
    println!("  Average per operation: {:?}", solid_state_time / (iterations * 2));
    
    // Test mechanical relay configuration
    println!("\nMechanical Configuration (15μs write delay, 20ms state change):");
    let mechanical_config = QwiicRelayConfig::for_mechanical(4);
    let mechanical_time = benchmark_timing_config(mechanical_config, iterations);
    println!("  Time for {} iterations: {:?}", iterations, mechanical_time);
    println!("  Average per operation: {:?}", mechanical_time / (iterations * 2));
    
    // Test custom aggressive configuration
    println!("\nAggressive Configuration (2μs write delay, 2ms state change):");
    let aggressive_config = QwiicRelayConfig::with_timing(4, 2, 2, 100);
    let aggressive_time = benchmark_timing_config(aggressive_config, iterations);
    println!("  Time for {} iterations: {:?}", iterations, aggressive_time);
    println!("  Average per operation: {:?}", aggressive_time / (iterations * 2));
    
    // Test custom conservative configuration
    println!("\nConservative Configuration (25μs write delay, 30ms state change):");
    let conservative_config = QwiicRelayConfig::with_timing(4, 25, 30, 300);
    let conservative_time = benchmark_timing_config(conservative_config, iterations);
    println!("  Time for {} iterations: {:?}", iterations, conservative_time);
    println!("  Average per operation: {:?}", conservative_time / (iterations * 2));
    
    // Summary
    println!("\n==============================");
    println!("Summary:");
    if standard_time > Duration::from_secs(0) {
        println!("Standard config baseline: {:?}", standard_time);
        
        if solid_state_time > Duration::from_secs(0) {
            let speedup = standard_time.as_secs_f64() / solid_state_time.as_secs_f64();
            println!("Solid state speedup: {:.2}x", speedup);
        }
        
        if mechanical_time > Duration::from_secs(0) {
            let slowdown = mechanical_time.as_secs_f64() / standard_time.as_secs_f64();
            println!("Mechanical slowdown: {:.2}x", slowdown);
        }
        
        if aggressive_time > Duration::from_secs(0) {
            let speedup = standard_time.as_secs_f64() / aggressive_time.as_secs_f64();
            println!("Aggressive speedup: {:.2}x", speedup);
        }
        
        if conservative_time > Duration::from_secs(0) {
            let slowdown = conservative_time.as_secs_f64() / standard_time.as_secs_f64();
            println!("Conservative slowdown: {:.2}x", slowdown);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qwiic_relay_rs::QwiicRelayConfig;

    #[test]
    fn test_timing_config_variations() {
        // Test that different configs produce different timing values
        let standard = QwiicRelayConfig::new(4);
        let solid_state = QwiicRelayConfig::for_solid_state(4);
        let mechanical = QwiicRelayConfig::for_mechanical(4);
        
        assert_ne!(standard.write_delay_us, solid_state.write_delay_us);
        assert_ne!(standard.write_delay_us, mechanical.write_delay_us);
        assert_ne!(solid_state.write_delay_us, mechanical.write_delay_us);
        
        assert!(solid_state.write_delay_us < standard.write_delay_us);
        assert!(standard.write_delay_us < mechanical.write_delay_us);
    }

    #[test]
    fn test_timing_ranges() {
        // Test that timing values are within reasonable ranges
        let configs = vec![
            QwiicRelayConfig::new(4),
            QwiicRelayConfig::for_solid_state(4),
            QwiicRelayConfig::for_mechanical(4),
            QwiicRelayConfig::with_timing(4, 1, 1, 50),
            QwiicRelayConfig::with_timing(4, 100, 100, 1000),
        ];
        
        for config in configs {
            // Write delay should be between 1μs and 1000μs (1ms)
            assert!(config.write_delay_us >= 1 && config.write_delay_us <= 1000);
            
            // State change delay should be between 1ms and 1000ms (1s)
            assert!(config.state_change_delay_ms >= 1 && config.state_change_delay_ms <= 1000);
            
            // Init delay should be between 50ms and 5000ms (5s)
            assert!(config.init_delay_ms >= 50 && config.init_delay_ms <= 5000);
        }
    }
}