//! Communication module for satellite system
//!
//! Handles multi-band communication including K-band, Ka-band, X-band, S-band, and UHF-band.
//! Implements band selection based on message priority and environmental conditions.

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
#[derive(Debug, Clone)]
pub struct BandConfig {
    /// Frequency band type
    pub band_type: BandType,
    /// Current power level (0-100%)
    pub power_level: u8,
    /// Is band currently active
    pub is_active: bool,
    /// Current data rate in bps
    pub data_rate: u64,
    /// Signal quality (0-100%)
    pub signal_quality: u8,
}

/// Communication manager state
pub struct CommunicationManager {
    /// Active communication bands
    bands: Vec<BandConfig, 5>,
    /// Current primary band
    primary_band: BandType,
    /// Emergency mode flag
    emergency_mode: bool,
}

impl CommunicationManager {
    /// Create new communication manager
    pub fn new() -> Self {
        let mut bands = Vec::new();

        // Initialize all supported bands
        let _ = bands.push(BandConfig {
            band_type: BandType::UhfBand,
            power_level: 50,
            is_active: true,
            data_rate: 9600, // 9.6 kbps
            signal_quality: 95,
        });

        let _ = bands.push(BandConfig {
            band_type: BandType::SBand,
            power_level: 75,
            is_active: true,
            data_rate: 2_000_000, // 2 Mbps
            signal_quality: 90,
        });

        let _ = bands.push(BandConfig {
            band_type: BandType::XBand,
            power_level: 80,
            is_active: true,
            data_rate: 100_000_000, // 100 Mbps
            signal_quality: 85,
        });

        let _ = bands.push(BandConfig {
            band_type: BandType::KBand,
            power_level: 90,
            is_active: true,
            data_rate: 1_000_000_000, // 1 Gbps
            signal_quality: 80,
        });

        let _ = bands.push(BandConfig {
            band_type: BandType::KaBand,
            power_level: 95,
            is_active: true,
            data_rate: 10_000_000_000, // 10 Gbps
            signal_quality: 75,
        });

        Self {
            bands,
            primary_band: BandType::SBand,
            emergency_mode: false,
        }
    }

    /// Select optimal band for message transmission
    pub fn select_optimal_band(&self, message: &Message) -> BandType {
        if self.emergency_mode {
            return BandType::UhfBand; // Most reliable in emergency
        }

        match message.priority {
            MessagePriority::Emergency => BandType::UhfBand,  // Most reliable
            MessagePriority::Critical => BandType::SBand,    // Fast and reliable
            MessagePriority::High => BandType::XBand,        // Good balance
            MessagePriority::Medium => BandType::KBand,      // High speed
            MessagePriority::Low => BandType::KaBand,        // Highest speed
        }
    }

    /// Get band configuration
    pub fn get_band_config(&self, band_type: BandType) -> Option<&BandConfig> {
        self.bands.iter().find(|b| b.band_type == band_type)
    }

    /// Set emergency mode
    pub fn set_emergency_mode(&mut self, emergency: bool) {
        self.emergency_mode = emergency;
        if emergency {
            self.primary_band = BandType::UhfBand;
        }
    }
}

/// Global communication manager instance
static mut COMM_MANAGER: Option<CommunicationManager> = None;

/// Initialize communication system
pub async fn initialize() -> Result<()> {
    unsafe {
        COMM_MANAGER = Some(CommunicationManager::new());
    }

    // Initialize hardware transceivers
    hardware::initialize_transceivers().await?;

    Ok(())
}

/// Send high priority message
pub async fn send_high_priority(message: &Message) -> Result<()> {
    let manager = unsafe { COMM_MANAGER.as_ref().unwrap() };
    let band = manager.select_optimal_band(message);

    // Create CCSDS packet
    let packet = create_message_packet(message, band)?;

    // Transmit with high priority timing
    transmit_packet_on_band(&packet, band, Some(Duration::from_millis(10))).await
}

/// Send medium priority message
pub async fn send_medium_priority(message: &Message) -> Result<()> {
    let manager = unsafe { COMM_MANAGER.as_ref().unwrap() };
    let band = manager.select_optimal_band(message);

    let packet = create_message_packet(message, band)?;
    transmit_packet_on_band(&packet, band, Some(Duration::from_millis(100))).await
}

/// Send low priority message
pub async fn send_low_priority(message: &Message) -> Result<()> {
    let manager = unsafe { COMM_MANAGER.as_ref().unwrap() };
    let band = manager.select_optimal_band(message);

    let packet = create_message_packet(message, band)?;
    transmit_packet_on_band(&packet, band, Some(Duration::from_secs(1))).await
}

/// Transmit telemetry packet
pub async fn transmit_telemetry(packet: &TelemetryPacket) -> Result<()> {
    // Create CCSDS packet for telemetry
    let ccsds_packet = create_telemetry_packet(packet)?;

    // Transmit on designated band
    transmit_packet_on_band(&ccsds_packet, packet.band, None).await
}

/// Create CCSDS packet from message
fn create_message_packet(message: &Message, band: BandType) -> Result<SpacePacket> {
    // Serialize message payload
    let mut payload_data = Vec::<u8, 2048>::new();

    match &message.payload {
        space_comms_shared::messaging::MessagePayload::Command { command_id, parameters } => {
            // Add command ID
            let cmd_bytes = command_id.to_be_bytes();
            payload_data.extend_from_slice(&cmd_bytes).map_err(|_| {
                SpaceCommError::memory_error(
                    space_comms_shared::error::MemoryErrorType::BufferOverflow,
                    Some(4)
                )
            })?;

            // Add parameters
            payload_data.extend_from_slice(parameters).map_err(|_| {
                SpaceCommError::memory_error(
                    space_comms_shared::error::MemoryErrorType::BufferOverflow,
                    Some(parameters.len())
                )
            })?;
        }
        space_comms_shared::messaging::MessagePayload::Emergency { alert_level, description, data } => {
            // Emergency packet format
            payload_data.push(*alert_level).map_err(|_| {
                SpaceCommError::memory_error(
                    space_comms_shared::error::MemoryErrorType::BufferOverflow,
                    Some(1)
                )
            })?;

            payload_data.extend_from_slice(description.as_bytes()).map_err(|_| {
                SpaceCommError::memory_error(
                    space_comms_shared::error::MemoryErrorType::BufferOverflow,
                    Some(description.len())
                )
            })?;

            payload_data.extend_from_slice(data).map_err(|_| {
                SpaceCommError::memory_error(
                    space_comms_shared::error::MemoryErrorType::BufferOverflow,
                    Some(data.len())
                )
            })?;
        }
        _ => {
            // Generic payload handling
            // For now, just add a placeholder
            payload_data.push(0x00).map_err(|_| {
                SpaceCommError::memory_error(
                    space_comms_shared::error::MemoryErrorType::BufferOverflow,
                    Some(1)
                )
            })?;
        }
    }

    // Create CCSDS packet
    let apid = match message.priority {
        MessagePriority::Emergency => 0x001,
        MessagePriority::Critical => 0x002,
        MessagePriority::High => 0x003,
        MessagePriority::Medium => 0x004,
        MessagePriority::Low => 0x005,
    };

    SpacePacket::new(
        PacketType::Command,
        apid,
        message.id.value() as u16,
        &payload_data,
        None,
    )
}

/// Create CCSDS packet from telemetry packet
fn create_telemetry_packet(packet: &TelemetryPacket) -> Result<SpacePacket> {
    // Serialize telemetry data (simplified)
    let mut payload_data = Vec::<u8, 2048>::new();

    // Add timestamp
    let timestamp_bytes = packet.data.timestamp.to_be_bytes();
    payload_data.extend_from_slice(&timestamp_bytes).map_err(|_| {
        SpaceCommError::memory_error(
            space_comms_shared::error::MemoryErrorType::BufferOverflow,
            Some(8)
        )
    })?;

    // Add measurements count
    let measurement_count = packet.data.measurements.len() as u16;
    let count_bytes = measurement_count.to_be_bytes();
    payload_data.extend_from_slice(&count_bytes).map_err(|_| {
        SpaceCommError::memory_error(
            space_comms_shared::error::MemoryErrorType::BufferOverflow,
            Some(2)
        )
    })?;

    // Add simplified measurement data
    for measurement in &packet.data.measurements {
        let id_bytes = measurement.measurement_id.to_be_bytes();
        payload_data.extend_from_slice(&id_bytes).map_err(|_| {
            SpaceCommError::memory_error(
                space_comms_shared::error::MemoryErrorType::BufferOverflow,
                Some(2)
            )
        })?;

        // Add measurement value (simplified as f32)
        match &measurement.value {
            space_comms_shared::telemetry::MeasurementValue::Float(val) => {
                let val_bytes = (*val as f32).to_be_bytes();
                payload_data.extend_from_slice(&val_bytes).map_err(|_| {
                    SpaceCommError::memory_error(
                        space_comms_shared::error::MemoryErrorType::BufferOverflow,
                        Some(4)
                    )
                })?;
            }
            space_comms_shared::telemetry::MeasurementValue::Integer(val) => {
                let val_bytes = (*val as i32).to_be_bytes();
                payload_data.extend_from_slice(&val_bytes).map_err(|_| {
                    SpaceCommError::memory_error(
                        space_comms_shared::error::MemoryErrorType::BufferOverflow,
                        Some(4)
                    )
                })?;
            }
            _ => {
                // Other types - add placeholder
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

    SpacePacket::new(
        PacketType::Telemetry,
        0x100, // Telemetry APID
        packet.sequence as u16,
        &payload_data,
        None,
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
