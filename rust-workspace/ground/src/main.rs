//! Ground Station Communication System
//!
//! Implements the ground station side of the satellite-ground communication link.
//! Handles command uplink, telemetry downlink, and mission operations.
//!
//! # Requirements Traceability
//! - REQ-IF-002: CCSDS Compliance (CCSDS packet creation and parsing)
//! - REQ-FN-007: Multi-Band Communication (frequency band selection support)
//! - REQ-PF-001: Command Response Time (command acknowledgment processing)
//! - REQ-FN-001: Priority Classification (command priority handling)
//!
//! # Architecture
//! The ground station operates as a UDP-based communication gateway that:
//! - Receives telemetry data from satellites via UDP sockets
//! - Sends commands to satellites with proper priority classification
//! - Maintains connection status monitoring and telemetry history
//! - Provides interactive mission control interface
//!
//! # Communication Protocol
//! - Uses CCSDS Space Packet Protocol for all communications
//! - Supports all five frequency bands (UHF, S, X, K, Ka)
//! - Implements priority-based command transmission
//! - Maintains telemetry packet history for analysis

use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use space_comms_shared::{
    ccsds::{PacketType, SpacePacket},
    messaging::{Message, MessageId, MessagePayload, MessagePriority},
    telemetry::{TelemetryData, TelemetryPacket},
    types::BandType,
    Result, SpaceCommError,
};

/// Ground station configuration
///
/// Contains all necessary parameters for ground station operation including
/// location, supported communication bands, and network configuration.
/// REQ-FN-007: Multi-Band Communication - Defines supported frequency bands
#[derive(Debug, Clone)]
pub struct GroundStationConfig {
    /// Station identifier for tracking and logging purposes
    pub station_id: String,

    /// Station location as (latitude, longitude, altitude in meters)
    /// Used for orbital mechanics calculations and link budget analysis
    pub location: (f64, f64, f64),

    /// Supported communication bands for satellite communication
    /// REQ-FN-007: Multi-Band Communication - Five frequency band support
    pub supported_bands: Vec<BandType>,

    /// UDP listening port for incoming telemetry data from satellites
    pub telemetry_port: u16,

    /// UDP port for outgoing command transmission to satellites
    pub command_port: u16,
}

impl Default for GroundStationConfig {
    /// Create default ground station configuration
    ///
    /// Provides sensible defaults for development and testing:
    /// - Los Angeles location for ground station positioning
    /// - All five frequency bands supported per REQ-FN-007
    /// - Standard UDP ports for telemetry and command channels
    fn default() -> Self {
        Self {
            // Standard ground station identifier format
            station_id: "GST-001".to_string(),

            // Los Angeles coordinates (lat, lon, altitude_m)
            // 34.0522°N, 118.2437°W, 75m elevation
            location: (34.0522, -118.2437, 75.0),

            // REQ-FN-007: Multi-Band Communication - Support all five frequency bands
            // Listed in order of increasing frequency and data rate capability
            supported_bands: vec![
                BandType::UhfBand, // 0.3-3 GHz: Most reliable, limited bandwidth
                BandType::SBand,   // 2-4 GHz: Reliable, all-weather communication
                BandType::XBand,   // 8-12 GHz: Balanced performance and reliability
                BandType::KBand,   // 20-30 GHz: High data rate, weather-sensitive
                BandType::KaBand,  // 26.5-40 GHz: Maximum data rate, atmospheric effects
            ],

            // Network configuration for ground station operations
            telemetry_port: 8081, // Incoming telemetry from satellites
            command_port: 8082,   // Outgoing commands to satellites
        }
    }
}

/// Ground station state and operational data
///
/// Maintains all runtime state for ground station operations including
/// network sockets, telemetry history, and connection status.
/// Thread-safe design using Arc<Mutex<T>> for shared state.
pub struct GroundStation {
    /// Ground station configuration parameters
    config: GroundStationConfig,

    /// UDP socket for receiving telemetry data from satellites
    /// REQ-PF-001: Command Response Time - Low-latency telemetry reception
    telemetry_socket: UdpSocket,

    /// UDP socket for sending commands to satellites
    /// REQ-FN-001: Priority Classification - Priority-based command transmission
    command_socket: UdpSocket,

    /// Thread-safe storage for received telemetry packets
    /// Maintains rolling history for analysis and monitoring
    telemetry_history: Arc<Mutex<Vec<TelemetryPacket>>>,

    /// Thread-safe command sequence number generator
    /// Ensures unique identification of each transmitted command
    command_sequence: Arc<Mutex<u16>>,

    /// Thread-safe connection status flag
    /// Tracks real-time connectivity with satellite systems
    is_connected: Arc<Mutex<bool>>,
}

impl GroundStation {
    /// Create new ground station instance
    ///
    /// Initializes UDP sockets, configures timeouts, and prepares thread-safe
    /// shared state for multi-threaded operation.
    ///
    /// # Arguments
    /// * `config` - Ground station configuration parameters
    ///
    /// # Returns
    /// * `Result<Self>` - Ground station instance or error
    ///
    /// # Requirements Traceability
    /// - REQ-PF-001: Command Response Time (socket timeout configuration)
    /// - REQ-NF-004: Fault Tolerance (error handling for socket creation)
    pub fn new(config: GroundStationConfig) -> Result<Self> {
        // Create UDP sockets for bi-directional communication
        // Bind to localhost for development/simulation environment
        let telemetry_socket = UdpSocket::bind(format!("127.0.0.1:{}", config.telemetry_port))
            .map_err(|e| {
                SpaceCommError::communication_timeout(
                    1000,
                    &format!("Failed to bind telemetry socket: {}", e),
                )
            })?;

        let command_socket = UdpSocket::bind(format!("127.0.0.1:{}", config.command_port))
            .map_err(|e| {
                SpaceCommError::communication_timeout(
                    1000,
                    &format!("Failed to bind command socket: {}", e),
                )
            })?;

        // REQ-PF-001: Command Response Time - Configure socket timeouts for responsiveness
        // 100ms timeout prevents blocking operations while maintaining responsiveness
        telemetry_socket
            .set_read_timeout(Some(Duration::from_millis(100)))
            .map_err(|e| {
                SpaceCommError::communication_timeout(
                    100,
                    &format!("Failed to set telemetry timeout: {}", e),
                )
            })?;

        command_socket
            .set_read_timeout(Some(Duration::from_millis(100)))
            .map_err(|e| {
                SpaceCommError::communication_timeout(
                    100,
                    &format!("Failed to set command timeout: {}", e),
                )
            })?;

        // Initialize thread-safe shared state using Arc<Mutex<T>> pattern
        // This enables safe concurrent access from multiple threads
        Ok(Self {
            config,
            telemetry_socket,
            command_socket,
            // Rolling telemetry history with automatic size management
            telemetry_history: Arc::new(Mutex::new(Vec::new())),
            // Monotonic command sequence for unique identification
            command_sequence: Arc::new(Mutex::new(0)),
            // Real-time connection status tracking
            is_connected: Arc::new(Mutex::new(false)),
        })
    }

    /// Start ground station operations
    ///
    /// Initializes all operational threads for telemetry reception, command processing,
    /// and system monitoring. This method spawns background threads and returns
    /// immediately, allowing the ground station to operate concurrently.
    ///
    /// # Returns
    /// * `Result<()>` - Success or initialization error
    ///
    /// # Requirements Traceability
    /// - REQ-NF-003: System Availability (robust multi-threaded architecture)
    /// - REQ-PF-001: Command Response Time (concurrent processing threads)
    pub fn start(&self) -> Result<()> {
        // Display ground station startup information
        println!("Starting Ground Station {}", self.config.station_id);
        println!(
            "Location: {:.4}°N, {:.4}°W, {:.1}m",
            self.config.location.0,
            -self.config.location.1, // Convert to West longitude
            self.config.location.2
        );

        // Start telemetry receiver thread for incoming satellite data
        // REQ-PF-001: Command Response Time - Dedicated thread for low-latency reception
        self.start_telemetry_receiver()?;

        // Start command processor thread for outgoing commands
        // REQ-FN-001: Priority Classification - Handles priority-based command processing
        self.start_command_processor()?;

        // Start monitoring thread for connection status and health checks
        // REQ-NF-003: System Availability - Continuous system health monitoring
        self.start_monitoring()?;

        println!("Ground station operational");
        Ok(())
    }

    /// Start telemetry receiver thread
    ///
    /// Spawns a dedicated thread for receiving and processing telemetry data
    /// from satellites. Implements continuous polling with timeout handling
    /// and automatic connection status updates.
    ///
    /// # Returns
    /// * `Result<()>` - Success or thread creation error
    ///
    /// # Requirements Traceability
    /// - REQ-IF-002: CCSDS Compliance (CCSDS packet parsing)
    /// - REQ-PF-001: Command Response Time (low-latency telemetry processing)
    /// - REQ-NF-003: System Availability (robust error handling)
    fn start_telemetry_receiver(&self) -> Result<()> {
        // Clone socket for thread ownership
        let socket = self.telemetry_socket.try_clone().map_err(|e| {
            SpaceCommError::communication_timeout(
                1000,
                &format!("Failed to clone telemetry socket: {}", e),
            )
        })?;

        // Clone Arc references for thread-safe access to shared state
        let telemetry_history = Arc::clone(&self.telemetry_history);
        let is_connected = Arc::clone(&self.is_connected);

        // Spawn dedicated telemetry processing thread
        thread::spawn(move || {
            // 4KB buffer for telemetry packets - sized for typical CCSDS packets
            let mut buffer = [0u8; 4096];

            // Continuous telemetry reception loop
            loop {
                match socket.recv_from(&mut buffer) {
                    Ok((size, addr)) => {
                        println!("Received {} bytes from {}", size, addr);

                        // REQ-NF-003: System Availability - Update connection status
                        *is_connected.lock().unwrap() = true;

                        // REQ-IF-002: CCSDS Compliance - Parse received telemetry packet
                        match parse_telemetry_packet(&buffer[..size]) {
                            Ok(packet) => {
                                println!("Telemetry packet parsed successfully");
                                display_telemetry(&packet);

                                // Store in thread-safe telemetry history
                                let mut history = telemetry_history.lock().unwrap();
                                history.push(packet);

                                // Maintain rolling history - keep only last 1000 packets
                                // Prevents unbounded memory growth during long operations
                                if history.len() > 1000 {
                                    history.remove(0);
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to parse telemetry packet: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        // REQ-NF-004: Fault Tolerance - Handle timeouts gracefully
                        // Timeout is expected when no data is available
                        if e.kind() != std::io::ErrorKind::WouldBlock {
                            eprintln!("Telemetry receive error: {}", e);
                        }
                    }
                }

                // Small delay to prevent excessive CPU usage while maintaining responsiveness
                thread::sleep(Duration::from_millis(10));
            }
        });

        Ok(())
    }

    /// Start command processor thread
    fn start_command_processor(&self) -> Result<()> {
        let socket = self.command_socket.try_clone().map_err(|e| {
            SpaceCommError::communication_timeout(
                1000,
                &format!("Failed to clone command socket: {}", e),
            )
        })?;

        thread::spawn(move || {
            // Simulate command processing
            loop {
                thread::sleep(Duration::from_secs(5));
                // Commands would be processed here
            }
        });

        Ok(())
    }

    /// Start monitoring thread
    fn start_monitoring(&self) -> Result<()> {
        let is_connected = Arc::clone(&self.is_connected);

        thread::spawn(move || {
            let mut last_connection_check = SystemTime::now();

            loop {
                let now = SystemTime::now();

                // Check connection status every 10 seconds
                if now
                    .duration_since(last_connection_check)
                    .unwrap_or_default()
                    > Duration::from_secs(10)
                {
                    let connected = *is_connected.lock().unwrap();

                    if connected {
                        println!("Satellite link: ACTIVE");
                        // Reset connection flag to detect future disconnections
                        *is_connected.lock().unwrap() = false;
                    } else {
                        println!("Satellite link: NO SIGNAL");
                    }

                    last_connection_check = now;
                }

                thread::sleep(Duration::from_millis(1000));
            }
        });

        Ok(())
    }

    /// Send command to satellite
    ///
    /// Constructs and transmits a command packet to the satellite using CCSDS
    /// protocol with proper priority handling and sequence numbering.
    ///
    /// # Arguments
    /// * `command` - Command structure containing ID, priority, and parameters
    ///
    /// # Returns
    /// * `Result<()>` - Success or transmission error
    ///
    /// # Requirements Traceability
    /// - REQ-FN-001: Priority Classification (command priority handling)
    /// - REQ-IF-002: CCSDS Compliance (CCSDS packet creation)
    /// - REQ-PF-001: Command Response Time (low-latency command transmission)
    pub fn send_command(&self, command: Command) -> Result<()> {
        // Generate unique command sequence number
        // REQ-FN-001: Priority Classification - Each command gets unique ID
        let mut sequence = self.command_sequence.lock().unwrap();
        *sequence += 1;

        // Create message structure with proper priority classification
        let message = Message {
            id: MessageId::new(*sequence as u32),
            priority: command.priority, // REQ-FN-001: Priority Classification
            payload: MessagePayload::Command {
                command_id: command.command_id,
                parameters: command.parameters.clone(),
            },
        };

        // REQ-IF-002: CCSDS Compliance - Create standard CCSDS packet
        let packet = create_command_packet(&message)?;
        let packet_bytes = packet.to_bytes()?;

        // Transmit to satellite via UDP (simulated space link)
        // In real implementation, this would interface with RF hardware
        let satellite_addr: SocketAddr = "127.0.0.1:8080".parse().map_err(|e| {
            SpaceCommError::communication_timeout(
                1000,
                &format!("Invalid satellite address: {}", e),
            )
        })?;

        // REQ-PF-001: Command Response Time - Direct socket transmission for low latency
        self.command_socket
            .send_to(&packet_bytes, satellite_addr)
            .map_err(|e| {
                SpaceCommError::communication_timeout(
                    1000,
                    &format!("Failed to send command: {}", e),
                )
            })?;

        println!(
            "Command sent: ID={}, Priority={:?}",
            command.command_id, command.priority
        );
        Ok(())
    }

    /// Get telemetry history
    pub fn get_telemetry_history(&self) -> Vec<TelemetryPacket> {
        self.telemetry_history.lock().unwrap().clone()
    }

    /// Get connection status
    pub fn is_connected_to_satellite(&self) -> bool {
        *self.is_connected.lock().unwrap()
    }
}

/// Command structure for satellite operations
///
/// Represents a command to be sent to the satellite with proper priority
/// classification and parameter encoding.
/// REQ-FN-001: Priority Classification - Command priority support
#[derive(Debug, Clone)]
pub struct Command {
    /// Unique command identifier for command type recognition
    pub command_id: u32,

    /// Command priority level for processing order
    /// REQ-FN-001: Priority Classification - Five-tier priority system
    pub priority: MessagePriority,

    /// Command-specific parameters as binary data
    /// Allows flexible parameter encoding for different command types
    pub parameters: Vec<u8>,
}

impl Command {
    /// Create new command with specified parameters
    ///
    /// # Arguments
    /// * `command_id` - Unique identifier for the command type
    /// * `priority` - Processing priority level (REQ-FN-001)
    /// * `parameters` - Binary encoded command parameters
    pub fn new(command_id: u32, priority: MessagePriority, parameters: Vec<u8>) -> Self {
        Self {
            command_id,
            priority,
            parameters,
        }
    }

    /// Create system status request command
    /// REQ-FN-005: Medium Priority Commands - Status and telemetry requests
    pub fn system_status_request() -> Self {
        Self::new(0x1001, MessagePriority::Medium, vec![])
    }

    /// Create telemetry request command
    /// REQ-FN-006: Low Priority Commands - Routine telemetry collection
    pub fn telemetry_request() -> Self {
        Self::new(0x1002, MessagePriority::Low, vec![])
    }

    /// Create emergency stop command
    /// REQ-FN-002: Emergency Command Set - Immediate termination of operations
    pub fn emergency_stop() -> Self {
        Self::new(0x9999, MessagePriority::Emergency, vec![0x01])
    }

    /// Create frequency band switch command
    /// REQ-FN-007: Multi-Band Communication - Dynamic band selection
    /// REQ-FN-004: High Priority Commands - Communication configuration
    pub fn switch_band(band: BandType) -> Self {
        // Map BandType to numeric identifier for transmission
        let band_id = match band {
            BandType::UhfBand => 0, // 0.3-3 GHz: Most reliable
            BandType::SBand => 1,   // 2-4 GHz: All-weather
            BandType::XBand => 2,   // 8-12 GHz: Balanced performance
            BandType::KBand => 3,   // 20-30 GHz: High data rate
            BandType::KaBand => 4,  // 26.5-40 GHz: Maximum data rate
        };
        Self::new(0x2001, MessagePriority::High, vec![band_id])
    }
}

/// Parse telemetry packet from received bytes
///
/// Processes incoming telemetry data according to CCSDS packet format
/// and creates structured telemetry packet for analysis.
///
/// # Arguments
/// * `bytes` - Raw packet bytes received from satellite
///
/// # Returns
/// * `Result<TelemetryPacket>` - Parsed telemetry packet or parsing error
///
/// # Requirements Traceability
/// - REQ-IF-002: CCSDS Compliance (CCSDS packet header parsing)
/// - REQ-NF-004: Fault Tolerance (robust packet validation)
fn parse_telemetry_packet(bytes: &[u8]) -> Result<TelemetryPacket> {
    // REQ-NF-004: Fault Tolerance - Validate minimum packet size
    if bytes.len() < 6 {
        return Err(SpaceCommError::invalid_packet("Packet too short", None));
    }

    // REQ-IF-002: CCSDS Compliance - Parse standard CCSDS header
    let header = space_comms_shared::ccsds::SpacePacketHeader::from_bytes(&bytes[0..6])?;

    // For simulation environment, create representative telemetry data
    // In real implementation, this would parse actual telemetry measurements
    let telemetry_data = TelemetryData {
        source: space_comms_shared::types::ComponentId::new(0x0001),
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64,
        measurements: Vec::new(),
        health_status: space_comms_shared::types::HealthStatus::Good,
    };

    // Create structured telemetry packet with parsed data
    Ok(TelemetryPacket {
        sequence: header.sequence_count as u32,
        band: BandType::SBand, // Default to S-Band for simulation
        data: telemetry_data,
    })
}

/// Create CCSDS command packet from message structure
///
/// Converts internal message format to standard CCSDS Space Packet format
/// for transmission to satellite systems.
///
/// # Arguments
/// * `message` - Internal message structure with command data
///
/// # Returns
/// * `Result<SpacePacket>` - CCSDS-compliant packet or creation error
///
/// # Requirements Traceability
/// - REQ-IF-002: CCSDS Compliance (Space Packet Protocol implementation)
/// - REQ-FN-001: Priority Classification (priority-based APID assignment)
fn create_command_packet(message: &Message) -> Result<SpacePacket> {
    let mut payload_data = Vec::new();

    // Extract command data from message payload
    if let MessagePayload::Command {
        command_id,
        parameters,
    } = &message.payload
    {
        // Encode command ID in big-endian format for network transmission
        payload_data.extend_from_slice(&command_id.to_be_bytes());

        // Append command parameters
        payload_data.extend_from_slice(parameters);
    }

    // REQ-FN-001: Priority Classification - Map message priority to CCSDS APID
    // Application Process Identifier (APID) indicates processing priority
    let apid = match message.priority {
        MessagePriority::Emergency => 0x001, // Highest priority APID
        MessagePriority::Critical => 0x002,  // Critical priority APID
        MessagePriority::High => 0x003,      // High priority APID
        MessagePriority::Medium => 0x004,    // Medium priority APID
        MessagePriority::Low => 0x005,       // Lowest priority APID
    };

    // REQ-IF-002: CCSDS Compliance - Create standard Space Packet
    SpacePacket::new(
        PacketType::Command,       // Command packet type
        apid,                      // Priority-based APID
        message.id.value() as u16, // Sequence count
        &payload_data,             // Command payload
        None,                      // No secondary header
    )
}

/// Display formatted telemetry information
///
/// Provides human-readable output of telemetry packet contents for
/// mission operations monitoring and debugging.
///
/// # Arguments
/// * `packet` - Telemetry packet to display
fn display_telemetry(packet: &TelemetryPacket) {
    println!("=== Telemetry Packet ===");
    println!("Sequence: {}", packet.sequence);
    println!("Band: {:?}", packet.band); // REQ-FN-007: Multi-Band Communication
    println!("Source: {:?}", packet.data.source);
    println!("Timestamp: {}", packet.data.timestamp);
    println!("Health: {:?}", packet.data.health_status);
    println!("Measurements: {}", packet.data.measurements.len());

    // Display individual measurements with detailed formatting
    for (i, measurement) in packet.data.measurements.iter().enumerate() {
        println!(
            "  [{}] ID: 0x{:04X}, Value: {:?}, Unit: {}, Quality: {:?}",
            i, measurement.measurement_id, measurement.value, measurement.unit, measurement.quality
        );
    }
    println!("========================");
}

/// Mission control interface
pub struct MissionControl {
    ground_station: GroundStation,
}

impl MissionControl {
    /// Create new mission control interface
    pub fn new(config: GroundStationConfig) -> Result<Self> {
        let ground_station = GroundStation::new(config)?;
        Ok(Self { ground_station })
    }

    /// Start mission operations
    pub fn start_operations(&self) -> Result<()> {
        self.ground_station.start()?;

        println!("Mission Control operational");

        // Start command loop
        self.command_loop();

        Ok(())
    }

    /// Interactive command loop
    fn command_loop(&self) {
        use std::io::{self, Write};

        println!("Mission Control Command Interface");
        println!("Available commands:");
        println!("  status   - Request system status");
        println!("  telem    - Request telemetry");
        println!("  band <n> - Switch to band (0=UHF, 1=S, 2=X, 3=K, 4=Ka)");
        println!("  stop     - Emergency stop");
        println!("  quit     - Exit mission control");

        loop {
            print!("MC> ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_err() {
                continue;
            }

            let parts: Vec<&str> = input.trim().split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            match parts[0] {
                "status" => {
                    if let Err(e) = self
                        .ground_station
                        .send_command(Command::system_status_request())
                    {
                        eprintln!("Failed to send status command: {}", e);
                    }
                }
                "telem" => {
                    if let Err(e) = self
                        .ground_station
                        .send_command(Command::telemetry_request())
                    {
                        eprintln!("Failed to send telemetry command: {}", e);
                    }
                }
                "band" => {
                    if parts.len() < 2 {
                        println!("Usage: band <0-4>");
                        continue;
                    }

                    let band = match parts[1] {
                        "0" => BandType::UhfBand,
                        "1" => BandType::SBand,
                        "2" => BandType::XBand,
                        "3" => BandType::KBand,
                        "4" => BandType::KaBand,
                        _ => {
                            println!("Invalid band number. Use 0-4.");
                            continue;
                        }
                    };

                    if let Err(e) = self.ground_station.send_command(Command::switch_band(band)) {
                        eprintln!("Failed to send band switch command: {}", e);
                    }
                }
                "stop" => {
                    if let Err(e) = self.ground_station.send_command(Command::emergency_stop()) {
                        eprintln!("Failed to send emergency stop: {}", e);
                    }
                }
                "quit" => {
                    println!("Mission Control shutting down");
                    break;
                }
                _ => {
                    println!("Unknown command: {}", parts[0]);
                }
            }
        }
    }
}

/// Example usage
fn main() -> Result<()> {
    // Create ground station configuration
    let config = GroundStationConfig::default();

    // Create mission control
    let mission_control = MissionControl::new(config)?;

    // Start operations
    mission_control.start_operations()?;

    Ok(())
}
