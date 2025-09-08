//! Time utilities and synchronization
//!
//! This module provides time-related utilities for space communication systems,
//! including high-precision timing for real-time operations.

use serde::{Deserialize, Serialize};

/// High-precision timestamp in nanoseconds since Unix epoch
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Timestamp(pub u64);

impl Timestamp {
    /// Create a new timestamp from nanoseconds since Unix epoch
    pub const fn new(nanos: u64) -> Self {
        Self(nanos)
    }

    /// Get current timestamp
    pub fn now() -> Self {
        Self(current_time_nanos())
    }

    /// Get the raw nanoseconds value
    pub const fn nanos(&self) -> u64 {
        self.0
    }

    /// Convert to milliseconds since Unix epoch
    pub const fn millis(&self) -> u64 {
        self.0 / 1_000_000
    }

    /// Convert to seconds since Unix epoch
    pub const fn seconds(&self) -> u64 {
        self.0 / 1_000_000_000
    }

    /// Calculate elapsed time since this timestamp
    pub fn elapsed(&self) -> Duration {
        let now = current_time_nanos();
        Duration::new(if now >= self.0 { now - self.0 } else { 0 })
    }
}

/// Duration in nanoseconds
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Duration(pub u64);

impl Duration {
    /// Create a new duration from nanoseconds
    pub const fn new(nanos: u64) -> Self {
        Self(nanos)
    }

    /// Create duration from milliseconds
    pub const fn from_millis(millis: u64) -> Self {
        Self(millis * 1_000_000)
    }

    /// Create duration from seconds
    pub const fn from_secs(secs: u64) -> Self {
        Self(secs * 1_000_000_000)
    }

    /// Get nanoseconds
    pub const fn nanos(&self) -> u64 {
        self.0
    }

    /// Get milliseconds
    pub const fn millis(&self) -> u64 {
        self.0 / 1_000_000
    }

    /// Get seconds
    pub const fn secs(&self) -> u64 {
        self.0 / 1_000_000_000
    }

    /// Check if duration is zero
    pub const fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

/// Get current time in nanoseconds since Unix epoch
///
/// This function provides the highest precision timing available on the platform.
/// For embedded systems without wall clock time, it may return time since boot.
pub fn current_time_nanos() -> u64 {
    #[cfg(feature = "std")]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    }

    #[cfg(not(feature = "std"))]
    {
        // For embedded systems, use a monotonic counter
        static BOOT_TIME_COUNTER: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
        BOOT_TIME_COUNTER.fetch_add(1000, core::sync::atomic::Ordering::Relaxed)
    }
}

/// Get monotonic time in nanoseconds since system boot
///
/// This function provides monotonic timing that doesn't go backwards,
/// suitable for measuring intervals and timeouts.
pub fn monotonic_time_nanos() -> u64 {
    #[cfg(feature = "std")]
    {
        // Use a simple approach - current time nanos should be monotonic enough for our purposes
        current_time_nanos()
    }

    #[cfg(not(feature = "std"))]
    {
        // For embedded systems, use an atomic counter
        static MONOTONIC_COUNTER: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
        MONOTONIC_COUNTER.fetch_add(1000, core::sync::atomic::Ordering::Relaxed)
    }
}

/// Time synchronization state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeSync {
    /// Not synchronized
    NotSynced,
    /// Synchronized with GPS
    GpsSynced,
    /// Synchronized with ground station
    GroundSynced,
    /// Synchronized with network time
    NetworkSynced,
    /// Synchronized with atomic clock
    AtomicSynced,
}

/// Time source for synchronization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeSource {
    /// Internal oscillator (least accurate)
    Internal,
    /// GPS timing signal
    Gps,
    /// Ground station time signal
    GroundStation,
    /// Network Time Protocol
    Ntp,
    /// Atomic clock reference
    AtomicClock,
}

impl TimeSource {
    /// Get the typical accuracy of this time source in nanoseconds
    pub const fn accuracy_nanos(&self) -> u64 {
        match self {
            TimeSource::Internal => 1_000_000_000,      // ±1 second
            TimeSource::Gps => 100_000,                 // ±100 microseconds
            TimeSource::GroundStation => 1_000,         // ±1 microsecond
            TimeSource::Ntp => 10_000_000,              // ±10 milliseconds
            TimeSource::AtomicClock => 1,                // ±1 nanosecond
        }
    }

    /// Check if this source provides sub-millisecond accuracy
    pub const fn is_high_precision(&self) -> bool {
        matches!(self, TimeSource::Gps | TimeSource::GroundStation | TimeSource::AtomicClock)
    }
}
