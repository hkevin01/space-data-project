//! Priority-based messaging system for space communications
//!
//! This module implements a priority queue system designed for real-time
//! space communication systems where message prioritization is critical
//! for mission success.
//!
//! # Requirements Traceability
//! - REQ-FN-001: Priority Classification (MessagePriority enum)
//! - REQ-FN-009: Message Queue Management (PriorityQueue implementation)
//! - REQ-FN-010: Real-Time Constraints (timing constraints in max_latency_ms)

use core::cmp::Ordering;
use heapless::binary_heap::{BinaryHeap, Max};
use serde::{Deserialize, Serialize};

use crate::error::{MemoryErrorType, Result, SpaceCommError};
use crate::types::{BandType, ComponentId, MessageId};

/// Message priority levels following NASA mission-critical classification
/// REQ-FN-001: Priority Classification - Five-tier priority system
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum MessagePriority {
    /// Lowest priority - routine housekeeping data
    /// Processing frequency: ~10 Hz
    /// REQ-FN-006: Low Priority Commands
    Low = 1,

    /// Medium priority - normal telemetry and data
    /// Processing frequency: ~100 Hz
    /// REQ-FN-005: Medium Priority Commands
    Medium = 2,

    /// High priority - important system status
    /// Processing frequency: ~500 Hz
    /// REQ-FN-004: High Priority Commands
    High = 3,

    /// Critical priority - emergency commands and alerts
    /// Processing frequency: ~1000 Hz, latency <1ms
    /// REQ-FN-003: Critical Command Set
    Critical = 4,

    /// Emergency priority - life-safety and mission-critical
    /// Processing frequency: immediate, latency <0.5ms
    /// REQ-FN-002: Emergency Command Set
    Emergency = 5,
}

impl MessagePriority {
    /// Get the maximum processing frequency for this priority level in Hz
    pub const fn max_frequency_hz(&self) -> u32 {
        match self {
            MessagePriority::Low => 10,
            MessagePriority::Medium => 100,
            MessagePriority::High => 500,
            MessagePriority::Critical => 1000,
            MessagePriority::Emergency => 2000,
        }
    }

    /// Get the maximum acceptable latency in milliseconds
    /// REQ-FN-010: Real-Time Constraints - Processing latency requirements
    pub const fn max_latency_ms(&self) -> u32 {
        match self {
            MessagePriority::Low => 10000,   // 10 seconds - REQ-FN-006
            MessagePriority::Medium => 1000, // 1 second - REQ-FN-005
            MessagePriority::High => 100,    // 100 ms - REQ-FN-004
            MessagePriority::Critical => 10, // 10 ms - REQ-FN-003
            MessagePriority::Emergency => 1, // 1 ms - REQ-FN-002
        }
    }

    /// Check if this priority requires real-time processing
    pub const fn is_real_time(&self) -> bool {
        matches!(self, MessagePriority::Critical | MessagePriority::Emergency)
    }
}

/// Core message structure for space communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique message identifier
    pub id: MessageId,

    /// Message priority level
    pub priority: MessagePriority,

    /// Source component that generated the message
    pub source: ComponentId,

    /// Destination component for the message
    pub destination: ComponentId,

    /// Timestamp when message was created (nanoseconds since epoch)
    pub timestamp: u64,

    /// Message payload data
    pub payload: MessagePayload,

    /// Preferred communication band
    pub preferred_band: BandType,

    /// Message time-to-live in seconds (0 = no expiration)
    pub ttl_seconds: u32,

    /// Retry count for failed transmissions
    pub retry_count: u8,

    /// Maximum number of retry attempts
    pub max_retries: u8,
}

/// Message payload types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagePayload {
    /// Telemetry data payload
    Telemetry {
        /// Telemetry data
        data: heapless::Vec<u8, 1024>,
        /// Data format identifier
        format: u16,
    },

    /// Command payload
    Command {
        /// Command identifier
        command_id: u32,
        /// Command parameters
        parameters: heapless::Vec<u8, 256>,
    },

    /// Status update payload
    Status {
        /// Status code
        status_code: u16,
        /// Status message
        message: heapless::String<128>,
    },

    /// Raw binary data
    Raw {
        /// Binary data
        data: heapless::Vec<u8, 2048>,
    },

    /// Emergency alert payload
    Emergency {
        /// Alert level (0-255, higher = more critical)
        alert_level: u8,
        /// Alert description
        description: heapless::String<256>,
        /// Associated data
        data: heapless::Vec<u8, 512>,
    },
}

impl MessagePayload {
    /// Get the size of the payload in bytes
    pub fn size(&self) -> usize {
        match self {
            MessagePayload::Telemetry { data, .. } => data.len(),
            MessagePayload::Command { parameters, .. } => parameters.len(),
            MessagePayload::Status { message, .. } => message.len(),
            MessagePayload::Raw { data } => data.len(),
            MessagePayload::Emergency {
                description, data, ..
            } => description.len() + data.len(),
        }
    }

    /// Check if payload is empty
    pub fn is_empty(&self) -> bool {
        self.size() == 0
    }
}

/// Priority queue message wrapper for heap ordering
#[derive(Debug, Clone)]
pub struct PriorityMessage {
    /// The actual message
    pub message: Message,
    /// Sequence number for FIFO ordering within same priority
    pub sequence: u64,
}

impl PartialEq for PriorityMessage {
    fn eq(&self, other: &Self) -> bool {
        self.message.priority == other.message.priority && self.sequence == other.sequence
    }
}

impl Eq for PriorityMessage {}

impl PartialOrd for PriorityMessage {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PriorityMessage {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority first, then older messages first (lower sequence number)
        match self.message.priority.cmp(&other.message.priority) {
            Ordering::Equal => other.sequence.cmp(&self.sequence), // Reverse for FIFO
            other => other,
        }
    }
}

/// Fixed-size priority queue for embedded systems
///
/// This queue maintains messages in priority order with FIFO semantics
/// within each priority level.
pub struct PriorityQueue<const N: usize> {
    /// Binary heap for priority ordering
    heap: BinaryHeap<PriorityMessage, Max, N>,
    /// Sequence counter for FIFO ordering
    sequence_counter: u64,
}

impl<const N: usize> PriorityQueue<N> {
    /// Create a new priority queue
    pub const fn new() -> Self {
        Self {
            heap: BinaryHeap::new(),
            sequence_counter: 0,
        }
    }

    /// Add a message to the queue
    ///
    /// Returns an error if the queue is full.
    pub fn push(&mut self, message: Message) -> Result<()> {
        let priority_message = PriorityMessage {
            message,
            sequence: self.sequence_counter,
        };

        self.sequence_counter = self.sequence_counter.wrapping_add(1);

        self.heap
            .push(priority_message)
            .map_err(|_| SpaceCommError::memory_error(MemoryErrorType::BufferOverflow, Some(N)))
    }

    /// Remove and return the highest priority message
    pub fn pop(&mut self) -> Option<Message> {
        self.heap.pop().map(|pm| pm.message)
    }

    /// Peek at the highest priority message without removing it
    pub fn peek(&self) -> Option<&Message> {
        self.heap.peek().map(|pm| &pm.message)
    }

    /// Get the number of messages in the queue
    pub fn len(&self) -> usize {
        self.heap.len()
    }

    /// Check if the queue is empty
    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    /// Check if the queue is full
    pub fn is_full(&self) -> bool {
        self.heap.len() >= N
    }

    /// Get the current capacity
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Remove expired messages based on TTL
    ///
    /// This method should be called periodically to clean up expired messages.
    pub fn remove_expired(&mut self, current_time_seconds: u64) {
        // Note: BinaryHeap doesn't support efficient removal of arbitrary elements
        // For a production system, consider using a more sophisticated data structure
        // or implementing a custom heap with removal capability

        // For now, we'll collect non-expired messages and rebuild the heap
        let mut temp_messages = heapless::Vec::<PriorityMessage, N>::new();

        while let Some(priority_message) = self.heap.pop() {
            let message_age = current_time_seconds.saturating_sub(
                priority_message.message.timestamp / 1_000_000_000, // Convert ns to seconds
            );

            if priority_message.message.ttl_seconds == 0
                || message_age < priority_message.message.ttl_seconds.into()
            {
                // Message is not expired, keep it
                if temp_messages.push(priority_message).is_err() {
                    // If we can't store it, we have to drop it (should not happen in normal operation)
                    break;
                }
            }
            // Expired messages are simply dropped
        }

        // Rebuild the heap with non-expired messages
        for priority_message in temp_messages {
            let _ = self.heap.push(priority_message); // Safe because we started with these messages
        }
    }

    /// Get statistics about the queue contents
    pub fn statistics(&self) -> QueueStatistics {
        let mut stats = QueueStatistics::default();

        // Count messages by priority
        // Note: This is inefficient as it requires cloning the heap
        // In a production system, maintain separate counters
        let temp_heap = self.heap.clone();
        for priority_message in temp_heap.into_vec() {
            match priority_message.message.priority {
                MessagePriority::Low => stats.low_priority += 1,
                MessagePriority::Medium => stats.medium_priority += 1,
                MessagePriority::High => stats.high_priority += 1,
                MessagePriority::Critical => stats.critical_priority += 1,
                MessagePriority::Emergency => stats.emergency_priority += 1,
            }
        }

        stats.total = self.len();
        stats.capacity = N;

        stats
    }
}

impl<const N: usize> Default for PriorityQueue<N> {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about queue contents
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QueueStatistics {
    /// Total number of messages
    pub total: usize,
    /// Queue capacity
    pub capacity: usize,
    /// Number of low priority messages
    pub low_priority: usize,
    /// Number of medium priority messages
    pub medium_priority: usize,
    /// Number of high priority messages
    pub high_priority: usize,
    /// Number of critical priority messages
    pub critical_priority: usize,
    /// Number of emergency priority messages
    pub emergency_priority: usize,
}

impl QueueStatistics {
    /// Calculate queue utilization as a percentage
    pub fn utilization_percent(&self) -> f32 {
        if self.capacity == 0 {
            0.0
        } else {
            (self.total as f32 / self.capacity as f32) * 100.0
        }
    }

    /// Check if queue is approaching capacity
    pub fn is_near_capacity(&self, threshold_percent: f32) -> bool {
        self.utilization_percent() >= threshold_percent
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ComponentId, MessageId};

    fn create_test_message(priority: MessagePriority, id: u64) -> Message {
        Message {
            id: MessageId::new(id),
            priority,
            source: ComponentId::new(1),
            destination: ComponentId::new(2),
            timestamp: 0,
            payload: MessagePayload::Raw {
                data: heapless::Vec::new(),
            },
            preferred_band: BandType::SBand,
            ttl_seconds: 0,
            retry_count: 0,
            max_retries: 3,
        }
    }

    #[test]
    fn test_priority_ordering() {
        assert!(MessagePriority::Emergency > MessagePriority::Critical);
        assert!(MessagePriority::Critical > MessagePriority::High);
        assert!(MessagePriority::High > MessagePriority::Medium);
        assert!(MessagePriority::Medium > MessagePriority::Low);
    }

    #[test]
    fn test_priority_queue() {
        let mut queue: PriorityQueue<10> = PriorityQueue::new();

        // Add messages in random order
        queue
            .push(create_test_message(MessagePriority::Low, 1))
            .unwrap();
        queue
            .push(create_test_message(MessagePriority::Emergency, 2))
            .unwrap();
        queue
            .push(create_test_message(MessagePriority::Medium, 3))
            .unwrap();

        // Should pop in priority order
        assert_eq!(queue.pop().unwrap().priority, MessagePriority::Emergency);
        assert_eq!(queue.pop().unwrap().priority, MessagePriority::Medium);
        assert_eq!(queue.pop().unwrap().priority, MessagePriority::Low);
    }

    #[test]
    fn test_fifo_within_priority() {
        let mut queue: PriorityQueue<10> = PriorityQueue::new();

        // Add multiple messages with same priority
        queue
            .push(create_test_message(MessagePriority::High, 1))
            .unwrap();
        queue
            .push(create_test_message(MessagePriority::High, 2))
            .unwrap();
        queue
            .push(create_test_message(MessagePriority::High, 3))
            .unwrap();

        // Should pop in FIFO order (first added, first out)
        assert_eq!(queue.pop().unwrap().id.value(), 1);
        assert_eq!(queue.pop().unwrap().id.value(), 2);
        assert_eq!(queue.pop().unwrap().id.value(), 3);
    }

    #[test]
    fn test_queue_capacity() {
        let mut queue: PriorityQueue<2> = PriorityQueue::new();

        assert!(queue
            .push(create_test_message(MessagePriority::Low, 1))
            .is_ok());
        assert!(queue
            .push(create_test_message(MessagePriority::Low, 2))
            .is_ok());

        // Third message should fail
        assert!(queue
            .push(create_test_message(MessagePriority::Low, 3))
            .is_err());
    }

    #[test]
    fn test_queue_statistics() {
        let mut queue: PriorityQueue<10> = PriorityQueue::new();

        queue
            .push(create_test_message(MessagePriority::Low, 1))
            .unwrap();
        queue
            .push(create_test_message(MessagePriority::High, 2))
            .unwrap();
        queue
            .push(create_test_message(MessagePriority::Emergency, 3))
            .unwrap();

        let stats = queue.statistics();
        assert_eq!(stats.total, 3);
        assert_eq!(stats.low_priority, 1);
        assert_eq!(stats.high_priority, 1);
        assert_eq!(stats.emergency_priority, 1);
        assert_eq!(stats.utilization_percent(), 30.0);
    }
}
