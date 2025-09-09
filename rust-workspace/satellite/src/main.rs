//! Satellite Communication System
//!
//! This is the main entry point for the embedded satellite communication system.
//! It implements a real-time system using Embassy async runtime for deterministic
//! task scheduling and communication management.
//!
//! # Architecture
//! - Embassy async runtime for task scheduling
//! - Priority-based message handling
//! - Multi-band communication support (K, Ka, S, X, UHF bands)
//! - Hardware abstraction layer for RF transceivers
//! - Fault tolerance with watchdog timers
//! - CCSDS-compliant packet processing
//!
//! # Requirements Traceability
//! - REQ-FN-010: Real-Time Constraints (Embassy async runtime with task timing)
//! - REQ-NF-002: Memory Constraints (heapless collections, static allocation)
//! - REQ-NF-005: Cross-Platform Support (ARM Cortex-M target)
//! - REQ-SF-002: Watchdog Protection (watchdog timer implementation)

#![no_std]
#![no_main]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

// External crate imports
use cortex_m;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{Channel, Receiver, Sender};
use heapless::String;

// Internal module imports
mod communication;
mod hardware;
mod error_handling;
mod command;
mod watchdog;
mod hardware;
mod error_handling;

// Shared library imports
use space_comms_shared::{
    messaging::{Message, MessagePriority, PriorityQueue},
    telemetry::{TelemetryData, TelemetryPacket},
    types::{ComponentId, BandType, HealthStatus},
    ccsds::{SpacePacket, PacketType},
    Result, SpaceCommError,
};

/// Maximum number of messages in priority queue (embedded constraint)
const MAX_QUEUE_SIZE: usize = 32;

/// Maximum number of telemetry measurements per packet
const MAX_TELEMETRY_MEASUREMENTS: usize = 16;

/// System heartbeat interval in milliseconds
const HEARTBEAT_INTERVAL_MS: u64 = 1000;

/// Critical message processing interval in milliseconds
const CRITICAL_PROCESSING_INTERVAL_MS: u64 = 1;

/// Telemetry transmission interval in milliseconds
const TELEMETRY_INTERVAL_MS: u64 = 100;

/// Communication channels for inter-task messaging
type MessageChannel = Channel<CriticalSectionRawMutex, Message, 16>;
type TelemetryChannel = Channel<CriticalSectionRawMutex, TelemetryPacket, 8>;
type CommandChannel = Channel<CriticalSectionRawMutex, SpacePacket, 8>;

/// Global channels for task communication
static MESSAGE_QUEUE_CHANNEL: MessageChannel = Channel::new();
static TELEMETRY_CHANNEL: TelemetryChannel = Channel::new();
static COMMAND_CHANNEL: CommandChannel = Channel::new();

/// System health monitor
static mut SYSTEM_HEALTH: HealthStatus = HealthStatus::Unknown;

/// Main entry point for the satellite system
/// REQ-FN-010: Real-Time Constraints - Embassy async runtime for deterministic scheduling
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Initialize error handling system
    error_handling::initialize();
    error_handling::log_info("Satellite system starting up");

    // Initialize system health
    unsafe {
        SYSTEM_HEALTH = HealthStatus::Good;
    }

    // Initialize hardware transceivers
    match hardware::initialize_transceivers().await {
        Ok(_) => error_handling::log_info("Hardware transceivers initialized successfully"),
        Err(e) => {
            error_handling::log_critical("Failed to initialize hardware transceivers");
            // Continue with degraded operation
        }
    }

    // Initialize communication system
    match communication::initialize().await {
        Ok(_) => error_handling::log_info("Communication system initialized successfully"),
        Err(e) => {
            error_handling::log_error("Failed to initialize communication system");
        }
    }

    // Initialize watchdog timer
    // REQ-SF-002: Watchdog Protection - Hardware and software watchdog timers
    watchdog::initialize();

    // Spawn high-priority tasks
    // REQ-FN-010: Real-Time Constraints - Task spawning with priority-based scheduling
    spawner.spawn(critical_message_processor()).unwrap();  // Emergency/Critical processing
    spawner.spawn(telemetry_collector()).unwrap();         // Real-time telemetry
    spawner.spawn(command_processor()).unwrap();           // Command execution

    // Spawn medium-priority tasks
    spawner.spawn(communication_manager()).unwrap();       // RF communication management
    spawner.spawn(health_monitor()).unwrap();              // System health monitoring
    spawner.spawn(error_handling::health_check_task()).unwrap(); // Error detection

    // Spawn low-priority tasks
    spawner.spawn(housekeeping_task()).unwrap();           // Routine maintenance
    spawner.spawn(system_heartbeat()).unwrap();            // System heartbeat

    // Main loop - should never exit
    loop {
        Timer::after(Duration::from_secs(1)).await;
        watchdog::reset();
    }
}

/// Critical message processor - highest priority task (1000Hz)
///
/// Processes emergency and critical priority messages with minimal latency.
/// This task has the highest priority and preempts all other tasks.
/// REQ-FN-002: Emergency Command Set - <1ms processing time
/// REQ-FN-003: Critical Command Set - <10ms processing time
/// REQ-FN-010: Real-Time Constraints - Priority-based processing
#[embassy_executor::task]
async fn critical_message_processor() {
    let mut queue: PriorityQueue<MAX_QUEUE_SIZE> = PriorityQueue::new();
    let receiver = MESSAGE_QUEUE_CHANNEL.receiver();

    loop {
        // Check for new messages
        if let Ok(message) = receiver.try_receive() {
            if message.priority >= MessagePriority::Critical {
                if let Err(e) = process_critical_message(&message).await {
                    error_handling::log_error("Critical message processing failed", &e);
                }
            } else {
                // Queue non-critical messages for later processing
                if let Err(e) = queue.push(message) {
                    error_handling::log_error("Message queue overflow", &e);
                }
            }
        }

        // Process queued messages if no critical messages pending
        if let Some(message) = queue.pop() {
            if let Err(e) = process_message(&message).await {
                error_handling::log_error("Message processing failed", &e);
            }
        }

        // Critical timing requirement: process at 1000Hz
        Timer::after(Duration::from_millis(CRITICAL_PROCESSING_INTERVAL_MS)).await;
        watchdog::reset();
    }
}

/// Process critical priority messages immediately
async fn process_critical_message(message: &Message) -> Result<()> {
    match &message.payload {
        space_comms_shared::messaging::MessagePayload::Emergency { alert_level, description, .. } => {
            // Handle emergency alerts
            emergency_alert_handler(*alert_level, description).await?;
        }
        space_comms_shared::messaging::MessagePayload::Command { command_id, parameters } => {
            // Execute critical commands immediately
            command::execute_critical_command(*command_id, parameters).await?;
        }
        _ => {
            // Other message types shouldn't be critical, but handle gracefully
            process_message(message).await?;
        }
    }
    Ok(())
}

/// Process regular priority messages
async fn process_message(message: &Message) -> Result<()> {
    match message.priority {
        MessagePriority::Emergency | MessagePriority::Critical => {
            process_critical_message(message).await
        }
        MessagePriority::High => {
            communication::send_high_priority(message).await
        }
        MessagePriority::Medium => {
            communication::send_medium_priority(message).await
        }
        MessagePriority::Low => {
            communication::send_low_priority(message).await
        }
    }
}

/// Telemetry collection task (100Hz)
///
/// Collects system telemetry data and packages it for transmission.
#[embassy_executor::task]
async fn telemetry_collector() {
    let sender = TELEMETRY_CHANNEL.sender();
    let mut sequence_counter: u32 = 0;

    loop {
        // Collect telemetry from various subsystems
        let telemetry = collect_system_telemetry().await;

        // Create telemetry packet
        let packet = TelemetryPacket::new(
            sequence_counter,
            telemetry,
            BandType::SBand, // Default to S-Band for telemetry
        );

        // Send to communication manager
        if let Err(_) = sender.try_send(packet) {
            error_handling::log_warning("Telemetry channel full, dropping packet");
        }

        sequence_counter = sequence_counter.wrapping_add(1);

        Timer::after(Duration::from_millis(TELEMETRY_INTERVAL_MS)).await;
    }
}

/// Collect telemetry data from various satellite subsystems
async fn collect_telemetry_data() -> TelemetryData {
    let mut measurements = Vec::<Measurement, MAX_TELEMETRY_MEASUREMENTS>::new();

    // Temperature measurements
    for sensor_id in 0..4 {
        if let Ok(reading) = hardware::read_temperature_sensor(sensor_id).await {
            let measurement = space_comms_shared::telemetry::Measurement {
                measurement_id: 0x0001 + sensor_id,
                value: space_comms_shared::telemetry::MeasurementValue::Float(reading.value),
                unit: reading.units,
                quality: space_comms_shared::telemetry::MeasurementQuality::Good,
            };
            let _ = measurements.push(measurement);
        }
    }

    // Voltage measurements
    for sensor_id in 0..4 {
        if let Ok(reading) = hardware::read_voltage_sensor(sensor_id).await {
            let measurement = space_comms_shared::telemetry::Measurement {
                measurement_id: 0x0010 + sensor_id,
                value: space_comms_shared::telemetry::MeasurementValue::Float(reading.value),
                unit: reading.units,
                quality: space_comms_shared::telemetry::MeasurementQuality::Good,
            };
            let _ = measurements.push(measurement);
        }
    }

    // Current measurements
    for sensor_id in 0..4 {
        if let Ok(reading) = hardware::read_current_sensor(sensor_id).await {
            let measurement = space_comms_shared::telemetry::Measurement {
                measurement_id: 0x0020 + sensor_id,
                value: space_comms_shared::telemetry::MeasurementValue::Float(reading.value),
                unit: reading.units,
                quality: space_comms_shared::telemetry::MeasurementQuality::Good,
            };
            let _ = measurements.push(measurement);
        }
    }

    TelemetryData {
        source: ComponentId::new(0x0001), // Satellite system ID
        timestamp: get_system_time_ns(),
        measurements,
        health_status: get_system_health(),
    }
}

/// Command processor task
///
/// Processes incoming commands from ground stations.
#[embassy_executor::task]
async fn command_processor() {
    let receiver = COMMAND_CHANNEL.receiver();

    loop {
        if let Ok(packet) = receiver.receive().await {
            if let Err(e) = command::process_command_packet(&packet).await {
                error_handling::log_error("Command processing failed", &e);
            }
        }
    }
}

/// Communication manager task
///
/// Handles communication across different frequency bands.
#[embassy_executor::task]
async fn communication_manager() {
    let telemetry_receiver = TELEMETRY_CHANNEL.receiver();

    loop {
        // Process telemetry transmission
        if let Ok(packet) = telemetry_receiver.try_receive() {
            if let Err(e) = communication::transmit_telemetry(&packet).await {
                error_handling::log_error("Telemetry transmission failed", &e);
            }
        }

        // Listen for incoming commands
        if let Ok(command_packet) = communication::receive_command().await {
            let sender = COMMAND_CHANNEL.sender();
            if let Err(_) = sender.try_send(command_packet) {
                error_handling::log_warning("Command channel full, dropping command");
            }
        }

        Timer::after(Duration::from_millis(10)).await;
    }
}

/// System health monitor task
///
/// Monitors system health and updates global health status.
#[embassy_executor::task]
async fn health_monitor() {
    loop {
        let health = assess_system_health().await;

        unsafe {
            SYSTEM_HEALTH = health;
        }

        // If health is critical, trigger emergency procedures
        if health == HealthStatus::Critical {
            emergency_procedures().await;
        }

        Timer::after(Duration::from_secs(5)).await;
    }
}

/// Housekeeping task - lowest priority
///
/// Performs routine maintenance and cleanup operations.
#[embassy_executor::task]
async fn housekeeping_task() {
    loop {
        // Perform routine maintenance
        maintenance_operations().await;

        // Clean up expired messages
        cleanup_expired_data().await;

        // Update system statistics
        update_system_statistics().await;

        Timer::after(Duration::from_secs(60)).await;
    }
}

/// System heartbeat task
///
/// Provides regular system heartbeat indication.
#[embassy_executor::task]
async fn system_heartbeat() {
    loop {
        // Toggle heartbeat LED or send heartbeat signal
        hardware::toggle_heartbeat_led().await;

        Timer::after(Duration::from_millis(HEARTBEAT_INTERVAL_MS)).await;
    }
}

/// Emergency alert handler
async fn emergency_alert_handler(alert_level: u8, description: &str) -> Result<()> {
    // Log emergency alert
    error_handling::log_emergency("Emergency alert", alert_level, description);

    // Activate emergency protocols based on alert level
    match alert_level {
        200..=255 => {
            // Critical emergency - activate safe mode
            activate_safe_mode().await?;
        }
        100..=199 => {
            // High emergency - reduce functionality
            reduce_functionality().await?;
        }
        50..=99 => {
            // Medium emergency - increase monitoring
            increase_monitoring().await?;
        }
        _ => {
            // Low emergency - log and continue
            error_handling::log_warning("Low-level emergency alert received");
        }
    }

    Ok(())
}

/// Assess overall system health
async fn assess_system_health() -> HealthStatus {
    let mut health_score = 100u8;

    // Check temperature
    if let Ok(temp) = hardware::read_temperature().await {
        if temp > 80.0 {
            health_score = health_score.saturating_sub(30);
        } else if temp > 60.0 {
            health_score = health_score.saturating_sub(10);
        }
    }

    // Check battery voltage
    if let Ok(voltage) = hardware::read_battery_voltage().await {
        if voltage < 10.0 {
            health_score = health_score.saturating_sub(40);
        } else if voltage < 12.0 {
            health_score = health_score.saturating_sub(20);
        }
    }

    // Check communication status
    if !communication::is_communication_healthy().await {
        health_score = health_score.saturating_sub(25);
    }

    // Convert score to health status
    match health_score {
        90..=100 => HealthStatus::Excellent,
        70..=89 => HealthStatus::Good,
        50..=69 => HealthStatus::Fair,
        30..=49 => HealthStatus::Poor,
        0..=29 => HealthStatus::Critical,
    }
}

/// Get current system health status
fn get_system_health() -> HealthStatus {
    unsafe { SYSTEM_HEALTH }
}

/// Get system time in nanoseconds (placeholder implementation)
fn get_system_time_ns() -> u64 {
    // TODO: Implement proper time synchronization
    // For now, use a simple counter
    static mut TIME_COUNTER: u64 = 0;
    unsafe {
        TIME_COUNTER += 1_000_000; // Increment by 1ms
        TIME_COUNTER
    }
}

/// Activate safe mode
async fn activate_safe_mode() -> Result<()> {
    error_handling::log_warning("Activating safe mode");

    // Disable non-essential systems
    hardware::disable_non_essential_systems().await?;

    // Switch to minimal power mode
    hardware::set_minimal_power_mode().await?;

    // Use only S-band for communication
    communication::set_emergency_mode(BandType::SBand).await?;

    Ok(())
}

/// Reduce system functionality during emergency
async fn reduce_functionality() -> Result<()> {
    error_handling::log_warning("Reducing system functionality");

    // Reduce telemetry frequency
    // Switch to backup communication band
    communication::switch_to_backup_band().await?;

    Ok(())
}

/// Increase system monitoring
async fn increase_monitoring() -> Result<()> {
    error_handling::log_info("Increasing system monitoring");

    // Increase telemetry frequency
    // Add additional health checks

    Ok(())
}

/// Perform routine maintenance operations
async fn maintenance_operations() {
    // Clean up temporary data
    // Defragment memory if needed
    // Update system counters
}

/// Clean up expired data
async fn cleanup_expired_data() {
    // Remove expired log entries
    // Clean up old telemetry data
}

/// Update system statistics
async fn update_system_statistics() {
    // Update performance counters
    // Calculate system utilization
}

/// Emergency procedures for critical health status
async fn emergency_procedures() {
    error_handling::log_emergency_simple("Critical system health detected");

    // Switch to emergency communication mode
    let _ = communication::set_emergency_mode(BandType::UhfBand).await;

    // Activate safe mode
    let _ = activate_safe_mode().await;
}
