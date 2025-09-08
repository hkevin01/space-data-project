"""
Memory management utilities for space communication systems.

This module provides robust memory management with leak detection,
boundary checking, and graceful handling of memory constraints
in resource-limited space environments.

Author: Space Data Communication Team
Version: 1.0.0
NASA-STD-REQ: SWE-REQ-009, SWE-REQ-013
"""

import gc
import logging
import threading
import tracemalloc
import weakref
from collections import defaultdict
from dataclasses import dataclass
from typing import Any, Dict, List, Optional, Set
import psutil
import sys

logger = logging.getLogger(__name__)


@dataclass
class MemoryAllocation:
    """Represents a memory allocation with metadata."""
    identifier: str
    size_bytes: int
    timestamp: float
    description: Optional[str] = None
    stack_trace: Optional[List[str]] = None


class MemoryManager:
    """
    Advanced memory manager for space communication systems.

    Features:
    - Memory allocation tracking and limits
    - Leak detection and prevention
    - Graceful degradation under memory pressure
    - Automatic garbage collection tuning
    - Memory usage statistics and monitoring
    """

    def __init__(
        self,
        memory_limit_bytes: int,
        enable_leak_detection: bool = True,
        gc_threshold_percent: float = 80.0
    ):
        """
        Initialize memory manager.

        Args:
            memory_limit_bytes: Maximum memory usage limit
            enable_leak_detection: Whether to track allocations for leak detection
            gc_threshold_percent: Memory usage percent to trigger aggressive GC
        """
        self.memory_limit_bytes = memory_limit_bytes
        self.enable_leak_detection = enable_leak_detection
        self.gc_threshold_percent = gc_threshold_percent

        # Allocation tracking
        self._allocations: Dict[str, MemoryAllocation] = {}
        self._total_allocated = 0
        self._allocation_lock = threading.RLock()

        # Weak reference tracking for leak detection
        self._tracked_objects: weakref.WeakSet = weakref.WeakSet()

        # Statistics
        self._allocation_history: List[int] = []
        self._gc_triggers = 0
        self._memory_warnings = 0

        # Start tracemalloc if leak detection is enabled
        if self.enable_leak_detection and not tracemalloc.is_tracing():
            tracemalloc.start(10)  # Keep 10 frames

        logger.info(f"Initialized MemoryManager with {memory_limit_bytes:,} byte limit")

    def allocate(self, size_bytes: int, identifier: str, description: str = None) -> bool:
        """
        Allocate memory with tracking.

        Args:
            size_bytes: Number of bytes to allocate
            identifier: Unique identifier for this allocation
            description: Optional description for debugging

        Returns:
            bool: True if allocation succeeded, False if rejected
        """
        with self._allocation_lock:
            # Check if allocation would exceed limits
            if not self.can_allocate(size_bytes):
                logger.warning(
                    f"Memory allocation rejected: {size_bytes:,} bytes "
                    f"would exceed limit (current: {self._total_allocated:,}, "
                    f"limit: {self.memory_limit_bytes:,})"
                )
                return False

            # Create allocation record
            allocation = MemoryAllocation(
                identifier=identifier,
                size_bytes=size_bytes,
                timestamp=self._get_current_time(),
                description=description
            )

            # Add stack trace if leak detection is enabled
            if self.enable_leak_detection:
                allocation.stack_trace = self._get_stack_trace()

            # Record allocation
            self._allocations[identifier] = allocation
            self._total_allocated += size_bytes
            self._allocation_history.append(self._total_allocated)

            # Check if we need aggressive garbage collection
            current_usage_percent = (self._total_allocated / self.memory_limit_bytes) * 100
            if current_usage_percent > self.gc_threshold_percent:
                self._trigger_aggressive_gc()

            logger.debug(
                f"Allocated {size_bytes:,} bytes for {identifier} "
                f"(total: {self._total_allocated:,}/{self.memory_limit_bytes:,})"
            )

            return True

    def deallocate(self, identifier: str) -> bool:
        """
        Deallocate memory by identifier.

        Args:
            identifier: Identifier of allocation to free

        Returns:
            bool: True if deallocation succeeded
        """
        with self._allocation_lock:
            if identifier not in self._allocations:
                logger.warning(f"Attempted to deallocate unknown identifier: {identifier}")
                return False

            allocation = self._allocations.pop(identifier)
            self._total_allocated -= allocation.size_bytes

            logger.debug(
                f"Deallocated {allocation.size_bytes:,} bytes for {identifier} "
                f"(total: {self._total_allocated:,}/{self.memory_limit_bytes:,})"
            )

            return True

    def can_allocate(self, size_bytes: int) -> bool:
        """
        Check if allocation of given size is possible.

        Args:
            size_bytes: Size to check

        Returns:
            bool: True if allocation would be within limits
        """
        return (self._total_allocated + size_bytes) <= self.memory_limit_bytes

    def get_usage_stats(self) -> Dict[str, Any]:
        """Get comprehensive memory usage statistics."""
        with self._allocation_lock:
            # Get system memory info
            process = psutil.Process()
            process_memory = process.memory_info()

            # Calculate percentages
            allocated_percent = (self._total_allocated / self.memory_limit_bytes) * 100

            # Top allocations by size
            top_allocations = sorted(
                self._allocations.items(),
                key=lambda x: x[1].size_bytes,
                reverse=True
            )[:10]

            return {
                'total_allocated_bytes': self._total_allocated,
                'memory_limit_bytes': self.memory_limit_bytes,
                'allocated_percent': allocated_percent,
                'allocation_count': len(self._allocations),
                'system_memory_rss_bytes': process_memory.rss,
                'system_memory_vms_bytes': process_memory.vms,
                'gc_triggers': self._gc_triggers,
                'memory_warnings': self._memory_warnings,
                'top_allocations': [
                    {
                        'identifier': identifier,
                        'size_bytes': allocation.size_bytes,
                        'description': allocation.description
                    }
                    for identifier, allocation in top_allocations
                ],
                'available_bytes': self.memory_limit_bytes - self._total_allocated
            }

    def detect_leaks(self) -> List[Dict[str, Any]]:
        """
        Detect potential memory leaks.

        Returns:
            List of potential leak information
        """
        if not self.enable_leak_detection:
            logger.warning("Leak detection not enabled")
            return []

        potential_leaks = []
        current_time = self._get_current_time()

        with self._allocation_lock:
            # Look for long-lived allocations
            for identifier, allocation in self._allocations.items():
                age_seconds = current_time - allocation.timestamp

                # Flag allocations older than 5 minutes as potential leaks
                if age_seconds > 300:
                    potential_leaks.append({
                        'identifier': identifier,
                        'size_bytes': allocation.size_bytes,
                        'age_seconds': age_seconds,
                        'description': allocation.description,
                        'stack_trace': allocation.stack_trace
                    })

        if potential_leaks:
            logger.warning(f"Detected {len(potential_leaks)} potential memory leaks")

        return potential_leaks

    def force_cleanup(self) -> int:
        """
        Force cleanup of tracked allocations and run garbage collection.

        Returns:
            int: Number of bytes freed
        """
        with self._allocation_lock:
            initial_allocated = self._total_allocated

            # Clear all tracked allocations (this doesn't free actual memory,
            # just stops tracking it)
            self._allocations.clear()
            freed_tracked = self._total_allocated
            self._total_allocated = 0

            # Force garbage collection
            self._trigger_aggressive_gc()

            logger.info(f"Force cleanup freed {freed_tracked:,} bytes from tracking")
            return freed_tracked

    def cleanup(self):
        """Clean up memory manager resources."""
        self.force_cleanup()

        if tracemalloc.is_tracing():
            tracemalloc.stop()

        logger.info("MemoryManager cleanup completed")

    def _trigger_aggressive_gc(self):
        """Trigger aggressive garbage collection."""
        self._gc_triggers += 1

        # Run garbage collection multiple times
        for generation in range(3):
            collected = gc.collect()
            logger.debug(f"GC generation {generation}: collected {collected} objects")

        # Compact memory if possible (Python 3.10+)
        if hasattr(gc, 'freeze'):
            gc.freeze()

        logger.info("Triggered aggressive garbage collection")

    def _get_stack_trace(self) -> List[str]:
        """Get current stack trace for leak detection."""
        try:
            import traceback
            return traceback.format_stack()
        except Exception:
            return []

    def _get_current_time(self) -> float:
        """Get current time for timestamp."""
        import time
        return time.time()


class BoundaryValidator:
    """
    Validates boundary conditions for memory and other constraints.

    Prevents buffer overflows, validates input ranges, and ensures
    system stability under edge conditions.
    """

    @staticmethod
    def validate_positive_integer(
        value: int,
        name: str,
        min_value: int = 1,
        max_value: Optional[int] = None
    ):
        """
        Validate that value is a positive integer within bounds.

        Args:
            value: Value to validate
            name: Name of parameter for error messages
            min_value: Minimum allowed value
            max_value: Maximum allowed value (None for no limit)

        Raises:
            ValueError: If validation fails
        """
        if not isinstance(value, int):
            raise ValueError(f"{name} must be an integer, got {type(value)}")

        if value < min_value:
            raise ValueError(f"{name} must be >= {min_value}, got {value}")

        if max_value is not None and value > max_value:
            raise ValueError(f"{name} must be <= {max_value}, got {value}")

    @staticmethod
    def validate_memory_size(size_bytes: int, max_size: Optional[int] = None):
        """
        Validate memory size parameters.

        Args:
            size_bytes: Size in bytes to validate
            max_size: Maximum allowed size

        Raises:
            ValueError: If size is invalid
            MemoryError: If size exceeds system limits
        """
        BoundaryValidator.validate_positive_integer(size_bytes, "size_bytes", 0)

        if max_size is not None and size_bytes > max_size:
            raise MemoryError(f"Requested size {size_bytes:,} exceeds limit {max_size:,}")

        # Check against available system memory
        available_memory = psutil.virtual_memory().available
        if size_bytes > available_memory:
            raise MemoryError(
                f"Requested size {size_bytes:,} exceeds available memory {available_memory:,}"
            )

    @staticmethod
    def validate_string_length(
        text: str,
        name: str,
        max_length: int = 1_000_000
    ):
        """
        Validate string length to prevent excessive memory usage.

        Args:
            text: String to validate
            name: Parameter name for error messages
            max_length: Maximum allowed length

        Raises:
            ValueError: If string is too long
        """
        if not isinstance(text, str):
            raise ValueError(f"{name} must be a string, got {type(text)}")

        if len(text) > max_length:
            raise ValueError(
                f"{name} length {len(text):,} exceeds maximum {max_length:,}"
            )

    @staticmethod
    def validate_buffer_bounds(
        buffer_size: int,
        access_offset: int,
        access_length: int
    ):
        """
        Validate buffer access bounds to prevent overflows.

        Args:
            buffer_size: Total buffer size
            access_offset: Starting offset for access
            access_length: Length of access

        Raises:
            ValueError: If access would exceed buffer bounds
        """
        if access_offset < 0:
            raise ValueError(f"Access offset {access_offset} cannot be negative")

        if access_length < 0:
            raise ValueError(f"Access length {access_length} cannot be negative")

        if access_offset + access_length > buffer_size:
            raise ValueError(
                f"Access beyond buffer bounds: offset {access_offset} + "
                f"length {access_length} > buffer size {buffer_size}"
            )

    @staticmethod
    def validate_enum_value(value: Any, enum_class: type, name: str):
        """
        Validate that value is a member of the specified enum.

        Args:
            value: Value to validate
            enum_class: Enum class to check against
            name: Parameter name for error messages

        Raises:
            ValueError: If value is not a valid enum member
        """
        if not isinstance(value, enum_class):
            valid_values = [member.name for member in enum_class]
            raise ValueError(
                f"{name} must be one of {valid_values}, got {value}"
            )


class MemoryPool:
    """
    Memory pool for efficient allocation of fixed-size objects.

    Reduces garbage collection pressure by reusing allocated memory
    for frequently created/destroyed objects.
    """

    def __init__(self, object_size: int, pool_size: int = 1000):
        """
        Initialize memory pool.

        Args:
            object_size: Size of objects in bytes
            pool_size: Maximum number of objects to pool
        """
        self.object_size = object_size
        self.pool_size = pool_size
        self._pool: List[bytes] = []
        self._lock = threading.Lock()

        # Pre-allocate initial pool
        for _ in range(min(100, pool_size)):
            self._pool.append(bytearray(object_size))

    def acquire(self) -> bytes:
        """Acquire an object from the pool."""
        with self._lock:
            if self._pool:
                return self._pool.pop()
            else:
                # Pool exhausted, create new object
                return bytearray(self.object_size)

    def release(self, obj: bytes):
        """Return an object to the pool."""
        if len(obj) != self.object_size:
            return  # Don't pool objects of wrong size

        with self._lock:
            if len(self._pool) < self.pool_size:
                # Reset object and return to pool
                if isinstance(obj, bytearray):
                    obj[:] = b'\x00' * len(obj)
                self._pool.append(obj)


__all__ = [
    'MemoryManager',
    'MemoryAllocation',
    'BoundaryValidator',
    'MemoryPool'
]
