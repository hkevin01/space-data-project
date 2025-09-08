"""
Advanced error handling and fault tolerance for space communication systems.

This module provides comprehensive error handling, graceful degradation,
and system recovery mechanisms designed for mission-critical space applications.

Author: Space Data Communication Team
Version: 1.0.0
NASA-STD-REQ: SWE-REQ-010, SWE-REQ-016
"""

import asyncio
import logging
import threading
import time
import traceback
from collections import defaultdict, deque
from dataclasses import dataclass, field
from enum import Enum
from typing import Any, Callable, Dict, List, Optional, Set, Union
import sys
import json

logger = logging.getLogger(__name__)


class ErrorSeverity(Enum):
    """Error severity levels for space communication systems."""
    TRACE = 0      # Debug information
    INFO = 1       # Informational messages
    WARNING = 2    # Warning conditions
    ERROR = 3      # Error conditions
    CRITICAL = 4   # Critical system errors
    FATAL = 5      # System failure conditions


class ErrorCategory(Enum):
    """Categories of errors in space communication."""
    COMMUNICATION = "communication"
    HARDWARE = "hardware"
    SOFTWARE = "software"
    PROTOCOL = "protocol"
    SECURITY = "security"
    PERFORMANCE = "performance"
    RESOURCE = "resource"
    CONFIGURATION = "configuration"


@dataclass
class ErrorRecord:
    """Comprehensive error record for tracking and analysis."""
    timestamp: float
    severity: ErrorSeverity
    category: ErrorCategory
    error_code: str
    message: str
    exception: Optional[Exception] = None
    stack_trace: Optional[str] = None
    context: Dict[str, Any] = field(default_factory=dict)
    recovery_action: Optional[str] = None
    recovered: bool = False
    recovery_time: Optional[float] = None

    def to_dict(self) -> Dict[str, Any]:
        """Convert error record to dictionary for serialization."""
        return {
            'timestamp': self.timestamp,
            'severity': self.severity.name,
            'category': self.category.value,
            'error_code': self.error_code,
            'message': self.message,
            'exception_type': type(self.exception).__name__ if self.exception else None,
            'stack_trace': self.stack_trace,
            'context': self.context,
            'recovery_action': self.recovery_action,
            'recovered': self.recovered,
            'recovery_time': self.recovery_time
        }


class CriticalError(Exception):
    """Exception for critical system errors requiring immediate attention."""

    def __init__(self, message: str, error_code: str = None, context: Dict[str, Any] = None):
        super().__init__(message)
        self.error_code = error_code or "CRITICAL_ERROR"
        self.context = context or {}
        self.timestamp = time.time()


class RecoverableError(Exception):
    """Exception for errors that can potentially be recovered from."""

    def __init__(self, message: str, error_code: str = None, retry_possible: bool = True):
        super().__init__(message)
        self.error_code = error_code or "RECOVERABLE_ERROR"
        self.retry_possible = retry_possible
        self.timestamp = time.time()


class ErrorHandler:
    """
    Advanced error handler for space communication systems.

    Features:
    - Comprehensive error logging and tracking
    - Automatic recovery mechanisms
    - Graceful degradation strategies
    - Error pattern analysis
    - System health monitoring
    - Crash prevention and recovery
    """

    def __init__(
        self,
        max_error_history: int = 10000,
        critical_error_threshold: int = 5,
        recovery_timeout: float = 30.0
    ):
        """
        Initialize error handler.

        Args:
            max_error_history: Maximum number of errors to keep in history
            critical_error_threshold: Number of critical errors before emergency mode
            recovery_timeout: Maximum time to attempt recovery (seconds)
        """
        self.max_error_history = max_error_history
        self.critical_error_threshold = critical_error_threshold
        self.recovery_timeout = recovery_timeout

        # Error tracking
        self.error_history: deque = deque(maxlen=max_error_history)
        self.error_counts: Dict[str, int] = defaultdict(int)
        self.error_patterns: Dict[str, List[float]] = defaultdict(list)
        self._lock = threading.RLock()

        # Recovery mechanisms
        self.recovery_strategies: Dict[str, Callable] = {}
        self.recovery_attempts: Dict[str, int] = defaultdict(int)
        self.max_recovery_attempts = 3

        # System state
        self.emergency_mode = False
        self.system_healthy = True
        self.last_health_check = time.time()

        # Performance tracking
        self.error_handling_times: List[float] = []

        logger.info("Initialized ErrorHandler for space communication system")

    def handle_error(
        self,
        error: Union[Exception, str],
        severity: ErrorSeverity = ErrorSeverity.ERROR,
        category: ErrorCategory = ErrorCategory.SOFTWARE,
        context: Optional[Dict[str, Any]] = None,
        error_code: Optional[str] = None
    ) -> ErrorRecord:
        """
        Handle an error with comprehensive logging and recovery.

        Args:
            error: Exception or error message
            severity: Error severity level
            category: Error category
            context: Additional context information
            error_code: Unique error code

        Returns:
            ErrorRecord: Complete error record
        """
        start_time = time.perf_counter()

        try:
            # Create error record
            error_record = self._create_error_record(
                error, severity, category, context, error_code
            )

            # Log error with appropriate level
            self._log_error(error_record)

            # Store error in history
            with self._lock:
                self.error_history.append(error_record)
                self.error_counts[error_record.error_code] += 1
                self.error_patterns[error_record.error_code].append(error_record.timestamp)

            # Attempt recovery if appropriate
            if severity in [ErrorSeverity.ERROR, ErrorSeverity.CRITICAL, ErrorSeverity.FATAL]:
                self._attempt_recovery(error_record)

            # Check for emergency conditions
            if severity >= ErrorSeverity.CRITICAL:
                self._check_emergency_conditions()

            # Update system health
            self._update_system_health(error_record)

            # Track performance
            handling_time = time.perf_counter() - start_time
            self.error_handling_times.append(handling_time)
            if len(self.error_handling_times) > 1000:
                self.error_handling_times = self.error_handling_times[-500:]

            return error_record

        except Exception as e:
            # Error in error handler - log to stderr to avoid recursion
            sys.stderr.write(f"ERROR in ErrorHandler.handle_error: {e}\n")
            sys.stderr.flush()
            raise

    def _create_error_record(
        self,
        error: Union[Exception, str],
        severity: ErrorSeverity,
        category: ErrorCategory,
        context: Optional[Dict[str, Any]],
        error_code: Optional[str]
    ) -> ErrorRecord:
        """Create comprehensive error record."""
        timestamp = time.time()

        # Extract error information
        if isinstance(error, Exception):
            message = str(error)
            exception = error
            stack_trace = traceback.format_exc()
            if error_code is None:
                error_code = f"{category.value}_{type(error).__name__}"
        else:
            message = str(error)
            exception = None
            stack_trace = None
            if error_code is None:
                error_code = f"{category.value}_GENERIC"

        # Add system context
        enhanced_context = {
            'thread_id': threading.get_ident(),
            'process_id': threading.current_thread().ident,
            'system_time': timestamp,
            'emergency_mode': self.emergency_mode,
            **(context or {})
        }

        return ErrorRecord(
            timestamp=timestamp,
            severity=severity,
            category=category,
            error_code=error_code,
            message=message,
            exception=exception,
            stack_trace=stack_trace,
            context=enhanced_context
        )

    def _log_error(self, error_record: ErrorRecord):
        """Log error with appropriate level and formatting."""
        log_message = (
            f"[{error_record.category.value.upper()}] "
            f"{error_record.error_code}: {error_record.message}"
        )

        # Add context if available
        if error_record.context:
            context_str = ", ".join(f"{k}={v}" for k, v in error_record.context.items())
            log_message += f" | Context: {context_str}"

        # Log at appropriate level
        if error_record.severity == ErrorSeverity.TRACE:
            logger.debug(log_message)
        elif error_record.severity == ErrorSeverity.INFO:
            logger.info(log_message)
        elif error_record.severity == ErrorSeverity.WARNING:
            logger.warning(log_message)
        elif error_record.severity == ErrorSeverity.ERROR:
            logger.error(log_message)
        elif error_record.severity >= ErrorSeverity.CRITICAL:
            logger.critical(log_message)

            # Also log stack trace for critical errors
            if error_record.stack_trace:
                logger.critical(f"Stack trace: {error_record.stack_trace}")

    def _attempt_recovery(self, error_record: ErrorRecord) -> bool:
        """
        Attempt to recover from error using registered strategies.

        Args:
            error_record: Error to recover from

        Returns:
            bool: True if recovery was successful
        """
        error_code = error_record.error_code

        # Check if we've exceeded recovery attempts
        if self.recovery_attempts[error_code] >= self.max_recovery_attempts:
            logger.warning(f"Max recovery attempts exceeded for {error_code}")
            return False

        # Find applicable recovery strategy
        recovery_strategy = self._find_recovery_strategy(error_record)
        if not recovery_strategy:
            logger.debug(f"No recovery strategy found for {error_code}")
            return False

        # Attempt recovery
        recovery_start = time.time()
        self.recovery_attempts[error_code] += 1

        try:
            logger.info(f"Attempting recovery for {error_code} (attempt {self.recovery_attempts[error_code]})")

            # Execute recovery with timeout
            recovery_successful = asyncio.wait_for(
                recovery_strategy(error_record),
                timeout=self.recovery_timeout
            )

            if recovery_successful:
                recovery_time = time.time() - recovery_start
                error_record.recovered = True
                error_record.recovery_action = recovery_strategy.__name__
                error_record.recovery_time = recovery_time

                logger.info(f"Recovery successful for {error_code} in {recovery_time:.3f}s")

                # Reset recovery attempts counter on success
                self.recovery_attempts[error_code] = 0
                return True
            else:
                logger.warning(f"Recovery failed for {error_code}")
                return False

        except asyncio.TimeoutError:
            logger.error(f"Recovery timeout for {error_code} after {self.recovery_timeout}s")
            return False
        except Exception as recovery_error:
            logger.error(f"Recovery strategy failed for {error_code}: {recovery_error}")
            return False

    def _find_recovery_strategy(self, error_record: ErrorRecord) -> Optional[Callable]:
        """Find appropriate recovery strategy for error."""
        # Try exact error code match first
        if error_record.error_code in self.recovery_strategies:
            return self.recovery_strategies[error_record.error_code]

        # Try category-based match
        category_key = f"{error_record.category.value}_generic"
        if category_key in self.recovery_strategies:
            return self.recovery_strategies[category_key]

        # Try severity-based match
        severity_key = f"severity_{error_record.severity.name.lower()}"
        if severity_key in self.recovery_strategies:
            return self.recovery_strategies[severity_key]

        return None

    def register_recovery_strategy(
        self,
        error_pattern: str,
        recovery_function: Callable[[ErrorRecord], bool]
    ):
        """
        Register a recovery strategy for specific error patterns.

        Args:
            error_pattern: Error code or pattern to match
            recovery_function: Function to execute for recovery
        """
        self.recovery_strategies[error_pattern] = recovery_function
        logger.info(f"Registered recovery strategy for pattern: {error_pattern}")

    def _check_emergency_conditions(self):
        """Check if system should enter emergency mode."""
        with self._lock:
            # Count critical errors in last 5 minutes
            current_time = time.time()
            recent_critical = sum(
                1 for record in self.error_history
                if (record.severity >= ErrorSeverity.CRITICAL and
                    current_time - record.timestamp < 300)
            )

            if recent_critical >= self.critical_error_threshold and not self.emergency_mode:
                logger.critical("ENTERING EMERGENCY MODE due to critical error threshold")
                self.emergency_mode = True
                self._enter_emergency_mode()
            elif recent_critical < self.critical_error_threshold // 2 and self.emergency_mode:
                logger.info("Exiting emergency mode - critical errors reduced")
                self.emergency_mode = False

    def _enter_emergency_mode(self):
        """Enter emergency mode with reduced functionality."""
        try:
            # Disable non-essential features
            logger.critical("Emergency mode activated - reducing system functionality")

            # Clear error history to free memory
            self.error_history.clear()

            # Force garbage collection
            import gc
            gc.collect()

            # Reduce logging verbosity
            logging.getLogger().setLevel(logging.ERROR)

        except Exception as e:
            sys.stderr.write(f"Error entering emergency mode: {e}\n")

    def _update_system_health(self, error_record: ErrorRecord):
        """Update overall system health assessment."""
        current_time = time.time()

        # Consider system unhealthy if:
        # - In emergency mode
        # - Multiple critical errors in last minute
        # - High error rate

        if self.emergency_mode:
            self.system_healthy = False
        else:
            # Check error rate in last minute
            recent_errors = sum(
                1 for record in self.error_history
                if (current_time - record.timestamp < 60 and
                    record.severity >= ErrorSeverity.ERROR)
            )

            self.system_healthy = recent_errors < 10  # Max 10 errors per minute

        self.last_health_check = current_time

    def get_error_statistics(self) -> Dict[str, Any]:
        """Get comprehensive error statistics."""
        with self._lock:
            current_time = time.time()

            # Calculate error rates
            recent_errors = [
                record for record in self.error_history
                if current_time - record.timestamp < 3600  # Last hour
            ]

            error_rate_per_hour = len(recent_errors)

            # Group by severity
            severity_counts = defaultdict(int)
            for record in recent_errors:
                severity_counts[record.severity.name] += 1

            # Group by category
            category_counts = defaultdict(int)
            for record in recent_errors:
                category_counts[record.category.value] += 1

            # Recovery statistics
            recovery_stats = {
                'total_recovery_attempts': sum(self.recovery_attempts.values()),
                'successful_recoveries': sum(
                    1 for record in self.error_history if record.recovered
                ),
                'recovery_success_rate': 0.0
            }

            if recovery_stats['total_recovery_attempts'] > 0:
                recovery_stats['recovery_success_rate'] = (
                    recovery_stats['successful_recoveries'] /
                    recovery_stats['total_recovery_attempts']
                )

            # Performance statistics
            avg_handling_time = (
                sum(self.error_handling_times) / len(self.error_handling_times)
                if self.error_handling_times else 0.0
            )

            return {
                'total_errors': len(self.error_history),
                'error_rate_per_hour': error_rate_per_hour,
                'severity_distribution': dict(severity_counts),
                'category_distribution': dict(category_counts),
                'system_healthy': self.system_healthy,
                'emergency_mode': self.emergency_mode,
                'recovery_statistics': recovery_stats,
                'average_handling_time_ms': avg_handling_time * 1000,
                'last_health_check': self.last_health_check,
                'most_common_errors': dict(
                    sorted(self.error_counts.items(), key=lambda x: x[1], reverse=True)[:10]
                )
            }

    def export_error_log(self, filename: str, max_records: int = 1000):
        """Export error history to file for analysis."""
        try:
            with self._lock:
                records_to_export = list(self.error_history)[-max_records:]

            export_data = {
                'export_timestamp': time.time(),
                'total_records': len(records_to_export),
                'emergency_mode': self.emergency_mode,
                'system_healthy': self.system_healthy,
                'error_records': [record.to_dict() for record in records_to_export]
            }

            with open(filename, 'w') as f:
                json.dump(export_data, f, indent=2, default=str)

            logger.info(f"Exported {len(records_to_export)} error records to {filename}")

        except Exception as e:
            logger.error(f"Failed to export error log: {e}")

    def clear_error_history(self):
        """Clear error history (use with caution)."""
        with self._lock:
            self.error_history.clear()
            self.error_counts.clear()
            self.error_patterns.clear()
            self.recovery_attempts.clear()

        logger.info("Cleared error history")

    def is_system_healthy(self) -> bool:
        """Check if system is currently healthy."""
        return self.system_healthy and not self.emergency_mode


# Built-in recovery strategies
async def restart_component_recovery(error_record: ErrorRecord) -> bool:
    """Generic component restart recovery strategy."""
    try:
        component = error_record.context.get('component')
        if component and hasattr(component, 'restart'):
            await component.restart()
            return True
        return False
    except Exception:
        return False


async def memory_cleanup_recovery(error_record: ErrorRecord) -> bool:
    """Memory cleanup recovery strategy."""
    try:
        import gc
        gc.collect()
        return True
    except Exception:
        return False


async def connection_reset_recovery(error_record: ErrorRecord) -> bool:
    """Connection reset recovery strategy."""
    try:
        connection = error_record.context.get('connection')
        if connection and hasattr(connection, 'reset'):
            await connection.reset()
            return True
        return False
    except Exception:
        return False


# Global error handler instance
global_error_handler = ErrorHandler()


def handle_critical_error(
    error: Union[Exception, str],
    context: Optional[Dict[str, Any]] = None,
    error_code: Optional[str] = None
) -> ErrorRecord:
    """Convenience function for handling critical errors."""
    return global_error_handler.handle_error(
        error,
        severity=ErrorSeverity.CRITICAL,
        category=ErrorCategory.SOFTWARE,
        context=context,
        error_code=error_code
    )


def handle_communication_error(
    error: Union[Exception, str],
    context: Optional[Dict[str, Any]] = None
) -> ErrorRecord:
    """Convenience function for handling communication errors."""
    return global_error_handler.handle_error(
        error,
        severity=ErrorSeverity.ERROR,
        category=ErrorCategory.COMMUNICATION,
        context=context
    )


def graceful_error_handler(func: Callable) -> Callable:
    """
    Decorator for graceful error handling with automatic recovery.

    Args:
        func: Function to wrap with error handling

    Returns:
        Wrapped function with error handling
    """
    def wrapper(*args, **kwargs):
        try:
            return func(*args, **kwargs)
        except CriticalError as e:
            handle_critical_error(e, context={'function': func.__name__})
            raise
        except RecoverableError as e:
            global_error_handler.handle_error(
                e,
                severity=ErrorSeverity.WARNING,
                context={'function': func.__name__, 'recoverable': True}
            )
            # For recoverable errors, we might want to return a default value
            # or retry the operation
            if e.retry_possible:
                logger.info(f"Retrying {func.__name__} after recoverable error")
                try:
                    return func(*args, **kwargs)
                except Exception:
                    # If retry also fails, escalate to error
                    global_error_handler.handle_error(
                        f"Retry failed for {func.__name__}",
                        severity=ErrorSeverity.ERROR,
                        context={'function': func.__name__}
                    )
                    raise
            raise
        except Exception as e:
            global_error_handler.handle_error(
                e,
                context={'function': func.__name__}
            )
            raise

    return wrapper


__all__ = [
    'ErrorHandler',
    'ErrorRecord',
    'ErrorSeverity',
    'ErrorCategory',
    'CriticalError',
    'RecoverableError',
    'global_error_handler',
    'handle_critical_error',
    'handle_communication_error',
    'graceful_error_handler',
    'restart_component_recovery',
    'memory_cleanup_recovery',
    'connection_reset_recovery'
]
