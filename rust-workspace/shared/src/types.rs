//! Common data types for space communication systems
//!
//! This module defines fundamental types used throughout the space communication
//! system, ensuring type safety and clear interfaces.

use serde::{Deserialize, Serialize};

/// Unique identifier for communication packets
///
/// Provides a type-safe wrapper around packet identifiers to prevent
/// confusion with other numeric IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PacketId(pub u32);

impl PacketId {
    /// Create a new packet ID
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Get the raw ID value
    pub const fn value(&self) -> u32 {
        self.0
    }
}

/// Unique identifier for messages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageId(pub u64);

impl MessageId {
    /// Create a new message ID
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Get the raw ID value
    pub const fn value(&self) -> u64 {
        self.0
    }
}

/// Unique identifier for system components
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ComponentId(pub u16);

impl ComponentId {
    /// Create a new component ID
    pub const fn new(id: u16) -> Self {
        Self(id)
    }

    /// Get the raw ID value
    pub const fn value(&self) -> u16 {
        self.0
    }
}

/// Communication frequency bands
///
/// Represents the different frequency bands used in satellite communication.
/// Each band has specific characteristics and use cases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BandType {
    /// UHF Band (300 MHz - 3 GHz)
    /// Primary use: Emergency communication and backup systems
    UhfBand,

    /// S-Band (2 GHz - 4 GHz)
    /// Primary use: Telemetry, tracking, and command (TT&C)
    SBand,

    /// X-Band (8 GHz - 12 GHz)
    /// Primary use: Medium-speed data transmission
    XBand,

    /// K-Band (20 GHz - 30 GHz)
    /// Primary use: High-speed data transmission
    KBand,

    /// Ka-Band (26.5 GHz - 40 GHz)
    /// Primary use: Ultra-high bandwidth Earth and relay links
    KaBand,
}

impl BandType {
    /// Get the frequency range for this band in Hz
    pub const fn frequency_range(&self) -> (u64, u64) {
        match self {
            BandType::UhfBand => (300_000_000, 3_000_000_000),
            BandType::SBand => (2_000_000_000, 4_000_000_000),
            BandType::XBand => (8_000_000_000, 12_000_000_000),
            BandType::KBand => (20_000_000_000, 30_000_000_000),
            BandType::KaBand => (26_500_000_000, 40_000_000_000),
        }
    }

    /// Get the typical data rate range for this band in bits per second
    pub const fn typical_data_rate_range(&self) -> (u64, u64) {
        match self {
            BandType::UhfBand => (1_000, 10_000_000),           // 1 Kbps - 10 Mbps
            BandType::SBand => (1_000_000, 100_000_000),        // 1 Mbps - 100 Mbps
            BandType::XBand => (100_000_000, 1_000_000_000),    // 100 Mbps - 1 Gbps
            BandType::KBand => (1_000_000_000, 10_000_000_000), // 1 Gbps - 10 Gbps
            BandType::KaBand => (10_000_000_000, 100_000_000_000), // 10 Gbps - 100 Gbps
        }
    }

    /// Get the primary use case for this band
    pub const fn primary_use_case(&self) -> &'static str {
        match self {
            BandType::UhfBand => "Emergency communication and backup",
            BandType::SBand => "Telemetry, tracking, and command (TT&C)",
            BandType::XBand => "Medium-speed data transmission",
            BandType::KBand => "High-speed data transmission",
            BandType::KaBand => "Ultra-high bandwidth Earth and relay links",
        }
    }

    /// Get the weather sensitivity (0.0 = not sensitive, 1.0 = very sensitive)
    pub const fn weather_sensitivity(&self) -> f32 {
        match self {
            BandType::UhfBand => 0.1,
            BandType::SBand => 0.2,
            BandType::XBand => 0.4,
            BandType::KBand => 0.8,
            BandType::KaBand => 0.9,
        }
    }
}

/// System operational modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationalMode {
    /// Normal operational mode
    Normal,
    /// Reduced functionality mode (degraded performance)
    Degraded,
    /// Emergency mode (minimal functionality)
    Emergency,
    /// Maintenance mode (limited operations)
    Maintenance,
    /// Safe mode (protective state)
    Safe,
}

/// Power consumption levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PowerLevel {
    /// Very low power consumption
    VeryLow,
    /// Low power consumption
    Low,
    /// Medium power consumption
    Medium,
    /// High power consumption
    High,
    /// Very high power consumption
    VeryHigh,
}

impl PowerLevel {
    /// Get typical power consumption in watts
    pub const fn typical_watts(&self) -> f32 {
        match self {
            PowerLevel::VeryLow => 0.1,
            PowerLevel::Low => 1.0,
            PowerLevel::Medium => 10.0,
            PowerLevel::High => 100.0,
            PowerLevel::VeryHigh => 1000.0,
        }
    }
}

/// System health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum HealthStatus {
    /// System is operating optimally
    Excellent,
    /// System is operating well
    Good,
    /// System is operating with minor issues
    Fair,
    /// System is operating with significant issues
    Poor,
    /// System is critically compromised
    Critical,
    /// System status is unknown
    Unknown,
}

impl HealthStatus {
    /// Convert to numeric score (0-100)
    pub const fn score(&self) -> u8 {
        match self {
            HealthStatus::Excellent => 100,
            HealthStatus::Good => 80,
            HealthStatus::Fair => 60,
            HealthStatus::Poor => 40,
            HealthStatus::Critical => 20,
            HealthStatus::Unknown => 0,
        }
    }

    /// Check if status requires immediate attention
    pub const fn requires_attention(&self) -> bool {
        matches!(self, HealthStatus::Poor | HealthStatus::Critical)
    }
}

/// Geographic coordinates for ground stations and satellite positions
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Coordinates {
    /// Latitude in degrees (-90.0 to 90.0)
    pub latitude: f64,
    /// Longitude in degrees (-180.0 to 180.0)
    pub longitude: f64,
    /// Altitude in meters above sea level
    pub altitude: f64,
}

impl Coordinates {
    /// Create new coordinates
    pub const fn new(latitude: f64, longitude: f64, altitude: f64) -> Self {
        Self { latitude, longitude, altitude }
    }

    /// Calculate distance to another coordinate (simplified calculation)
    /// Returns distance in kilometers
    pub fn distance_to(&self, other: &Coordinates) -> f64 {
        // Simplified Haversine formula
        const EARTH_RADIUS_KM: f64 = 6371.0;

        let lat1_rad = self.latitude.to_radians();
        let lat2_rad = other.latitude.to_radians();
        let delta_lat = (other.latitude - self.latitude).to_radians();
        let delta_lon = (other.longitude - self.longitude).to_radians();

        let a = (delta_lat / 2.0).sin().powi(2) +
                lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        EARTH_RADIUS_KM * c
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_id() {
        let id = PacketId::new(12345);
        assert_eq!(id.value(), 12345);
    }

    #[test]
    fn test_band_type_properties() {
        let s_band = BandType::SBand;
        assert_eq!(s_band.frequency_range(), (2_000_000_000, 4_000_000_000));
        assert_eq!(s_band.primary_use_case(), "Telemetry, tracking, and command (TT&C)");
        assert!(s_band.weather_sensitivity() < 0.5);
    }

    #[test]
    fn test_health_status_ordering() {
        assert!(HealthStatus::Excellent > HealthStatus::Good);
        assert!(HealthStatus::Critical < HealthStatus::Poor);
        assert!(HealthStatus::Critical.requires_attention());
        assert!(!HealthStatus::Good.requires_attention());
    }

    #[test]
    fn test_coordinates_distance() {
        let coord1 = Coordinates::new(0.0, 0.0, 0.0);
        let coord2 = Coordinates::new(1.0, 1.0, 0.0);
        let distance = coord1.distance_to(&coord2);
        assert!(distance > 0.0);
        assert!(distance < 200.0); // Should be less than 200km
    }
}
