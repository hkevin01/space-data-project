//! # Space Communications Shared Library
//!
//! This library provides shared types, protocols, and utilities for space communication
//! systems, designed to meet NASA and DoD coding standards.
//!
//! ## Features
//! - CCSDS-compliant packet structures
//! - Priority-based messaging protocols
//! - Error correction and fault tolerance types
//! - Security and cryptographic primitives
//! - NASA/DoD standard data types

#![cfg_attr(feature = "no-std", no_std)]
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![warn(trivial_casts, trivial_numeric_casts)]
#![warn(unused_qualifications)]

#[cfg(feature = "std")]
extern crate std;

#[cfg(not(feature = "std"))]
extern crate core as std;

pub mod ccsds;
pub mod error;
pub mod messaging;
pub mod security;
pub mod telemetry;
pub mod time;
pub mod types;

// Re-export commonly used types
pub use error::{Result, SpaceCommError};
pub use messaging::{Message, MessagePriority, PriorityQueue};
pub use telemetry::{TelemetryData, TelemetryPacket};
pub use types::{BandType, ComponentId, MessageId, PacketId};
