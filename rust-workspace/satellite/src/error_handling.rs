//! Error handling and logging module for satellite systems
//!
//! Provides comprehensive error handling, logging, and fault recovery mechanisms
//! following NASA software engineering standards.

use embassy_time::{Duration, Instant, Timer};
use heapless::{String, Vec};

use space_comms_shared::{SpaceCommError, error::ErrorSeverity};

/// Log entry structure
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// Timestamp in milliseconds since boot
    pub timestamp: u64,
    /// Log level
    pub level: LogLevel,
    /// Message content
    pub message: String<256>,
    /// Error code if applicable
    pub error_code: Option<u32>,
    /// Component that generated the log
    pub component: String<32>,
}

/// Log levels
#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel {
    /// Critical errors requiring immediate attention
    Critical,
    /// Error conditions
    Error,
    /// Warning conditions
    Warning,
    /// Informational messages
    Info,
    /// Debug information
    Debug,
}

/// System fault types
#[derive(Debug, Clone)]
pub enum FaultType {
    /// Hardware failure
    Hardware { component: String<32>, error_code: u32 },
    /// Software fault
    Software { module: String<32>, error_code: u32 },
    /// Communication failure
    Communication { band: String<16>, error_code: u32 },
    /// Power system fault
    Power { subsystem: String<32>, error_code: u32 },
    /// Thermal fault
    Thermal { sensor_id: u16, temperature: f32 },
    /// Memory fault
    Memory { address: Option<u32>, error_code: u32 },
}

/// Fault recovery action
#[derive(Debug, Clone)]
pub enum RecoveryAction {
    /// No action required
    None,
    /// Restart component
    RestartComponent(String<32>),
    /// Switch to backup
    SwitchToBackup(String<32>),
    /// Power cycle hardware
    PowerCycle(String<32>),
    /// Enter safe mode
    SafeMode,
    /// Emergency shutdown
    EmergencyShutdown,
}

/// Error handler state
pub struct ErrorHandler {
    /// Recent log entries
    log_buffer: Vec<LogEntry, 256>,
    /// Active faults
    active_faults: Vec<FaultType, 32>,
    /// Error counts by component
    error_counts: Vec<(String<32>, u32), 16>,
    /// System health status
    system_health: SystemHealth,
}

/// System health status
#[derive(Debug, Clone)]
pub struct SystemHealth {
    /// Overall health percentage (0-100)
    pub overall_health: u8,
    /// Critical faults count
    pub critical_faults: u8,
    /// Warning count
    pub warnings: u8,
    /// Time since last critical fault
    pub time_since_critical: u64,
    /// Is system in safe mode
    pub is_safe_mode: bool,
}

impl ErrorHandler {
    /// Create new error handler
    pub fn new() -> Self {
        Self {
            log_buffer: Vec::new(),
            active_faults: Vec::new(),
            error_counts: Vec::new(),
            system_health: SystemHealth {
                overall_health: 100,
                critical_faults: 0,
                warnings: 0,
                time_since_critical: 0,
                is_safe_mode: false,
            },
        }
    }

    /// Add log entry
    pub fn add_log(&mut self, level: LogLevel, message: &str, component: &str, error_code: Option<u32>) {
        let entry = LogEntry {
            timestamp: Instant::now().as_millis(),
            level: level.clone(),
            message: String::try_from(message).unwrap_or_else(|_| String::from("Log message too long")),
            error_code,
            component: String::try_from(component).unwrap_or_else(|_| String::from("Unknown")),
        };

        // Add to buffer (remove oldest if full)
        if self.log_buffer.is_full() {
            self.log_buffer.remove(0);
        }
        let _ = self.log_buffer.push(entry);

        // Update health status based on log level
        match level {
            LogLevel::Critical => {
                self.system_health.critical_faults += 1;
                self.system_health.overall_health = self.system_health.overall_health.saturating_sub(20);
                self.system_health.time_since_critical = 0;
            }
            LogLevel::Error => {
                self.system_health.overall_health = self.system_health.overall_health.saturating_sub(10);
            }
            LogLevel::Warning => {
                self.system_health.warnings += 1;
                self.system_health.overall_health = self.system_health.overall_health.saturating_sub(2);
            }
            _ => {}
        }

        // Update error counts
        self.update_error_count(component);
    }

    /// Add active fault
    pub fn add_fault(&mut self, fault: FaultType) -> RecoveryAction {
        // Check if fault already exists
        let fault_exists = self.active_faults.iter().any(|f| {
            core::mem::discriminant(f) == core::mem::discriminant(&fault)
        });

        if !fault_exists && !self.active_faults.is_full() {
            let _ = self.active_faults.push(fault.clone());
        }

        // Determine recovery action
        self.determine_recovery_action(&fault)
    }

    /// Remove resolved fault
    pub fn remove_fault(&mut self, fault_type: &FaultType) {
        self.active_faults.retain(|f| {
            core::mem::discriminant(f) != core::mem::discriminant(fault_type)
        });

        // Improve health when faults are resolved
        if self.active_faults.is_empty() {
            self.system_health.overall_health =
                core::cmp::min(self.system_health.overall_health + 10, 100);
        }
    }

    /// Get system health
    pub fn get_health(&self) -> &SystemHealth {
        &self.system_health
    }

    /// Get recent logs
    pub fn get_recent_logs(&self, count: usize) -> &[LogEntry] {
        let start = self.log_buffer.len().saturating_sub(count);
        &self.log_buffer[start..]
    }

    /// Get active faults
    pub fn get_active_faults(&self) -> &[FaultType] {
        &self.active_faults
    }

    /// Update error count for component
    fn update_error_count(&mut self, component: &str) {
        let component_string = String::try_from(component).unwrap_or_else(|_| String::from("Unknown"));

        // Find existing entry or create new one
        if let Some(entry) = self.error_counts.iter_mut().find(|(name, _)| name == &component_string) {
            entry.1 += 1;
        } else if !self.error_counts.is_full() {
            let _ = self.error_counts.push((component_string, 1));
        }
    }

    /// Determine appropriate recovery action for fault
    fn determine_recovery_action(&self, fault: &FaultType) -> RecoveryAction {
        match fault {
            FaultType::Hardware { component, error_code } => {
                if *error_code >= 1000 {
                    RecoveryAction::EmergencyShutdown
                } else if *error_code >= 500 {
                    RecoveryAction::PowerCycle(component.clone())
                } else {
                    RecoveryAction::RestartComponent(component.clone())
                }
            }
            FaultType::Software { module, error_code } => {
                if *error_code >= 900 {
                    RecoveryAction::SafeMode
                } else {
                    RecoveryAction::RestartComponent(module.clone())
                }
            }
            FaultType::Communication { band, error_code } => {
                if *error_code >= 800 {
                    RecoveryAction::SwitchToBackup(band.clone())
                } else {
                    RecoveryAction::RestartComponent(band.clone())
                }
            }
            FaultType::Power { subsystem, error_code } => {
                if *error_code >= 700 {
                    RecoveryAction::EmergencyShutdown
                } else {
                    RecoveryAction::PowerCycle(subsystem.clone())
                }
            }
            FaultType::Thermal { temperature, .. } => {
                if *temperature > 80.0 || *temperature < -50.0 {
                    RecoveryAction::EmergencyShutdown
                } else if *temperature > 70.0 || *temperature < -40.0 {
                    RecoveryAction::SafeMode
                } else {
                    RecoveryAction::None
                }
            }
            FaultType::Memory { address: _, error_code } => {
                if *error_code >= 600 {
                    RecoveryAction::SafeMode
                } else {
                    RecoveryAction::None
                }
            }
        }
    }

    /// Periodic health update
    pub fn update_health(&mut self) {
        self.system_health.time_since_critical += 1;

        // Gradually improve health if no recent critical errors
        if self.system_health.time_since_critical > 3600 { // 1 hour
            self.system_health.overall_health =
                core::cmp::min(self.system_health.overall_health + 1, 100);
        }

        // Reset counters periodically
        if self.system_health.time_since_critical > 86400 { // 24 hours
            self.system_health.critical_faults = 0;
            self.system_health.warnings = 0;
        }
    }
}

/// Global error handler instance
static mut ERROR_HANDLER: Option<ErrorHandler> = None;

/// Initialize error handling system
pub fn initialize() {
    unsafe {
        ERROR_HANDLER = Some(ErrorHandler::new());
    }
}

/// Log critical error
pub fn log_critical(message: &str) {
    log_with_component(LogLevel::Critical, message, "SYSTEM", None);
}

/// Log error
pub fn log_error(message: &str) {
    log_with_component(LogLevel::Error, message, "SYSTEM", None);
}

/// Log warning
pub fn log_warning(message: &str) {
    log_with_component(LogLevel::Warning, message, "SYSTEM", None);
}

/// Log info
pub fn log_info(message: &str) {
    log_with_component(LogLevel::Info, message, "SYSTEM", None);
}

/// Log debug
pub fn log_debug(message: &str) {
    log_with_component(LogLevel::Debug, message, "SYSTEM", None);
}

/// Log with specific component
pub fn log_with_component(level: LogLevel, message: &str, component: &str, error_code: Option<u32>) {
    unsafe {
        if let Some(handler) = ERROR_HANDLER.as_mut() {
            handler.add_log(level, message, component, error_code);
        }
    }
}

/// Handle system fault
pub fn handle_fault(fault: FaultType) -> RecoveryAction {
    unsafe {
        if let Some(handler) = ERROR_HANDLER.as_mut() {
            let action = handler.add_fault(fault.clone());

            // Log the fault
            match &fault {
                FaultType::Hardware { component, error_code } => {
                    log_with_component(
                        LogLevel::Critical,
                        &format!("Hardware fault in {}", component),
                        component,
                        Some(*error_code)
                    );
                }
                FaultType::Software { module, error_code } => {
                    log_with_component(
                        LogLevel::Error,
                        &format!("Software fault in {}", module),
                        module,
                        Some(*error_code)
                    );
                }
                FaultType::Communication { band, error_code } => {
                    log_with_component(
                        LogLevel::Warning,
                        &format!("Communication fault on {}", band),
                        "COMM",
                        Some(*error_code)
                    );
                }
                FaultType::Power { subsystem, error_code } => {
                    log_with_component(
                        LogLevel::Critical,
                        &format!("Power fault in {}", subsystem),
                        "POWER",
                        Some(*error_code)
                    );
                }
                FaultType::Thermal { sensor_id, temperature } => {
                    log_with_component(
                        LogLevel::Warning,
                        &format!("Thermal fault: sensor {} at {}°C", sensor_id, temperature),
                        "THERMAL",
                        Some(*sensor_id as u32)
                    );
                }
                FaultType::Memory { address, error_code } => {
                    let addr_str = address.map(|a| format!("0x{:08X}", a)).unwrap_or_else(|| "Unknown".to_string());
                    log_with_component(
                        LogLevel::Error,
                        &format!("Memory fault at {}", addr_str),
                        "MEMORY",
                        Some(*error_code)
                    );
                }
            }

            action
        } else {
            RecoveryAction::None
        }
    }
}

/// Resolve fault
pub fn resolve_fault(fault: FaultType) {
    unsafe {
        if let Some(handler) = ERROR_HANDLER.as_mut() {
            handler.remove_fault(&fault);
            log_info("Fault resolved");
        }
    }
}

/// Get system health
pub fn get_system_health() -> SystemHealth {
    unsafe {
        ERROR_HANDLER.as_ref()
            .map(|h| h.get_health().clone())
            .unwrap_or_else(|| SystemHealth {
                overall_health: 0,
                critical_faults: 255,
                warnings: 255,
                time_since_critical: 0,
                is_safe_mode: true,
            })
    }
}

/// Handle SpaceCommError
pub fn handle_space_comm_error(error: &SpaceCommError) -> RecoveryAction {
    match error.severity() {
        ErrorSeverity::Critical => {
            log_critical(&format!("Critical SpaceComm error: {}", error));
            RecoveryAction::SafeMode
        }
        ErrorSeverity::High => {
            log_error(&format!("High severity SpaceComm error: {}", error));
            RecoveryAction::RestartComponent(String::from("COMM"))
        }
        ErrorSeverity::Medium => {
            log_warning(&format!("Medium severity SpaceComm error: {}", error));
            RecoveryAction::None
        }
        ErrorSeverity::Low => {
            log_info(&format!("Low severity SpaceComm error: {}", error));
            RecoveryAction::None
        }
    }
}

/// Execute recovery action
pub async fn execute_recovery_action(action: RecoveryAction) -> Result<(), &'static str> {
    match action {
        RecoveryAction::None => {
            log_debug("No recovery action required");
            Ok(())
        }
        RecoveryAction::RestartComponent(component) => {
            log_info(&format!("Restarting component: {}", component));
            Timer::after(Duration::from_millis(1000)).await;
            log_info(&format!("Component {} restarted", component));
            Ok(())
        }
        RecoveryAction::SwitchToBackup(component) => {
            log_warning(&format!("Switching to backup for: {}", component));
            Timer::after(Duration::from_millis(500)).await;
            log_info(&format!("Switched to backup for {}", component));
            Ok(())
        }
        RecoveryAction::PowerCycle(component) => {
            log_warning(&format!("Power cycling: {}", component));
            Timer::after(Duration::from_millis(2000)).await;
            log_info(&format!("Power cycle complete for {}", component));
            Ok(())
        }
        RecoveryAction::SafeMode => {
            log_critical("Entering safe mode");
            unsafe {
                if let Some(handler) = ERROR_HANDLER.as_mut() {
                    handler.system_health.is_safe_mode = true;
                }
            }
            // Safe mode would typically disable non-essential systems
            Ok(())
        }
        RecoveryAction::EmergencyShutdown => {
            log_critical("Executing emergency shutdown");
            // This would trigger hardware shutdown procedures
            Err("Emergency shutdown initiated")
        }
    }
}

/// Periodic health check task
pub async fn health_check_task() {
    loop {
        unsafe {
            if let Some(handler) = ERROR_HANDLER.as_mut() {
                handler.update_health();
            }
        }

        Timer::after(Duration::from_secs(60)).await; // Every minute
    }
}

/// Format helper functions (since we can't use std::format! in no-std)
fn format(args: &str) -> String<256> {
    String::try_from(args).unwrap_or_else(|_| String::from("Format error"))
}

/// Get formatted system status
pub fn get_system_status() -> String<512> {
    let health = get_system_health();

    let mut status = String::new();
    let _ = status.push_str("System Status: ");

    if health.overall_health >= 90 {
        let _ = status.push_str("EXCELLENT");
    } else if health.overall_health >= 70 {
        let _ = status.push_str("GOOD");
    } else if health.overall_health >= 50 {
        let _ = status.push_str("DEGRADED");
    } else {
        let _ = status.push_str("CRITICAL");
    }

    status
}
