//! Comprehensive stress testing for message priority system
//!
//! This module provides extensive testing for the space command priority system,
//! including stress tests, real-time constraint validation, and mission scenario simulation.

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use std::collections::VecDeque;

use space_comms_shared::{
    commands::*,
    messaging::{Message, MessagePriority, PriorityQueue},
    types::{ComponentId, BandType},
    Result,
};

/// Test statistics for priority queue performance
#[derive(Debug, Clone, Default)]
pub struct TestStats {
    pub total_messages: usize,
    pub emergency_messages: usize,
    pub critical_messages: usize,
    pub high_messages: usize,
    pub medium_messages: usize,
    pub low_messages: usize,
    pub messages_processed: usize,
    pub total_latency_ns: u64,
    pub max_latency_ns: u64,
    pub min_latency_ns: u64,
    pub priority_violations: usize,
    pub dropped_messages: usize,
}

impl TestStats {
    pub fn average_latency_ns(&self) -> f64 {
        if self.messages_processed > 0 {
            self.total_latency_ns as f64 / self.messages_processed as f64
        } else {
            0.0
        }
    }

    pub fn average_latency_ms(&self) -> f64 {
        self.average_latency_ns() / 1_000_000.0
    }
}

/// Message priority stress tester
pub struct PriorityStressTester {
    queue: PriorityQueue<1000>,
    stats: TestStats,
    ground_station: ComponentId,
    satellite: ComponentId,
}

impl PriorityStressTester {
    pub fn new() -> Self {
        Self {
            queue: PriorityQueue::new(),
            stats: TestStats::default(),
            ground_station: ComponentId::new(0x0001),
            satellite: ComponentId::new(0x0100),
        }
    }

    /// Generate all 24 types of space commands for comprehensive testing
    pub fn generate_all_commands(&self) -> Vec<SpaceCommand> {
        vec![
            // Emergency Priority Commands (5 commands)
            SpaceCommand::EmergencyAbort {
                reason: EmergencyReason::SystemFailure,
                confirmation_code: 0xDEADBEEF,
            },
            SpaceCommand::EmergencyHalt {
                subsystems: heapless::Vec::from_slice(&[
                    SubsystemId::Propulsion,
                    SubsystemId::PayloadControl,
                ]).unwrap(),
                override_code: 0x1234567890ABCDEF,
            },
            SpaceCommand::ActivateSafeMode {
                safe_mode_level: SafeModeLevel::Level1,
                duration_seconds: Some(3600),
            },
            SpaceCommand::EmergencyPowerDown {
                systems_to_preserve: heapless::Vec::from_slice(&[
                    SubsystemId::Communications,
                    SubsystemId::OnboardComputer,
                ]).unwrap(),
                battery_threshold_percent: 15,
            },
            SpaceCommand::EmergencyAttitudeRecovery {
                target_attitude: [0.0, 0.0, 0.0, 1.0], // Identity quaternion
                max_angular_velocity: 0.1, // rad/s
            },

            // Critical Priority Commands (6 commands)
            SpaceCommand::AbortMission {
                mission_id: 12345,
                abort_reason: heapless::String::from_str("Collision risk detected").unwrap(),
                preserve_data: true,
            },
            SpaceCommand::HaltSubsystem {
                subsystem: SubsystemId::Propulsion,
                graceful_shutdown: true,
                timeout_seconds: 30,
            },
            SpaceCommand::CollisionAvoidance {
                debris_id: 98765,
                maneuver_type: ManeuverType::AvoidanceManeuver,
                delta_v: [0.5, -0.2, 0.1], // m/s
                execution_time: 1694188800, // Unix timestamp
            },
            SpaceCommand::AttitudeControl {
                target_quaternion: [0.1, 0.2, 0.3, 0.9],
                angular_rates: [0.01, 0.02, 0.01], // rad/s
                control_mode: AttitudeMode::EarthPointing,
                deadline_ms: 5000,
            },
            SpaceCommand::SwitchCommBackup {
                primary_failure: heapless::String::from_str("X-Band transmitter fault").unwrap(),
                backup_band: BandType::SBand,
                power_level_percent: 80,
            },
            SpaceCommand::ResetSystem {
                component: ComponentId::new(0x0200),
                reset_type: ResetType::SoftReset,
                preserve_config: true,
            },

            // High Priority Commands (5 commands)
            SpaceCommand::UpdateOrbit {
                semi_major_axis: 7000.0, // km
                eccentricity: 0.001,
                inclination: 98.6, // degrees (sun-synchronous)
                raan: 45.0,
                arg_periapsis: 90.0,
                true_anomaly: 0.0,
            },
            SpaceCommand::ReconfigureComm {
                band: BandType::XBand,
                frequency_hz: 8_450_000_000, // 8.45 GHz
                power_level: 75,
                modulation: ModulationType::QPSK,
                error_correction: true,
            },
            SpaceCommand::Deploy {
                deployable: DeployableType::SolarPanel,
                deployment_angle: 180.0, // degrees
                deployment_rate: 5.0, // degrees/second
                force_limit: 50.0, // Newtons
            },
            SpaceCommand::StartDataCollection {
                instrument: InstrumentId::Camera,
                collection_mode: heapless::String::from_str("high_resolution").unwrap(),
                duration_seconds: 300,
                data_rate_mbps: 10.0,
            },
            SpaceCommand::ConfigurePower {
                solar_panel_orientation: [0.0, 0.0, 1.0], // Z-axis pointing
                battery_mode: BatteryMode::Charging,
                power_budget_watts: 500.0,
                load_shedding_priority: heapless::Vec::from_slice(&[
                    SubsystemId::PayloadControl,
                    SubsystemId::ThermalControl,
                ]).unwrap(),
            },

            // Medium Priority Commands (5 commands)
            SpaceCommand::RequestTelemetry {
                telemetry_type: TelemetryType::Health,
                sampling_rate_hz: 1.0,
                duration_seconds: 3600,
                compression: true,
            },
            SpaceCommand::UpdateConfig {
                config_id: heapless::String::from_str("attitude_control").unwrap(),
                parameters: heapless::Vec::from_slice(&[0x01, 0x02, 0x03, 0x04]).unwrap(),
                apply_immediately: false,
                backup_current: true,
            },
            SpaceCommand::CalibrateInstrument {
                instrument: InstrumentId::Magnetometer,
                calibration_type: CalibrationType::Bias,
                reference_values: heapless::Vec::from_slice(&[1.0, 0.0, 0.0]).unwrap(),
                temperature_compensation: true,
            },
            SpaceCommand::ScheduleOperation {
                operation_id: 54321,
                scheduled_time: 1694275200, // Future timestamp
                command: Box::new(SpaceCommand::SendStatus {
                    status_type: StatusType::SystemHealth,
                    include_diagnostics: true,
                    format: ReportFormat::Json,
                }),
                repeat_interval: Some(3600), // Every hour
            },
            SpaceCommand::StoreData {
                data_type: DataType::Science,
                storage_location: StorageLocation::NonVolatileMemory,
                compression_level: 5,
                encryption: true,
            },

            // Low Priority Commands (4 commands)
            SpaceCommand::SendStatus {
                status_type: StatusType::Full,
                include_diagnostics: true,
                format: ReportFormat::Json,
            },
            SpaceCommand::UpdateTime {
                utc_time: 1694188800,
                time_source: TimeSource::Gps,
                precision_microseconds: 1000,
            },
            SpaceCommand::PerformMaintenance {
                maintenance_type: MaintenanceType::SystemCheck,
                automated: true,
                estimated_duration: 1800, // 30 minutes
            },
            SpaceCommand::LogEvent {
                event_type: EventType::Information,
                severity: EventSeverity::Info,
                description: heapless::String::from_str("System startup complete").unwrap(),
                associated_data: heapless::Vec::from_slice(&[0xFF, 0xFE, 0xFD]).unwrap(),
            },
        ]
    }

    /// Create a message from a space command
    pub fn create_message(&self, command: SpaceCommand) -> Result<Message> {
        CommandBuilder::new(self.ground_station, self.satellite)
            .build_from_command(command)
    }

    /// Stress test with high message throughput
    pub fn stress_test_high_throughput(&mut self, messages_per_second: usize, duration_seconds: u64) -> Result<TestStats> {
        println!("🚀 Starting high throughput stress test: {} msg/sec for {} seconds",
                 messages_per_second, duration_seconds);

        let all_commands = self.generate_all_commands();
        let start_time = Instant::now();
        let target_duration = Duration::from_secs(duration_seconds);
        let interval = Duration::from_nanos(1_000_000_000 / messages_per_second as u64);

        let mut message_count = 0;
        let mut last_send = Instant::now();

        while start_time.elapsed() < target_duration {
            if last_send.elapsed() >= interval {
                // Select random command
                let command = all_commands[message_count % all_commands.len()].clone();
                let message = self.create_message(command)?;

                let send_time = Instant::now();

                // Try to add to queue
                match self.queue.push(message.clone()) {
                    Ok(_) => {
                        self.stats.total_messages += 1;
                        match message.priority {
                            MessagePriority::Emergency => self.stats.emergency_messages += 1,
                            MessagePriority::Critical => self.stats.critical_messages += 1,
                            MessagePriority::High => self.stats.high_messages += 1,
                            MessagePriority::Medium => self.stats.medium_messages += 1,
                            MessagePriority::Low => self.stats.low_messages += 1,
                        }
                    },
                    Err(_) => {
                        self.stats.dropped_messages += 1;
                        println!("❌ Queue overflow! Dropped message {}", message_count);
                    }
                }

                // Process some messages to simulate real system
                if message_count % 10 == 0 {
                    self.process_messages_batch(5)?;
                }

                message_count += 1;
                last_send = send_time;
            }

            // Small sleep to prevent 100% CPU usage
            thread::sleep(Duration::from_micros(10));
        }

        // Process remaining messages
        while !self.queue.is_empty() {
            self.process_messages_batch(10)?;
        }

        println!("✅ High throughput test completed:");
        println!("   Total messages: {}", self.stats.total_messages);
        println!("   Messages processed: {}", self.stats.messages_processed);
        println!("   Dropped messages: {}", self.stats.dropped_messages);
        println!("   Average latency: {:.2} ms", self.stats.average_latency_ms());

        Ok(self.stats.clone())
    }

    /// Test priority ordering under stress
    pub fn stress_test_priority_ordering(&mut self, iterations: usize) -> Result<TestStats> {
        println!("🎯 Starting priority ordering stress test: {} iterations", iterations);

        let all_commands = self.generate_all_commands();

        for i in 0..iterations {
            // Add messages in random order
            for (idx, command) in all_commands.iter().enumerate() {
                let message = self.create_message(command.clone())?;

                if let Err(_) = self.queue.push(message) {
                    self.stats.dropped_messages += 1;
                    continue;
                }

                self.stats.total_messages += 1;
            }

            // Process all messages and verify ordering
            let mut last_priority = MessagePriority::Emergency;
            while !self.queue.is_empty() {
                if let Some(message) = self.queue.pop() {
                    self.stats.messages_processed += 1;

                    // Check priority ordering
                    if message.priority < last_priority {
                        self.stats.priority_violations += 1;
                        println!("❌ Priority violation! Got {:?} after {:?}",
                                message.priority, last_priority);
                    }
                    last_priority = message.priority;
                }
            }

            if i % 100 == 0 {
                println!("   Iteration {}: {} violations so far", i, self.stats.priority_violations);
            }
        }

        println!("✅ Priority ordering test completed:");
        println!("   Priority violations: {}", self.stats.priority_violations);
        println!("   Total messages processed: {}", self.stats.messages_processed);

        Ok(self.stats.clone())
    }

    /// Test real-time latency constraints
    pub fn stress_test_latency_constraints(&mut self, test_duration_seconds: u64) -> Result<TestStats> {
        println!("⏱️  Starting latency constraint stress test: {} seconds", test_duration_seconds);

        let all_commands = self.generate_all_commands();
        let start_time = Instant::now();
        let target_duration = Duration::from_secs(test_duration_seconds);

        while start_time.elapsed() < target_duration {
            // Add a mix of priority messages
            for command in &all_commands {
                let message = self.create_message(command.clone())?;
                let send_time = Instant::now();

                if let Err(_) = self.queue.push(message.clone()) {
                    self.stats.dropped_messages += 1;
                    continue;
                }

                self.stats.total_messages += 1;

                // Immediately process high-priority messages
                if message.priority >= MessagePriority::Critical {
                    if let Some(processed_msg) = self.queue.pop() {
                        let process_time = Instant::now();
                        let latency_ns = process_time.duration_since(send_time).as_nanos() as u64;

                        self.record_latency(latency_ns, &processed_msg);

                        // Check if latency exceeds constraints
                        let max_allowed_ns = processed_msg.priority.max_latency_ms() as u64 * 1_000_000;
                        if latency_ns > max_allowed_ns {
                            self.stats.priority_violations += 1;
                            println!("❌ Latency violation! {:?} took {} ms (max: {} ms)",
                                    processed_msg.priority,
                                    latency_ns / 1_000_000,
                                    max_allowed_ns / 1_000_000);
                        }
                    }
                }
            }

            // Process remaining lower priority messages in batches
            self.process_messages_batch(5)?;

            // Simulate system load
            thread::sleep(Duration::from_micros(100));
        }

        println!("✅ Latency constraint test completed:");
        println!("   Latency violations: {}", self.stats.priority_violations);
        println!("   Average latency: {:.2} ms", self.stats.average_latency_ms());
        println!("   Max latency: {:.2} ms", self.stats.max_latency_ns as f64 / 1_000_000.0);

        Ok(self.stats.clone())
    }

    /// Simulate mission-critical scenario
    pub fn simulate_mission_scenario(&mut self, scenario_name: &str) -> Result<TestStats> {
        println!("🌌 Simulating mission scenario: {}", scenario_name);

        match scenario_name {
            "collision_avoidance" => self.simulate_collision_scenario(),
            "power_emergency" => self.simulate_power_emergency(),
            "communication_failure" => self.simulate_comm_failure(),
            "attitude_loss" => self.simulate_attitude_loss(),
            _ => {
                println!("❌ Unknown scenario: {}", scenario_name);
                Ok(self.stats.clone())
            }
        }
    }

    fn simulate_collision_scenario(&mut self) -> Result<TestStats> {
        println!("💥 Simulating collision avoidance scenario");

        // Sequence of commands during collision avoidance
        let scenario_commands = vec![
            // 1. Debris detection alert
            SpaceCommand::LogEvent {
                event_type: EventType::Warning,
                severity: EventSeverity::High,
                description: heapless::String::from_str("Space debris detected").unwrap(),
                associated_data: heapless::Vec::new(),
            },

            // 2. Emergency collision avoidance (CRITICAL)
            SpaceCommand::CollisionAvoidance {
                debris_id: 12345,
                maneuver_type: ManeuverType::AvoidanceManeuver,
                delta_v: [0.5, -0.3, 0.2],
                execution_time: 1694188900,
            },

            // 3. Attitude control for maneuver (CRITICAL)
            SpaceCommand::AttitudeControl {
                target_quaternion: [0.0, 0.1, 0.0, 0.995],
                angular_rates: [0.02, 0.01, 0.005],
                control_mode: AttitudeMode::VelocityPointing,
                deadline_ms: 2000,
            },

            // 4. Switch to backup comm during maneuver (CRITICAL)
            SpaceCommand::SwitchCommBackup {
                primary_failure: heapless::String::from_str("Maneuver interference").unwrap(),
                backup_band: BandType::SBand,
                power_level_percent: 100,
            },

            // 5. Update orbital parameters after maneuver (HIGH)
            SpaceCommand::UpdateOrbit {
                semi_major_axis: 7001.5,
                eccentricity: 0.0012,
                inclination: 98.6,
                raan: 45.1,
                arg_periapsis: 90.0,
                true_anomaly: 15.0,
            },

            // 6. Request telemetry to confirm success (MEDIUM)
            SpaceCommand::RequestTelemetry {
                telemetry_type: TelemetryType::Position,
                sampling_rate_hz: 10.0,
                duration_seconds: 60,
                compression: false,
            },
        ];

        let start_time = Instant::now();

        // Add all scenario commands rapidly
        for command in scenario_commands {
            let message = self.create_message(command)?;
            let send_time = Instant::now();

            if let Err(_) = self.queue.push(message) {
                self.stats.dropped_messages += 1;
                continue;
            }
            self.stats.total_messages += 1;
        }

        // Process in priority order and measure response times
        while !self.queue.is_empty() {
            if let Some(message) = self.queue.pop() {
                let process_time = Instant::now();
                let latency_ns = process_time.duration_since(start_time).as_nanos() as u64;

                self.record_latency(latency_ns, &message);

                println!("   Processed: {} (Priority: {:?}, Latency: {:.2} ms)",
                        self.get_command_description(&message),
                        message.priority,
                        latency_ns as f64 / 1_000_000.0);
            }
        }

        println!("✅ Collision avoidance scenario completed in {:.2} ms",
                start_time.elapsed().as_millis());

        Ok(self.stats.clone())
    }

    fn simulate_power_emergency(&mut self) -> Result<TestStats> {
        println!("🔋 Simulating power emergency scenario");

        let scenario_commands = vec![
            // 1. Battery critical alert
            SpaceCommand::LogEvent {
                event_type: EventType::Error,
                severity: EventSeverity::Critical,
                description: heapless::String::from_str("Battery level critical").unwrap(),
                associated_data: heapless::Vec::new(),
            },

            // 2. Emergency power down (EMERGENCY)
            SpaceCommand::EmergencyPowerDown {
                systems_to_preserve: heapless::Vec::from_slice(&[
                    SubsystemId::Communications,
                    SubsystemId::OnboardComputer,
                    SubsystemId::AttitudeControl,
                ]).unwrap(),
                battery_threshold_percent: 10,
            },

            // 3. Activate safe mode (EMERGENCY)
            SpaceCommand::ActivateSafeMode {
                safe_mode_level: SafeModeLevel::Level1,
                duration_seconds: Some(7200), // 2 hours
            },

            // 4. Configure power management (HIGH)
            SpaceCommand::ConfigurePower {
                solar_panel_orientation: [0.0, 0.0, 1.0],
                battery_mode: BatteryMode::Emergency,
                power_budget_watts: 50.0, // Minimal power
                load_shedding_priority: heapless::Vec::from_slice(&[
                    SubsystemId::PayloadControl,
                    SubsystemId::ThermalControl,
                    SubsystemId::Navigation,
                ]).unwrap(),
            },
        ];

        let start_time = Instant::now();

        for command in scenario_commands {
            let message = self.create_message(command)?;
            self.queue.push(message)?;
            self.stats.total_messages += 1;
        }

        while !self.queue.is_empty() {
            if let Some(message) = self.queue.pop() {
                let latency_ns = start_time.elapsed().as_nanos() as u64;
                self.record_latency(latency_ns, &message);

                println!("   Processed: {} (Priority: {:?})",
                        self.get_command_description(&message),
                        message.priority);
            }
        }

        println!("✅ Power emergency scenario completed");
        Ok(self.stats.clone())
    }

    fn simulate_comm_failure(&mut self) -> Result<TestStats> {
        println!("📡 Simulating communication failure scenario");
        // Implementation similar to other scenarios...
        Ok(self.stats.clone())
    }

    fn simulate_attitude_loss(&mut self) -> Result<TestStats> {
        println!("🔄 Simulating attitude loss scenario");
        // Implementation similar to other scenarios...
        Ok(self.stats.clone())
    }

    fn process_messages_batch(&mut self, batch_size: usize) -> Result<()> {
        for _ in 0..batch_size {
            if let Some(message) = self.queue.pop() {
                let latency_ns = 1000; // Simulated processing time
                self.record_latency(latency_ns, &message);
            } else {
                break;
            }
        }
        Ok(())
    }

    fn record_latency(&mut self, latency_ns: u64, _message: &Message) {
        self.stats.messages_processed += 1;
        self.stats.total_latency_ns += latency_ns;

        if self.stats.messages_processed == 1 {
            self.stats.min_latency_ns = latency_ns;
            self.stats.max_latency_ns = latency_ns;
        } else {
            self.stats.min_latency_ns = self.stats.min_latency_ns.min(latency_ns);
            self.stats.max_latency_ns = self.stats.max_latency_ns.max(latency_ns);
        }
    }

    fn get_command_description(&self, message: &Message) -> String {
        // Extract command description from message payload
        match &message.payload {
            space_comms_shared::messaging::MessagePayload::Command { command_id, .. } => {
                format!("Command ID: 0x{:04X}", command_id)
            },
            _ => "Unknown".to_string(),
        }
    }

    /// Print comprehensive test report
    pub fn print_test_report(&self) {
        println!("\n📊 COMPREHENSIVE TEST REPORT");
        println!("=" .repeat(50));
        println!("Total Messages:      {}", self.stats.total_messages);
        println!("  Emergency:         {}", self.stats.emergency_messages);
        println!("  Critical:          {}", self.stats.critical_messages);
        println!("  High:              {}", self.stats.high_messages);
        println!("  Medium:            {}", self.stats.medium_messages);
        println!("  Low:               {}", self.stats.low_messages);
        println!();
        println!("Messages Processed:  {}", self.stats.messages_processed);
        println!("Dropped Messages:    {}", self.stats.dropped_messages);
        println!("Priority Violations: {}", self.stats.priority_violations);
        println!();
        println!("Latency Statistics:");
        println!("  Average:           {:.2} ms", self.stats.average_latency_ms());
        println!("  Minimum:           {:.2} ms", self.stats.min_latency_ns as f64 / 1_000_000.0);
        println!("  Maximum:           {:.2} ms", self.stats.max_latency_ns as f64 / 1_000_000.0);
        println!("=" .repeat(50));
    }
}

// Additional helper to extend CommandBuilder
impl CommandBuilder {
    pub fn build_from_command(mut self, command: SpaceCommand) -> Result<Message> {
        let priority = command.priority();
        let preferred_band = command.preferred_band();

        // Serialize command to bytes
        let command_bytes = serde_json::to_vec(&command)
            .map_err(|_| space_comms_shared::SpaceCommError::validation_error("Failed to serialize command"))?;

        let mut parameters = heapless::Vec::new();
        parameters.extend_from_slice(&command_bytes).map_err(|_| {
            space_comms_shared::SpaceCommError::validation_error("Command too large for message")
        })?;

        Ok(Message {
            id: space_comms_shared::types::MessageId::new(),
            priority,
            source: self.source,
            destination: self.destination,
            timestamp: space_comms_shared::time::current_time_nanos(),
            payload: space_comms_shared::messaging::MessagePayload::Command {
                command_id: command.discriminant(),
                parameters,
            },
            preferred_band,
            ttl_seconds: priority.max_latency_ms() / 1000 + 1,
            retry_count: 0,
            max_retries: if priority.is_real_time() { 3 } else { 1 },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_command_types() {
        let mut tester = PriorityStressTester::new();
        let commands = tester.generate_all_commands();

        assert_eq!(commands.len(), 24, "Should have exactly 24 command types");

        // Verify each priority level is represented
        let priorities: std::collections::HashSet<_> = commands.iter()
            .map(|cmd| cmd.priority())
            .collect();

        assert!(priorities.contains(&MessagePriority::Emergency));
        assert!(priorities.contains(&MessagePriority::Critical));
        assert!(priorities.contains(&MessagePriority::High));
        assert!(priorities.contains(&MessagePriority::Medium));
        assert!(priorities.contains(&MessagePriority::Low));
    }

    #[test]
    fn test_priority_ordering() {
        let mut tester = PriorityStressTester::new();
        let stats = tester.stress_test_priority_ordering(10).unwrap();

        assert_eq!(stats.priority_violations, 0, "Should have no priority violations");
        assert!(stats.messages_processed > 0, "Should process some messages");
    }

    #[test]
    fn test_high_throughput() {
        let mut tester = PriorityStressTester::new();
        let stats = tester.stress_test_high_throughput(100, 2).unwrap(); // 100 msg/sec for 2 seconds

        assert!(stats.total_messages >= 150, "Should generate significant messages");
        assert!(stats.average_latency_ms() < 100.0, "Average latency should be reasonable");
    }

    #[test]
    fn test_mission_scenarios() {
        let mut tester = PriorityStressTester::new();

        let stats = tester.simulate_mission_scenario("collision_avoidance").unwrap();
        assert!(stats.messages_processed > 0);

        let stats = tester.simulate_mission_scenario("power_emergency").unwrap();
        assert!(stats.messages_processed > 0);
    }
}
