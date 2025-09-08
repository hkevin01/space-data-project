//! CCSDS (Consultative Committee for Space Data Systems) protocol implementation
//!
//! This module implements CCSDS space communication standards including:
//! - Space Packet Protocol (CCSDS 133.0-B-1)
//! - Space Data Link Protocol (CCSDS 132.0-B-2)
//! - Advanced Orbiting Systems Networks (CCSDS 135.0-B-4)

use serde::{Deserialize, Serialize};
use crate::error::{Result, SpaceCommError};
use crate::types::{PacketId, ComponentId};

/// CCSDS Space Packet primary header (6 bytes)
///
/// Implements the primary header structure defined in CCSDS 133.0-B-1
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpacePacketHeader {
    /// Packet Version Number (3 bits) - always 000 for CCSDS
    pub version: u8,

    /// Packet Type (1 bit) - 0=telemetry, 1=command
    pub packet_type: PacketType,

    /// Secondary Header Flag (1 bit) - indicates presence of secondary header
    pub secondary_header_flag: bool,

    /// Application Process Identifier (11 bits) - identifies the data source/destination
    pub apid: u16,

    /// Sequence Flags (2 bits) - indicates packet segmentation
    pub sequence_flags: SequenceFlags,

    /// Packet Sequence Count (14 bits) - packet counter for this APID
    pub sequence_count: u16,

    /// Packet Data Length (16 bits) - length of packet data field minus 1
    pub data_length: u16,
}

/// CCSDS packet types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PacketType {
    /// Telemetry packet (spacecraft to ground)
    Telemetry = 0,
    /// Command packet (ground to spacecraft)
    Command = 1,
}

/// CCSDS sequence flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SequenceFlags {
    /// Continuation segment (not first, not last)
    Continuation = 0b00,
    /// First segment of multi-packet data
    FirstSegment = 0b01,
    /// Last segment of multi-packet data
    LastSegment = 0b10,
    /// Unsegmented packet (complete data in single packet)
    Unsegmented = 0b11,
}

impl SpacePacketHeader {
    /// Create a new space packet header
    pub fn new(
        packet_type: PacketType,
        apid: u16,
        sequence_count: u16,
        data_length: u16,
        has_secondary_header: bool,
    ) -> Result<Self> {
        // Validate APID (11 bits max)
        if apid > 0x7FF {
            return Err(SpaceCommError::invalid_packet(
                "APID exceeds 11-bit maximum",
                None,
            ));
        }

        // Validate sequence count (14 bits max)
        if sequence_count > 0x3FFF {
            return Err(SpaceCommError::invalid_packet(
                "Sequence count exceeds 14-bit maximum",
                None,
            ));
        }

        Ok(Self {
            version: 0, // Always 0 for CCSDS
            packet_type,
            secondary_header_flag: has_secondary_header,
            apid,
            sequence_flags: SequenceFlags::Unsegmented,
            sequence_count,
            data_length,
        })
    }

    /// Serialize header to bytes (big-endian, CCSDS standard)
    pub fn to_bytes(&self) -> [u8; 6] {
        let mut bytes = [0u8; 6];

        // First 16 bits: Version(3) + Type(1) + SecHdr(1) + APID(11)
        let first_word = ((self.version as u16) << 13) |
                         ((self.packet_type as u16) << 12) |
                         ((self.secondary_header_flag as u16) << 11) |
                         self.apid;
        bytes[0] = (first_word >> 8) as u8;
        bytes[1] = first_word as u8;

        // Second 16 bits: SequenceFlags(2) + SequenceCount(14)
        let second_word = ((self.sequence_flags as u16) << 14) | self.sequence_count;
        bytes[2] = (second_word >> 8) as u8;
        bytes[3] = second_word as u8;

        // Third 16 bits: Data Length
        bytes[4] = (self.data_length >> 8) as u8;
        bytes[5] = self.data_length as u8;

        bytes
    }

    /// Deserialize header from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 6 {
            return Err(SpaceCommError::invalid_packet(
                "Insufficient bytes for CCSDS header",
                None,
            ));
        }

        // Parse first 16 bits
        let first_word = ((bytes[0] as u16) << 8) | (bytes[1] as u16);
        let version = ((first_word >> 13) & 0x07) as u8;
        let packet_type = if (first_word >> 12) & 0x01 == 0 {
            PacketType::Telemetry
        } else {
            PacketType::Command
        };
        let secondary_header_flag = ((first_word >> 11) & 0x01) != 0;
        let apid = first_word & 0x7FF;

        // Parse second 16 bits
        let second_word = ((bytes[2] as u16) << 8) | (bytes[3] as u16);
        let sequence_flags = match (second_word >> 14) & 0x03 {
            0b00 => SequenceFlags::Continuation,
            0b01 => SequenceFlags::FirstSegment,
            0b10 => SequenceFlags::LastSegment,
            0b11 => SequenceFlags::Unsegmented,
            _ => unreachable!(),
        };
        let sequence_count = second_word & 0x3FFF;

        // Parse third 16 bits
        let data_length = ((bytes[4] as u16) << 8) | (bytes[5] as u16);

        // Validate version
        if version != 0 {
            return Err(SpaceCommError::invalid_packet(
                "Invalid CCSDS version number",
                None,
            ));
        }

        Ok(Self {
            version,
            packet_type,
            secondary_header_flag,
            apid,
            sequence_flags,
            sequence_count,
            data_length,
        })
    }

    /// Get the total packet length including header
    pub fn total_packet_length(&self) -> usize {
        6 + self.data_length as usize + 1 // +1 because data_length is length-1
    }
}

/// CCSDS Space Packet structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpacePacket {
    /// Primary header (always present)
    pub header: SpacePacketHeader,

    /// Secondary header (optional, depends on header flag)
    pub secondary_header: Option<SecondaryHeader>,

    /// Packet data field
    pub data: heapless::Vec<u8, 2048>,

    /// Packet error control (optional checksum/CRC)
    pub error_control: Option<u16>,
}

/// Secondary header structure (mission-specific)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecondaryHeader {
    /// Time stamp (format depends on mission)
    pub timestamp: u64,

    /// Additional mission-specific data
    pub mission_data: heapless::Vec<u8, 64>,
}

impl SpacePacket {
    /// Create a new space packet
    pub fn new(
        packet_type: PacketType,
        apid: u16,
        sequence_count: u16,
        data: &[u8],
        secondary_header: Option<SecondaryHeader>,
    ) -> Result<Self> {
        if data.len() > 2048 {
            return Err(SpaceCommError::invalid_packet(
                "Data exceeds maximum packet size",
                None,
            ));
        }

        let data_length = data.len() +
                         if secondary_header.is_some() { 8 } else { 0 } + // Estimated secondary header size
                         if secondary_header.is_some() { 2 } else { 0 };   // Error control size

        let header = SpacePacketHeader::new(
            packet_type,
            apid,
            sequence_count,
            (data_length.saturating_sub(1)) as u16, // CCSDS data_length is actual_length - 1
            secondary_header.is_some(),
        )?;

        let mut packet_data = heapless::Vec::new();
        packet_data.extend_from_slice(data).map_err(|_| {
            SpaceCommError::memory_error(
                crate::error::MemoryErrorType::BufferOverflow,
                Some(data.len()),
            )
        })?;

        Ok(Self {
            header,
            secondary_header,
            data: packet_data,
            error_control: None,
        })
    }

    /// Calculate and set CRC-16 error control
    pub fn calculate_crc(&mut self) {
        let mut crc = crc16_ccitt(0xFFFF, &self.header.to_bytes());

        if let Some(ref sec_hdr) = self.secondary_header {
            // Simplified secondary header CRC calculation
            let timestamp_bytes = sec_hdr.timestamp.to_be_bytes();
            crc = crc16_ccitt(crc, &timestamp_bytes);
            crc = crc16_ccitt(crc, &sec_hdr.mission_data);
        }

        crc = crc16_ccitt(crc, &self.data);
        self.error_control = Some(crc);
    }

    /// Verify packet CRC
    pub fn verify_crc(&self) -> bool {
        if let Some(stored_crc) = self.error_control {
            let mut calculated_crc = crc16_ccitt(0xFFFF, &self.header.to_bytes());

            if let Some(ref sec_hdr) = self.secondary_header {
                let timestamp_bytes = sec_hdr.timestamp.to_be_bytes();
                calculated_crc = crc16_ccitt(calculated_crc, &timestamp_bytes);
                calculated_crc = crc16_ccitt(calculated_crc, &sec_hdr.mission_data);
            }

            calculated_crc = crc16_ccitt(calculated_crc, &self.data);
            calculated_crc == stored_crc
        } else {
            false // No CRC to verify
        }
    }

    /// Serialize packet to bytes
    pub fn to_bytes(&self) -> Result<heapless::Vec<u8, 4096>> {
        let mut bytes = heapless::Vec::new();

        // Add primary header
        bytes.extend_from_slice(&self.header.to_bytes()).map_err(|_| {
            SpaceCommError::memory_error(
                crate::error::MemoryErrorType::BufferOverflow,
                Some(6),
            )
        })?;

        // Add secondary header if present
        if let Some(ref sec_hdr) = self.secondary_header {
            bytes.extend_from_slice(&sec_hdr.timestamp.to_be_bytes()).map_err(|_| {
                SpaceCommError::memory_error(
                    crate::error::MemoryErrorType::BufferOverflow,
                    Some(8),
                )
            })?;
            bytes.extend_from_slice(&sec_hdr.mission_data).map_err(|_| {
                SpaceCommError::memory_error(
                    crate::error::MemoryErrorType::BufferOverflow,
                    Some(sec_hdr.mission_data.len()),
                )
            })?;
        }

        // Add data
        bytes.extend_from_slice(&self.data).map_err(|_| {
            SpaceCommError::memory_error(
                crate::error::MemoryErrorType::BufferOverflow,
                Some(self.data.len()),
            )
        })?;

        // Add error control if present
        if let Some(crc) = self.error_control {
            bytes.extend_from_slice(&crc.to_be_bytes()).map_err(|_| {
                SpaceCommError::memory_error(
                    crate::error::MemoryErrorType::BufferOverflow,
                    Some(2),
                )
            })?;
        }

        Ok(bytes)
    }

    /// Get unique packet identifier
    pub fn packet_id(&self) -> PacketId {
        // Combine APID and sequence count for unique ID
        let id = ((self.header.apid as u32) << 16) | (self.header.sequence_count as u32);
        PacketId::new(id)
    }
}

/// Calculate CRC-16-CCITT (polynomial 0x1021)
fn crc16_ccitt(mut crc: u16, data: &[u8]) -> u16 {
    for &byte in data {
        crc ^= (byte as u16) << 8;
        for _ in 0..8 {
            if crc & 0x8000 != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}

/// Virtual Channel Access Unit (VCAU) for Space Data Link Protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualChannelAccessUnit {
    /// Master Channel Identifier
    pub mc_id: u8,

    /// Virtual Channel Identifier
    pub vc_id: u8,

    /// Frame sequence number
    pub frame_sequence: u16,

    /// Space packet(s) in this frame
    pub packets: heapless::Vec<SpacePacket, 8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_space_packet_header_serialization() {
        let header = SpacePacketHeader::new(
            PacketType::Telemetry,
            0x123,
            0x456,
            100,
            false,
        ).unwrap();

        let bytes = header.to_bytes();
        let deserialized = SpacePacketHeader::from_bytes(&bytes).unwrap();

        assert_eq!(header, deserialized);
    }

    #[test]
    fn test_space_packet_creation() {
        let data = b"Hello, space!";
        let packet = SpacePacket::new(
            PacketType::Command,
            0x100,
            1,
            data,
            None,
        ).unwrap();

        assert_eq!(packet.header.packet_type, PacketType::Command);
        assert_eq!(packet.header.apid, 0x100);
        assert_eq!(packet.data.as_slice(), data);
    }

    #[test]
    fn test_crc_calculation() {
        let mut packet = SpacePacket::new(
            PacketType::Telemetry,
            0x200,
            42,
            b"test data",
            None,
        ).unwrap();

        packet.calculate_crc();
        assert!(packet.error_control.is_some());
        assert!(packet.verify_crc());
    }

    #[test]
    fn test_invalid_apid() {
        let result = SpacePacketHeader::new(
            PacketType::Telemetry,
            0x800, // Too large for 11 bits
            0,
            0,
            false,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_crc16_ccitt() {
        // Test vector for CRC-16-CCITT
        let data = b"123456789";
        let crc = crc16_ccitt(0xFFFF, data);
        assert_eq!(crc, 0x29B1); // Known result for this test vector
    }
}
