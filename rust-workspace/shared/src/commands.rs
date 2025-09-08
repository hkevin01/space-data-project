//! Mission-critical space commands with priority classification
//!
//! This module defines a comprehensive set of satellite and ground station
//! commands used in real space missions, classified by criticality and
//! priority levels for proper message queue handling.

use core::fmt;
use serde::{Deserialize, Serialize};
use heapless::{String, Vec};

use crate::messaging::{Message, MessagePayload, MessagePriority};
use crate::types::{ComponentId, MessageId, BandType};
use crate::error::{Result, SpaceCommError};

/// Comprehensive space mission command types with NASA-standard classifications
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpaceCommand {
    // ==================== EMERGENCY PRIORITY COMMANDS ====================
    /// Immediate emergency abort - terminates all operations
    EmergencyAbort {
        reason: EmergencyReason,
        confirmation_code: u32,
    },

    /// Hard stop - immediately halt all satellite operations
    EmergencyHalt {
        subsystems: Vec<SubsystemId, 16>,
        override_code: u64,
    },

    /// Safe mode activation - minimal power configuration
    ActivateSafeMode {
        safe_mode_level: SafeModeLevel,
        duration_seconds: Option<u32>,
    },

    /// Emergency power down of non-critical systems
    EmergencyPowerDown {
        systems_to_preserve: Vec<SubsystemId, 8>,
        battery_threshold_percent: u8,
    },

    /// Immediate attitude recovery for spin stabilization
    EmergencyAttitudeRecovery {
        target_attitude: [f32; 4], // Quaternion
        max_angular_velocity: f32,
    },

    // ==================== CRITICAL PRIORITY COMMANDS ====================
    /// Abort current mission sequence
    AbortMission {
        mission_id: u32,
        abort_reason: String<128>,
        preserve_data: bool,
    },

    /// Halt specific subsystem operation
    HaltSubsystem {
        subsystem: SubsystemId,
        graceful_shutdown: bool,
        timeout_seconds: u32,
    },

    /// Execute collision avoidance maneuver
    CollisionAvoidance {
        debris_id: u64,
        maneuver_type: ManeuverType,
        delta_v: [f32; 3], // m/s in X, Y, Z
        execution_time: u64, // Unix timestamp
    },

    /// Immediate attitude control command
    AttitudeControl {
        target_quaternion: [f32; 4],
        angular_rates: [f32; 3], // rad/s
        control_mode: AttitudeMode,
        deadline_ms: u32,
    },

    /// Switch to backup communication system
    SwitchCommBackup {
        primary_failure: String<64>,
        backup_band: BandType,
        power_level_percent: u8,
    },

    /// Reset critical system component
    ResetSystem {
        component: ComponentId,
        reset_type: ResetType,
        preserve_config: bool,
    },

    // ==================== HIGH PRIORITY COMMANDS ====================
    /// Update orbital parameters
    UpdateOrbit {
        semi_major_axis: f64,    // km
        eccentricity: f64,
        inclination: f64,        // degrees
        raan: f64,              // Right Ascension of Ascending Node
        arg_periapsis: f64,     // Argument of periapsis
        true_anomaly: f64,      // degrees
    },

    /// Reconfigure communication band settings
    ReconfigureComm {
        band: BandType,
        frequency_hz: u64,
        power_level: u8,        // 0-100%
        modulation: ModulationType,
        error_correction: bool,
    },

    /// Deploy solar panels or antenna
    Deploy {
        deployable: DeployableType,
        deployment_angle: f32,   // degrees
        deployment_rate: f32,    // degrees/second
        force_limit: f32,        // Newtons
    },

    /// Start science data collection
    StartDataCollection {
        instrument: InstrumentId,
        collection_mode: String<32>,
        duration_seconds: u32,
        data_rate_mbps: f32,
    },

    /// Configure power management system
    ConfigurePower {
        solar_panel_orientation: [f32; 3],
        battery_mode: BatteryMode,
        power_budget_watts: f32,
        load_shedding_priority: Vec<SubsystemId, 16>,
    },

    // ==================== MEDIUM PRIORITY COMMANDS ====================
    /// Request telemetry data
    RequestTelemetry {
        telemetry_type: TelemetryType,
        sampling_rate_hz: f32,
        duration_seconds: u32,
        compression: bool,
    },

    /// Update software configuration
    UpdateConfig {
        config_id: String<32>,
        parameters: Vec<u8, 512>,
        apply_immediately: bool,
        backup_current: bool,
    },

    /// Calibrate instrument or sensor
    CalibrateInstrument {
        instrument: InstrumentId,
        calibration_type: CalibrationType,
        reference_values: Vec<f32, 16>,
        temperature_compensation: bool,
    },

    /// Schedule future operations
    ScheduleOperation {
        operation_id: u64,
        scheduled_time: u64,    // Unix timestamp
        command: Box<SpaceCommand>,
        repeat_interval: Option<u32>, // seconds
    },

    /// Store data to onboard memory
    StoreData {
        data_type: DataType,
        storage_location: StorageLocation,
        compression_level: u8,
        encryption: bool,
    },

    // ==================== LOW PRIORITY COMMANDS ====================
    /// Send status report
    SendStatus {
        status_type: StatusType,
        include_diagnostics: bool,
        format: ReportFormat,
    },

    /// Update time synchronization
    UpdateTime {
        utc_time: u64,          // Unix timestamp
        time_source: TimeSource,
        precision_microseconds: u32,
    },

    /// Perform routine maintenance
    PerformMaintenance {
        maintenance_type: MaintenanceType,
        automated: bool,
        estimated_duration: u32, // seconds
    },

    /// Log system event
    LogEvent {
        event_type: EventType,
        severity: EventSeverity,
        description: String<256>,
        associated_data: Vec<u8, 128>,
    },
}

// ==================== SUPPORTING ENUMS AND STRUCTURES ====================

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SafeModeLevel {
    Level1, // Minimal operations, communications only
    Level2, // Basic attitude control + communications
    Level3, // Add power management
    Level4, // Add thermal control
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttitudeMode {
    Inertial,
    EarthPointing,
    SunPointing,
    VelocityPointing,
    TargetPointing,
    SpinStabilized,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResetType {
    SoftReset,
    HardReset,
    WatchdogReset,
    PowerCycle,
    FactoryReset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModulationType {
    BPSK,
    QPSK,
    PSK8,
    QAM16,
    QAM64,
    OFDM,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeployableType {
    SolarPanel,
    Antenna,
    Magnetometer,
    Sensor,
    CameraLens,
    Radiator,
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatteryMode {
    Charging,
    Discharging,
    Maintenance,
    Emergency,
    Hibernate,
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CalibrationType {
    Bias,
    Scale,
    Temperature,
    Linearity,
    Cross_axis,
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataType {
    Telemetry,
    Science,
    Images,
    Logs,
    Configuration,
    Diagnostic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageLocation {
    VolatileMemory,
    NonVolatileMemory,
    BackupStorage,
    ExternalStorage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatusType {
    SystemHealth,
    MissionStatus,
    ComponentStatus,
    PowerStatus,
    CommunicationStatus,
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportFormat {
    Binary,
    Json,
    Csv,
    Compressed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeSource {
    GroundStation,
    Gps,
    OnboardClock,
    NetworkTime,
    AtomicClock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaintenanceType {
    SystemCheck,
    Calibration,
    SoftwareUpdate,
    HardwareTest,
    Performance,
    Preventive,
}

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
    pub fn priority(&self) -> MessagePriority {
        match self {
            // Emergency Priority - Immediate response required
            SpaceCommand::EmergencyAbort { .. } => MessagePriority::Emergency,
            SpaceCommand::EmergencyHalt { .. } => MessagePriority::Emergency,
            SpaceCommand::ActivateSafeMode { .. } => MessagePriority::Emergency,
            SpaceCommand::EmergencyPowerDown { .. } => MessagePriority::Emergency,
            SpaceCommand::EmergencyAttitudeRecovery { .. } => MessagePriority::Emergency,

            // Critical Priority - Mission-critical operations
            SpaceCommand::AbortMission { .. } => MessagePriority::Critical,
            SpaceCommand::HaltSubsystem { .. } => MessagePriority::Critical,
            SpaceCommand::CollisionAvoidance { .. } => MessagePriority::Critical,
            SpaceCommand::AttitudeControl { .. } => MessagePriority::Critical,
            SpaceCommand::SwitchCommBackup { .. } => MessagePriority::Critical,
            SpaceCommand::ResetSystem { .. } => MessagePriority::Critical,

            // High Priority - Important operations
            SpaceCommand::UpdateOrbit { .. } => MessagePriority::High,
            SpaceCommand::ReconfigureComm { .. } => MessagePriority::High,
            SpaceCommand::Deploy { .. } => MessagePriority::High,
            SpaceCommand::StartDataCollection { .. } => MessagePriority::High,
            SpaceCommand::ConfigurePower { .. } => MessagePriority::High,

            // Medium Priority - Normal operations
            SpaceCommand::RequestTelemetry { .. } => MessagePriority::Medium,
            SpaceCommand::UpdateConfig { .. } => MessagePriority::Medium,
            SpaceCommand::CalibrateInstrument { .. } => MessagePriority::Medium,
            SpaceCommand::ScheduleOperation { .. } => MessagePriority::Medium,
            SpaceCommand::StoreData { .. } => MessagePriority::Medium,

            // Low Priority - Routine operations
            SpaceCommand::SendStatus { .. } => MessagePriority::Low,
            SpaceCommand::UpdateTime { .. } => MessagePriority::Low,
            SpaceCommand::PerformMaintenance { .. } => MessagePriority::Low,
            SpaceCommand::LogEvent { .. } => MessagePriority::Low,
        }
    }

    /// Get the preferred communication band for this command
    pub fn preferred_band(&self) -> BandType {
        match self.priority() {
            MessagePriority::Emergency => BandType::KBand,  // Highest bandwidth for immediate response
            MessagePriority::Critical => BandType::XBand,   // High reliability
            MessagePriority::High => BandType::SBand,       // Good balance of reliability and efficiency
            MessagePriority::Medium => BandType::SBand,     // Standard operations
            MessagePriority::Low => BandType::UhfBand,      // Low bandwidth sufficient
        }
    }

    /// Get maximum allowed execution time in milliseconds
    pub fn max_execution_time_ms(&self) -> u32 {
        match self.priority() {
            MessagePriority::Emergency => 1,      // Must execute within 1ms
            MessagePriority::Critical => 10,      // Must execute within 10ms
            MessagePriority::High => 100,         // Must execute within 100ms
            MessagePriority::Medium => 1000,      // Must execute within 1 second
            MessagePriority::Low => 10000,        // Must execute within 10 seconds
        }
    }

    /// Check if command requires confirmation before execution
    pub fn requires_confirmation(&self) -> bool {
        matches!(
            self,
            SpaceCommand::EmergencyAbort { .. }
                | SpaceCommand::EmergencyHalt { .. }
                | SpaceCommand::ActivateSafeMode { .. }
                | SpaceCommand::AbortMission { .. }
                | SpaceCommand::CollisionAvoidance { .. }
                | SpaceCommand::ResetSystem { .. }
                | SpaceCommand::Deploy { .. }
        )
    }

    /// Get command description for logging
    pub fn description(&self) -> &'static str {
        match self {
            SpaceCommand::EmergencyAbort { .. } => "Emergency abort - terminate all operations",
            SpaceCommand::EmergencyHalt { .. } => "Emergency halt - stop all satellite operations",
            SpaceCommand::ActivateSafeMode { .. } => "Activate safe mode - minimal power configuration",
            SpaceCommand::EmergencyPowerDown { .. } => "Emergency power down non-critical systems",
            SpaceCommand::EmergencyAttitudeRecovery { .. } => "Emergency attitude recovery maneuver",
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
        write!(f, "{} (Priority: {:?})", self.description(), self.priority())
    }
}

/// Command builder for creating space commands with proper validation
pub struct CommandBuilder {
    command: Option<SpaceCommand>,
    source: ComponentId,
    destination: ComponentId,
}

impl CommandBuilder {
    pub fn new(source: ComponentId, destination: ComponentId) -> Self {
        Self {
            command: None,
            source,
            destination,
        }
    }

    /// Create emergency abort command
    pub fn emergency_abort(mut self, reason: EmergencyReason, confirmation_code: u32) -> Self {
        self.command = Some(SpaceCommand::EmergencyAbort {
            reason,
            confirmation_code,
        });
        self
    }

    /// Create collision avoidance command
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
    pub fn build(self) -> Result<Message> {
        let command = self.command.ok_or_else(|| {
            SpaceCommError::validation_error("No command specified in builder")
        })?;

        let priority = command.priority();
        let preferred_band = command.preferred_band();

        // Serialize command to bytes
        let command_bytes = serde_json::to_vec(&command)
            .map_err(|_| SpaceCommError::validation_error("Failed to serialize command"))?;

        let mut parameters = Vec::new();
        parameters.extend_from_slice(&command_bytes).map_err(|_| {
            SpaceCommError::validation_error("Command too large for message")
        })?;

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
            ttl_seconds: priority.max_latency_ms() / 1000 + 1,
            retry_count: 0,
            max_retries: if priority.is_real_time() { 3 } else { 1 },
        })
    }
}

impl SpaceCommand {
    /// Get discriminant value for command type identification
    pub fn discriminant(&self) -> u32 {
        match self {
            SpaceCommand::EmergencyAbort { .. } => 0x0001,
            SpaceCommand::EmergencyHalt { .. } => 0x0002,
            SpaceCommand::ActivateSafeMode { .. } => 0x0003,
            SpaceCommand::EmergencyPowerDown { .. } => 0x0004,
            SpaceCommand::EmergencyAttitudeRecovery { .. } => 0x0005,
            SpaceCommand::AbortMission { .. } => 0x0010,
            SpaceCommand::HaltSubsystem { .. } => 0x0011,
            SpaceCommand::CollisionAvoidance { .. } => 0x0012,
            SpaceCommand::AttitudeControl { .. } => 0x0013,
            SpaceCommand::SwitchCommBackup { .. } => 0x0014,
            SpaceCommand::ResetSystem { .. } => 0x0015,
            SpaceCommand::UpdateOrbit { .. } => 0x0020,
            SpaceCommand::ReconfigureComm { .. } => 0x0021,
            SpaceCommand::Deploy { .. } => 0x0022,
            SpaceCommand::StartDataCollection { .. } => 0x0023,
            SpaceCommand::ConfigurePower { .. } => 0x0024,
            SpaceCommand::RequestTelemetry { .. } => 0x0030,
            SpaceCommand::UpdateConfig { .. } => 0x0031,
            SpaceCommand::CalibrateInstrument { .. } => 0x0032,
            SpaceCommand::ScheduleOperation { .. } => 0x0033,
            SpaceCommand::StoreData { .. } => 0x0034,
            SpaceCommand::SendStatus { .. } => 0x0040,
            SpaceCommand::UpdateTime { .. } => 0x0041,
            SpaceCommand::PerformMaintenance { .. } => 0x0042,
            SpaceCommand::LogEvent { .. } => 0x0043,
        }
    }
}
