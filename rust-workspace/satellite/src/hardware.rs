//! Hardware abstraction layer for satellite systems
//!
//! # Satellite Hardware Abstraction Layer (HAL)
//!
//! Provides comprehensive abstraction for RF transceivers, sensors, and other satellite hardware
//! components. Designed for embedded systems with no-std compatibility and real-time constraints.
//!
//! ## Requirements Fulfilled:
//! - REQ-IF-001: RF transceiver hardware interface and control
//! - REQ-FN-007: Multi-band transceiver support (UHF, S, X, K, Ka bands)
//! - REQ-NF-004: Power management and efficiency control
//! - REQ-SF-002: Emergency hardware shutdown and safe mode operations
//! - REQ-NF-002: Real-time hardware response and timing constraints
//! - REQ-PF-002: Hardware performance monitoring and optimization
//! - REQ-NF-001: System health monitoring and sensor data collection
//!
//! ## Architecture:
//! - Individual transceiver abstractions for each frequency band
//! - Centralized hardware manager for coordination and control
//! - Embassy async integration for non-blocking hardware operations
//! - Temperature, voltage, and current sensor interfaces
//! - Emergency protocols for hardware protection and survival

use embassy_time::{Duration, Timer};
use embedded_hal::digital::v2::{InputPin, OutputPin};
use heapless::Vec;

use space_comms_shared::{Result, SpaceCommError, types::BandType};

/// Transceiver status structure
///
/// Comprehensive status information for RF transceivers including operational
/// parameters, health metrics, and environmental conditions.
///
/// Requirements Fulfilled:
/// - REQ-NF-001: Hardware status monitoring and reporting
/// - REQ-PF-002: Performance metrics collection
/// - REQ-SF-002: Safety parameter monitoring
#[derive(Debug, Clone)]
pub struct TransceiverStatus {
    /// Is transceiver powered on and operational
    /// REQ-SF-002: Power state monitoring for emergency protocols
    pub is_powered: bool,

    /// Current operating frequency in Hz
    /// REQ-FN-007: Frequency management for multi-band operations
    pub frequency: u64,

    /// Transmit power level (0-100%)
    /// REQ-NF-004: Power level control for efficiency optimization
    pub tx_power: u8,

    /// Signal strength in dBm (received signal strength indicator)
    /// REQ-PF-002: Link quality monitoring and optimization
    pub signal_strength: i16,

    /// Operating temperature in Celsius
    /// REQ-SF-002: Thermal monitoring for hardware protection
    pub temperature: i16,

    /// Is transceiver locked to carrier frequency
    /// REQ-IF-001: Phase-locked loop status for reliable communication
    pub is_locked: bool,
}

/// UHF Band Transceiver (400-450 MHz)
///
/// Provides reliable, low-power communication capability primarily used for
/// emergency communications and command uplink due to excellent range and
/// atmospheric penetration characteristics.
///
/// Requirements Fulfilled:
/// - REQ-FN-007: UHF band communication support
/// - REQ-SF-002: Emergency communication reliability
/// - REQ-IF-001: Hardware transceiver interface
pub struct UhfTransceiver {
    /// Current transceiver operational status
    status: TransceiverStatus,

    /// Hardware enable flag for power management
    /// REQ-NF-004: Power control and conservation
    enabled: bool,
    // Note: Real hardware pins would be defined here for actual implementation
    // using embedded-hal traits for GPIO, SPI, I2C interfaces
}

impl UhfTransceiver {
    /// Create new UHF transceiver instance
    ///
    /// Initializes UHF transceiver with space-qualified default settings
    /// optimized for reliability and emergency communications.
    ///
    /// Requirements Fulfilled:
    /// - REQ-FN-007: UHF band initialization with 435 MHz frequency
    /// - REQ-SF-002: Conservative settings for emergency reliability
    /// - REQ-NF-004: 50% power level for efficiency balance
    ///
    /// Returns:
    /// UhfTransceiver instance ready for emergency communications
    pub fn new() -> Self {
        Self {
            status: TransceiverStatus {
                is_powered: true,
                frequency: 435_000_000,   // REQ-FN-007: 435 MHz amateur satellite band
                tx_power: 50,             // REQ-NF-004: Conservative power for reliability
                signal_strength: -80,     // Typical space link budget
                temperature: 25,          // Room temperature startup
                is_locked: true,          // REQ-IF-001: PLL locked status
            },
            enabled: true,                // REQ-SF-002: Ready for emergency use
        }
    }

    /// Transmit data on UHF band
    ///
    /// Performs UHF transmission with appropriate timing based on 9.6 kbps
    /// data rate, suitable for emergency commands and basic telemetry.
    ///
    /// Parameters:
    /// - data: Byte array to transmit
    ///
    /// Requirements Fulfilled:
    /// - REQ-PF-002: 9.6 kbps data rate for emergency communications
    /// - REQ-SF-002: Hardware readiness validation before transmission
    /// - REQ-NF-002: Non-blocking async transmission timing
    ///
    /// Returns:
    /// Result<()> indicating transmission success or hardware failure
    pub async fn transmit(&mut self, data: &[u8]) -> Result<()> {
        // Hardware readiness check (REQ-SF-002)
        if !self.status.is_powered || !self.enabled {
            return Err(SpaceCommError::hardware_failure("UHF transceiver not ready", 1));
        }

        // Calculate transmission time based on UHF data rate (REQ-PF-002)
        // 9.6 kbps = 9600 bits/second
        let transmission_time = (data.len() * 8 * 1000) / 9600; // milliseconds
        Timer::after(Duration::from_millis(transmission_time as u64)).await;

        Ok(())
    }

    /// Receive data from UHF band
    ///
    /// Attempts to receive data from UHF band with hardware validation
    /// and appropriate timing for 9.6 kbps operations.
    ///
    /// Requirements Fulfilled:
    /// - REQ-SF-002: Hardware readiness validation for reliable reception
    /// - REQ-PF-002: UHF reception timing and buffer management
    /// - REQ-NF-002: Non-blocking async reception
    ///
    /// Returns:
    /// Result<Vec<u8, 512>> received data or hardware error
    pub async fn receive(&mut self) -> Result<Vec<u8, 512>> {
        // Hardware readiness check (REQ-SF-002)
        if !self.status.is_powered || !self.enabled {
            return Err(SpaceCommError::hardware_failure("UHF transceiver not ready", 1));
        }

        // Simulate checking for received data (REQ-NF-002)
        Timer::after(Duration::from_millis(10)).await;

        // For simulation, return empty data (real implementation would read hardware buffer)
        Ok(Vec::new())
    }

    /// Get current transceiver status
    ///
    /// Provides read-only access to transceiver operational status
    /// for monitoring and diagnostic purposes.
    ///
    /// Requirements Fulfilled:
    /// - REQ-NF-001: Hardware status monitoring access
    ///
    /// Returns:
    /// Reference to current TransceiverStatus
    pub fn get_status(&self) -> &TransceiverStatus {
        &self.status
    }
}

/// S-Band Transceiver (2.0-2.3 GHz)
///
/// Provides balanced performance between reliability and data rate, commonly used
/// for command uplink and telemetry downlink in space missions. Offers good
/// atmospheric penetration with moderate data rates.
///
/// Requirements Fulfilled:
/// - REQ-FN-007: S-Band communication support (2.2 GHz operational frequency)
/// - REQ-PF-002: 2 Mbps data rate for command and telemetry operations
/// - REQ-IF-001: S-Band hardware transceiver interface
pub struct SBandTransceiver {
    /// Current transceiver operational status
    status: TransceiverStatus,

    /// Hardware enable flag for power management
    /// REQ-NF-004: Power control and conservation
    enabled: bool,
}

impl SBandTransceiver {
    /// Create new S-Band transceiver instance
    ///
    /// Initializes S-Band transceiver with optimal settings for standard
    /// space operations balancing reliability and performance.
    ///
    /// Requirements Fulfilled:
    /// - REQ-FN-007: S-Band initialization with 2.2 GHz frequency
    /// - REQ-PF-002: 2 Mbps data rate configuration
    /// - REQ-NF-004: 75% power level for performance balance
    ///
    /// Returns:
    /// SBandTransceiver instance ready for command/telemetry operations
    pub fn new() -> Self {
        Self {
            status: TransceiverStatus {
                is_powered: true,
                frequency: 2_200_000_000, // REQ-FN-007: 2.2 GHz S-Band frequency
                tx_power: 75,             // REQ-NF-004: Higher power for performance
                signal_strength: -75,     // Good space link performance
                temperature: 30,          // Slightly higher due to power level
                is_locked: true,          // REQ-IF-001: PLL locked for operations
            },
            enabled: true,                // Ready for standard operations
        }
    }

    /// Transmit data on S-Band
    ///
    /// Performs S-Band transmission with timing based on 2 Mbps data rate,
    /// suitable for command uplink and standard telemetry operations.
    ///
    /// Parameters:
    /// - data: Byte array to transmit
    ///
    /// Requirements Fulfilled:
    /// - REQ-PF-002: 2 Mbps data rate for command/telemetry
    /// - REQ-SF-002: Hardware validation before transmission
    /// - REQ-NF-002: Async transmission with proper timing
    ///
    /// Returns:
    /// Result<()> indicating transmission success or failure
    pub async fn transmit(&mut self, data: &[u8]) -> Result<()> {
        // Hardware readiness validation (REQ-SF-002)
        if !self.status.is_powered || !self.enabled {
            return Err(SpaceCommError::hardware_failure("S-Band transceiver not ready", 2));
        }

        // Calculate transmission time for 2 Mbps (REQ-PF-002)
        let transmission_time = (data.len() * 8) / 2_000; // microseconds to milliseconds
        Timer::after(Duration::from_millis(transmission_time.max(1) as u64)).await;

        Ok(())
    }

    /// Receive data from S-Band
    ///
    /// Attempts to receive data from S-Band with appropriate timing
    /// and buffer management for 2 Mbps operations.
    ///
    /// Requirements Fulfilled:
    /// - REQ-SF-002: Hardware readiness validation
    /// - REQ-PF-002: S-Band reception performance
    /// - REQ-NF-002: Non-blocking async reception
    ///
    /// Returns:
    /// Result<Vec<u8, 2048>> received data or hardware error
    pub async fn receive(&mut self) -> Result<Vec<u8, 2048>> {
        if !self.status.is_powered || !self.enabled {
            return Err(SpaceCommError::hardware_failure("S-Band transceiver not ready", 2));
        }

        // Faster reception check for higher data rate
        Timer::after(Duration::from_millis(5)).await;
        Ok(Vec::new())
    }

    /// Get current transceiver status
    ///
    /// Requirements Fulfilled:
    /// - REQ-NF-001: Hardware status monitoring access
    pub fn get_status(&self) -> &TransceiverStatus {
        &self.status
    }
}

/// X-Band Transceiver (8.0-12.0 GHz)
///
/// High-performance transceiver for science data downlink and high-rate telemetry.
/// Provides excellent data rates with good reliability for deep space missions.
///
/// Requirements Fulfilled:
/// - REQ-FN-007: X-Band communication support (8.4 GHz frequency)
/// - REQ-PF-002: 100 Mbps data rate for science data transmission
/// - REQ-IF-001: X-Band hardware interface for high-rate operations
pub struct XBandTransceiver {
    status: TransceiverStatus,
    enabled: bool,
}

impl XBandTransceiver {
    /// Create new X-Band transceiver instance
    /// REQ-FN-007: X-Band initialization with 8.4 GHz frequency
    /// REQ-PF-002: 100 Mbps data rate for science operations
    pub fn new() -> Self {
        Self {
            status: TransceiverStatus {
                is_powered: true,
                frequency: 8_400_000_000, // REQ-FN-007: 8.4 GHz X-Band
                tx_power: 80,             // REQ-NF-004: High power for range
                signal_strength: -70,     // Good performance characteristics
                temperature: 35,          // Higher due to power requirements
                is_locked: true,          // REQ-IF-001: PLL locked status
            },
            enabled: true,
        }
    }

    /// Transmit data on X-Band with 100 Mbps performance
    /// REQ-PF-002: High-speed transmission for science data
    /// REQ-SF-002: Hardware validation for reliable operation
    pub async fn transmit(&mut self, data: &[u8]) -> Result<()> {
        if !self.status.is_powered || !self.enabled {
            return Err(SpaceCommError::hardware_failure("X-Band transceiver not ready", 3));
        }

        // 100 Mbps transmission timing (REQ-PF-002)
        let transmission_time = (data.len() * 8) / 100_000; // microseconds to milliseconds
        Timer::after(Duration::from_millis(transmission_time.max(1) as u64)).await;
        Ok(())
    }

    /// Receive data from X-Band
    /// REQ-PF-002: High-rate reception capability
    pub async fn receive(&mut self) -> Result<Vec<u8, 4096>> {
        if !self.status.is_powered || !self.enabled {
            return Err(SpaceCommError::hardware_failure("X-Band transceiver not ready", 3));
        }
        Timer::after(Duration::from_millis(2)).await;
        Ok(Vec::new())
    }

    pub fn get_status(&self) -> &TransceiverStatus { &self.status }
}

/// K-Band Transceiver (18-27 GHz)
///
/// Very high-speed transceiver for bulk data transfer and high-resolution imagery.
/// Requires precise pointing and atmospheric compensation.
///
/// Requirements Fulfilled:
/// - REQ-FN-007: K-Band communication support (22 GHz frequency)
/// - REQ-PF-002: 1 Gbps data rate for bulk data operations
/// - REQ-NF-004: High power requirements for performance
pub struct KBandTransceiver {
    status: TransceiverStatus,
    enabled: bool,
}

impl KBandTransceiver {
    /// Create new K-Band transceiver instance
    /// REQ-FN-007: K-Band initialization with 22 GHz frequency
    /// REQ-PF-002: 1 Gbps data rate configuration
    pub fn new() -> Self {
        Self {
            status: TransceiverStatus {
                is_powered: true,
                frequency: 22_000_000_000, // REQ-FN-007: 22 GHz K-Band
                tx_power: 90,              // REQ-NF-004: Very high power for performance
                signal_strength: -65,      // Excellent performance when conditions allow
                temperature: 40,           // High due to power amplifier heat
                is_locked: true,           // REQ-IF-001: Critical for high-rate ops
            },
            enabled: true,
        }
    }

    /// Transmit data on K-Band with 1 Gbps performance
    /// REQ-PF-002: Ultra-high-speed transmission capability
    pub async fn transmit(&mut self, data: &[u8]) -> Result<()> {
        if !self.status.is_powered || !self.enabled {
            return Err(SpaceCommError::hardware_failure("K-Band transceiver not ready", 4));
        }

        // 1 Gbps transmission timing (REQ-PF-002)
        let transmission_time = (data.len() * 8) / 1_000_000; // nanoseconds to milliseconds
        Timer::after(Duration::from_millis(transmission_time.max(1) as u64)).await;
        Ok(())
    }

    /// Receive data from K-Band
    pub async fn receive(&mut self) -> Result<Vec<u8, 8192>> {
        if !self.status.is_powered || !self.enabled {
            return Err(SpaceCommError::hardware_failure("K-Band transceiver not ready", 4));
        }
        Timer::after(Duration::from_millis(1)).await;
        Ok(Vec::new())
    }

    pub fn get_status(&self) -> &TransceiverStatus { &self.status }
}

/// Ka-Band Transceiver (27-40 GHz)
///
/// Maximum performance transceiver for the highest data rate operations.
/// Highly sensitive to atmospheric conditions but provides unmatched throughput.
///
/// Requirements Fulfilled:
/// - REQ-FN-007: Ka-Band communication support (32 GHz frequency)
/// - REQ-PF-002: 10 Gbps maximum data rate capability
/// - REQ-NF-004: Maximum power for ultimate performance
pub struct KaBandTransceiver {
    status: TransceiverStatus,
    enabled: bool,
}

impl KaBandTransceiver {
    /// Create new Ka-Band transceiver instance
    /// REQ-FN-007: Ka-Band initialization with 32 GHz frequency
    /// REQ-PF-002: 10 Gbps maximum data rate configuration
    pub fn new() -> Self {
        Self {
            status: TransceiverStatus {
                is_powered: true,
                frequency: 32_000_000_000, // REQ-FN-007: 32 GHz Ka-Band
                tx_power: 95,              // REQ-NF-004: Maximum power for best performance
                signal_strength: -60,      // Best possible performance characteristics
                temperature: 45,           // Highest due to maximum power requirements
                is_locked: true,           // REQ-IF-001: Essential for 10 Gbps operations
            },
            enabled: true,
        }
    }

    /// Transmit data on Ka-Band with 10 Gbps performance
    /// REQ-PF-002: Maximum data rate transmission capability
    pub async fn transmit(&mut self, data: &[u8]) -> Result<()> {
        if !self.status.is_powered || !self.enabled {
            return Err(SpaceCommError::hardware_failure("Ka-Band transceiver not ready", 5));
        }

        // 10 Gbps transmission timing (REQ-PF-002)
        let transmission_time = (data.len() * 8) / 10_000_000; // nanoseconds to milliseconds
        Timer::after(Duration::from_millis(transmission_time.max(1) as u64)).await;
        Ok(())
    }

    /// Receive data from Ka-Band
    pub async fn receive(&mut self) -> Result<Vec<u8, 16384>> {
        if !self.status.is_powered || !self.enabled {
            return Err(SpaceCommError::hardware_failure("Ka-Band transceiver not ready", 5));
        }
        Timer::after(Duration::from_millis(1)).await;
        Ok(Vec::new())
    }

    pub fn get_status(&self) -> &TransceiverStatus { &self.status }
}
        let transmission_time = (data.len() * 8) / 1_000_000; // ns to ms
        Timer::after(Duration::from_millis(transmission_time.max(1) as u64)).await;

        Ok(())
    }

    pub async fn receive(&mut self) -> Result<Vec<u8, 8192>> {
        if !self.status.is_powered || !self.enabled {
            return Err(SpaceCommError::hardware_failure("K-Band transceiver not ready", 4));
        }

        Timer::after(Duration::from_millis(1)).await;
        Ok(Vec::new())
    }

    pub fn get_status(&self) -> &TransceiverStatus {
        &self.status
    }
}

/// Ka-Band Transceiver (27-40 GHz)
pub struct KaBandTransceiver {
    status: TransceiverStatus,
    enabled: bool,
}

impl KaBandTransceiver {
    pub fn new() -> Self {
        Self {
            status: TransceiverStatus {
                is_powered: true,
                frequency: 32_000_000_000, // 32 GHz
                tx_power: 95,
                signal_strength: -60,
                temperature: 45,
                is_locked: true,
            },
            enabled: true,
        }
    }

    pub async fn transmit(&mut self, data: &[u8]) -> Result<()> {
        if !self.status.is_powered || !self.enabled {
            return Err(SpaceCommError::hardware_failure("Ka-Band transceiver not ready", 5));
        }

        // Simulate transmission time based on data rate (10 Gbps)
        let transmission_time = (data.len() * 8) / 10_000_000; // ns to ms
        Timer::after(Duration::from_millis(transmission_time.max(1) as u64)).await;

        Ok(())
    }

    pub async fn receive(&mut self) -> Result<Vec<u8, 16384>> {
        if !self.status.is_powered || !self.enabled {
            return Err(SpaceCommError::hardware_failure("Ka-Band transceiver not ready", 5));
        }

        Timer::after(Duration::from_millis(1)).await;
        Ok(Vec::new())
    }

    pub fn get_status(&self) -> &TransceiverStatus {
        &self.status
    }
}

/// Hardware manager for all transceivers
///
/// Centralized management system for all RF transceivers, providing coordination,
/// monitoring, and control capabilities for the complete communication subsystem.
///
/// Requirements Fulfilled:
/// - REQ-FN-007: Multi-band transceiver coordination and management
/// - REQ-NF-001: System-wide hardware monitoring and status reporting
/// - REQ-SF-002: Emergency protocols and hardware protection
/// - REQ-NF-004: Power management across all communication bands
pub struct HardwareManager {
    /// UHF transceiver for emergency communications
    uhf: UhfTransceiver,

    /// S-Band transceiver for command and telemetry
    s_band: SBandTransceiver,

    /// X-Band transceiver for science data
    x_band: XBandTransceiver,

    /// K-Band transceiver for high-rate operations
    k_band: KBandTransceiver,

    /// Ka-Band transceiver for maximum throughput
    ka_band: KaBandTransceiver,
}

impl HardwareManager {
    /// Create new hardware manager instance
    ///
    /// Initializes all transceiver instances with their respective configurations
    /// for comprehensive multi-band communication capability.
    ///
    /// Requirements Fulfilled:
    /// - REQ-FN-007: Initialize all 5 communication bands
    /// - REQ-NF-001: Hardware management system initialization
    ///
    /// Returns:
    /// HardwareManager instance with all transceivers ready
    pub fn new() -> Self {
        Self {
            uhf: UhfTransceiver::new(),       // REQ-SF-002: Emergency communications
            s_band: SBandTransceiver::new(),  // Standard operations
            x_band: XBandTransceiver::new(),  // Science data
            k_band: KBandTransceiver::new(),  // High-rate operations
            ka_band: KaBandTransceiver::new(), // Maximum throughput
        }
    }

    /// Get all transceiver statuses
    ///
    /// Provides comprehensive status information for all transceivers
    /// for system monitoring and health assessment.
    ///
    /// Requirements Fulfilled:
    /// - REQ-NF-001: Complete system status monitoring
    /// - REQ-SF-002: Hardware health assessment for safety
    ///
    /// Returns:
    /// Array of (name, status) tuples for all transceivers
    pub fn get_all_statuses(&self) -> [(&'static str, &TransceiverStatus); 5] {
        [
            ("UHF", self.uhf.get_status()),      // Emergency band status
            ("S-Band", self.s_band.get_status()), // Standard ops status
            ("X-Band", self.x_band.get_status()), // Science data status
            ("K-Band", self.k_band.get_status()), // High-rate status
            ("Ka-Band", self.ka_band.get_status()), // Max throughput status
        ]
    }

    /// Power cycle a transceiver
    ///
    /// Performs controlled power cycle of specified transceiver for recovery
    /// from lock-up conditions or thermal reset requirements.
    ///
    /// Parameters:
    /// - band: Frequency band to power cycle
    ///
    /// Requirements Fulfilled:
    /// - REQ-SF-002: Hardware recovery and reset capability
    /// - REQ-NF-004: Power management and control
    /// - REQ-IF-001: Hardware control interface
    ///
    /// Returns:
    /// Result<()> indicating power cycle success or failure
    pub async fn power_cycle_transceiver(&mut self, band: BandType) -> Result<()> {
        match band {
            BandType::UhfBand => {
                self.uhf.status.is_powered = false;      // Power down
                Timer::after(Duration::from_millis(1000)).await; // Wait for discharge
                self.uhf.status.is_powered = true;       // Power up
            }
            BandType::SBand => {
                self.s_band.status.is_powered = false;
                Timer::after(Duration::from_millis(1000)).await;
                self.s_band.status.is_powered = true;
            }
            BandType::XBand => {
                self.x_band.status.is_powered = false;
                Timer::after(Duration::from_millis(1000)).await;
                self.x_band.status.is_powered = true;
            }
            BandType::KBand => {
                self.k_band.status.is_powered = false;
                Timer::after(Duration::from_millis(1000)).await;
                self.k_band.status.is_powered = true;
            }
            BandType::KaBand => {
                self.ka_band.status.is_powered = false;
                Timer::after(Duration::from_millis(1000)).await;
                self.ka_band.status.is_powered = true;
            }
        }

        // Wait for hardware startup and PLL lock (REQ-IF-001)
        Timer::after(Duration::from_millis(500)).await;
        Ok(())
    }
}

/// Global hardware manager instance
static mut HARDWARE_MANAGER: Option<HardwareManager> = None;

/// Initialize all transceivers
pub async fn initialize_transceivers() -> Result<()> {
    unsafe {
        HARDWARE_MANAGER = Some(HardwareManager::new());
    }

    // Startup delay for all transceivers
    Timer::after(Duration::from_millis(2000)).await;

    Ok(())
}

/// Transmit on UHF band
pub async fn transmit_uhf(data: &[u8]) -> Result<()> {
    let manager = unsafe { HARDWARE_MANAGER.as_mut().unwrap() };
    manager.uhf.transmit(data).await
}

/// Transmit on S-Band
pub async fn transmit_s_band(data: &[u8]) -> Result<()> {
    let manager = unsafe { HARDWARE_MANAGER.as_mut().unwrap() };
    manager.s_band.transmit(data).await
}

/// Transmit on X-Band
pub async fn transmit_x_band(data: &[u8]) -> Result<()> {
    let manager = unsafe { HARDWARE_MANAGER.as_mut().unwrap() };
    manager.x_band.transmit(data).await
}

/// Transmit on K-Band
pub async fn transmit_k_band(data: &[u8]) -> Result<()> {
    let manager = unsafe { HARDWARE_MANAGER.as_mut().unwrap() };
    manager.k_band.transmit(data).await
}

/// Transmit on Ka-Band
pub async fn transmit_ka_band(data: &[u8]) -> Result<()> {
    let manager = unsafe { HARDWARE_MANAGER.as_mut().unwrap() };
    manager.ka_band.transmit(data).await
}

/// Receive from UHF band
pub async fn receive_uhf() -> Result<Vec<u8, 512>> {
    let manager = unsafe { HARDWARE_MANAGER.as_mut().unwrap() };
    manager.uhf.receive().await
}

/// Receive from S-Band
pub async fn receive_s_band() -> Result<Vec<u8, 2048>> {
    let manager = unsafe { HARDWARE_MANAGER.as_mut().unwrap() };
    manager.s_band.receive().await
}

/// Receive from X-Band
pub async fn receive_x_band() -> Result<Vec<u8, 4096>> {
    let manager = unsafe { HARDWARE_MANAGER.as_mut().unwrap() };
    manager.x_band.receive().await
}

/// Get hardware health status
pub fn get_hardware_health() -> [(&'static str, bool); 5] {
    let manager = unsafe { HARDWARE_MANAGER.as_ref().unwrap() };
    let statuses = manager.get_all_statuses();

    [
        (statuses[0].0, statuses[0].1.is_powered && statuses[0].1.is_locked),
        (statuses[1].0, statuses[1].1.is_powered && statuses[1].1.is_locked),
        (statuses[2].0, statuses[2].1.is_powered && statuses[2].1.is_locked),
        (statuses[3].0, statuses[3].1.is_powered && statuses[3].1.is_locked),
        (statuses[4].0, statuses[4].1.is_powered && statuses[4].1.is_locked),
    ]
}

/// Emergency hardware shutdown
pub async fn emergency_shutdown() -> Result<()> {
    let manager = unsafe { HARDWARE_MANAGER.as_mut().unwrap() };

    // Power down all transceivers except UHF (emergency communications)
    manager.s_band.status.is_powered = false;
    manager.x_band.status.is_powered = false;
    manager.k_band.status.is_powered = false;
    manager.ka_band.status.is_powered = false;

    // Reduce UHF power to minimum
    manager.uhf.status.tx_power = 10;

    Ok(())
}

/// Sensor reading structure
#[derive(Debug, Clone)]
pub struct SensorReading {
    /// Sensor ID
    pub sensor_id: u16,
    /// Reading value
    pub value: f32,
    /// Units (e.g., "C", "V", "A", "rpm")
    pub units: &'static str,
    /// Timestamp (milliseconds since boot)
    pub timestamp: u64,
}

/// Read temperature sensor
pub async fn read_temperature_sensor(sensor_id: u16) -> Result<SensorReading> {
    // Simulate sensor reading delay
    Timer::after(Duration::from_millis(50)).await;

    // Simulate temperature reading
    let temp = match sensor_id {
        0 => 25.5,  // Main board
        1 => 30.2,  // RF section
        2 => 22.8,  // Battery
        3 => 45.1,  // Power amplifier
        _ => 25.0,  // Default
    };

    Ok(SensorReading {
        sensor_id,
        value: temp,
        units: "C",
        timestamp: embassy_time::Instant::now().as_millis(),
    })
}

/// Read voltage sensor
pub async fn read_voltage_sensor(sensor_id: u16) -> Result<SensorReading> {
    Timer::after(Duration::from_millis(30)).await;

    let voltage = match sensor_id {
        0 => 12.1,  // Main bus
        1 => 5.05,  // Digital supply
        2 => 3.32,  // Analog supply
        3 => 28.5,  // Battery
        _ => 12.0,  // Default
    };

    Ok(SensorReading {
        sensor_id,
        value: voltage,
        units: "V",
        timestamp: embassy_time::Instant::now().as_millis(),
    })
}

/// Read current sensor
pub async fn read_current_sensor(sensor_id: u16) -> Result<SensorReading> {
    Timer::after(Duration::from_millis(40)).await;

    let current = match sensor_id {
        0 => 2.15,  // Total system
        1 => 0.85,  // Digital section
        2 => 0.45,  // RF section
        3 => 0.95,  // Transmitter
        _ => 1.0,   // Default
    };

    Ok(SensorReading {
        sensor_id,
        value: current,
        units: "A",
        timestamp: embassy_time::Instant::now().as_millis(),
    })
}

/// Check if hardware is in safe state
pub fn is_hardware_safe() -> bool {
    let manager = unsafe { HARDWARE_MANAGER.as_ref().unwrap() };
    let statuses = manager.get_all_statuses();

    // Check temperatures are within safe range
    for (_, status) in &statuses {
        if status.temperature > 70 || status.temperature < -40 {
            return false;
        }
    }

    // Check at least one transceiver is operational
    statuses.iter().any(|(_, status)| status.is_powered && status.is_locked)
}
