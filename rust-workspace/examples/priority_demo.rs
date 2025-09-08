//! Space Communication Priority System Demo
//!
//! This application demonstrates the comprehensive space command priority system
//! with real-time stress testing and mission scenario simulation.

use std::time::Duration;
use std::thread;

use space_comms_shared::{
    commands::*,
    messaging::{MessagePriority, PriorityQueue},
    types::{ComponentId, BandType},
    Result,
};

mod priority_stress_tests;
use priority_stress_tests::PriorityStressTester;

fn main() -> Result<()> {
    println!("🚀 SPACE COMMUNICATION PRIORITY SYSTEM DEMONSTRATION");
    println!("=" .repeat(60));
    println!();

    // Create stress tester
    let mut tester = PriorityStressTester::new();

    // Demonstration 1: Show all 24 command types
    demonstrate_all_commands(&mut tester)?;

    // Demonstration 2: Priority ordering verification
    demonstrate_priority_ordering(&mut tester)?;

    // Demonstration 3: High throughput stress test
    demonstrate_high_throughput(&mut tester)?;

    // Demonstration 4: Real-time latency testing
    demonstrate_latency_constraints(&mut tester)?;

    // Demonstration 5: Mission scenarios
    demonstrate_mission_scenarios(&mut tester)?;

    // Final report
    tester.print_test_report();

    println!("\n🎉 All demonstrations completed successfully!");
    Ok(())
}

fn demonstrate_all_commands(tester: &mut PriorityStressTester) -> Result<()> {
    println!("📋 DEMONSTRATION 1: All 24 Space Command Types");
    println!("-" .repeat(50));

    let commands = tester.generate_all_commands();

    for (i, command) in commands.iter().enumerate() {
        let priority = command.priority();
        let band = command.preferred_band();
        let max_latency = priority.max_latency_ms();
        let requires_confirm = command.requires_confirmation();

        println!("{:2}. {} (Priority: {:?})", i + 1, command.description(), priority);
        println!("    Band: {:?}, Max Latency: {} ms, Confirmation: {}",
                band, max_latency, requires_confirm);

        // Create and validate message
        let message = tester.create_message(command.clone())?;
        println!("    Message ID: {}, TTL: {} sec, Max Retries: {}",
                message.id.value(), message.ttl_seconds, message.max_retries);
        println!();
    }

    // Group by priority
    println!("📊 Commands by Priority Level:");
    for priority in [
        MessagePriority::Emergency,
        MessagePriority::Critical,
        MessagePriority::High,
        MessagePriority::Medium,
        MessagePriority::Low,
    ] {
        let count = commands.iter().filter(|cmd| cmd.priority() == priority).count();
        println!("  {:?}: {} commands", priority, count);
    }

    println!("✅ Demonstration 1 completed\n");
    Ok(())
}

fn demonstrate_priority_ordering(tester: &mut PriorityStressTester) -> Result<()> {
    println!("🎯 DEMONSTRATION 2: Priority Ordering Verification");
    println!("-" .repeat(50));

    println!("Adding commands in random order and verifying priority sorting...");

    let stats = tester.stress_test_priority_ordering(50)?;

    println!("Results:");
    println!("  Total messages: {}", stats.total_messages);
    println!("  Messages processed: {}", stats.messages_processed);
    println!("  Priority violations: {}", stats.priority_violations);

    if stats.priority_violations == 0 {
        println!("✅ Perfect priority ordering maintained!");
    } else {
        println!("❌ Priority violations detected: {}", stats.priority_violations);
    }

    println!("✅ Demonstration 2 completed\n");
    Ok(())
}

fn demonstrate_high_throughput(tester: &mut PriorityStressTester) -> Result<()> {
    println!("🚄 DEMONSTRATION 3: High Throughput Stress Test");
    println!("-" .repeat(50));

    println!("Testing system under high message load...");

    // Test at different throughput levels
    let test_scenarios = vec![
        (50, 3),   // 50 msg/sec for 3 seconds
        (100, 3),  // 100 msg/sec for 3 seconds
        (200, 2),  // 200 msg/sec for 2 seconds
        (500, 1),  // 500 msg/sec for 1 second
    ];

    for (rate, duration) in test_scenarios {
        println!("\nTesting {} messages/second for {} seconds:", rate, duration);

        let stats = tester.stress_test_high_throughput(rate, duration)?;

        println!("  Messages sent: {}", stats.total_messages);
        println!("  Messages processed: {}", stats.messages_processed);
        println!("  Dropped: {}", stats.dropped_messages);
        println!("  Average latency: {:.2} ms", stats.average_latency_ms());

        let success_rate = (stats.messages_processed as f64 / stats.total_messages as f64) * 100.0;
        println!("  Success rate: {:.1}%", success_rate);

        if success_rate > 95.0 {
            println!("  ✅ Excellent performance!");
        } else if success_rate > 80.0 {
            println!("  ⚠️  Good performance with some drops");
        } else {
            println!("  ❌ Performance degradation detected");
        }
    }

    println!("✅ Demonstration 3 completed\n");
    Ok(())
}

fn demonstrate_latency_constraints(tester: &mut PriorityStressTester) -> Result<()> {
    println!("⏱️  DEMONSTRATION 4: Real-Time Latency Constraints");
    println!("-" .repeat(50));

    println!("Testing real-time latency requirements...");
    println!("Priority levels and their latency constraints:");

    for priority in [
        MessagePriority::Emergency,
        MessagePriority::Critical,
        MessagePriority::High,
        MessagePriority::Medium,
        MessagePriority::Low,
    ] {
        println!("  {:?}: max {} ms, frequency {} Hz",
                priority, priority.max_latency_ms(), priority.max_frequency_hz());
    }

    let stats = tester.stress_test_latency_constraints(5)?;

    println!("\nLatency Test Results:");
    println!("  Messages processed: {}", stats.messages_processed);
    println!("  Average latency: {:.2} ms", stats.average_latency_ms());
    println!("  Max latency: {:.2} ms", stats.max_latency_ns as f64 / 1_000_000.0);
    println!("  Min latency: {:.2} ms", stats.min_latency_ns as f64 / 1_000_000.0);
    println!("  Latency violations: {}", stats.priority_violations);

    if stats.priority_violations == 0 {
        println!("  ✅ All latency constraints met!");
    } else {
        println!("  ❌ {} latency constraint violations", stats.priority_violations);
    }

    println!("✅ Demonstration 4 completed\n");
    Ok(())
}

fn demonstrate_mission_scenarios(tester: &mut PriorityStressTester) -> Result<()> {
    println!("🌌 DEMONSTRATION 5: Mission-Critical Scenarios");
    println!("-" .repeat(50));

    let scenarios = vec![
        "collision_avoidance",
        "power_emergency",
        "communication_failure",
        "attitude_loss",
    ];

    for scenario in scenarios {
        println!("\n🎬 Scenario: {}", scenario.replace('_', " ").to_uppercase());
        println!("-" .repeat(30));

        let stats = tester.simulate_mission_scenario(scenario)?;

        println!("Scenario completed:");
        println!("  Messages processed: {}", stats.messages_processed);
        if stats.average_latency_ms() > 0.0 {
            println!("  Average response time: {:.2} ms", stats.average_latency_ms());
        }

        // Small delay between scenarios for visibility
        thread::sleep(Duration::from_millis(500));
    }

    println!("✅ Demonstration 5 completed\n");
    Ok(())
}

/// Demonstrate specific emergency commands
fn demonstrate_emergency_commands() -> Result<()> {
    println!("🚨 EMERGENCY COMMAND DEMONSTRATION");
    println!("-" .repeat(40));

    let ground_station = ComponentId::new(0x0001);
    let satellite = ComponentId::new(0x0100);

    // Create emergency abort command
    let abort_msg = CommandBuilder::new(ground_station, satellite)
        .emergency_abort(EmergencyReason::CollisionImminent, 0xDEADBEEF)
        .build()?;

    println!("Emergency Abort Command:");
    println!("  Priority: {:?}", abort_msg.priority);
    println!("  Band: {:?}", abort_msg.preferred_band);
    println!("  TTL: {} seconds", abort_msg.ttl_seconds);
    println!("  Max Retries: {}", abort_msg.max_retries);

    // Create collision avoidance command
    let collision_msg = CommandBuilder::new(ground_station, satellite)
        .collision_avoidance(
            98765,
            ManeuverType::AvoidanceManeuver,
            [0.5, -0.2, 0.1],
            1694188800,
        )
        .build()?;

    println!("\nCollision Avoidance Command:");
    println!("  Priority: {:?}", collision_msg.priority);
    println!("  Band: {:?}", collision_msg.preferred_band);
    println!("  Execution time constraint: {} ms", collision_msg.priority.max_latency_ms());

    // Create attitude control command
    let attitude_msg = CommandBuilder::new(ground_station, satellite)
        .attitude_control(
            [0.0, 0.0, 0.0, 1.0],
            [0.01, 0.01, 0.01],
            AttitudeMode::EarthPointing,
            5000,
        )
        .build()?;

    println!("\nAttitude Control Command:");
    println!("  Priority: {:?}", attitude_msg.priority);
    println!("  Real-time: {}", attitude_msg.priority.is_real_time());
    println!("  Max frequency: {} Hz", attitude_msg.priority.max_frequency_hz());

    println!("✅ Emergency commands demonstration completed\n");
    Ok(())
}

/// Demonstrate priority queue behavior
fn demonstrate_priority_queue_behavior() -> Result<()> {
    println!("🔄 PRIORITY QUEUE BEHAVIOR DEMONSTRATION");
    println!("-" .repeat(45));

    let mut queue: PriorityQueue<20> = PriorityQueue::new();
    let ground_station = ComponentId::new(0x0001);
    let satellite = ComponentId::new(0x0100);

    // Add messages with different priorities
    let commands_and_descriptions = vec![
        (SpaceCommand::SendStatus {
            status_type: StatusType::SystemHealth,
            include_diagnostics: false,
            format: ReportFormat::Json
        }, "Status Report"),

        (SpaceCommand::EmergencyAbort {
            reason: EmergencyReason::SystemFailure,
            confirmation_code: 0x12345678
        }, "Emergency Abort"),

        (SpaceCommand::RequestTelemetry {
            telemetry_type: TelemetryType::Health,
            sampling_rate_hz: 1.0,
            duration_seconds: 60,
            compression: true
        }, "Telemetry Request"),

        (SpaceCommand::CollisionAvoidance {
            debris_id: 54321,
            maneuver_type: ManeuverType::AvoidanceManeuver,
            delta_v: [0.1, 0.2, -0.1],
            execution_time: 1694188800
        }, "Collision Avoidance"),

        (SpaceCommand::UpdateConfig {
            config_id: heapless::String::from_str("test_config").unwrap(),
            parameters: heapless::Vec::new(),
            apply_immediately: false,
            backup_current: true
        }, "Config Update"),
    ];

    println!("Adding messages in random order:");
    for (i, (command, description)) in commands_and_descriptions.iter().enumerate() {
        let message = CommandBuilder::new(ground_station, satellite)
            .build_from_command(command.clone())?;

        queue.push(message.clone())?;

        println!("  {}. Added: {} (Priority: {:?})",
                i + 1, description, message.priority);
    }

    println!("\nProcessing messages (should be in priority order):");
    let mut order = 1;
    while !queue.is_empty() {
        if let Some(message) = queue.pop() {
            let command_desc = match message.priority {
                MessagePriority::Emergency => "🚨 EMERGENCY",
                MessagePriority::Critical => "🔴 CRITICAL",
                MessagePriority::High => "🟡 HIGH",
                MessagePriority::Medium => "🔵 MEDIUM",
                MessagePriority::Low => "⚪ LOW",
            };

            println!("  {}. Processed: {} {:?}", order, command_desc, message.priority);
            order += 1;
        }
    }

    println!("✅ Priority queue demonstration completed\n");
    Ok(())
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_full_system_integration() {
        let mut tester = PriorityStressTester::new();

        // Test basic functionality
        let commands = tester.generate_all_commands();
        assert_eq!(commands.len(), 24);

        // Test message creation
        for command in commands {
            let message = tester.create_message(command.clone());
            assert!(message.is_ok());

            let msg = message.unwrap();
            assert_eq!(msg.priority, command.priority());
            assert_eq!(msg.preferred_band, command.preferred_band());
        }

        // Test stress scenarios
        let stats = tester.stress_test_priority_ordering(5).unwrap();
        assert_eq!(stats.priority_violations, 0);

        let stats = tester.stress_test_high_throughput(10, 1).unwrap();
        assert!(stats.total_messages > 0);
        assert!(stats.messages_processed > 0);
    }

    #[test]
    fn test_emergency_command_priorities() {
        let emergency_commands = vec![
            SpaceCommand::EmergencyAbort {
                reason: EmergencyReason::SystemFailure,
                confirmation_code: 0x12345678
            },
            SpaceCommand::EmergencyHalt {
                subsystems: heapless::Vec::new(),
                override_code: 0x87654321
            },
            SpaceCommand::ActivateSafeMode {
                safe_mode_level: SafeModeLevel::Level1,
                duration_seconds: Some(3600)
            },
        ];

        for command in emergency_commands {
            assert_eq!(command.priority(), MessagePriority::Emergency);
            assert!(command.priority().is_real_time());
            assert_eq!(command.priority().max_latency_ms(), 0);
            assert!(command.requires_confirmation());
        }
    }

    #[test]
    fn test_command_band_selection() {
        let test_cases = vec![
            (MessagePriority::Emergency, BandType::KBand),
            (MessagePriority::Critical, BandType::XBand),
            (MessagePriority::High, BandType::SBand),
            (MessagePriority::Medium, BandType::SBand),
            (MessagePriority::Low, BandType::UhfBand),
        ];

        for (priority, expected_band) in test_cases {
            // Find a command with this priority
            let command = match priority {
                MessagePriority::Emergency => SpaceCommand::EmergencyAbort {
                    reason: EmergencyReason::SystemFailure,
                    confirmation_code: 0x12345
                },
                MessagePriority::Critical => SpaceCommand::CollisionAvoidance {
                    debris_id: 123,
                    maneuver_type: ManeuverType::AvoidanceManeuver,
                    delta_v: [0.0, 0.0, 0.0],
                    execution_time: 0
                },
                MessagePriority::High => SpaceCommand::Deploy {
                    deployable: DeployableType::SolarPanel,
                    deployment_angle: 180.0,
                    deployment_rate: 5.0,
                    force_limit: 50.0
                },
                MessagePriority::Medium => SpaceCommand::RequestTelemetry {
                    telemetry_type: TelemetryType::Health,
                    sampling_rate_hz: 1.0,
                    duration_seconds: 60,
                    compression: true
                },
                MessagePriority::Low => SpaceCommand::SendStatus {
                    status_type: StatusType::SystemHealth,
                    include_diagnostics: false,
                    format: ReportFormat::Json
                },
            };

            assert_eq!(command.priority(), priority);
            assert_eq!(command.preferred_band(), expected_band);
        }
    }
}
