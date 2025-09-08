# Space Communication Priority System - Testing Guide

## 🚀 Overview

This document provides comprehensive instructions for building, testing, and validating the **Space Communication Priority System** built in Rust. The system implements 24 different mission-critical space commands with a 5-tier priority system designed for real-time space operations.

## 📋 Prerequisites

### Required Software

1. **Rust Toolchain (1.70+)**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   ```

2. **Embedded Targets (for satellite deployment)**
   ```bash
   rustup target add thumbv7em-none-eabihf
   rustup target add aarch64-unknown-linux-gnu
   ```

3. **Development Tools**
   ```bash
   rustup component add rustfmt clippy
   cargo install cargo-audit cargo-tarpaulin cargo-embed
   ```

### System Requirements
- **Memory**: Minimum 8GB RAM (16GB recommended)
- **Storage**: 5GB free space for build artifacts
- **OS**: Linux (Ubuntu 20.04+), macOS 12+, Windows 11+

## 🎯 Priority System Architecture

### Command Priority Levels

| Priority | Latency Constraint | Frequency | Use Case |
|----------|-------------------|-----------|----------|
| **Emergency** | <1ms | 2000 Hz | Life-safety, mission-critical |
| **Critical** | <10ms | 1000 Hz | Collision avoidance, system failures |
| **High** | <100ms | 500 Hz | Orbital maneuvers, deployments |
| **Medium** | <1000ms | 100 Hz | Configuration, telemetry requests |
| **Low** | <10000ms | 10 Hz | Status reports, maintenance |

### Complete Command Set (24 Commands)

#### Emergency Priority (5 commands)
1. **EmergencyAbort** - Immediate termination of all operations
2. **EmergencyHalt** - Hard stop of all satellite operations
3. **ActivateSafeMode** - Minimal power configuration
4. **EmergencyPowerDown** - Shutdown non-critical systems
5. **EmergencyAttitudeRecovery** - Spin stabilization recovery

#### Critical Priority (6 commands)
6. **AbortMission** - Terminate current mission sequence
7. **HaltSubsystem** - Stop specific subsystem operation
8. **CollisionAvoidance** - Execute avoidance maneuver
9. **AttitudeControl** - Immediate attitude adjustment
10. **SwitchCommBackup** - Failover to backup communication
11. **ResetSystem** - Component reset and recovery

#### High Priority (5 commands)
12. **UpdateOrbit** - Modify orbital parameters
13. **ReconfigureComm** - Change communication settings
14. **Deploy** - Deploy solar panels or antenna
15. **StartDataCollection** - Begin science operations
16. **ConfigurePower** - Power management configuration

#### Medium Priority (5 commands)
17. **RequestTelemetry** - Data collection request
18. **UpdateConfig** - Software configuration update
19. **CalibrateInstrument** - Sensor calibration
20. **ScheduleOperation** - Future operation scheduling
21. **StoreData** - Data storage operation

#### Low Priority (3 commands)
22. **SendStatus** - Status report transmission
23. **UpdateTime** - Time synchronization
24. **PerformMaintenance** - Routine maintenance
25. **LogEvent** - System event logging

## 🔧 Building the System

### 1. Quick Build
```bash
cd rust-workspace
cargo build --workspace
```

### 2. Release Build
```bash
cargo build --workspace --release
```

### 3. Embedded Build (Satellite)
```bash
cd satellite
cargo build --target thumbv7em-none-eabihf
```

### 4. Complete Build & Test Script
```bash
chmod +x build_and_test.sh
./build_and_test.sh
```

## 🧪 Testing Framework

### 1. Unit Tests
```bash
cargo test --workspace --lib
```

### 2. Integration Tests
```bash
cargo test --workspace --test "*"
```

### 3. Priority System Stress Tests
```bash
cargo test --test priority_stress_tests -- --nocapture
```

### 4. Interactive Demo
```bash
cargo run --example priority_demo
```

## 🚀 Stress Testing Scenarios

### High Throughput Testing
Tests message processing under extreme load:
- **50 msg/sec** for 3 seconds (baseline)
- **100 msg/sec** for 3 seconds (normal operations)
- **200 msg/sec** for 2 seconds (peak load)
- **500 msg/sec** for 1 second (emergency burst)

### Priority Ordering Verification
Validates that messages are processed in correct priority order:
```rust
// Emergency messages always processed first
// Critical messages before High priority
// FIFO ordering within same priority level
```

### Latency Constraint Testing
Ensures real-time requirements are met:
- Emergency: <1ms processing time
- Critical: <10ms processing time
- High: <100ms processing time
- Medium: <1000ms processing time
- Low: <10000ms processing time

### Mission Scenario Simulations

#### 1. Collision Avoidance Scenario
```
1. Debris detection alert (LOW)
2. Emergency collision avoidance (CRITICAL)
3. Attitude control for maneuver (CRITICAL)
4. Switch to backup comm (CRITICAL)
5. Update orbital parameters (HIGH)
6. Request telemetry confirmation (MEDIUM)
```

#### 2. Power Emergency Scenario
```
1. Battery critical alert (LOW)
2. Emergency power down (EMERGENCY)
3. Activate safe mode (EMERGENCY)
4. Configure power management (HIGH)
```

#### 3. Communication Failure Scenario
```
1. Primary comm failure detection (MEDIUM)
2. Switch to backup system (CRITICAL)
3. Reconfigure backup band (HIGH)
4. Test communication link (MEDIUM)
```

#### 4. Attitude Loss Scenario
```
1. Attitude loss detection (HIGH)
2. Emergency attitude recovery (EMERGENCY)
3. Activate reaction wheels (CRITICAL)
4. Fine attitude adjustment (HIGH)
```

## 📊 Performance Metrics

### Target Performance
- **Queue Capacity**: 1000 messages
- **Processing Rate**: Up to 2000 messages/second
- **Memory Usage**: <64KB RAM for embedded satellite
- **Flash Usage**: <256KB for satellite firmware
- **Latency**: 99.9% of messages meet priority constraints

### Validation Criteria
- ✅ Zero priority violations under normal load
- ✅ <1% message drops under 500 msg/sec load
- ✅ All emergency commands processed within 1ms
- ✅ All critical commands processed within 10ms
- ✅ Perfect FIFO ordering within priority levels

## 🛡️ Security Features

### Command Authentication
- **Confirmation codes** for emergency commands
- **Digital signatures** for critical operations
- **Replay protection** with timestamps
- **Source validation** for all commands

### Communication Security
- **Post-quantum cryptography** (CRYSTALS-Kyber/Dilithium)
- **AES-256-GCM** for payload encryption
- **HMAC verification** for message integrity
- **Band-specific security** levels

## 🔬 Advanced Testing

### Property-Based Testing
```bash
cargo test --features "proptest" -- property_tests
```

### Fault Injection Testing
```bash
cargo test --features "fault-injection" -- fault_tests
```

### Hardware-in-the-Loop Testing
```bash
cargo test --features "hardware-sim" -- hil_tests
```

### Benchmarking
```bash
cargo bench --workspace
```

## 🚀 Deployment Instructions

### Ground Station Deployment
```bash
cargo run --bin space-comms-ground
```

### Mission Control Deployment
```bash
cargo run --bin mission-control
```

### Satellite Firmware Deployment
```bash
cd satellite
cargo embed --target thumbv7em-none-eabihf
```

### Docker Deployment
```bash
docker-compose up --build
```

## 📈 Performance Validation

### Expected Test Results

#### Priority Stress Test
```
✅ Total messages: 2400+
✅ Priority violations: 0
✅ Average latency: <5ms
✅ Max latency: <50ms
✅ Success rate: >99%
```

#### High Throughput Test
```
✅ 500 msg/sec sustained for 1 second
✅ <1% message drops
✅ Queue never full
✅ Real-time constraints maintained
```

#### Mission Scenarios
```
✅ Collision avoidance: <100ms total response
✅ Power emergency: <50ms safe mode activation
✅ Comm failure: <200ms backup activation
✅ Attitude loss: <10ms recovery initiation
```

## 🐛 Troubleshooting

### Common Issues

1. **Build Failures**
   ```bash
   # Clean and rebuild
   cargo clean
   cargo build --workspace
   ```

2. **Test Failures**
   ```bash
   # Run specific test with output
   cargo test test_name -- --nocapture
   ```

3. **Embedded Build Issues**
   ```bash
   # Check target installation
   rustup target list --installed
   rustup target add thumbv7em-none-eabihf
   ```

4. **Memory Issues**
   ```bash
   # Check memory usage
   cargo build --release
   # Reduce queue size in configuration
   ```

### Debug Mode
```bash
RUST_LOG=debug cargo test -- --nocapture
```

## 📚 Documentation

### Generate Documentation
```bash
cargo doc --workspace --open
```

### Code Coverage
```bash
cargo tarpaulin --workspace --out Html
```

### Security Audit
```bash
cargo audit
```

## 🌟 Success Criteria

A successful test run should demonstrate:

1. ✅ All 24 command types correctly prioritized
2. ✅ Zero priority violations under stress
3. ✅ Real-time latency constraints met
4. ✅ High throughput handling (>200 msg/sec)
5. ✅ Mission scenarios execute flawlessly
6. ✅ Embedded compatibility verified
7. ✅ Memory constraints satisfied (<64KB)
8. ✅ Security features operational

## 🚀 Ready for Space!

When all tests pass, the system is ready for:
- **Laboratory validation** with hardware-in-the-loop
- **Integration testing** with actual satellite hardware
- **Mission simulation** with ground station equipment
- **Space deployment** on real satellite missions

---

**🌌 The future of space communication is here - built with Rust! 🦀🚀**
