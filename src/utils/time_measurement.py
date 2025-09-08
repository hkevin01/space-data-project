"""
Precision timing utilities for space communication systems.

This module provides high-precision timing measurements essential for
real-time space communication requirements with microsecond accuracy.

Author: Space Data Communication Team
Version: 1.0.0
NASA-STD-REQ: SWE-REQ-008, SWE-REQ-012
"""

import time
import threading
from contextlib import contextmanager
from dataclasses import dataclass
from enum import Enum
from typing import Dict, List, Optional, Union
import statistics


class TimeUnit(Enum):
    """Supported time units for measurements."""
    NANOSECONDS = 1e-9
    MICROSECONDS = 1e-6
    MILLISECONDS = 1e-3
    SECONDS = 1.0


@dataclass
class TimeMeasurement:
    """Container for timing measurement data."""
    duration: float
    unit: TimeUnit
    start_time: float
    end_time: float
    description: Optional[str] = None
    
    def to_unit(self, target_unit: TimeUnit) -> float:
        """Convert measurement to specified time unit."""
        return self.duration * (self.unit.value / target_unit.value)
    
    def __str__(self) -> str:
        return f"{self.duration:.6f} {self.unit.name.lower()}"


class PrecisionTimer:
    """
    High-precision timer for space communication timing requirements.
    
    Provides microsecond-level accuracy for measuring operation durations,
    with support for nested timing, statistics collection, and performance
    monitoring.
    """
    
    def __init__(self, default_unit: TimeUnit = TimeUnit.MICROSECONDS):
        """
        Initialize precision timer.
        
        Args:
            default_unit: Default time unit for measurements
        """
        self.default_unit = default_unit
        self._measurements: Dict[str, List[TimeMeasurement]] = {}
        self._active_timers: Dict[str, float] = {}
        self._lock = threading.RLock()
        
        # Use the most precise timer available
        self._timer_func = time.perf_counter_ns if hasattr(time, 'perf_counter_ns') else time.perf_counter
        self._timer_resolution = 1e-9 if hasattr(time, 'perf_counter_ns') else 1e-6
    
    def start_measurement(self, name: Optional[str] = None) -> str:
        """
        Start a timing measurement.
        
        Args:
            name: Optional name for the measurement
            
        Returns:
            str: Measurement identifier
        """
        if name is None:
            name = f"measurement_{id(self)}_{len(self._active_timers)}"
        
        start_time = self._get_precise_time()
        
        with self._lock:
            self._active_timers[name] = start_time
        
        return name
    
    def end_measurement(
        self, 
        measurement_id: str, 
        unit: Optional[TimeUnit] = None,
        description: Optional[str] = None
    ) -> float:
        """
        End a timing measurement and return duration.
        
        Args:
            measurement_id: Measurement identifier from start_measurement
            unit: Time unit for result (uses default if None)
            description: Optional description for the measurement
            
        Returns:
            float: Duration in specified time unit
            
        Raises:
            ValueError: If measurement_id is not found
        """
        end_time = self._get_precise_time()
        
        with self._lock:
            if measurement_id not in self._active_timers:
                raise ValueError(f"No active measurement found for ID: {measurement_id}")
            
            start_time = self._active_timers.pop(measurement_id)
        
        # Calculate duration in seconds
        duration_seconds = (end_time - start_time) * self._timer_resolution
        
        # Convert to requested unit
        if unit is None:
            unit = self.default_unit
        
        duration = duration_seconds / unit.value
        
        # Store measurement
        measurement = TimeMeasurement(
            duration=duration,
            unit=unit,
            start_time=start_time * self._timer_resolution,
            end_time=end_time * self._timer_resolution,
            description=description
        )
        
        with self._lock:
            if measurement_id not in self._measurements:
                self._measurements[measurement_id] = []
            self._measurements[measurement_id].append(measurement)
        
        return duration
    
    @contextmanager
    def measure(
        self, 
        name: str, 
        unit: Optional[TimeUnit] = None,
        description: Optional[str] = None
    ):
        """
        Context manager for timing code blocks.
        
        Args:
            name: Name for the measurement
            unit: Time unit for result
            description: Optional description
            
        Example:
            with timer.measure("critical_operation", TimeUnit.MICROSECONDS):
                # code to time
                pass
        """
        measurement_id = self.start_measurement(name)
        try:
            yield measurement_id
        finally:
            self.end_measurement(measurement_id, unit, description)
    
    def get_statistics(self, measurement_name: str) -> Dict[str, float]:
        """
        Get statistical summary for a measurement.
        
        Args:
            measurement_name: Name of measurement to analyze
            
        Returns:
            Dict containing min, max, mean, median, std_dev
        """
        with self._lock:
            if measurement_name not in self._measurements:
                return {}
            
            measurements = self._measurements[measurement_name]
            if not measurements:
                return {}
            
            durations = [m.duration for m in measurements]
            
            return {
                'count': len(durations),
                'min': min(durations),
                'max': max(durations),
                'mean': statistics.mean(durations),
                'median': statistics.median(durations),
                'std_dev': statistics.stdev(durations) if len(durations) > 1 else 0.0,
                'unit': measurements[0].unit.name
            }
    
    def get_recent_measurements(
        self, 
        measurement_name: str, 
        count: int = 10
    ) -> List[TimeMeasurement]:
        """Get the most recent measurements for a given name."""
        with self._lock:
            if measurement_name not in self._measurements:
                return []
            return self._measurements[measurement_name][-count:]
    
    def clear_measurements(self, measurement_name: Optional[str] = None):
        """
        Clear stored measurements.
        
        Args:
            measurement_name: Specific measurement to clear, or None for all
        """
        with self._lock:
            if measurement_name is None:
                self._measurements.clear()
            elif measurement_name in self._measurements:
                del self._measurements[measurement_name]
    
    def _get_precise_time(self) -> float:
        """Get the most precise time measurement available."""
        return self._timer_func()
    
    def get_timer_resolution(self) -> float:
        """Get the timer resolution in seconds."""
        return self._timer_resolution


class PerformanceMonitor:
    """
    Monitor performance of space communication operations.
    
    Tracks timing, memory usage, and other performance metrics
    for critical space communication functions.
    """
    
    def __init__(self):
        self.timer = PrecisionTimer()
        self._counters: Dict[str, int] = {}
        self._lock = threading.RLock()
    
    def time_operation(self, operation_name: str):
        """Decorator for timing function execution."""
        def decorator(func):
            def wrapper(*args, **kwargs):
                with self.timer.measure(f"{operation_name}_{func.__name__}"):
                    result = func(*args, **kwargs)
                self.increment_counter(f"{operation_name}_calls")
                return result
            return wrapper
        return decorator
    
    def increment_counter(self, counter_name: str, amount: int = 1):
        """Increment a performance counter."""
        with self._lock:
            self._counters[counter_name] = self._counters.get(counter_name, 0) + amount
    
    def get_counter(self, counter_name: str) -> int:
        """Get current value of a counter."""
        with self._lock:
            return self._counters.get(counter_name, 0)
    
    def get_performance_summary(self) -> Dict[str, Dict[str, Union[int, float, str]]]:
        """Get comprehensive performance summary."""
        summary = {
            'counters': dict(self._counters),
            'timing_stats': {}
        }
        
        for measurement_name in self.timer._measurements:
            stats = self.timer.get_statistics(measurement_name)
            if stats:
                summary['timing_stats'][measurement_name] = stats
        
        return summary


# Global performance monitor instance
performance_monitor = PerformanceMonitor()


def benchmark_function(operation_name: str):
    """
    Decorator for benchmarking function performance.
    
    Args:
        operation_name: Name to use for the benchmark
        
    Example:
        @benchmark_function("message_processing")
        def process_message(msg):
            # function code
            pass
    """
    return performance_monitor.time_operation(operation_name)


def time_critical_operation(func):
    """
    Decorator for timing critical operations with automatic logging.
    
    Logs warning if operation takes longer than expected thresholds.
    """
    def wrapper(*args, **kwargs):
        start_time = time.perf_counter()
        try:
            result = func(*args, **kwargs)
            return result
        finally:
            duration = time.perf_counter() - start_time
            
            # Log warnings for slow operations
            if duration > 0.1:  # 100ms threshold
                import logging
                logger = logging.getLogger(__name__)
                logger.warning(
                    f"Slow operation detected: {func.__name__} took {duration*1000:.2f}ms"
                )
            
            performance_monitor.timer.end_measurement(
                performance_monitor.timer.start_measurement(func.__name__),
                TimeUnit.MILLISECONDS
            )
    
    return wrapper


__all__ = [
    'TimeUnit',
    'TimeMeasurement', 
    'PrecisionTimer',
    'PerformanceMonitor',
    'performance_monitor',
    'benchmark_function',
    'time_critical_operation'
]
