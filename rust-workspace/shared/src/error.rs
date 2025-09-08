//! Error types and handling for space communication systems
//!
//! This module provides comprehensive error handling following NASA and DoD standards
//! for mission-critical systems. All errors are designed to be informative and
//! actionable for debugging and system recovery.

use core::fmt;

#[cfg(feature = "std")]
use std::error::Error as StdError;

use serde::{Deserialize, Serialize};

/// Standard result type for space communication operations
pub type Result<T> = core::result::Result<T, SpaceCommError>;

/// Comprehensive error types for space communication systems
///
/// Each error variant provides specific context for debugging and recovery.
/// Error codes follow NASA-STD-8719.13C guidelines for error classification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SpaceCommError {
    /// Communication timeout occurred
    ///
    /// This error indicates that a communication operation exceeded the
    /// maximum allowed time limit.
    CommunicationTimeout {
        /// Duration of timeout in milliseconds
        timeout_ms: u64,
        /// Operation that timed out
        operation: &'static str,
    },

    /// Invalid packet structure or content
    ///
    /// Indicates that a received packet does not conform to expected format
    /// or contains invalid data.
    InvalidPacket {
        /// Reason for packet invalidity
        reason: &'static str,
        /// Packet ID if available
        packet_id: Option<u32>,
    },

    /// Hardware component failure
    ///
    /// Represents a failure in a specific hardware component that affects
    /// communication capabilities.
    HardwareFailure {
        /// Name of failed component
        component: &'static str,
        /// Error code from hardware
        error_code: u32,
    },

    /// Memory allocation or management error
    ///
    /// Indicates insufficient memory or memory management issues.
    MemoryError {
        /// Type of memory error
        error_type: MemoryErrorType,
        /// Size of allocation that failed (if applicable)
        size: Option<usize>,
    },

    /// Cryptographic operation failure
    ///
    /// Security-related errors including encryption, decryption, and
    /// authentication failures.
    CryptographicError {
        /// Type of cryptographic operation that failed
        operation: CryptoOperation,
        /// Specific error details
        details: &'static str,
    },

    /// Protocol violation or incompatibility
    ///
    /// Indicates that communication protocol requirements were not met.
    ProtocolError {
        /// Expected protocol version
        expected_version: u8,
        /// Received protocol version
        received_version: u8,
        /// Protocol name
        protocol: &'static str,
    },

    /// System resource exhaustion
    ///
    /// Indicates that a system resource (other than memory) has been exhausted.
    ResourceExhausted {
        /// Type of resource
        resource: &'static str,
        /// Current usage level
        current_usage: u32,
        /// Maximum allowed usage
        max_usage: u32,
    },

    /// Configuration or parameter error
    ///
    /// Indicates invalid configuration parameters or system setup.
    ConfigurationError {
        /// Parameter name that is invalid
        parameter: &'static str,
        /// Invalid value
        value: &'static str,
        /// Reason why value is invalid
        reason: &'static str,
    },

    /// Checksum or integrity verification failure
    ///
    /// Data integrity check failed, indicating possible corruption.
    IntegrityError {
        /// Type of integrity check that failed
        check_type: &'static str,
        /// Expected checksum/hash
        expected: &'static str,
        /// Calculated checksum/hash
        calculated: &'static str,
    },
}

/// Memory error subtypes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryErrorType {
    /// Allocation failure
    AllocationFailed,
    /// Buffer overflow prevention
    BufferOverflow,
    /// Out of memory condition
    OutOfMemory,
    /// Memory corruption detected
    CorruptionDetected,
}

/// Cryptographic operation types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CryptoOperation {
    /// Encryption operation
    Encryption,
    /// Decryption operation
    Decryption,
    /// Digital signature creation
    Signing,
    /// Digital signature verification
    Verification,
    /// Key generation
    KeyGeneration,
    /// Key exchange
    KeyExchange,
}

impl fmt::Display for SpaceCommError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpaceCommError::CommunicationTimeout { timeout_ms, operation } => {
                write!(f, "Communication timeout after {}ms during {}", timeout_ms, operation)
            }
            SpaceCommError::InvalidPacket { reason, packet_id } => {
                if let Some(id) = packet_id {
                    write!(f, "Invalid packet (ID: {}): {}", id, reason)
                } else {
                    write!(f, "Invalid packet: {}", reason)
                }
            }
            SpaceCommError::HardwareFailure { component, error_code } => {
                write!(f, "Hardware failure in {}: error code {}", component, error_code)
            }
            SpaceCommError::MemoryError { error_type, size } => {
                match size {
                    Some(s) => write!(f, "Memory error ({:?}): size {}", error_type, s),
                    None => write!(f, "Memory error: {:?}", error_type),
                }
            }
            SpaceCommError::CryptographicError { operation, details } => {
                write!(f, "Cryptographic error during {:?}: {}", operation, details)
            }
            SpaceCommError::ProtocolError { expected_version, received_version, protocol } => {
                write!(
                    f,
                    "Protocol error in {}: expected version {}, received {}",
                    protocol, expected_version, received_version
                )
            }
            SpaceCommError::ResourceExhausted { resource, current_usage, max_usage } => {
                write!(
                    f,
                    "Resource exhausted: {} ({}/{})",
                    resource, current_usage, max_usage
                )
            }
            SpaceCommError::ConfigurationError { parameter, value, reason } => {
                write!(f, "Configuration error: {} = '{}' ({})", parameter, value, reason)
            }
            SpaceCommError::IntegrityError { check_type, expected, calculated } => {
                write!(
                    f,
                    "Integrity check failed ({}): expected {}, got {}",
                    check_type, expected, calculated
                )
            }
        }
    }
}

#[cfg(feature = "std")]
impl StdError for SpaceCommError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        None
    }
}

impl SpaceCommError {
    /// Create a communication timeout error
    pub const fn communication_timeout(timeout_ms: u64, operation: &'static str) -> Self {
        Self::CommunicationTimeout { timeout_ms, operation }
    }

    /// Create an invalid packet error
    pub const fn invalid_packet(reason: &'static str, packet_id: Option<u32>) -> Self {
        Self::InvalidPacket { reason, packet_id }
    }

    /// Create a hardware failure error
    pub const fn hardware_failure(component: &'static str, error_code: u32) -> Self {
        Self::HardwareFailure { component, error_code }
    }

    /// Create a memory error
    pub const fn memory_error(error_type: MemoryErrorType, size: Option<usize>) -> Self {
        Self::MemoryError { error_type, size }
    }

    /// Create a cryptographic error
    pub const fn cryptographic_error(operation: CryptoOperation, details: &'static str) -> Self {
        Self::CryptographicError { operation, details }
    }

    /// Check if error is recoverable
    ///
    /// Some errors indicate conditions that may be temporary and worth retrying,
    /// while others indicate permanent failures.
    pub const fn is_recoverable(&self) -> bool {
        match self {
            SpaceCommError::CommunicationTimeout { .. } => true,
            SpaceCommError::InvalidPacket { .. } => false,
            SpaceCommError::HardwareFailure { .. } => false,
            SpaceCommError::MemoryError { error_type, .. } => {
                matches!(error_type, MemoryErrorType::AllocationFailed)
            }
            SpaceCommError::CryptographicError { .. } => false,
            SpaceCommError::ProtocolError { .. } => false,
            SpaceCommError::ResourceExhausted { .. } => true,
            SpaceCommError::ConfigurationError { .. } => false,
            SpaceCommError::IntegrityError { .. } => false,
        }
    }

    /// Get error severity level (0=info, 1=warning, 2=error, 3=critical)
    pub const fn severity(&self) -> u8 {
        match self {
            SpaceCommError::CommunicationTimeout { .. } => 1,
            SpaceCommError::InvalidPacket { .. } => 2,
            SpaceCommError::HardwareFailure { .. } => 3,
            SpaceCommError::MemoryError { .. } => 3,
            SpaceCommError::CryptographicError { .. } => 3,
            SpaceCommError::ProtocolError { .. } => 2,
            SpaceCommError::ResourceExhausted { .. } => 2,
            SpaceCommError::ConfigurationError { .. } => 2,
            SpaceCommError::IntegrityError { .. } => 3,
        }
    }
}
