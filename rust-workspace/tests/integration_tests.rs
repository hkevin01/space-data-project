//! Integration tests for the satellite communication system
//!
//! Tests the complete satellite system including hardware abstraction,
//! communication modules, and error handling.

#![cfg(test)]

use space_comms_shared::{
    messaging::{Message, MessageId, MessagePriority, MessagePayload},
    telemetry::{TelemetryPacket, TelemetryData, Measurement, MeasurementValue, MeasurementQuality},
    types::{BandType, ComponentId, HealthStatus},
    ccsds::{SpacePacket, PacketType},
    Result, SpaceCommError,
};

/// Test message creation and processing
#[test]
fn test_message_creation() {
    let message = Message {
        id: MessageId::new(1),
        priority: MessagePriority::High,
        payload: MessagePayload::Command {
            command_id: 0x1001,
            parameters: vec![0x01, 0x02, 0x03],
        },
    };

    assert_eq!(message.id.value(), 1);
    assert_eq!(message.priority, MessagePriority::High);

    if let MessagePayload::Command { command_id, parameters } = &message.payload {
        assert_eq!(*command_id, 0x1001);
        assert_eq!(parameters.len(), 3);
    } else {
        panic!("Expected Command payload");
    }
}

/// Test telemetry packet creation
#[test]
fn test_telemetry_packet() {
    let mut measurements = heapless::Vec::<Measurement, 16>::new();

    let temp_measurement = Measurement {
        measurement_id: 0x0001,
        value: MeasurementValue::Float(25.5),
        unit: "C",
        quality: MeasurementQuality::Good,
    };

    measurements.push(temp_measurement).unwrap();

    let telemetry_data = TelemetryData {
        source: ComponentId::new(0x0001),
        timestamp: 1234567890,
        measurements,
        health_status: HealthStatus::Good,
    };

    let packet = TelemetryPacket {
        sequence: 42,
        band: BandType::SBand,
        data: telemetry_data,
    };

    assert_eq!(packet.sequence, 42);
    assert_eq!(packet.band, BandType::SBand);
    assert_eq!(packet.data.measurements.len(), 1);
    assert_eq!(packet.data.health_status, HealthStatus::Good);
}

/// Test CCSDS packet creation
#[test]
fn test_ccsds_packet() {
    let data = vec![0x01, 0x02, 0x03, 0x04];

    let packet = SpacePacket::new(
        PacketType::Telemetry,
        0x100,
        42,
        &data,
        None,
    ).unwrap();

    assert_eq!(packet.header.packet_type, PacketType::Telemetry);
    assert_eq!(packet.header.apid, 0x100);
    assert_eq!(packet.header.sequence_count, 42);
    assert_eq!(packet.data.len(), 4);
}

/// Test error handling
#[test]
fn test_error_handling() {
    let error = SpaceCommError::communication_timeout(1000, "Test timeout");

    assert!(format!("{}", error).contains("timeout"));
    assert!(format!("{}", error).contains("1000"));
}

/// Test band type enumeration
#[test]
fn test_band_types() {
    let bands = vec![
        BandType::UhfBand,
        BandType::SBand,
        BandType::XBand,
        BandType::KBand,
        BandType::KaBand,
    ];

    assert_eq!(bands.len(), 5);

    // Test serialization
    for band in bands {
        let serialized = format!("{:?}", band);
        assert!(!serialized.is_empty());
    }
}

/// Test measurement values
#[test]
fn test_measurement_values() {
    let float_val = MeasurementValue::Float(3.14159);
    let int_val = MeasurementValue::Integer(42);
    let bool_val = MeasurementValue::Boolean(true);

    match float_val {
        MeasurementValue::Float(val) => assert!((val - 3.14159).abs() < f64::EPSILON),
        _ => panic!("Expected float value"),
    }

    match int_val {
        MeasurementValue::Integer(val) => assert_eq!(val, 42),
        _ => panic!("Expected integer value"),
    }

    match bool_val {
        MeasurementValue::Boolean(val) => assert!(val),
        _ => panic!("Expected boolean value"),
    }
}

/// Test component ID functionality
#[test]
fn test_component_id() {
    let id = ComponentId::new(0x1234);
    assert_eq!(id.value(), 0x1234);

    let id2 = ComponentId::new(0x1234);
    assert_eq!(id, id2);
}

/// Test message ID functionality
#[test]
fn test_message_id() {
    let id = MessageId::new(42);
    assert_eq!(id.value(), 42);

    let id2 = MessageId::new(42);
    assert_eq!(id, id2);

    let id3 = MessageId::new(43);
    assert_ne!(id, id3);
}

/// Test health status
#[test]
fn test_health_status() {
    assert_eq!(HealthStatus::Good as u8, 0);
    assert_eq!(HealthStatus::Degraded as u8, 1);
    assert_eq!(HealthStatus::Critical as u8, 2);
}

/// Test memory error creation
#[test]
fn test_memory_errors() {
    use space_comms_shared::error::{MemoryErrorType, ErrorSeverity};

    let error = SpaceCommError::memory_error(MemoryErrorType::BufferOverflow, Some(1024));
    assert_eq!(error.severity(), ErrorSeverity::High);

    let error2 = SpaceCommError::memory_error(MemoryErrorType::AllocationFailure, None);
    assert_eq!(error2.severity(), ErrorSeverity::Critical);
}

/// Benchmark test for message processing
#[test]
fn test_message_processing_performance() {
    use std::time::Instant;

    let start = Instant::now();

    // Create 1000 messages
    for i in 0..1000 {
        let message = Message {
            id: MessageId::new(i),
            priority: MessagePriority::Medium,
            payload: MessagePayload::Command {
                command_id: i,
                parameters: vec![0x01, 0x02, 0x03, 0x04],
            },
        };

        // Simulate processing
        assert_eq!(message.id.value(), i);
    }

    let duration = start.elapsed();
    println!("Message processing time for 1000 messages: {:?}", duration);

    // Should complete within reasonable time (< 10ms)
    assert!(duration.as_millis() < 10);
}

/// Test concurrent message handling
#[cfg(feature = "std")]
#[test]
fn test_concurrent_messaging() {
    use std::sync::{Arc, Mutex};
    use std::thread;

    let message_count = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    // Spawn 10 threads, each creating 100 messages
    for thread_id in 0..10 {
        let count = Arc::clone(&message_count);

        let handle = thread::spawn(move || {
            for i in 0..100 {
                let message = Message {
                    id: MessageId::new(thread_id * 100 + i),
                    priority: MessagePriority::Low,
                    payload: MessagePayload::Command {
                        command_id: 0x2000 + i,
                        parameters: vec![thread_id as u8, i as u8],
                    },
                };

                // Simulate processing
                let mut count = count.lock().unwrap();
                *count += 1;
            }
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    let final_count = *message_count.lock().unwrap();
    assert_eq!(final_count, 1000);
}

/// Integration test for the complete system
#[test]
fn test_system_integration() {
    // Test that all components can work together

    // 1. Create a message
    let message = Message {
        id: MessageId::new(1),
        priority: MessagePriority::Critical,
        payload: MessagePayload::Emergency {
            alert_level: 3,
            description: "System test".to_string(),
            data: vec![0x01, 0x02, 0x03],
        },
    };

    // 2. Create telemetry data
    let mut measurements = heapless::Vec::<Measurement, 16>::new();
    measurements.push(Measurement {
        measurement_id: 0x0001,
        value: MeasurementValue::Float(25.0),
        unit: "C",
        quality: MeasurementQuality::Good,
    }).unwrap();

    let telemetry_data = TelemetryData {
        source: ComponentId::new(0x0001),
        timestamp: 1234567890,
        measurements,
        health_status: HealthStatus::Good,
    };

    let telemetry_packet = TelemetryPacket {
        sequence: 1,
        band: BandType::SBand,
        data: telemetry_data,
    };

    // 3. Create CCSDS packets
    let command_packet = SpacePacket::new(
        PacketType::Command,
        0x001,
        1,
        &[0x01, 0x02, 0x03],
        None,
    ).unwrap();

    let telemetry_ccsds = SpacePacket::new(
        PacketType::Telemetry,
        0x100,
        1,
        &[0x04, 0x05, 0x06],
        None,
    ).unwrap();

    // 4. Verify serialization
    let command_bytes = command_packet.to_bytes().unwrap();
    let telemetry_bytes = telemetry_ccsds.to_bytes().unwrap();

    assert!(command_bytes.len() >= 6); // Minimum CCSDS header size
    assert!(telemetry_bytes.len() >= 6);

    // 5. Test error scenarios
    let error = SpaceCommError::invalid_packet("Test error", Some(command_bytes));
    assert!(format!("{}", error).contains("invalid packet"));

    println!("System integration test completed successfully");
}
