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
    /// Construct a fully-validated, integrity-protected CCSDS Space Packet.
    ///
    /// - **ID**: FN-PKT-001
    /// - **Requirement**: Produce a well-formed CCSDS 133.0-B-2 Space Packet
    ///   with a computed Packet Error Control (PEC) field on every construction.
    /// - **Purpose**: Ensure every packet entering the transmission pipeline
    ///   already carries a valid CRC so callers cannot inadvertently omit
    ///   integrity protection.
    /// - **Rationale**: Requiring callers to invoke `calculate_crc()` separately
    ///   creates an error-prone pattern forbidden in safety-critical software.
    ///   Embedding the call in the constructor enforces the invariant.
    /// - **Inputs**:
    ///   - `packet_type`: `Command` or `Telemetry` per CCSDS §3.2.
    ///   - `apid`: Application Process Identifier, range `0x000`–`0x7FF` (11 bits).
    ///   - `sequence_count`: Packet counter for this APID, range `0`–`0x3FFF` (14 bits).
    ///   - `data`: Payload bytes; maximum 2 048 bytes.
    ///   - `secondary_header`: Optional mission-specific secondary header.
    /// - **Outputs**: `Ok(SpacePacket)` with `error_control` already populated.
    /// - **Preconditions**: `apid ≤ 0x7FF`, `sequence_count ≤ 0x3FFF`,
    ///   `data.len() ≤ 2048`.
    /// - **Postconditions**: `packet.error_control.is_some() == true`;
    ///   `packet.verify_crc() == true`.
    /// - **Failure Modes**: Returns `Err(InvalidPacket)` on constraint violations;
    ///   returns `Err(MemoryError)` if the internal buffer overflows.
    /// - **Error Handling**: All constraint violations produce typed errors for
    ///   caller-level fault isolation.
    /// - **Constraints**: Stack frame ≤ 2 KB; no heap allocation.
    /// - **Verification**: Unit test `test_space_packet_creation` must pass;
    ///   `verify_crc()` must return `true` immediately after construction.
    /// - **References**: CCSDS 133.0-B-2 §4.1; CCSDS 132.0-B-2 §4.1.4.
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

        let mut packet = Self {
            header,
            secondary_header,
            data: packet_data,
            error_control: None,
        };

        // Auto-compute CRC so every packet leaves the constructor integrity-protected.
        // Postcondition: verify_crc() == true immediately after new().
        packet.calculate_crc();

        Ok(packet)
    }

    /// Compute and store the CRC-16/CCITT-FALSE Packet Error Control field.
    ///
    /// - **ID**: FN-PKT-002
    /// - **Requirement**: Populate `error_control` with CRC-16/CCITT-FALSE over
    ///   the complete packet (primary header + optional secondary header + data).
    /// - **Purpose**: Enable the receiver to detect transmission errors on the
    ///   satellite–ground RF link per CCSDS 132.0-B-2 §4.1.4.
    /// - **Inputs**: Mutable reference to `self`; uses internal fields only.
    /// - **Outputs**: Sets `self.error_control`; no return value.
    /// - **Preconditions**: `self.header`, `self.secondary_header`, and `self.data`
    ///   must be fully populated before calling.
    /// - **Postconditions**: `self.error_control == Some(valid_crc)`;
    ///   `self.verify_crc() == true`.
    /// - **Side Effects**: Mutates `self.error_control` only.
    /// - **Failure Modes**: None — CRC computation is infallible for bounded inputs.
    /// - **References**: CCSDS 132.0-B-2 §4.1.4; CCSDS 133.0-B-2 §4.1.5.
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

    /// Verify the stored CRC-16/CCITT-FALSE against a freshly computed value.
    ///
    /// - **ID**: FN-PKT-003
    /// - **Requirement**: Return `true` if and only if the stored `error_control`
    ///   field matches the CRC computed over the current packet contents.
    /// - **Purpose**: Allow the receiver to reject corrupted packets before
    ///   processing their payload, satisfying CCSDS data-integrity requirements.
    /// - **Inputs**: Shared reference to `self`.
    /// - **Outputs**: `true` — packet integrity verified; `false` — corrupted or
    ///   no CRC present.
    /// - **Preconditions**: `calculate_crc()` has been called (or `new()` used).
    /// - **Side Effects**: None — read-only operation.
    /// - **Failure Modes**: Returns `false` (safe default) if `error_control` is
    ///   absent rather than panicking.
    /// - **References**: CCSDS 132.0-B-2 §4.1.4.
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

/// Pre-computed CRC-16/CCITT-FALSE lookup table (polynomial 0x1021).
///
/// - **ID**: OPT-CRC-001
/// - **Requirement**: Compute CRC-16/CCITT-FALSE error control for CCSDS packets
///   in O(n) time with a single table lookup per byte.
/// - **Purpose**: Replace the 8-iteration bit-shift inner loop with a single indexed
///   read, reducing CPU cycles per byte from ~8 to ~1 while preserving identical
///   output values.
/// - **Rationale**: Lookup-table CRC is a well-known aerospace data-integrity
///   optimization used in flight software where processing budget is constrained.
///   The table is generated at compile time (zero runtime overhead).
/// - **Constraints**: Table occupies exactly 512 bytes of read-only flash/ROM.
/// - **Verification**: Any CRC value computed by this table must equal the value
///   produced by the reference bit-by-bit algorithm for all 256 single-byte inputs.
/// - **References**: ITU-T V.42 Annex B; CCSDS 132.0-B-2 §4.1.4.
const CRC16_TABLE: [u16; 256] = {
    let mut table = [0u16; 256];
    let mut i = 0usize;
    while i < 256 {
        let mut crc = (i as u16) << 8;
        let mut j = 0;
        while j < 8 {
            if crc & 0x8000 != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
            j += 1;
        }
        table[i] = crc;
        i += 1;
    }
    table
};

/// Compute CRC-16/CCITT-FALSE over `data`, accumulating into an existing `crc` state.
///
/// - **ID**: FN-CRC-001
/// - **Requirement**: Produce a CRC-16/CCITT-FALSE checksum compatible with
///   CCSDS Packet Error Control (CCSDS 133.0-B-2 §4.1.5).
/// - **Purpose**: Detect single and burst bit errors in space packets traversing
///   noisy RF links according to the CCSDS error-control standard.
/// - **Rationale**: Uses the pre-computed `CRC16_TABLE` for O(1)-per-byte
///   computation instead of an 8-iteration bit loop, reducing cycle count by ~8×
///   while producing bit-identical results.
/// - **Inputs**:
///   - `crc`: Running CRC state; **must** be initialised to `0xFFFF` for the
///     first call in a packet calculation chain.
///   - `data`: Byte slice over which the checksum is computed.
/// - **Outputs**: Updated 16-bit CRC accumulator (chain across multiple calls).
/// - **Preconditions**: `CRC16_TABLE` is fully populated (guaranteed at compile time).
/// - **Postconditions**: Return value equals the CRC-16/CCITT-FALSE of the
///   concatenation of all `data` slices processed since `crc = 0xFFFF`.
/// - **Side Effects**: None — pure function.
/// - **Failure Modes**: None; operates on immutable slice with no allocation.
/// - **Constraints**: Processes at most `usize::MAX` bytes per call.
/// - **Verification**: Cross-validate against reference vector:
///   `crc16_ccitt(0xFFFF, b"123456789")` must equal `0x29B1`.
/// - **References**: CCSDS 132.0-B-2 §4.1.4; ITU-T V.42 Annex B.
fn crc16_ccitt(mut crc: u16, data: &[u8]) -> u16 {
    for &byte in data {
        let idx = (((crc >> 8) ^ (byte as u16)) & 0xFF) as usize;
        crc = (crc << 8) ^ CRC16_TABLE[idx];
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
