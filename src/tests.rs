use crate::*;
use i2cdev::linux::LinuxI2CError;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[cfg(test)]
mod verification_tests {
    use super::*;

    #[test]
    fn test_verification_config_default() {
        let config = VerificationConfig::default();
        assert!(matches!(config.mode, VerificationMode::Strict));
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.retry_delay_ms, 50);
        assert_eq!(config.verification_delay_ms, 20);
        assert_eq!(config.timeout_ms, 1000);
    }

    #[test]
    fn test_verification_config_strict() {
        let config = VerificationConfig::strict();
        assert!(matches!(config.mode, VerificationMode::Strict));
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_verification_config_lenient() {
        let config = VerificationConfig::lenient();
        assert!(matches!(config.mode, VerificationMode::Lenient));
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.retry_delay_ms, 100);
        assert_eq!(config.verification_delay_ms, 50);
        assert_eq!(config.timeout_ms, 2000);
    }

    #[test]
    fn test_verification_config_disabled() {
        let config = VerificationConfig::disabled();
        assert!(matches!(config.mode, VerificationMode::Disabled));
        assert_eq!(config.max_retries, 0);
        assert_eq!(config.retry_delay_ms, 0);
        assert_eq!(config.verification_delay_ms, 0);
        assert_eq!(config.timeout_ms, 0);
    }

    #[test]
    fn test_verification_config_builder() {
        let config = VerificationConfig::default()
            .with_mode(VerificationMode::Lenient)
            .with_max_retries(10)
            .with_retry_delay(200)
            .with_verification_delay(100)
            .with_timeout(5000);

        assert!(matches!(config.mode, VerificationMode::Lenient));
        assert_eq!(config.max_retries, 10);
        assert_eq!(config.retry_delay_ms, 200);
        assert_eq!(config.verification_delay_ms, 100);
        assert_eq!(config.timeout_ms, 5000);
    }

    #[test]
    fn test_verification_config_durations() {
        let config = VerificationConfig::default()
            .with_retry_delay(150)
            .with_verification_delay(75)
            .with_timeout(3000);

        assert_eq!(config.retry_delay(), Duration::from_millis(150));
        assert_eq!(config.verification_delay(), Duration::from_millis(75));
        assert_eq!(config.timeout(), Duration::from_millis(3000));
    }

    #[test]
    fn test_relay_config_with_verification() {
        let verification = VerificationConfig::lenient();
        let config = QwiicRelayConfig::with_verification(2, verification);
        
        assert_eq!(config.relay_count, 2);
        assert!(matches!(config.verification.mode, VerificationMode::Lenient));
    }

    #[test]
    fn test_relay_error_display() {
        let i2c_error = RelayError::I2C(LinuxI2CError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Test error"
        )));
        assert!(format!("{}", i2c_error).contains("I2C error"));

        let verification_error = RelayError::StateVerificationFailed {
            relay_num: Some(2),
            expected: true,
            actual: false,
            attempts: 3,
        };
        let msg = format!("{}", verification_error);
        assert!(msg.contains("relay 2"));
        assert!(msg.contains("expected ON"));
        assert!(msg.contains("got OFF"));
        assert!(msg.contains("3 attempts"));

        let verification_error_no_relay = RelayError::StateVerificationFailed {
            relay_num: None,
            expected: false,
            actual: true,
            attempts: 1,
        };
        let msg = format!("{}", verification_error_no_relay);
        assert!(msg.contains("relay"));
        assert!(msg.contains("expected OFF"));
        assert!(msg.contains("got ON"));

        let timeout_error = RelayError::Timeout {
            relay_num: Some(3),
            operation: "set_relay_on".to_string(),
            duration_ms: 1500,
        };
        let msg = format!("{}", timeout_error);
        assert!(msg.contains("relay 3"));
        assert!(msg.contains("set_relay_on"));
        assert!(msg.contains("1500ms"));

        let config_error = RelayError::InvalidConfiguration("Test error".to_string());
        assert_eq!(format!("{}", config_error), "Invalid configuration: Test error");
    }

    #[test]
    fn test_relay_error_from_i2c() {
        let i2c_err = LinuxI2CError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Test error"
        ));
        let relay_err: RelayError = i2c_err.into();
        assert!(matches!(relay_err, RelayError::I2C(_)));
    }
}

#[cfg(test)]
mod mock_tests {
    use super::*;

    // Mock I2C device for testing
    struct MockI2CDevice {
        relay_states: Arc<Mutex<[bool; 5]>>, // Index 0 unused, 1-4 for relays
        fail_count: Arc<Mutex<u8>>,
        should_fail_toggle: bool,
    }

    impl MockI2CDevice {
        fn new() -> Self {
            MockI2CDevice {
                relay_states: Arc::new(Mutex::new([false; 5])),
                fail_count: Arc::new(Mutex::new(0)),
                should_fail_toggle: false,
            }
        }

        fn with_failures(fail_count: u8) -> Self {
            MockI2CDevice {
                relay_states: Arc::new(Mutex::new([false; 5])),
                fail_count: Arc::new(Mutex::new(fail_count)),
                should_fail_toggle: true,
            }
        }

        fn toggle_relay(&self, relay_num: u8) -> Result<(), RelayError> {
            if self.should_fail_toggle {
                let mut count = self.fail_count.lock().unwrap();
                if *count > 0 {
                    *count -= 1;
                    // Simulate failure - don't actually toggle
                    return Ok(());
                }
            }

            let mut states = self.relay_states.lock().unwrap();
            states[relay_num as usize] = !states[relay_num as usize];
            Ok(())
        }

        fn get_state(&self, relay_num: u8) -> bool {
            let states = self.relay_states.lock().unwrap();
            states[relay_num as usize]
        }

        fn set_state(&self, relay_num: u8, state: bool) -> Result<(), RelayError> {
            let states = self.relay_states.lock().unwrap();
            let current = states[relay_num as usize];
            drop(states);
            
            if current != state {
                self.toggle_relay(relay_num)?;
            }
            Ok(())
        }
    }

    #[test]
    fn test_successful_relay_on_with_verification() {
        let mock_device = MockI2CDevice::new();
        
        // Simulate turning relay 1 on
        assert!(!mock_device.get_state(1));
        mock_device.set_state(1, true).unwrap();
        assert!(mock_device.get_state(1));
    }

    #[test]
    fn test_successful_relay_off_with_verification() {
        let mock_device = MockI2CDevice::new();
        
        // Set relay 1 to on first
        mock_device.set_state(1, true).unwrap();
        assert!(mock_device.get_state(1));
        
        // Turn it off
        mock_device.set_state(1, false).unwrap();
        assert!(!mock_device.get_state(1));
    }

    #[test]
    fn test_retry_logic_succeeds_after_failures() {
        let mock_device = MockI2CDevice::with_failures(2);
        
        // Should fail twice, then succeed on third attempt
        assert!(!mock_device.get_state(1));
        
        // First two toggles will be ignored due to failures
        mock_device.toggle_relay(1).unwrap();
        assert!(!mock_device.get_state(1)); // Still off
        
        mock_device.toggle_relay(1).unwrap();
        assert!(!mock_device.get_state(1)); // Still off
        
        // Third toggle should work
        mock_device.toggle_relay(1).unwrap();
        assert!(mock_device.get_state(1)); // Now on
    }

    #[test]
    fn test_timeout_simulation() {
        // Test that timeout calculation works
        let config = VerificationConfig::default()
            .with_timeout(100); // 100ms timeout
        
        let start = Instant::now();
        thread::sleep(Duration::from_millis(150));
        
        assert!(start.elapsed() > config.timeout());
    }

    #[test]
    fn test_state_verification_error_creation() {
        let error = RelayError::StateVerificationFailed {
            relay_num: Some(1),
            expected: true,
            actual: false,
            attempts: 3,
        };
        
        match error {
            RelayError::StateVerificationFailed { relay_num, expected, actual, attempts } => {
                assert_eq!(relay_num, Some(1));
                assert_eq!(expected, true);
                assert_eq!(actual, false);
                assert_eq!(attempts, 3);
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_timeout_error_creation() {
        let error = RelayError::Timeout {
            relay_num: Some(2),
            operation: "test_op".to_string(),
            duration_ms: 1000,
        };
        
        match error {
            RelayError::Timeout { relay_num, operation, duration_ms } => {
                assert_eq!(relay_num, Some(2));
                assert_eq!(operation, "test_op");
                assert_eq!(duration_ms, 1000);
            }
            _ => panic!("Wrong error type"),
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    #[ignore] // Requires actual hardware
    fn test_relay_on_with_strict_verification() {
        let verification = VerificationConfig::strict();
        let config = QwiicRelayConfig::with_verification(4, verification);
        let mut relay = QwiicRelay::new(config, "/dev/i2c-1", 0x08)
            .expect("Failed to create relay");

        // Turn relay 1 on with verification
        relay.set_relay_on(Some(1))
            .expect("Failed to turn relay on with verification");

        // Verify it's actually on
        let state = relay.get_relay_state(Some(1))
            .expect("Failed to get relay state");
        assert!(state, "Relay should be on after verified set_relay_on");

        // Turn it off
        relay.set_relay_off(Some(1))
            .expect("Failed to turn relay off with verification");

        // Verify it's actually off
        let state = relay.get_relay_state(Some(1))
            .expect("Failed to get relay state");
        assert!(!state, "Relay should be off after verified set_relay_off");
    }

    #[test]
    #[ignore] // Requires actual hardware
    fn test_relay_with_lenient_verification() {
        let verification = VerificationConfig::lenient();
        let config = QwiicRelayConfig::with_verification(4, verification);
        let mut relay = QwiicRelay::new(config, "/dev/i2c-1", 0x08)
            .expect("Failed to create relay");

        // Test with lenient mode - more retries and longer delays
        for i in 1..=4 {
            relay.set_relay_on(Some(i))
                .expect(&format!("Failed to turn relay {} on", i));
            
            thread::sleep(Duration::from_millis(100));
            
            relay.set_relay_off(Some(i))
                .expect(&format!("Failed to turn relay {} off", i));
        }
    }

    #[test]
    #[ignore] // Requires actual hardware
    fn test_relay_with_disabled_verification() {
        let verification = VerificationConfig::disabled();
        let config = QwiicRelayConfig::with_verification(4, verification);
        let mut relay = QwiicRelay::new(config, "/dev/i2c-1", 0x08)
            .expect("Failed to create relay");

        // With disabled verification, operations should be faster
        let start = Instant::now();
        
        for _ in 0..10 {
            relay.set_relay_on(Some(1)).expect("Failed to turn on");
            relay.set_relay_off(Some(1)).expect("Failed to turn off");
        }
        
        let elapsed = start.elapsed();
        println!("20 operations without verification took {:?}", elapsed);
        
        // Compare with verified operations
        let verification = VerificationConfig::strict();
        let config = QwiicRelayConfig::with_verification(4, verification);
        let mut relay_verified = QwiicRelay::new(config, "/dev/i2c-1", 0x08)
            .expect("Failed to create relay");
        
        let start = Instant::now();
        
        for _ in 0..10 {
            relay_verified.set_relay_on(Some(1)).expect("Failed to turn on");
            relay_verified.set_relay_off(Some(1)).expect("Failed to turn off");
        }
        
        let elapsed_verified = start.elapsed();
        println!("20 operations with verification took {:?}", elapsed_verified);
        
        // Verified operations should take longer due to verification delays
        assert!(elapsed_verified > elapsed);
    }

    #[test]
    #[ignore] // Requires actual hardware
    fn test_all_relays_with_verification() {
        let verification = VerificationConfig::default()
            .with_max_retries(2)
            .with_verification_delay(30);
        let config = QwiicRelayConfig::with_verification(4, verification);
        let mut relay = QwiicRelay::new(config, "/dev/i2c-1", 0x08)
            .expect("Failed to create relay");

        // Test all relays operations (these don't have individual verification yet)
        relay.set_all_relays_on()
            .expect("Failed to turn all relays on");
        thread::sleep(Duration::from_millis(500));

        // Verify all are on
        for i in 1..=4 {
            let state = relay.get_relay_state(Some(i))
                .expect(&format!("Failed to get state of relay {}", i));
            assert!(state, "Relay {} should be on", i);
        }

        relay.set_all_relays_off()
            .expect("Failed to turn all relays off");
        thread::sleep(Duration::from_millis(500));

        // Verify all are off
        for i in 1..=4 {
            let state = relay.get_relay_state(Some(i))
                .expect(&format!("Failed to get state of relay {}", i));
            assert!(!state, "Relay {} should be off", i);
        }
    }
}