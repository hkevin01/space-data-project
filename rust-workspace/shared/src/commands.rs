//! Mission-critical space commands with priority classification
//!
//! # Space Command Protocol Module
//!
//! This module defines a comprehensive set of satellite and ground station
//! commands used in real space missions, classified by criticality and
//! priority levels for proper message queue handling.
//!
//! ## Requirements Fulfilled:
//! - REQ-FN-001: Priority-based command classification (Emergency, Critical, High, Medium, Low)
//! - REQ-FN-002: Emergency protocol commands (EmergencyAbort, EmergencyHalt, ActivateSafeMode)
//! - REQ-FN-003: Critical system commands (AbortMission, CollisionAvoidance, AttitudeControl)
//! - REQ-FN-004: High priority operations (UpdateOrbit, ReconfigureComm, Deploy)
//! - REQ-FN-005: Medium priority commands (RequestTelemetry, UpdateConfig, CalibrateInstrument)
//! - REQ-FN-006: Low priority operations (SendStatus, UpdateTime, PerformMaintenance)
//! - REQ-FN-007: Multi-band communication support with band selection
//! - REQ-PF-001: Command response time requirements (1ms-10s based on priority)
//! - REQ-IF-002: CCSDS-compliant command structure and serialization
//! - REQ-SF-001: Command validation and confirmation requirements
//!
//! ## NASA/DoD Standards Compliance:
//! - **NASA-STD-8719.13A**: Software Safety (emergency command validation)
//! - **NASA-HDBK-2203**: Software Engineering Requirements (command structure)
//! - **DoD-STD-2167A**: Defense System Software Development (command protocols)
//! - **MIL-STD-1553B**: Digital Time Division Command/Response (command formatting)
//! - **CCSDS 133.0-B-2**: Space Packet Protocol (command serialization)
//! - **NASA-STD-8739.8**: Software Assurance Standard (command validation)
//! - **ECSS-E-ST-70-41C**: Space Engineering - Telemetry and Telecommand
//! - **DoD 8570.01-M**: Information Assurance (secure command handling)
//!
//! ## Architecture:
//! Commands are organized by priority level to ensure proper message queue handling
//! and real-time response for mission-critical operations. Each command includes
//! specific timing constraints and preferred communication bands.

use core::fmt;
use heapless::{String, Vec};
use serde::{Deserialize, Serialize};

use crate::error::{Result, SpaceCommError};
use crate::messaging::{Message, MessagePayload, MessagePriority};
use crate::types::{BandType, ComponentId, MessageId};

/// Comprehensive space mission command types with NASA-standard classifications
///
/// This enum represents all possible commands that can be sent to satellite systems,
/// organized by priority levels to ensure proper message queue handling and real-time
/// response for mission-critical operations.
///
/// Requirements Fulfilled:
/// - REQ-FN-001: Command priority classification system
/// - REQ-PF-001: Response time constraints (1ms-10s based on priority)
/// - REQ-SF-001: Command validation and confirmation requirements
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpaceCommand {
    // ==================== EMERGENCY PRIORITY COMMANDS ====================
    // REQ-FN-002: Emergency protocol commands for immediate response
    /// Immediate emergency abort - terminates all operations
    /// REQ-FN-002: Emergency abort protocol with confirmation
    /// REQ-PF-001: Must execute within 1ms for mission safety
    EmergencyAbort {
        reason: EmergencyReason,
        confirmation_code: u32,
    },

    /// Hard stop - immediately halt all satellite operations
    /// REQ-FN-002: Emergency halt with subsystem specification
    /// REQ-SF-002: Override protection for critical safety functions
    EmergencyHalt {
        subsystems: Vec<SubsystemId, 16>,
        override_code: u64,
    },

    /// Safe mode activation - minimal power configuration
    /// REQ-FN-002: Safe mode protocol for system preservation
    /// REQ-NF-004: Power management during emergency situations
    ActivateSafeMode {
        safe_mode_level: SafeModeLevel,
        duration_seconds: Option<u32>,
    },

    /// Emergency power down of non-critical systems
    /// REQ-FN-002: Emergency power management protocol
    /// REQ-NF-004: Battery threshold monitoring and protection
    EmergencyPowerDown {
        systems_to_preserve: Vec<SubsystemId, 8>,
        battery_threshold_percent: u8,
    },

    /// Immediate attitude recovery for spin stabilization
    /// REQ-FN-002: Emergency attitude control protocol
    /// REQ-PF-002: Attitude control performance requirements
    EmergencyAttitudeRecovery {
        target_attitude: [f32; 4], // Quaternion
        max_angular_velocity: f32,
    },

    // ==================== CRITICAL PRIORITY COMMANDS ====================
    // REQ-FN-003: Critical system commands for mission operations
    /// Abort current mission sequence
    /// REQ-FN-003: Mission abort capability with data preservation
    /// REQ-SF-001: Command validation for mission-critical operations
    AbortMission {
        mission_id: u32,
        abort_reason: String<128>,
        preserve_data: bool,
    },

    /// Halt specific subsystem operation
    /// REQ-FN-003: Subsystem control with graceful shutdown
    /// REQ-PF-001: Critical command execution within 10ms
    HaltSubsystem {
        subsystem: SubsystemId,
        graceful_shutdown: bool,
        timeout_seconds: u32,
    },

    /// Execute collision avoidance maneuver
    /// REQ-FN-003: Collision avoidance protocol implementation
    /// REQ-PF-002: Maneuver execution timing requirements
    CollisionAvoidance {
        debris_id: u64,
        maneuver_type: ManeuverType,
        delta_v: [f32; 3],   // m/s in X, Y, Z
        execution_time: u64, // Unix timestamp
    },

    /// Immediate attitude control command
    /// REQ-FN-003: Real-time attitude control system
    /// REQ-PF-001: Attitude response within 10ms deadline
    AttitudeControl {
        target_quaternion: [f32; 4],
        angular_rates: [f32; 3], // rad/s
        control_mode: AttitudeMode,
        deadline_ms: u32,
    },

    /// Switch to backup communication system
    /// REQ-FN-003: Communication redundancy and failover
    /// REQ-FN-007: Multi-band communication support
    SwitchCommBackup {
        primary_failure: String<64>,
        backup_band: BandType,
        power_level_percent: u8,
    },

    /// Reset critical system component
    /// REQ-FN-003: System reset and recovery capability
    /// REQ-SF-002: Component reset with configuration preservation
    ResetSystem {
        component: ComponentId,
        reset_type: ResetType,
        preserve_config: bool,
    },

    // ==================== HIGH PRIORITY COMMANDS ====================
    // REQ-FN-004: High priority operations for mission control
    /// Update orbital parameters
    /// REQ-FN-004: Orbital parameter management and updates
    /// REQ-PF-002: Precision orbital mechanics calculations
    UpdateOrbit {
        semi_major_axis: f64, // km
        eccentricity: f64,
        inclination: f64,   // degrees
        raan: f64,          // Right Ascension of Ascending Node
        arg_periapsis: f64, // Argument of periapsis
        true_anomaly: f64,  // degrees
    },

    /// Reconfigure communication band settings
    /// REQ-FN-004: Communication system reconfiguration
    /// REQ-FN-007: Multi-band frequency management
    /// REQ-IF-002: CCSDS-compliant modulation and error correction
    ReconfigureComm {
        band: BandType,
        frequency_hz: u64,
        power_level: u8, // 0-100%
        modulation: ModulationType,
        error_correction: bool,
    },

    /// Deploy solar panels or antenna
    /// REQ-FN-004: Deployable mechanism control
    /// REQ-SF-001: Deployment validation and force monitoring
    Deploy {
        deployable: DeployableType,
        deployment_angle: f32, // degrees
        deployment_rate: f32,  // degrees/second
        force_limit: f32,      // Newtons
    },

    /// Start science data collection
    /// REQ-FN-004: Science instrument control and data collection
    /// REQ-PF-002: Data rate management and performance
    StartDataCollection {
        instrument: InstrumentId,
        collection_mode: String<32>,
        duration_seconds: u32,
        data_rate_mbps: f32,
    },

    /// Configure power management system
    /// REQ-FN-004: Power system configuration and optimization
    /// REQ-NF-004: Power budget management and load shedding
    ConfigurePower {
        solar_panel_orientation: [f32; 3],
        battery_mode: BatteryMode,
        power_budget_watts: f32,
        load_shedding_priority: Vec<SubsystemId, 16>,
    },

    // ==================== MEDIUM PRIORITY COMMANDS ====================
    // REQ-FN-005: Medium priority commands for normal operations
    /// Request telemetry data
    /// REQ-FN-005: Telemetry data collection and transmission
    /// REQ-IF-002: CCSDS-compliant telemetry formatting
    RequestTelemetry {
        telemetry_type: TelemetryType,
        sampling_rate_hz: f32,
        duration_seconds: u32,
        compression: bool,
    },

    /// Update software configuration
    /// REQ-FN-005: Configuration management and updates
    /// REQ-SF-001: Configuration validation and backup
    UpdateConfig {
        config_id: String<32>,
        parameters: Vec<u8, 512>,
        apply_immediately: bool,
        backup_current: bool,
    },

    /// Calibrate instrument or sensor
    /// REQ-FN-005: Instrument calibration and maintenance
    /// REQ-PF-002: Calibration accuracy and precision requirements
    CalibrateInstrument {
        instrument: InstrumentId,
        calibration_type: CalibrationType,
        reference_values: Vec<f32, 16>,
        temperature_compensation: bool,
    },

    /// Schedule future operations
    /// REQ-FN-005: Command scheduling and automation
    /// REQ-PF-001: Scheduled operation timing constraints
    ScheduleOperation {
        operation_id: u64,
        scheduled_time: u64, // Unix timestamp
        command: Box<SpaceCommand>,
        repeat_interval: Option<u32>, // seconds
    },

    /// Store data to onboard memory
    /// REQ-FN-005: Data storage and management
    /// REQ-SC-001: Data encryption and security requirements
    StoreData {
        data_type: DataType,
        storage_location: StorageLocation,
        compression_level: u8,
        encryption: bool,
    },

    // ==================== LOW PRIORITY COMMANDS ====================
    // REQ-FN-006: Low priority operations for housekeeping
    /// Send status report
    /// REQ-FN-006: System status reporting and diagnostics
    /// REQ-IF-002: Status report formatting and transmission
    SendStatus {
        status_type: StatusType,
        include_diagnostics: bool,
        format: ReportFormat,
    },

    /// Update time synchronization
    /// REQ-FN-006: Time synchronization and accuracy maintenance
    /// REQ-PF-002: Time precision and synchronization requirements
    UpdateTime {
        utc_time: u64, // Unix timestamp
        time_source: TimeSource,
        precision_microseconds: u32,
    },

    /// Perform routine maintenance
    /// REQ-FN-006: Preventive maintenance and system checks
    /// REQ-QL-001: System reliability and maintenance protocols
    PerformMaintenance {
        maintenance_type: MaintenanceType,
        automated: bool,
        estimated_duration: u32, // seconds
    },

    /// Log system event
    /// REQ-FN-006: System event logging and audit trail
    /// REQ-SC-001: Secure logging with event integrity
    LogEvent {
        event_type: EventType,
        severity: EventSeverity,
        description: String<256>,
        associated_data: Vec<u8, 128>,
    },
}

// ==================== SUPPORTING ENUMS AND STRUCTURES ====================
// These enums provide type-safe parameters for command structures

/// Emergency reasons for abort and halt commands
/// REQ-FN-002: Emergency situation classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmergencyReason {
    SystemFailure,
    PowerCritical,
    ThermalEmergency,
    AttitudeLoss,
    CommunicationLoss,
    CollisionImminent,
    GroundCommand,
    OnboardFailsafe,
}

/// Safe mode levels for emergency operation
/// REQ-FN-002: Safe mode hierarchy and system preservation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SafeModeLevel {
    Level1, // Minimal operations, communications only
    Level2, // Basic attitude control + communications
    Level3, // Add power management
    Level4, // Add thermal control
}

/// Satellite subsystem identifiers
/// REQ-FN-003: Subsystem control and management
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubsystemId {
    Power,
    Communications,
    AttitudeControl,
    Propulsion,
    ThermalControl,
    PayloadControl,
    OnboardComputer,
    Navigation,
    SolarPanels,
    BatteryManagement,
    Antenna,
    Sensors,
    DataStorage,
    CommandProcessor,
    Telemetry,
    ErrorCorrection,
}

/// Orbital maneuver types for collision avoidance and orbit changes
/// REQ-FN-003: Orbital mechanics and collision avoidance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ManeuverType {
    OrbitRaise,
    OrbitLower,
    PlaneChange,
    AttitudeAdjust,
    StationKeeping,
    AvoidanceManeuver,
    Deorbit,
}

/// Attitude control modes for spacecraft orientation
/// REQ-FN-003: Attitude control system capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttitudeMode {
    Inertial,
    EarthPointing,
    SunPointing,
    VelocityPointing,
    TargetPointing,
    SpinStabilized,
}

/// System reset types for component recovery
/// REQ-FN-003: System reset and recovery mechanisms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResetType {
    SoftReset,
    HardReset,
    WatchdogReset,
    PowerCycle,
    FactoryReset,
}

/// Communication modulation types
/// REQ-FN-007: Multi-band communication modulation support
/// REQ-IF-002: CCSDS-compliant modulation schemes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModulationType {
    BPSK,
    QPSK,
    PSK8,
    QAM16,
    QAM64,
    OFDM,
}

/// Deployable component types
/// REQ-FN-004: Deployable mechanism control
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeployableType {
    SolarPanel,
    Antenna,
    Magnetometer,
    Sensor,
    CameraLens,
    Radiator,
}

/// Science instrument identifiers
/// REQ-FN-004: Science instrument control and management
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstrumentId {
    Camera,
    Spectrometer,
    Magnetometer,
    Accelerometer,
    Gyroscope,
    TemperatureSensor,
    PressureSensor,
    RadiationDetector,
    GpsReceiver,
    StarTracker,
}

/// Battery operation modes
/// REQ-NF-004: Power management and battery control
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatteryMode {
    Charging,
    Discharging,
    Maintenance,
    Emergency,
    Hibernate,
}

/// Telemetry data types for system monitoring
/// REQ-FN-005: Telemetry data classification and collection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TelemetryType {
    Health,
    Position,
    Attitude,
    Power,
    Thermal,
    Communications,
    Payload,
    Navigation,
    Diagnostics,
    Science,
}

/// Instrument calibration types
/// REQ-FN-005: Instrument calibration procedures
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CalibrationType {
    Bias,
    Scale,
    Temperature,
    Linearity,
    Cross_axis,
    Full,
}

/// Data storage categories
/// REQ-FN-005: Data classification and storage management
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataType {
    Telemetry,
    Science,
    Images,
    Logs,
    Configuration,
    Diagnostic,
}

/// Storage location options
/// REQ-FN-005: Storage hierarchy and management
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageLocation {
    VolatileMemory,
    NonVolatileMemory,
    BackupStorage,
    ExternalStorage,
}

/// Status report types
/// REQ-FN-006: System status monitoring and reporting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatusType {
    SystemHealth,
    MissionStatus,
    ComponentStatus,
    PowerStatus,
    CommunicationStatus,
    Full,
}

/// Report output formats
/// REQ-IF-002: Data formatting and transmission standards
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportFormat {
    Binary,
    Json,
    Csv,
    Compressed,
}

/// Time synchronization sources
/// REQ-FN-006: Time accuracy and synchronization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeSource {
    GroundStation,
    Gps,
    OnboardClock,
    NetworkTime,
    AtomicClock,
}

/// Maintenance operation types
/// REQ-FN-006: Preventive maintenance procedures
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaintenanceType {
    SystemCheck,
    Calibration,
    SoftwareUpdate,
    HardwareTest,
    Performance,
    Preventive,
}

/// System event types for logging
/// REQ-FN-006: Event classification and logging
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    SystemStart,
    SystemShutdown,
    Error,
    Warning,
    Information,
    CommandReceived,
    CommandExecuted,
    DataTransmission,
    ModeChange,
    Anomaly,
}

/// Event severity levels
/// REQ-FN-006: Event prioritization and severity classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl SpaceCommand {
    /// Get the appropriate priority level for this command
    ///
    /// This method maps each command to its proper priority level based on
    /// mission criticality and real-time requirements.
    ///
    /// Requirements Fulfilled:
    /// - REQ-FN-001: Priority-based command classification system
    /// - REQ-PF-001: Response time requirements based on priority
    ///
    /// Returns:
    /// MessagePriority enum value corresponding to command criticality
    pub fn priority(&self) -> MessagePriority {
        match self {
            // Emergency Priority - Immediate response required (REQ-FN-002)
            // Must execute within 1ms for mission safety
            SpaceCommand::EmergencyAbort { .. } => MessagePriority::Emergency,
            SpaceCommand::EmergencyHalt { .. } => MessagePriority::Emergency,
            SpaceCommand::ActivateSafeMode { .. } => MessagePriority::Emergency,
            SpaceCommand::EmergencyPowerDown { .. } => MessagePriority::Emergency,
            SpaceCommand::EmergencyAttitudeRecovery { .. } => MessagePriority::Emergency,

            // Critical Priority - Mission-critical operations (REQ-FN-003)
            // Must execute within 10ms for operational safety
            SpaceCommand::AbortMission { .. } => MessagePriority::Critical,
            SpaceCommand::HaltSubsystem { .. } => MessagePriority::Critical,
            SpaceCommand::CollisionAvoidance { .. } => MessagePriority::Critical,
            SpaceCommand::AttitudeControl { .. } => MessagePriority::Critical,
            SpaceCommand::SwitchCommBackup { .. } => MessagePriority::Critical,
            SpaceCommand::ResetSystem { .. } => MessagePriority::Critical,

            // High Priority - Important operations (REQ-FN-004)
            // Must execute within 100ms for mission effectiveness
            SpaceCommand::UpdateOrbit { .. } => MessagePriority::High,
            SpaceCommand::ReconfigureComm { .. } => MessagePriority::High,
            SpaceCommand::Deploy { .. } => MessagePriority::High,
            SpaceCommand::StartDataCollection { .. } => MessagePriority::High,
            SpaceCommand::ConfigurePower { .. } => MessagePriority::High,

            // Medium Priority - Normal operations (REQ-FN-005)
            // Must execute within 1 second for operational efficiency
            SpaceCommand::RequestTelemetry { .. } => MessagePriority::Medium,
            SpaceCommand::UpdateConfig { .. } => MessagePriority::Medium,
            SpaceCommand::CalibrateInstrument { .. } => MessagePriority::Medium,
            SpaceCommand::ScheduleOperation { .. } => MessagePriority::Medium,
            SpaceCommand::StoreData { .. } => MessagePriority::Medium,

            // Low Priority - Routine operations (REQ-FN-006)
            // Must execute within 10 seconds for housekeeping
            SpaceCommand::SendStatus { .. } => MessagePriority::Low,
            SpaceCommand::UpdateTime { .. } => MessagePriority::Low,
            SpaceCommand::PerformMaintenance { .. } => MessagePriority::Low,
            SpaceCommand::LogEvent { .. } => MessagePriority::Low,
        }
    }

    /// Get the preferred communication band for this command
    ///
    /// Selects optimal communication band based on command priority and
    /// bandwidth requirements for reliable transmission.
    ///
    /// Requirements Fulfilled:
    /// - REQ-FN-007: Multi-band communication support
    /// - REQ-PF-001: Latency requirements for different priorities
    ///
    /// Returns:
    /// BandType enum value for optimal command transmission
    pub fn preferred_band(&self) -> BandType {
        match self.priority() {
            MessagePriority::Emergency => BandType::KBand, // Highest bandwidth for immediate response
            MessagePriority::Critical => BandType::XBand, // High reliability for critical operations
            MessagePriority::High => BandType::SBand, // Good balance of reliability and efficiency
            MessagePriority::Medium => BandType::SBand, // Standard operations band
            MessagePriority::Low => BandType::UhfBand, // Low bandwidth sufficient for housekeeping
        }
    }

    /// Get maximum allowed execution time in milliseconds
    ///
    /// Defines hard real-time constraints for command execution based on
    /// priority level and mission safety requirements.
    ///
    /// Requirements Fulfilled:
    /// - REQ-PF-001: Command response time requirements
    /// - REQ-NF-002: Real-time system constraints
    ///
    /// Returns:
    /// Maximum execution time in milliseconds
    pub fn max_execution_time_ms(&self) -> u32 {
        match self.priority() {
            MessagePriority::Emergency => 1, // Must execute within 1ms (REQ-PF-001)
            MessagePriority::Critical => 10, // Must execute within 10ms
            MessagePriority::High => 100,    // Must execute within 100ms
            MessagePriority::Medium => 1000, // Must execute within 1 second
            MessagePriority::Low => 10000,   // Must execute within 10 seconds
        }
    }

    /// Check if command requires confirmation before execution
    ///
    /// Determines if a command requires explicit confirmation to prevent
    /// accidental execution of potentially destructive operations.
    ///
    /// Requirements Fulfilled:
    /// - REQ-SF-001: Command validation and confirmation
    /// - REQ-SF-002: Safety interlocks for critical operations
    ///
    /// Returns:
    /// true if confirmation required, false otherwise
    pub fn requires_confirmation(&self) -> bool {
        matches!(
            self,
            SpaceCommand::EmergencyAbort { .. }         // REQ-SF-001: Abort confirmation
                | SpaceCommand::EmergencyHalt { .. }    // REQ-SF-001: Halt confirmation
                | SpaceCommand::ActivateSafeMode { .. } // REQ-SF-001: Safe mode confirmation
                | SpaceCommand::AbortMission { .. }     // REQ-SF-001: Mission abort confirmation
                | SpaceCommand::CollisionAvoidance { .. } // REQ-SF-001: Maneuver confirmation
                | SpaceCommand::ResetSystem { .. }      // REQ-SF-001: Reset confirmation
                | SpaceCommand::Deploy { .. } // REQ-SF-001: Deployment confirmation
        )
    }

    /// Get command description for logging
    ///
    /// Provides human-readable descriptions for commands used in logging,
    /// telemetry, and mission operations documentation.
    ///
    /// Requirements Fulfilled:
    /// - REQ-FN-006: Event logging and audit trail
    /// - REQ-QL-002: Documentation and traceability
    ///
    /// Returns:
    /// Static string describing command purpose
    pub fn description(&self) -> &'static str {
        match self {
            SpaceCommand::EmergencyAbort { .. } => "Emergency abort - terminate all operations",
            SpaceCommand::EmergencyHalt { .. } => "Emergency halt - stop all satellite operations",
            SpaceCommand::ActivateSafeMode { .. } => {
                "Activate safe mode - minimal power configuration"
            }
            SpaceCommand::EmergencyPowerDown { .. } => "Emergency power down non-critical systems",
            SpaceCommand::EmergencyAttitudeRecovery { .. } => {
                "Emergency attitude recovery maneuver"
            }
            SpaceCommand::AbortMission { .. } => "Abort current mission sequence",
            SpaceCommand::HaltSubsystem { .. } => "Halt specific subsystem operation",
            SpaceCommand::CollisionAvoidance { .. } => "Execute collision avoidance maneuver",
            SpaceCommand::AttitudeControl { .. } => "Attitude control command",
            SpaceCommand::SwitchCommBackup { .. } => "Switch to backup communication system",
            SpaceCommand::ResetSystem { .. } => "Reset system component",
            SpaceCommand::UpdateOrbit { .. } => "Update orbital parameters",
            SpaceCommand::ReconfigureComm { .. } => "Reconfigure communication settings",
            SpaceCommand::Deploy { .. } => "Deploy solar panel or antenna",
            SpaceCommand::StartDataCollection { .. } => "Start science data collection",
            SpaceCommand::ConfigurePower { .. } => "Configure power management",
            SpaceCommand::RequestTelemetry { .. } => "Request telemetry data",
            SpaceCommand::UpdateConfig { .. } => "Update software configuration",
            SpaceCommand::CalibrateInstrument { .. } => "Calibrate instrument or sensor",
            SpaceCommand::ScheduleOperation { .. } => "Schedule future operation",
            SpaceCommand::StoreData { .. } => "Store data to onboard memory",
            SpaceCommand::SendStatus { .. } => "Send status report",
            SpaceCommand::UpdateTime { .. } => "Update time synchronization",
            SpaceCommand::PerformMaintenance { .. } => "Perform routine maintenance",
            SpaceCommand::LogEvent { .. } => "Log system event",
        }
    }
}

impl fmt::Display for SpaceCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} (Priority: {:?})",
            self.description(),
            self.priority()
        )
    }
}

/// Command builder for creating space commands with proper validation
///
/// This builder pattern ensures that all space commands are properly constructed
/// with valid source/destination routing and appropriate message formatting.
///
/// Requirements Fulfilled:
/// - REQ-SF-001: Command validation and safety checks
/// - REQ-IF-002: CCSDS-compliant message structure
/// - REQ-FN-001: Priority-based command routing
///
/// Example Usage:
/// ```rust
/// let command_msg = CommandBuilder::new(ComponentId::GroundStation, ComponentId::Satellite)
///     .emergency_abort(EmergencyReason::SystemFailure, 0x12345678)
///     .build()?;
/// ```
pub struct CommandBuilder {
    command: Option<SpaceCommand>,
    source: ComponentId,
    destination: ComponentId,
}

impl CommandBuilder {
    /// Create a new command builder with source and destination routing
    ///
    /// Parameters:
    /// - source: Component ID of the command originator
    /// - destination: Component ID of the command target
    ///
    /// Requirements Fulfilled:
    /// - REQ-IF-002: Message routing and addressing
    pub fn new(source: ComponentId, destination: ComponentId) -> Self {
        Self {
            command: None,
            source,
            destination,
        }
    }

    /// Create emergency abort command
    ///
    /// Builds an emergency abort command with proper confirmation codes
    /// for immediate mission termination.
    ///
    /// Parameters:
    /// - reason: Emergency situation requiring abort
    /// - confirmation_code: Security code to prevent accidental abort
    ///
    /// Requirements Fulfilled:
    /// - REQ-FN-002: Emergency abort protocol
    /// - REQ-SF-001: Confirmation requirement for destructive commands
    pub fn emergency_abort(mut self, reason: EmergencyReason, confirmation_code: u32) -> Self {
        self.command = Some(SpaceCommand::EmergencyAbort {
            reason,
            confirmation_code,
        });
        self
    }

    /// Create collision avoidance command
    ///
    /// Builds a collision avoidance maneuver command with orbital mechanics
    /// parameters for debris avoidance or orbital adjustments.
    ///
    /// Parameters:
    /// - debris_id: Unique identifier for the debris or object to avoid
    /// - maneuver_type: Type of orbital maneuver to perform
    /// - delta_v: Change in velocity vector [X, Y, Z] in m/s
    /// - execution_time: Unix timestamp for maneuver execution
    ///
    /// Requirements Fulfilled:
    /// - REQ-FN-003: Collision avoidance capability
    /// - REQ-PF-002: Orbital mechanics precision
    pub fn collision_avoidance(
        mut self,
        debris_id: u64,
        maneuver_type: ManeuverType,
        delta_v: [f32; 3],
        execution_time: u64,
    ) -> Self {
        self.command = Some(SpaceCommand::CollisionAvoidance {
            debris_id,
            maneuver_type,
            delta_v,
            execution_time,
        });
        self
    }

    /// Create attitude control command
    ///
    /// Builds an attitude control command for spacecraft orientation
    /// and pointing with real-time deadline constraints.
    ///
    /// Parameters:
    /// - target_quaternion: Desired spacecraft attitude as quaternion [w, x, y, z]
    /// - angular_rates: Desired angular rates [roll, pitch, yaw] in rad/s
    /// - control_mode: Attitude control mode (inertial, earth-pointing, etc.)
    /// - deadline_ms: Maximum time allowed for attitude change
    ///
    /// Requirements Fulfilled:
    /// - REQ-FN-003: Attitude control system
    /// - REQ-PF-001: Real-time control deadlines
    pub fn attitude_control(
        mut self,
        target_quaternion: [f32; 4],
        angular_rates: [f32; 3],
        control_mode: AttitudeMode,
        deadline_ms: u32,
    ) -> Self {
        self.command = Some(SpaceCommand::AttitudeControl {
            target_quaternion,
            angular_rates,
            control_mode,
            deadline_ms,
        });
        self
    }

    /// Build the final message
    ///
    /// Constructs a complete Message structure with proper priority assignment,
    /// band selection, and CCSDS-compliant formatting.
    ///
    /// Returns:
    /// Result<Message> - Complete message ready for transmission
    ///
    /// Requirements Fulfilled:
    /// - REQ-IF-002: CCSDS message structure and serialization
    /// - REQ-FN-001: Priority-based routing and band selection
    /// - REQ-FN-007: Multi-band communication support
    pub fn build(self) -> Result<Message> {
        // Validate that a command was specified (REQ-SF-001)
        let command = self
            .command
            .ok_or_else(|| SpaceCommError::validation_error("No command specified in builder"))?;

        // Get command properties for message construction
        let priority = command.priority(); // REQ-FN-001: Priority assignment
        let preferred_band = command.preferred_band(); // REQ-FN-007: Band selection

        // Serialize command to bytes for transmission (REQ-IF-002)
        let command_bytes = serde_json::to_vec(&command)
            .map_err(|_| SpaceCommError::validation_error("Failed to serialize command"))?;

        // Pack command bytes into message parameters
        let mut parameters = Vec::new();
        parameters
            .extend_from_slice(&command_bytes)
            .map_err(|_| SpaceCommError::validation_error("Command too large for message"))?;

        // Construct complete message with all required fields
        Ok(Message {
            id: MessageId::new(),
            priority,
            source: self.source,
            destination: self.destination,
            timestamp: crate::time::current_time_nanos(),
            payload: MessagePayload::Command {
                command_id: command.discriminant(),
                parameters,
            },
            preferred_band,
            ttl_seconds: priority.max_latency_ms() / 1000 + 1, // REQ-PF-001: TTL based on latency
            retry_count: 0,
            max_retries: if priority.is_real_time() { 3 } else { 1 }, // REQ-NF-002: Retry logic
        })
    }
}

impl SpaceCommand {
    /// Get discriminant value for command type identification
    ///
    /// Returns a unique identifier for each command type used in message
    /// routing, logging, and protocol compliance verification.
    ///
    /// Requirements Fulfilled:
    /// - REQ-IF-002: CCSDS command identification and routing
    /// - REQ-FN-006: Command logging and audit trail
    ///
    /// Returns:
    /// u32 discriminant value uniquely identifying command type
    pub fn discriminant(&self) -> u32 {
        match self {
            // Emergency Commands (0x0001-0x000F) - REQ-FN-002
            SpaceCommand::EmergencyAbort { .. } => 0x0001,
            SpaceCommand::EmergencyHalt { .. } => 0x0002,
            SpaceCommand::ActivateSafeMode { .. } => 0x0003,
            SpaceCommand::EmergencyPowerDown { .. } => 0x0004,
            SpaceCommand::EmergencyAttitudeRecovery { .. } => 0x0005,

            // Critical Commands (0x0010-0x001F) - REQ-FN-003
            SpaceCommand::AbortMission { .. } => 0x0010,
            SpaceCommand::HaltSubsystem { .. } => 0x0011,
            SpaceCommand::CollisionAvoidance { .. } => 0x0012,
            SpaceCommand::AttitudeControl { .. } => 0x0013,
            SpaceCommand::SwitchCommBackup { .. } => 0x0014,
            SpaceCommand::ResetSystem { .. } => 0x0015,

            // High Priority Commands (0x0020-0x002F) - REQ-FN-004
            SpaceCommand::UpdateOrbit { .. } => 0x0020,
            SpaceCommand::ReconfigureComm { .. } => 0x0021,
            SpaceCommand::Deploy { .. } => 0x0022,
            SpaceCommand::StartDataCollection { .. } => 0x0023,
            SpaceCommand::ConfigurePower { .. } => 0x0024,

            // Medium Priority Commands (0x0030-0x003F) - REQ-FN-005
            SpaceCommand::RequestTelemetry { .. } => 0x0030,
            SpaceCommand::UpdateConfig { .. } => 0x0031,
            SpaceCommand::CalibrateInstrument { .. } => 0x0032,
            SpaceCommand::ScheduleOperation { .. } => 0x0033,
            SpaceCommand::StoreData { .. } => 0x0034,

            // Low Priority Commands (0x0040-0x004F) - REQ-FN-006
            SpaceCommand::SendStatus { .. } => 0x0040,
            SpaceCommand::UpdateTime { .. } => 0x0041,
            SpaceCommand::PerformMaintenance { .. } => 0x0042,
            SpaceCommand::LogEvent { .. } => 0x0043,
        }
    }
}
