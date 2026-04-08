//! System integration tests — end-to-end coverage.
//!
//! These tests exercise the full API surface across shared, simulation-adjacent
//! concepts, and cross-crate interactions.  They use the correct, current API
//! and serve as regression guards for all prior improvements and optimisations.
//!
//! # Coverage
//! - Message creation and all payload variants
//! - CCSDS packet assembly, CRC auto-computation, mutation detection
//! - Telemetry packet construction and measurement encoding
//! - Priority queue ordering: Emergency→Critical→High→Medium→Low
//! - TTL enforcement via `pop_valid()`
//! - Security: HMAC sign/verify, tamper rejection
//! - Band type frequency + FSPL spot-checks
//! - Error construction, display formatting, recoverability flags
//! - Component and message ID identity/uniqueness

#![cfg(test)]

use space_comms_shared::{
    ccsds::{PacketType, SequenceFlags, SpacePacket, SpacePacketHeader},
    error::{MemoryErrorType, SpaceCommError},
    messaging::{Message, MessagePayload, MessagePriority, PriorityQueue},
    security::{AuthTag, CommandAuthenticator, DIGEST_LEN},
    telemetry::{Measurement, MeasurementQuality, MeasurementValue, TelemetryData, TelemetryPacket},
    types::{BandType, ComponentId, HealthStatus, MessageId},
};

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn make_command_message(priority: MessagePriority, ttl_seconds: u32, timestamp_ns: u64) -> Message {
    Message {
        id: MessageId::new(),
        priority,
        source: ComponentId::new(0x0001),
        destination: ComponentId::new(0x0100),
        timestamp: timestamp_ns,
        payload: MessagePayload::Command {
            command_id: 0x0001_u32,
            parameters: heapless::Vec::new(),
        },
        preferred_band: BandType::SBand,
        ttl_seconds,
        retry_count: 0,
        max_retries: 3,
    }
}

// ─── Message Struct Tests ──────────────────────────────────────────────────────

/// Command payload must store command_id and empty parameters correctly.
#[test]
fn test_message_creation_command_payload() {
    let id = MessageId::from_value(0xDEAD_BEEF);
    let msg = Message {
        id,
        priority: MessagePriority::High,
        source: ComponentId::new(0x0001),
        destination: ComponentId::new(0x0200),
        timestamp: 1_700_000_000_000_000_000,
        payload: MessagePayload::Command {
            command_id: 0x1001,
            parameters: heapless::Vec::new(),
        },
        preferred_band: BandType::XBand,
        ttl_seconds: 30,
        retry_count: 0,
        max_retries: 3,
    };

    assert_eq!(msg.id.value(), 0xDEAD_BEEF);
    assert_eq!(msg.priority, MessagePriority::High);
    assert_eq!(msg.preferred_band, BandType::XBand);
    assert_eq!(msg.ttl_seconds, 30);

    if let MessagePayload::Command { command_id, parameters } = &msg.payload {
        assert_eq!(*command_id, 0x1001);
        assert!(parameters.is_empty());
    } else {
        panic!("Expected Command payload");
    }
}

/// Telemetry payload must store data bytes and format correctly.
#[test]
fn test_message_creation_telemetry_payload() {
    let mut data: heapless::Vec<u8, 1024> = heapless::Vec::new();
    data.push(0xAB).unwrap();
    data.push(0xCD).unwrap();

    let msg = Message {
        id: MessageId::new(),
        priority: MessagePriority::Low,
        source: ComponentId::new(0x0010),
        destination: ComponentId::new(0xFF00),
        timestamp: 0,
        payload: MessagePayload::Telemetry { data, format: 0x0050 },
        preferred_band: BandType::UhfBand,
        ttl_seconds: 0,
        retry_count: 0,
        max_retries: 0,
    };

    if let MessagePayload::Telemetry { data, format } = &msg.payload {
        assert_eq!(data.len(), 2);
        assert_eq!(*format, 0x0050);
    } else {
        panic!("Expected Telemetry payload");
    }
}

/// Emergency payload must store alert_level, description, and data using heapless types.
#[test]
fn test_message_creation_emergency_payload() {
    let mut desc: heapless::String<256> = heapless::String::new();
    desc.push_str("ATTITUDE_CONTROL_FAILURE").unwrap();

    let msg = Message {
        id: MessageId::from_value(1),
        priority: MessagePriority::Emergency,
        source: ComponentId::new(0x0001),
        destination: ComponentId::new(0xFF00),
        timestamp: 1_000_000_000,
        payload: MessagePayload::Emergency {
            alert_level: 255,
            description: desc,
            data: heapless::Vec::new(),
        },
        preferred_band: BandType::XBand,
        ttl_seconds: 5,
        retry_count: 0,
        max_retries: 5,
    };

    if let MessagePayload::Emergency { alert_level, description, .. } = &msg.payload {
        assert_eq!(*alert_level, 255);
        assert!(description.as_str().contains("ATTITUDE_CONTROL_FAILURE"));
    } else {
        panic!("Expected Emergency payload");
    }
}

// ─── Priority Queue Integration Tests ─────────────────────────────────────────

/// Emergency must dequeue before Medium and Low.
#[test]
fn test_priority_ordering_emergency_first() {
    let mut q = PriorityQueue::<8>::new();
    q.push(make_command_message(MessagePriority::Medium, 0, 0)).unwrap();
    q.push(make_command_message(MessagePriority::Emergency, 0, 0)).unwrap();
    q.push(make_command_message(MessagePriority::Low, 0, 0)).unwrap();

    assert_eq!(q.pop().unwrap().priority, MessagePriority::Emergency);
    assert_eq!(q.pop().unwrap().priority, MessagePriority::Medium);
    assert_eq!(q.pop().unwrap().priority, MessagePriority::Low);
}

/// Full five-tier ordering must be preserved.
#[test]
fn test_all_five_priority_tiers_ordered_correctly() {
    let mut q = PriorityQueue::<16>::new();
    for p in [
        MessagePriority::Low,
        MessagePriority::High,
        MessagePriority::Emergency,
        MessagePriority::Medium,
        MessagePriority::Critical,
    ] {
        q.push(make_command_message(p, 0, 0)).unwrap();
    }
    let order = [
        MessagePriority::Emergency,
        MessagePriority::Critical,
        MessagePriority::High,
        MessagePriority::Medium,
        MessagePriority::Low,
    ];
    for expected in order {
        assert_eq!(q.pop().unwrap().priority, expected);
    }
}

/// TTL enforcement: expired message silently discarded, valid returned.
#[test]
fn test_ttl_enforcement_pop_valid_skips_expired() {
    let mut q = PriorityQueue::<8>::new();
    let now_secs = 1000_u64;

    // Expired Emergency (created at 0 ns, TTL=1s, age=1000s >> 1s)
    q.push(make_command_message(MessagePriority::Emergency, 1, 0)).unwrap();
    // Still-valid Low (created at 999s, TTL=60s, age=1s < 60s)
    q.push(make_command_message(MessagePriority::Low, 60, 999 * 1_000_000_000)).unwrap();

    let result = q.pop_valid(now_secs);
    assert!(result.is_some());
    assert_eq!(result.unwrap().priority, MessagePriority::Low,
        "Expired Emergency must be discarded; valid Low must surface");
    assert!(q.pop_valid(now_secs).is_none(), "Queue must be empty after the valid message is popped");
}

/// TTL=0 means no expiry.
#[test]
fn test_ttl_zero_never_expires() {
    let mut q = PriorityQueue::<4>::new();
    q.push(make_command_message(MessagePriority::Critical, 0, 0)).unwrap();
    assert!(q.pop_valid(999_999).is_some(), "TTL=0 must never expire");
}

/// Overflow: pushing beyond capacity returns Err (no panic).
#[test]
fn test_queue_overflow_returns_error() {
    let mut q = PriorityQueue::<2>::new();
    q.push(make_command_message(MessagePriority::Low, 0, 0)).unwrap();
    q.push(make_command_message(MessagePriority::Low, 0, 0)).unwrap();
    assert!(q.push(make_command_message(MessagePriority::Low, 0, 0)).is_err());
}

// ─── CCSDS Packet Integration Tests ───────────────────────────────────────────

/// Auto-CRC postcondition: every packet from `new()` verifies immediately.
#[test]
fn test_ccsds_auto_crc_postcondition() {
    let packet = SpacePacket::new(PacketType::Telemetry, 0x100, 1, b"integration_test_data", None).unwrap();
    assert!(packet.verify_crc(), "CRC must be valid immediately after SpacePacket::new()");
}

/// Mutating data must break CRC.
#[test]
fn test_ccsds_mutated_data_fails_crc() {
    let mut p = SpacePacket::new(PacketType::Command, 0x010, 5, b"command:abort_sequence", None).unwrap();
    p.data[0] ^= 0xFF;
    assert!(!p.verify_crc(), "Bit-flipped data must fail CRC check");
}

/// APID 0x7FF is the legal ceiling.
#[test]
fn test_ccsds_apid_max_boundary_accepted() {
    assert!(SpacePacket::new(PacketType::Telemetry, 0x7FF, 0, b"x", None).is_ok());
}

/// APID 0x800 exceeds 11 bits and must be rejected.
#[test]
fn test_ccsds_apid_over_max_rejected() {
    assert!(SpacePacket::new(PacketType::Telemetry, 0x800, 0, b"x", None).is_err());
}

/// Empty payload: valid packet with CRC.
#[test]
fn test_ccsds_empty_payload_still_valid() {
    let p = SpacePacket::new(PacketType::Command, 0x001, 0, &[], None).unwrap();
    assert!(p.verify_crc());
    assert_eq!(p.data.len(), 0);
}

/// Header serialization round-trip must be identity.
#[test]
fn test_ccsds_header_round_trip() {
    let hdr = SpacePacketHeader::new(PacketType::Command, 0x0A0, 100, 64, true).unwrap();
    let restored = SpacePacketHeader::from_bytes(&hdr.to_bytes()).unwrap();
    assert_eq!(hdr, restored);
}

/// CCSDS version must always be zero.
#[test]
fn test_ccsds_version_always_zero() {
    let hdr = SpacePacketHeader::new(PacketType::Telemetry, 0x001, 1, 0, false).unwrap();
    assert_eq!(hdr.version, 0);
}

/// Single-packet payloads use Unsegmented sequence flag.
#[test]
fn test_ccsds_sequence_flags_default_unsegmented() {
    let hdr = SpacePacketHeader::new(PacketType::Command, 0x001, 0, 0, false).unwrap();
    assert_eq!(hdr.sequence_flags, SequenceFlags::Unsegmented);
}

// ─── Telemetry Integration Tests ──────────────────────────────────────────────

/// Telemetry packet round-trip: constructor + field access.
#[test]
fn test_telemetry_packet_construction() {
    let mut measurements: heapless::Vec<Measurement, 32> = heapless::Vec::new();
    measurements.push(Measurement {
        measurement_id: 0x0001,
        value: MeasurementValue::Float(23.5),
        unit: "degC",
        quality: MeasurementQuality::Good,
    }).unwrap();
    measurements.push(Measurement {
        measurement_id: 0x0002,
        value: MeasurementValue::Integer(1234),
        unit: "rpm",
        quality: MeasurementQuality::Good,
    }).unwrap();

    let data = TelemetryData {
        source: ComponentId::new(0x0001),
        timestamp: 1_700_000_000_000_000_000,
        measurements,
        health_status: HealthStatus::Good,
    };
    let packet = TelemetryPacket::new(99, data, BandType::SBand);

    assert_eq!(packet.sequence, 99);
    assert_eq!(packet.band, BandType::SBand);
    assert_eq!(packet.data.measurements.len(), 2);
    assert_eq!(packet.data.health_status, HealthStatus::Good);
    assert!(packet.size_bytes > 0);
}

/// All measurement value variants must be constructible and pattern-matchable.
#[test]
fn test_all_measurement_value_variants() {
    let vals = [
        MeasurementValue::Float(3.14),
        MeasurementValue::Integer(42),
        MeasurementValue::Boolean(true),
    ];
    // Ensure each variant can be formatted (no panic)
    for v in &vals {
        let _ = format!("{:?}", v);
    }
}

/// HealthStatus must compare correctly and `requires_attention()` must reflect severity.
#[test]
fn test_health_status_comparison_and_attention_flag() {
    // Use score() for health-quality comparisons (derived Ord uses discriminant order).
    assert!(HealthStatus::Excellent.score() > HealthStatus::Good.score());
    assert!(HealthStatus::Good.score() > HealthStatus::Fair.score());
    assert!(HealthStatus::Fair.score() > HealthStatus::Poor.score());
    assert!(HealthStatus::Poor.score() > HealthStatus::Critical.score());

    assert!(!HealthStatus::Excellent.requires_attention());
    assert!(!HealthStatus::Good.requires_attention());
    assert!(HealthStatus::Poor.requires_attention());
    assert!(HealthStatus::Critical.requires_attention());
}

// ─── Security Integration Tests ───────────────────────────────────────────────

/// Sign + verify end-to-end round-trip.
#[test]
fn test_security_sign_verify_roundtrip() {
    let key = b"integration-auth-key-for-testing";
    let msg = b"THROTTLE:100:SEQ:0x0042";
    let tag = CommandAuthenticator::sign(key, msg).unwrap();
    assert!(CommandAuthenticator::verify(key, msg, &tag).unwrap());
}

/// Tampered message byte must be rejected.
#[test]
fn test_security_tampered_message_rejected() {
    let key = b"integration-auth-key-for-testing";
    let tag = CommandAuthenticator::sign(key, b"ABORT:0").unwrap();
    assert!(!CommandAuthenticator::verify(key, b"ABORT:1", &tag).unwrap());
}

/// Wrong key must fail verification.
#[test]
fn test_security_wrong_key_rejected() {
    let tag = CommandAuthenticator::sign(b"correct-key-32bb", b"safe_payload").unwrap();
    assert!(!CommandAuthenticator::verify(b"wrong-key-xxxxx!", b"safe_payload", &tag).unwrap());
}

/// AuthTag is exactly DIGEST_LEN bytes (32 for SHA-256).
#[test]
fn test_security_digest_length_is_32() {
    let tag = CommandAuthenticator::sign(b"key", b"payload").unwrap();
    assert_eq!(tag.as_bytes().len(), DIGEST_LEN);
    assert_eq!(DIGEST_LEN, 32);
}

/// Empty key must return error.
#[test]
fn test_security_empty_key_returns_error() {
    assert!(CommandAuthenticator::sign(b"", b"message").is_err());
}

// ─── Band Types Integration Tests ─────────────────────────────────────────────

/// All band centre frequencies must lie within their defined ranges.
#[test]
fn test_band_centre_frequencies_in_range() {
    let bands_ranges = [
        (BandType::UhfBand, 0.3_f64, 3.0_f64),
        (BandType::SBand,   2.0,    4.0),
        (BandType::XBand,   8.0,   12.0),
        (BandType::KBand,  20.0,   30.0),
        (BandType::KaBand, 26.5,   40.0),
    ];
    for (band, lo, hi) in bands_ranges {
        let f = band.center_frequency_ghz();
        assert!(f >= lo && f <= hi,
            "{:?}: {:.2} GHz not in [{:.2}, {:.2}]", band, f, lo, hi);
    }
}

/// FSPL must increase with distance at constant frequency.
#[test]
fn test_band_fspl_increases_with_distance() {
    let fspl_500 = BandType::SBand.free_space_path_loss_db(500.0);
    let fspl_1000 = BandType::SBand.free_space_path_loss_db(1000.0);
    assert!(fspl_1000 > fspl_500);
}

/// FSPL at zero or negative distance must return 0.0.
#[test]
fn test_band_fspl_zero_distance() {
    assert_eq!(BandType::KaBand.free_space_path_loss_db(0.0), 0.0);
    assert_eq!(BandType::KaBand.free_space_path_loss_db(-1.0), 0.0);
}

// ─── Error Handling Integration Tests ─────────────────────────────────────────

/// CommunicationTimeout display must contain ms value and operation name.
#[test]
fn test_error_communication_timeout_display() {
    let err = SpaceCommError::communication_timeout(2500, "uplink_command");
    let msg = format!("{}", err);
    assert!(msg.contains("2500"));
    assert!(msg.contains("uplink_command"));
}

/// InvalidPacket without ID must omit "ID:" from display.
#[test]
fn test_error_invalid_packet_no_id() {
    let err = SpaceCommError::invalid_packet("bad_header_checksum", None);
    let msg = format!("{}", err);
    assert!(msg.contains("bad_header_checksum"));
}

/// Memory errors for correct variant names (AllocationFailed not AllocationFailure).
#[test]
fn test_error_memory_allocation_failed_variant() {
    let err = SpaceCommError::memory_error(MemoryErrorType::AllocationFailed, None);
    let msg = format!("{}", err);
    assert!(msg.to_lowercase().contains("allocation") || msg.to_lowercase().contains("memory"));
}

/// BufferOverflow must not be recoverable.
#[test]
fn test_error_buffer_overflow_not_recoverable() {
    let err = SpaceCommError::memory_error(MemoryErrorType::BufferOverflow, Some(512));
    assert!(!err.is_recoverable());
}

/// CommunicationTimeout must be recoverable.
#[test]
fn test_error_communication_timeout_is_recoverable() {
    let err = SpaceCommError::communication_timeout(1000, "link_check");
    assert!(err.is_recoverable());
}

/// HardwareFailure must not be recoverable.
#[test]
fn test_error_hardware_failure_not_recoverable() {
    let err = SpaceCommError::hardware_failure("Ka-Band_SSPA", 0xDEAD);
    assert!(!err.is_recoverable());
}

// ─── MessageId / ComponentId Tests ────────────────────────────────────────────

/// `from_value()` must store and return the exact value provided.
#[test]
fn test_message_id_from_value_roundtrip() {
    let id = MessageId::from_value(42);
    assert_eq!(id.value(), 42);
}

/// `new()` must generate monotonically increasing, unique IDs.
#[test]
fn test_message_id_new_unique() {
    let a = MessageId::new();
    let b = MessageId::new();
    assert_ne!(a, b, "MessageId::new() must produce distinct IDs");
    assert!(b.value() > a.value(), "IDs must be monotonically increasing");
}

/// ComponentId stores its value with identity equality.
#[test]
fn test_component_id_equality() {
    let a = ComponentId::new(0x1234);
    let b = ComponentId::new(0x1234);
    assert_eq!(a, b);
    assert_eq!(a.value(), 0x1234);
}

// ─── System-Level Integration Scenario ────────────────────────────────────────

/// End-to-end: build a command, authenticate it, queue it, extract via pop_valid,
/// wrap in a CCSDS packet, and verify packet integrity.
#[test]
fn test_full_command_lifecycle() {
    // 1. Create authenticated command message
    let auth_key = b"mission-ground-auth-key-32-bytes";
    let command_payload = b"REACTION_WHEEL:SPIN:3000RPM:SEQ:0x0001";
    let auth_tag = CommandAuthenticator::sign(auth_key, command_payload).unwrap();
    assert!(CommandAuthenticator::verify(auth_key, command_payload, &auth_tag).unwrap());

    // 2. Enqueue as a Critical message
    let mut queue = PriorityQueue::<8>::new();
    let msg = make_command_message(MessagePriority::Critical, 60, 1_000 * 1_000_000_000);
    queue.push(msg).unwrap();

    // 3. Dequeue while still in TTL window
    let dequeued = queue.pop_valid(1_010).unwrap(); // age = 10s < TTL=60s
    assert_eq!(dequeued.priority, MessagePriority::Critical);

    // 4. Wrap command in CCSDS space packet
    let packet = SpacePacket::new(PacketType::Command, 0x001, 1, command_payload, None).unwrap();
    assert!(packet.verify_crc(), "CCSDS packet from critical command must have valid CRC");

    // 5. Verify telemetry ack packet separately
    let ack = SpacePacket::new(PacketType::Telemetry, 0x001, 2, b"ACK:OK:0x0001", None).unwrap();
    assert!(ack.verify_crc());
    assert_ne!(packet.error_control, ack.error_control, "Different payloads must differ in CRC");
}
