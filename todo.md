# TODO List

## Future Improvements

### High Priority
- [ ] Add support for changing relay hardware address (partially implemented in roadmap)
- [ ] Add async/await support for non-blocking I2C operations
- [ ] Create more comprehensive integration tests with mock I2C devices
- [ ] Add retry logic for I2C communication failures

### Medium Priority
- [ ] Add support for relay board discovery (scan I2C addresses)
- [ ] Implement relay state caching to reduce I2C reads
- [ ] Add configuration validation (ensure relay_num is within bounds)
- [ ] Create builder pattern for QwiicRelay initialization
- [ ] Add support for relay pulse mode (on for X milliseconds)

### Low Priority
- [ ] Add examples directory with more use cases
- [ ] Create benchmarks for I2C operations
- [ ] Add support for relay board diagnostics
- [ ] Implement debug trait with better formatting
- [ ] Add logging support with `log` crate
- [ ] Create CLI tool for relay control
- [ ] Add support for relay sequencing/patterns

### Documentation
- [ ] Add hardware setup guide in documentation
- [ ] Create troubleshooting guide for common I2C issues
- [ ] Add wiring diagrams for different board types
- [ ] Document power requirements and limitations

### Testing
- [ ] Add property-based tests with `proptest`
- [ ] Create test fixtures for different relay board types
- [ ] Add stress tests for rapid relay switching
- [ ] Implement mock I2C device for CI testing

### Code Quality
- [ ] Consider using `const` generics for relay count
- [ ] Evaluate using typestate pattern for relay states
- [ ] Add `#[must_use]` attributes where appropriate
- [ ] Consider making enums non-exhaustive for future compatibility