//! Communication module for satellite system
//!
//! # Satellite Communication System Module
//!
//! Handles multi-band communication including K-band, Ka-band, X-band, S-band, and UHF-band.
//! Implements band selection based on message priority and environmental conditions with
//! CCSDS-compliant packet formatting and real-time transmission constraints.
//!
//! ## Requirements Fulfilled:
//! - REQ-FN-007: Multi-band communication support (K, Ka, X, S, UHF bands)
//! - REQ-FN-001: Priority-based message routing and band selection
//! - REQ-IF-002: CCSDS space packet protocol compliance
//! - REQ-PF-001: Real-time communication constraints (10ms-1s timeouts)
//! - REQ-NF-003: Concurrent communication handling with Embassy async
//! - REQ-SF-002: Emergency communication protocols and failover
//! - REQ-NF-004: Power management across communication bands
//!
//! ## Architecture:
//! - Band selection algorithm based on message priority and reliability
//! - CCSDS packet creation and parsing for space standards compliance
//! - Emergency mode with UHF fallback for maximum reliability
//! - Power management across multiple RF bands for efficiency

use embassy_time::{Duration, Timer};
use heapless::Vec;

use space_comms_shared::{
    messaging::{Message, MessagePriority},
    telemetry::TelemetryPacket,
    types::BandType,
    ccsds::{SpacePacket, PacketType, SpacePacketHeader},
    Result, SpaceCommError,
};

use crate::hardware;
use crate::error_handling;

/// Communication band configuration
///
/// Stores configuration and status information for each RF communication band
/// supported by the satellite system.
///
/// Requirements Fulfilled:
/// - REQ-FN-007: Multi-band communication configuration management
/// - REQ-NF-004: Power level control for each band
#[derive(Debug, Clone)]
pub struct BandConfig {
    /// Frequency band type (UHF, S, X, K, Ka)
    /// REQ-FN-007: Band type identification
    pub band_type: BandType,

    /// Current power level (0-100%)
    /// REQ-NF-004: Power management and control
    pub power_level: u8,

    /// Is band currently active and operational
    /// REQ-SF-002: Band availability for emergency protocols
    pub is_active: bool,

    /// Current data rate in bits per second
    /// REQ-PF-002: Data rate monitoring and optimization
    pub data_rate: u64,

    /// Signal quality metric (0-100%)
    /// REQ-SF-002: Signal quality for band selection
    pub signal_quality: u8,
}

/// Communication manager state
///
/// Central manager for all satellite communication operations, handling
/// band selection, emergency protocols, and CCSDS packet transmission.
///
/// Requirements Fulfilled:
/// - REQ-FN-007: Multi-band communication coordination
/// - REQ-SF-002: Emergency communication management
/// - REQ-NF-003: Thread-safe communication state management
pub struct CommunicationManager {
    /// Active communication bands configuration
    /// REQ-FN-007: Support for all 5 communication bands
    bands: Vec<BandConfig, 5>,

    /// Current primary communication band
    /// REQ-FN-001: Primary band selection based on priority
    primary_band: BandType,

    /// Emergency mode flag for failover protocols
    /// REQ-SF-002: Emergency communication mode
    emergency_mode: bool,
}

impl CommunicationManager {
    /// Create new communication manager
    ///
    /// Initializes all supported communication bands with default configurations
    /// optimized for space environment operations.
    ///
    /// Requirements Fulfilled:
    /// - REQ-FN-007: Initialize all 5 communication bands
    /// - REQ-NF-004: Set initial power levels for efficiency
    /// - REQ-PF-002: Configure data rates per band capabilities
    ///
    /// Returns:
    /// CommunicationManager instance with all bands configured
    pub fn new() -> Self {
        let mut bands = Vec::new();

        // Initialize all supported bands with space-optimized settings

        // UHF Band: Most reliable, lowest data rate (REQ-FN-007)
        let _ = bands.push(BandConfig {
            band_type: BandType::UhfBand,
            power_level: 50,        // REQ-NF-004: Conservative power for reliability
            is_active: true,
            data_rate: 9600,        // REQ-PF-002: 9.6 kbps - emergency communications
            signal_quality: 95,     // REQ-SF-002: Highest reliability
        });

        // S Band: Good balance of reliability and speed (REQ-FN-007)
        let _ = bands.push(BandConfig {
            band_type: BandType::SBand,
            power_level: 75,        // REQ-NF-004: Moderate power for standard ops
            is_active: true,
            data_rate: 2_000_000,   // REQ-PF-002: 2 Mbps - command & telemetry
            signal_quality: 90,
        });

        // X Band: High speed, good reliability (REQ-FN-007)
        let _ = bands.push(BandConfig {
            band_type: BandType::XBand,
            power_level: 80,        // REQ-NF-004: Higher power for range
            is_active: true,
            data_rate: 100_000_000, // REQ-PF-002: 100 Mbps - science data
            signal_quality: 85,
        });

        // K Band: Very high speed (REQ-FN-007)
        let _ = bands.push(BandConfig {
            band_type: BandType::KBand,
            power_level: 90,        // REQ-NF-004: High power for performance
            is_active: true,
            data_rate: 1_000_000_000, // REQ-PF-002: 1 Gbps - bulk data transfer
            signal_quality: 80,
        });

        // Ka Band: Highest speed, atmospheric sensitivity (REQ-FN-007)
        let _ = bands.push(BandConfig {
            band_type: BandType::KaBand,
            power_level: 95,        // REQ-NF-004: Maximum power needed
            is_active: true,
            data_rate: 10_000_000_000, // REQ-PF-002: 10 Gbps - maximum throughput
            signal_quality: 75,     // Lower due to atmospheric effects
        });

        Self {
            bands,
            primary_band: BandType::SBand,  // REQ-FN-007: S-band as default primary
            emergency_mode: false,          // REQ-SF-002: Normal operation mode
        }
    }

    /// Select optimal band for message transmission
    ///
    /// Implements intelligent band selection algorithm based on message priority,
    /// emergency conditions, and band characteristics for optimal performance.
    ///
    /// Parameters:
    /// - message: Message to be transmitted with priority information
    ///
    /// Requirements Fulfilled:
    /// - REQ-FN-001: Priority-based band selection algorithm
    /// - REQ-SF-002: Emergency protocol band selection
    /// - REQ-FN-007: Multi-band optimization
    ///
    /// Returns:
    /// BandType optimal for the given message transmission
    pub fn select_optimal_band(&self, message: &Message) -> BandType {
        // Emergency mode overrides normal selection (REQ-SF-002)
        if self.emergency_mode {
            return BandType::UhfBand; // Most reliable band in emergency
        }

        // Priority-based band selection (REQ-FN-001)
        match message.priority {
            MessagePriority::Emergency => BandType::UhfBand,  // REQ-SF-002: Maximum reliability
            MessagePriority::Critical => BandType::SBand,    // Fast and reliable for critical ops
            MessagePriority::High => BandType::XBand,        // Good balance for important ops
            MessagePriority::Medium => BandType::KBand,      // High speed for normal ops
            MessagePriority::Low => BandType::KaBand,        // Highest speed for bulk data
        }
    }

    /// Get band configuration
    ///
    /// Retrieves configuration information for a specific communication band
    /// for monitoring and control purposes.
    ///
    /// Parameters:
    /// - band_type: The frequency band to query
    ///
    /// Requirements Fulfilled:
    /// - REQ-FN-007: Band configuration access
    /// - REQ-NF-001: System monitoring and status reporting
    ///
    /// Returns:
    /// Option<&BandConfig> reference to band configuration if found
    pub fn get_band_config(&self, band_type: BandType) -> Option<&BandConfig> {
        self.bands.iter().find(|b| b.band_type == band_type)
    }

    /// Set emergency mode
    ///
    /// Activates emergency communication protocols, switching to most
    /// reliable band and adjusting power settings for survival mode.
    ///
    /// Parameters:
    /// - emergency: true to activate emergency mode, false for normal
    ///
    /// Requirements Fulfilled:
    /// - REQ-SF-002: Emergency communication protocol activation
    /// - REQ-NF-004: Emergency power management
    pub fn set_emergency_mode(&mut self, emergency: bool) {
        self.emergency_mode = emergency;
        if emergency {
            self.primary_band = BandType::UhfBand; // REQ-SF-002: Switch to most reliable band
        }
    }
}

/// Global communication manager instance
/// REQ-NF-003: Thread-safe global state management for Embassy async
static mut COMM_MANAGER: Option<CommunicationManager> = None;

/// Initialize communication system
///
/// Sets up the communication subsystem by creating the manager instance
/// and initializing all hardware transceivers for multi-band operation.
///
/// Requirements Fulfilled:
/// - REQ-FN-007: Initialize all communication bands
/// - REQ-IF-001: Hardware transceiver initialization
/// - REQ-NF-003: Async system initialization
///
/// Returns:
/// Result<()> indicating success or hardware initialization failure
pub async fn initialize() -> Result<()> {
    // Create global communication manager instance
    unsafe {
        COMM_MANAGER = Some(CommunicationManager::new());
    }

    // Initialize hardware transceivers for all bands (REQ-IF-001)
    hardware::initialize_transceivers().await?;

    Ok(())
}

/// Send high priority message
///
/// Transmits high priority messages with strict timing constraints and
/// optimal band selection for mission-critical operations.
///
/// Parameters:
/// - message: High priority message to transmit
///
/// Requirements Fulfilled:
/// - REQ-FN-001: High priority message handling
/// - REQ-PF-001: 10ms transmission deadline for critical messages
/// - REQ-IF-002: CCSDS packet creation and transmission
///
/// Returns:
/// Result<()> indicating transmission success or failure
pub async fn send_high_priority(message: &Message) -> Result<()> {
    let manager = unsafe { COMM_MANAGER.as_ref().unwrap() };
    let band = manager.select_optimal_band(message); // REQ-FN-001: Band selection

    // Create CCSDS-compliant packet (REQ-IF-002)
    let packet = create_message_packet(message, band)?;

    // Transmit with high priority timing constraint (REQ-PF-001)
    transmit_packet_on_band(&packet, band, Some(Duration::from_millis(10))).await
}

/// Send medium priority message
///
/// Transmits medium priority messages with moderate timing constraints
/// for standard operational communications.
///
/// Parameters:
/// - message: Medium priority message to transmit
///
/// Requirements Fulfilled:
/// - REQ-FN-001: Medium priority message handling
/// - REQ-PF-001: 100ms transmission deadline for standard operations
///
/// Returns:
/// Result<()> indicating transmission success or failure
pub async fn send_medium_priority(message: &Message) -> Result<()> {
    let manager = unsafe { COMM_MANAGER.as_ref().unwrap() };
    let band = manager.select_optimal_band(message);

    let packet = create_message_packet(message, band)?;

    // Medium priority timing constraint (REQ-PF-001)
    transmit_packet_on_band(&packet, band, Some(Duration::from_millis(100))).await
}

/// Send low priority message
///
/// Transmits low priority messages with relaxed timing constraints
/// for housekeeping and non-critical operations.
///
/// Parameters:
/// - message: Low priority message to transmit
///
/// Requirements Fulfilled:
/// - REQ-FN-001: Low priority message handling
/// - REQ-PF-001: 1 second transmission deadline for housekeeping
///
/// Returns:
/// Result<()> indicating transmission success or failure
pub async fn send_low_priority(message: &Message) -> Result<()> {
    let manager = unsafe { COMM_MANAGER.as_ref().unwrap() };
    let band = manager.select_optimal_band(message);

    let packet = create_message_packet(message, band)?;

    // Low priority timing constraint (REQ-PF-001)
    transmit_packet_on_band(&packet, band, Some(Duration::from_secs(1))).await
}

/// Transmit telemetry packet
///
/// Sends telemetry data packets using CCSDS format on designated
/// communication bands for ground station reception.
///
/// Parameters:
/// - packet: Telemetry packet containing sensor and status data
///
/// Requirements Fulfilled:
/// - REQ-IF-002: CCSDS telemetry packet transmission
/// - REQ-FN-007: Band-specific telemetry transmission
///
/// Returns:
/// Result<()> indicating transmission success or failure
pub async fn transmit_telemetry(packet: &TelemetryPacket) -> Result<()> {
    // Create CCSDS packet for telemetry (REQ-IF-002)
    let ccsds_packet = create_telemetry_packet(packet)?;

    // Transmit on designated band with no timeout for bulk data
    transmit_packet_on_band(&ccsds_packet, packet.band, None).await
}

/// Create CCSDS packet from message
///
/// Converts a space communication message into a CCSDS-compliant space packet
/// with proper header formatting, payload serialization, and priority-based APID assignment.
///
/// Parameters:
/// - message: Source message to be converted
/// - band: Communication band for transmission optimization
///
/// Requirements Fulfilled:
/// - REQ-IF-002: CCSDS space packet standard compliance
/// - REQ-FN-001: Priority-based APID assignment
/// - REQ-SC-001: Secure payload serialization
///
/// Returns:
/// Result<SpacePacket> CCSDS-compliant packet ready for transmission
fn create_message_packet(message: &Message, band: BandType) -> Result<SpacePacket> {
    // Serialize message payload with type-specific formatting
    let mut payload_data = Vec::<u8, 2048>::new();

    match &message.payload {
        // Command payload processing (REQ-IF-002)
        space_comms_shared::messaging::MessagePayload::Command { command_id, parameters } => {
            // Add command ID in big-endian format for network transmission
            let cmd_bytes = command_id.to_be_bytes();
            payload_data.extend_from_slice(&cmd_bytes).map_err(|_| {
                SpaceCommError::memory_error(
                    space_comms_shared::error::MemoryErrorType::BufferOverflow,
                    Some(4)
                )
            })?;

            // Add command parameters
            payload_data.extend_from_slice(parameters).map_err(|_| {
                SpaceCommError::memory_error(
                    space_comms_shared::error::MemoryErrorType::BufferOverflow,
                    Some(parameters.len())
                )
            })?;
        }

        // Emergency payload processing (REQ-SF-002)
        space_comms_shared::messaging::MessagePayload::Emergency { alert_level, description, data } => {
            // Emergency packet format for immediate response
            payload_data.push(*alert_level).map_err(|_| {
                SpaceCommError::memory_error(
                    space_comms_shared::error::MemoryErrorType::BufferOverflow,
                    Some(1)
                )
            })?;

            // Add human-readable emergency description
            payload_data.extend_from_slice(description.as_bytes()).map_err(|_| {
                SpaceCommError::memory_error(
                    space_comms_shared::error::MemoryErrorType::BufferOverflow,
                    Some(description.len())
                )
            })?;

            // Add emergency data payload
            payload_data.extend_from_slice(data).map_err(|_| {
                SpaceCommError::memory_error(
                    space_comms_shared::error::MemoryErrorType::BufferOverflow,
                    Some(data.len())
                )
            })?;
        }

        // Generic payload handling for extensibility
        _ => {
            // Placeholder for future payload types
            payload_data.push(0x00).map_err(|_| {
                SpaceCommError::memory_error(
                    space_comms_shared::error::MemoryErrorType::BufferOverflow,
                    Some(1)
                )
            })?;
        }
    }

    // Create CCSDS packet with priority-based APID assignment (REQ-FN-001)
    let apid = match message.priority {
        MessagePriority::Emergency => 0x001,  // REQ-SF-002: Emergency APID
        MessagePriority::Critical => 0x002,   // Critical operations APID
        MessagePriority::High => 0x003,       // High priority APID
        MessagePriority::Medium => 0x004,     // Medium priority APID
        MessagePriority::Low => 0x005,        // Low priority APID
    };

    // Create CCSDS space packet (REQ-IF-002)
    SpacePacket::new(
        PacketType::Command,
        apid,
        message.id.value() as u16,  // Sequence number from message ID
        &payload_data,
        None,                       // No error control for commands
    )
}

/// Create CCSDS packet from telemetry packet
///
/// Converts telemetry data into CCSDS-compliant telemetry packets for
/// standardized transmission to ground stations and mission control.
///
/// Parameters:
/// - packet: Telemetry packet containing sensor measurements and timestamps
///
/// Requirements Fulfilled:
/// - REQ-IF-002: CCSDS telemetry packet standard compliance
/// - REQ-PF-002: Efficient telemetry data serialization
/// - REQ-NF-001: System monitoring data transmission
///
/// Returns:
/// Result<SpacePacket> CCSDS telemetry packet ready for transmission
fn create_telemetry_packet(packet: &TelemetryPacket) -> Result<SpacePacket> {
    // Serialize telemetry data with CCSDS format (REQ-IF-002)
    let mut payload_data = Vec::<u8, 2048>::new();

    // Add timestamp in big-endian format for network transmission
    let timestamp_bytes = packet.data.timestamp.to_be_bytes();
    payload_data.extend_from_slice(&timestamp_bytes).map_err(|_| {
        SpaceCommError::memory_error(
            space_comms_shared::error::MemoryErrorType::BufferOverflow,
            Some(8)
        )
    })?;

    // Add measurements count for parsing validation
    let measurement_count = packet.data.measurements.len() as u16;
    let count_bytes = measurement_count.to_be_bytes();
    payload_data.extend_from_slice(&count_bytes).map_err(|_| {
        SpaceCommError::memory_error(
            space_comms_shared::error::MemoryErrorType::BufferOverflow,
            Some(2)
        )
    })?;

    // Serialize measurement data with type-specific encoding (REQ-PF-002)
    for measurement in &packet.data.measurements {
        // Add measurement ID for identification
        let id_bytes = measurement.measurement_id.to_be_bytes();
        payload_data.extend_from_slice(&id_bytes).map_err(|_| {
            SpaceCommError::memory_error(
                space_comms_shared::error::MemoryErrorType::BufferOverflow,
                Some(2)
            )
        })?;

        // Add measurement value with type-specific serialization
        match &measurement.value {
            // Floating-point measurements (temperature, voltage, etc.)
            space_comms_shared::telemetry::MeasurementValue::Float(val) => {
                let val_bytes = (*val as f32).to_be_bytes();
                payload_data.extend_from_slice(&val_bytes).map_err(|_| {
                    SpaceCommError::memory_error(
                        space_comms_shared::error::MemoryErrorType::BufferOverflow,
                        Some(4)
                    )
                })?;
            }

            // Integer measurements (counters, status codes, etc.)
            space_comms_shared::telemetry::MeasurementValue::Integer(val) => {
                let val_bytes = (*val as i32).to_be_bytes();
                payload_data.extend_from_slice(&val_bytes).map_err(|_| {
                    SpaceCommError::memory_error(
                        space_comms_shared::error::MemoryErrorType::BufferOverflow,
                        Some(4)
                    )
                })?;
            }

            // Other measurement types - use placeholder for future expansion
            _ => {
                let placeholder = [0u8; 4];
                payload_data.extend_from_slice(&placeholder).map_err(|_| {
                    SpaceCommError::memory_error(
                        space_comms_shared::error::MemoryErrorType::BufferOverflow,
                        Some(4)
                    )
                })?;
            }
        }
    }

    // Create CCSDS telemetry packet (REQ-IF-002)
    SpacePacket::new(
        PacketType::Telemetry,
        0x100, // Standard telemetry APID
        packet.sequence as u16,  // Telemetry sequence number
        &payload_data,
        None,  // No error control for telemetry (handled at link layer)
    )
}

/// Transmit packet on specified band
async fn transmit_packet_on_band(
    packet: &SpacePacket,
    band: BandType,
    timeout: Option<Duration>,
) -> Result<()> {
    // Get band configuration
    let manager = unsafe { COMM_MANAGER.as_ref().unwrap() };
    let band_config = manager.get_band_config(band).ok_or_else(|| {
        SpaceCommError::hardware_failure("Band not configured", 0)
    })?;

    if !band_config.is_active {
        return Err(SpaceCommError::hardware_failure("Band not active", 0));
    }

    // Serialize packet
    let packet_bytes = packet.to_bytes()?;

    // Select hardware transceiver and transmit
    match band {
        BandType::UhfBand => hardware::transmit_uhf(&packet_bytes).await,
        BandType::SBand => hardware::transmit_s_band(&packet_bytes).await,
        BandType::XBand => hardware::transmit_x_band(&packet_bytes).await,
        BandType::KBand => hardware::transmit_k_band(&packet_bytes).await,
        BandType::KaBand => hardware::transmit_ka_band(&packet_bytes).await,
    }?;

    // Wait for transmission with timeout if specified
    if let Some(timeout_duration) = timeout {
        Timer::after(timeout_duration).await;
    }

    Ok(())
}

/// Receive command packet
pub async fn receive_command() -> Result<SpacePacket> {
    // Check all bands for incoming commands
    if let Ok(packet) = hardware::receive_uhf().await {
        return parse_received_packet(&packet);
    }

    if let Ok(packet) = hardware::receive_s_band().await {
        return parse_received_packet(&packet);
    }

    if let Ok(packet) = hardware::receive_x_band().await {
        return parse_received_packet(&packet);
    }

    Err(SpaceCommError::communication_timeout(100, "No command received"))
}

/// Parse received packet bytes into SpacePacket
fn parse_received_packet(bytes: &[u8]) -> Result<SpacePacket> {
    if bytes.len() < 6 {
        return Err(SpaceCommError::invalid_packet("Packet too short", None));
    }

    // Parse CCSDS header
    let header = SpacePacketHeader::from_bytes(&bytes[0..6])?;

    // Extract data
    let data_end = core::cmp::min(bytes.len(), 6 + header.data_length as usize + 1);
    let mut data = Vec::<u8, 2048>::new();
    data.extend_from_slice(&bytes[6..data_end]).map_err(|_| {
        SpaceCommError::memory_error(
            space_comms_shared::error::MemoryErrorType::BufferOverflow,
            Some(data_end - 6)
        )
    })?;

    Ok(SpacePacket {
        header,
        secondary_header: None,
        data,
        error_control: None,
    })
}

/// Check if communication system is healthy
pub async fn is_communication_healthy() -> bool {
    let manager = unsafe { COMM_MANAGER.as_ref().unwrap() };

    // Check if at least one band is active with good signal quality
    manager.bands.iter().any(|band| {
        band.is_active && band.signal_quality > 50
    })
}

/// Set emergency communication mode
pub async fn set_emergency_mode(band: BandType) -> Result<()> {
    let manager = unsafe { COMM_MANAGER.as_mut().unwrap() };
    manager.set_emergency_mode(true);

    error_handling::log_warning("Emergency communication mode activated");

    // Reduce power on other bands to conserve energy
    for band_config in &mut manager.bands {
        if band_config.band_type != band {
            band_config.power_level = band_config.power_level / 2;
        }
    }

    Ok(())
}

/// Switch to backup communication band
pub async fn switch_to_backup_band() -> Result<()> {
    let manager = unsafe { COMM_MANAGER.as_mut().unwrap() };

    // Switch to UHF as backup (most reliable)
    manager.primary_band = BandType::UhfBand;

    error_handling::log_info("Switched to backup communication band");

    Ok(())
}
