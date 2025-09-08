//! Ground Station Communication System
//!
//! Implements the ground station side of the satellite-ground communication link.
//! Handles command uplink, telemetry downlink, and mission operations.

use std::collections::HashMap;
use std::net::{UdpSocket, SocketAddr};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use space_comms_shared::{
    messaging::{Message, MessageId, MessagePriority, MessagePayload},
    telemetry::{TelemetryPacket, TelemetryData},
    ccsds::{SpacePacket, PacketType},
    types::BandType,
    Result, SpaceCommError,
};

/// Ground station configuration
#[derive(Debug, Clone)]
pub struct GroundStationConfig {
    /// Station identifier
    pub station_id: String,
    /// Station location (latitude, longitude, altitude)
    pub location: (f64, f64, f64),
    /// Supported communication bands
    pub supported_bands: Vec<BandType>,
    /// UDP listening port for telemetry
    pub telemetry_port: u16,
    /// UDP port for command transmission
    pub command_port: u16,
}

impl Default for GroundStationConfig {
    fn default() -> Self {
        Self {
            station_id: "GST-001".to_string(),
            location: (34.0522, -118.2437, 75.0), // Los Angeles coordinates
            supported_bands: vec![
                BandType::UhfBand,
                BandType::SBand,
                BandType::XBand,
                BandType::KBand,
                BandType::KaBand,
            ],
            telemetry_port: 8081,
            command_port: 8082,
        }
    }
}

/// Ground station state
pub struct GroundStation {
    /// Configuration
    config: GroundStationConfig,
    /// Telemetry socket
    telemetry_socket: UdpSocket,
    /// Command socket
    command_socket: UdpSocket,
    /// Received telemetry packets
    telemetry_history: Arc<Mutex<Vec<TelemetryPacket>>>,
    /// Command sequence number
    command_sequence: Arc<Mutex<u16>>,
    /// Connection status with satellite
    is_connected: Arc<Mutex<bool>>,
}

impl GroundStation {
    /// Create new ground station
    pub fn new(config: GroundStationConfig) -> Result<Self> {
        // Create UDP sockets
        let telemetry_socket = UdpSocket::bind(format!("127.0.0.1:{}", config.telemetry_port))
            .map_err(|e| SpaceCommError::communication_timeout(1000, &format!("Failed to bind telemetry socket: {}", e)))?;

        let command_socket = UdpSocket::bind(format!("127.0.0.1:{}", config.command_port))
            .map_err(|e| SpaceCommError::communication_timeout(1000, &format!("Failed to bind command socket: {}", e)))?;

        // Set socket timeouts
        telemetry_socket.set_read_timeout(Some(Duration::from_millis(100)))
            .map_err(|e| SpaceCommError::communication_timeout(100, &format!("Failed to set telemetry timeout: {}", e)))?;

        command_socket.set_read_timeout(Some(Duration::from_millis(100)))
            .map_err(|e| SpaceCommError::communication_timeout(100, &format!("Failed to set command timeout: {}", e)))?;

        Ok(Self {
            config,
            telemetry_socket,
            command_socket,
            telemetry_history: Arc::new(Mutex::new(Vec::new())),
            command_sequence: Arc::new(Mutex::new(0)),
            is_connected: Arc::new(Mutex::new(false)),
        })
    }

    /// Start ground station operations
    pub fn start(&self) -> Result<()> {
        println!("Starting Ground Station {}", self.config.station_id);
        println!("Location: {:.4}°N, {:.4}°W, {:.1}m",
                 self.config.location.0,
                 -self.config.location.1,
                 self.config.location.2);

        // Start telemetry receiver thread
        self.start_telemetry_receiver()?;

        // Start command processor thread
        self.start_command_processor()?;

        // Start monitoring thread
        self.start_monitoring()?;

        println!("Ground station operational");
        Ok(())
    }

    /// Start telemetry receiver thread
    fn start_telemetry_receiver(&self) -> Result<()> {
        let socket = self.telemetry_socket.try_clone()
            .map_err(|e| SpaceCommError::communication_timeout(1000, &format!("Failed to clone telemetry socket: {}", e)))?;

        let telemetry_history = Arc::clone(&self.telemetry_history);
        let is_connected = Arc::clone(&self.is_connected);

        thread::spawn(move || {
            let mut buffer = [0u8; 4096];

            loop {
                match socket.recv_from(&mut buffer) {
                    Ok((size, addr)) => {
                        println!("Received {} bytes from {}", size, addr);

                        // Update connection status
                        *is_connected.lock().unwrap() = true;

                        // Parse telemetry packet
                        match parse_telemetry_packet(&buffer[..size]) {
                            Ok(packet) => {
                                println!("Telemetry packet parsed successfully");
                                display_telemetry(&packet);

                                // Store in history
                                let mut history = telemetry_history.lock().unwrap();
                                history.push(packet);

                                // Keep only last 1000 packets
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
                        // Timeout is expected when no data is available
                        if e.kind() != std::io::ErrorKind::WouldBlock {
                            eprintln!("Telemetry receive error: {}", e);
                        }
                    }
                }

                thread::sleep(Duration::from_millis(10));
            }
        });

        Ok(())
    }

    /// Start command processor thread
    fn start_command_processor(&self) -> Result<()> {
        let socket = self.command_socket.try_clone()
            .map_err(|e| SpaceCommError::communication_timeout(1000, &format!("Failed to clone command socket: {}", e)))?;

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
                if now.duration_since(last_connection_check).unwrap_or_default() > Duration::from_secs(10) {
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
    pub fn send_command(&self, command: Command) -> Result<()> {
        let mut sequence = self.command_sequence.lock().unwrap();
        *sequence += 1;

        let message = Message {
            id: MessageId::new(*sequence as u32),
            priority: command.priority,
            payload: MessagePayload::Command {
                command_id: command.command_id,
                parameters: command.parameters.clone(),
            },
        };

        // Create CCSDS packet
        let packet = create_command_packet(&message)?;
        let packet_bytes = packet.to_bytes()?;

        // Send to satellite (simulated via UDP to localhost)
        let satellite_addr: SocketAddr = "127.0.0.1:8080".parse()
            .map_err(|e| SpaceCommError::communication_timeout(1000, &format!("Invalid satellite address: {}", e)))?;

        self.command_socket.send_to(&packet_bytes, satellite_addr)
            .map_err(|e| SpaceCommError::communication_timeout(1000, &format!("Failed to send command: {}", e)))?;

        println!("Command sent: ID={}, Priority={:?}", command.command_id, command.priority);
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

/// Command structure
#[derive(Debug, Clone)]
pub struct Command {
    /// Command identifier
    pub command_id: u32,
    /// Command priority
    pub priority: MessagePriority,
    /// Command parameters
    pub parameters: Vec<u8>,
}

impl Command {
    /// Create new command
    pub fn new(command_id: u32, priority: MessagePriority, parameters: Vec<u8>) -> Self {
        Self {
            command_id,
            priority,
            parameters,
        }
    }

    /// Create system status request
    pub fn system_status_request() -> Self {
        Self::new(0x1001, MessagePriority::Medium, vec![])
    }

    /// Create telemetry request
    pub fn telemetry_request() -> Self {
        Self::new(0x1002, MessagePriority::Low, vec![])
    }

    /// Create emergency stop command
    pub fn emergency_stop() -> Self {
        Self::new(0x9999, MessagePriority::Emergency, vec![0x01])
    }

    /// Create band switch command
    pub fn switch_band(band: BandType) -> Self {
        let band_id = match band {
            BandType::UhfBand => 0,
            BandType::SBand => 1,
            BandType::XBand => 2,
            BandType::KBand => 3,
            BandType::KaBand => 4,
        };
        Self::new(0x2001, MessagePriority::High, vec![band_id])
    }
}

/// Parse telemetry packet from bytes
fn parse_telemetry_packet(bytes: &[u8]) -> Result<TelemetryPacket> {
    if bytes.len() < 6 {
        return Err(SpaceCommError::invalid_packet("Packet too short", None));
    }

    // Parse CCSDS header
    let header = space_comms_shared::ccsds::SpacePacketHeader::from_bytes(&bytes[0..6])?;

    // For simulation, create a dummy telemetry packet
    let telemetry_data = TelemetryData {
        source: space_comms_shared::types::ComponentId::new(0x0001),
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64,
        measurements: Vec::new(),
        health_status: space_comms_shared::types::HealthStatus::Good,
    };

    Ok(TelemetryPacket {
        sequence: header.sequence_count as u32,
        band: BandType::SBand,
        data: telemetry_data,
    })
}

/// Create CCSDS command packet from message
fn create_command_packet(message: &Message) -> Result<SpacePacket> {
    let mut payload_data = Vec::new();

    if let MessagePayload::Command { command_id, parameters } = &message.payload {
        // Add command ID
        payload_data.extend_from_slice(&command_id.to_be_bytes());

        // Add parameters
        payload_data.extend_from_slice(parameters);
    }

    // Create CCSDS packet
    let apid = match message.priority {
        MessagePriority::Emergency => 0x001,
        MessagePriority::Critical => 0x002,
        MessagePriority::High => 0x003,
        MessagePriority::Medium => 0x004,
        MessagePriority::Low => 0x005,
    };

    SpacePacket::new(
        PacketType::Command,
        apid,
        message.id.value() as u16,
        &payload_data,
        None,
    )
}

/// Display telemetry information
fn display_telemetry(packet: &TelemetryPacket) {
    println!("=== Telemetry Packet ===");
    println!("Sequence: {}", packet.sequence);
    println!("Band: {:?}", packet.band);
    println!("Source: {:?}", packet.data.source);
    println!("Timestamp: {}", packet.data.timestamp);
    println!("Health: {:?}", packet.data.health_status);
    println!("Measurements: {}", packet.data.measurements.len());

    for (i, measurement) in packet.data.measurements.iter().enumerate() {
        println!("  [{}] ID: 0x{:04X}, Value: {:?}, Unit: {}, Quality: {:?}",
                 i,
                 measurement.measurement_id,
                 measurement.value,
                 measurement.unit,
                 measurement.quality);
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
                    if let Err(e) = self.ground_station.send_command(Command::system_status_request()) {
                        eprintln!("Failed to send status command: {}", e);
                    }
                }
                "telem" => {
                    if let Err(e) = self.ground_station.send_command(Command::telemetry_request()) {
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
