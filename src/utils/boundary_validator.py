"""
Boundary condition validators for space communication systems.

This module provides comprehensive validation utilities to ensure
system stability and prevent errors at boundary conditions.

Author: Space Data Communication Team
Version: 1.0.0
NASA-STD-REQ: SWE-REQ-014, SWE-REQ-015
"""

import math
import re
from typing import Any, List, Optional, Tuple, Union
from enum import Enum


class ValidationError(Exception):
    """Custom exception for validation failures."""
    pass


class BoundaryValidator:
    """
    Comprehensive boundary condition validator for space systems.

    Validates input parameters, ranges, and constraints to prevent
    system failures and ensure reliable operation under all conditions.
    """

    # Constants for space communication constraints
    MAX_FREQUENCY_HZ = 300e9  # 300 GHz upper limit for space communication
    MIN_FREQUENCY_HZ = 1e6    # 1 MHz lower limit
    MAX_BANDWIDTH_BPS = 100e9  # 100 Gbps theoretical maximum
    MIN_BANDWIDTH_BPS = 1      # 1 bps minimum
    MAX_POWER_WATTS = 10000    # 10 kW maximum transmit power
    MAX_MESSAGE_SIZE_BYTES = 100 * 1024 * 1024  # 100 MB maximum message

    @staticmethod
    def validate_positive_integer(
        value: int,
        name: str,
        min_value: int = 1,
        max_value: Optional[int] = None
    ) -> int:
        """
        Validate positive integer with optional bounds.

        Args:
            value: Integer value to validate
            name: Parameter name for error messages
            min_value: Minimum allowed value (inclusive)
            max_value: Maximum allowed value (inclusive, None for no limit)

        Returns:
            int: Validated value

        Raises:
            ValidationError: If validation fails
        """
        if not isinstance(value, int):
            raise ValidationError(f"{name} must be an integer, got {type(value).__name__}")

        if value < min_value:
            raise ValidationError(f"{name} must be >= {min_value}, got {value}")

        if max_value is not None and value > max_value:
            raise ValidationError(f"{name} must be <= {max_value}, got {value}")

        return value

    @staticmethod
    def validate_positive_float(
        value: float,
        name: str,
        min_value: float = 0.0,
        max_value: Optional[float] = None,
        allow_zero: bool = True
    ) -> float:
        """
        Validate positive float with optional bounds.

        Args:
            value: Float value to validate
            name: Parameter name for error messages
            min_value: Minimum allowed value
            max_value: Maximum allowed value (None for no limit)
            allow_zero: Whether zero is allowed

        Returns:
            float: Validated value

        Raises:
            ValidationError: If validation fails
        """
        if not isinstance(value, (int, float)):
            raise ValidationError(f"{name} must be numeric, got {type(value).__name__}")

        value = float(value)

        if math.isnan(value):
            raise ValidationError(f"{name} cannot be NaN")

        if math.isinf(value):
            raise ValidationError(f"{name} cannot be infinite")

        if not allow_zero and value == 0.0:
            raise ValidationError(f"{name} cannot be zero")

        if value < min_value:
            raise ValidationError(f"{name} must be >= {min_value}, got {value}")

        if max_value is not None and value > max_value:
            raise ValidationError(f"{name} must be <= {max_value}, got {value}")

        return value

    @staticmethod
    def validate_frequency(frequency_hz: float, name: str = "frequency") -> float:
        """
        Validate frequency for space communication.

        Args:
            frequency_hz: Frequency in Hz
            name: Parameter name for error messages

        Returns:
            float: Validated frequency

        Raises:
            ValidationError: If frequency is outside valid range
        """
        frequency_hz = BoundaryValidator.validate_positive_float(
            frequency_hz,
            name,
            min_value=BoundaryValidator.MIN_FREQUENCY_HZ,
            max_value=BoundaryValidator.MAX_FREQUENCY_HZ,
            allow_zero=False
        )

        return frequency_hz

    @staticmethod
    def validate_bandwidth(bandwidth_bps: int, name: str = "bandwidth") -> int:
        """
        Validate bandwidth for communication systems.

        Args:
            bandwidth_bps: Bandwidth in bits per second
            name: Parameter name for error messages

        Returns:
            int: Validated bandwidth

        Raises:
            ValidationError: If bandwidth is outside valid range
        """
        bandwidth_bps = BoundaryValidator.validate_positive_integer(
            bandwidth_bps,
            name,
            min_value=BoundaryValidator.MIN_BANDWIDTH_BPS,
            max_value=BoundaryValidator.MAX_BANDWIDTH_BPS
        )

        return bandwidth_bps

    @staticmethod
    def validate_power(power_watts: float, name: str = "power") -> float:
        """
        Validate power levels for transmitters.

        Args:
            power_watts: Power in watts
            name: Parameter name for error messages

        Returns:
            float: Validated power

        Raises:
            ValidationError: If power is outside safe range
        """
        power_watts = BoundaryValidator.validate_positive_float(
            power_watts,
            name,
            min_value=0.001,  # 1 mW minimum
            max_value=BoundaryValidator.MAX_POWER_WATTS,
            allow_zero=False
        )

        return power_watts

    @staticmethod
    def validate_string(
        text: str,
        name: str,
        min_length: int = 0,
        max_length: int = 10000,
        pattern: Optional[str] = None,
        allowed_chars: Optional[str] = None
    ) -> str:
        """
        Validate string with length and content constraints.

        Args:
            text: String to validate
            name: Parameter name for error messages
            min_length: Minimum allowed length
            max_length: Maximum allowed length
            pattern: Optional regex pattern to match
            allowed_chars: Optional set of allowed characters

        Returns:
            str: Validated string

        Raises:
            ValidationError: If string validation fails
        """
        if not isinstance(text, str):
            raise ValidationError(f"{name} must be a string, got {type(text).__name__}")

        if len(text) < min_length:
            raise ValidationError(f"{name} must be at least {min_length} characters, got {len(text)}")

        if len(text) > max_length:
            raise ValidationError(f"{name} must be at most {max_length} characters, got {len(text)}")

        if pattern and not re.match(pattern, text):
            raise ValidationError(f"{name} does not match required pattern: {pattern}")

        if allowed_chars and not all(c in allowed_chars for c in text):
            invalid_chars = set(text) - set(allowed_chars)
            raise ValidationError(f"{name} contains invalid characters: {invalid_chars}")

        return text

    @staticmethod
    def validate_enum_value(value: Any, enum_class: type, name: str) -> Any:
        """
        Validate that value is a member of the specified enum.

        Args:
            value: Value to validate
            enum_class: Enum class to check against
            name: Parameter name for error messages

        Returns:
            Validated enum value

        Raises:
            ValidationError: If value is not a valid enum member
        """
        if not isinstance(value, enum_class):
            if isinstance(value, str):
                # Try to convert string to enum
                try:
                    return enum_class[value.upper()]
                except KeyError:
                    pass

            valid_values = [member.name for member in enum_class]
            raise ValidationError(
                f"{name} must be one of {valid_values}, got {value}"
            )

        return value

    @staticmethod
    def validate_list(
        items: List[Any],
        name: str,
        min_length: int = 0,
        max_length: Optional[int] = None,
        item_validator: Optional[callable] = None
    ) -> List[Any]:
        """
        Validate list with length and item constraints.

        Args:
            items: List to validate
            name: Parameter name for error messages
            min_length: Minimum required list length
            max_length: Maximum allowed list length
            item_validator: Optional function to validate each item

        Returns:
            List: Validated list

        Raises:
            ValidationError: If list validation fails
        """
        if not isinstance(items, list):
            raise ValidationError(f"{name} must be a list, got {type(items).__name__}")

        if len(items) < min_length:
            raise ValidationError(f"{name} must have at least {min_length} items, got {len(items)}")

        if max_length is not None and len(items) > max_length:
            raise ValidationError(f"{name} must have at most {max_length} items, got {len(items)}")

        if item_validator:
            for i, item in enumerate(items):
                try:
                    item_validator(item)
                except Exception as e:
                    raise ValidationError(f"{name}[{i}] validation failed: {e}")

        return items

    @staticmethod
    def validate_range(
        value: Union[int, float],
        name: str,
        min_val: Union[int, float],
        max_val: Union[int, float],
        inclusive: bool = True
    ) -> Union[int, float]:
        """
        Validate that value is within specified range.

        Args:
            value: Value to validate
            name: Parameter name for error messages
            min_val: Minimum allowed value
            max_val: Maximum allowed value
            inclusive: Whether bounds are inclusive

        Returns:
            Validated value

        Raises:
            ValidationError: If value is outside range
        """
        if not isinstance(value, (int, float)):
            raise ValidationError(f"{name} must be numeric, got {type(value).__name__}")

        if isinstance(value, float) and (math.isnan(value) or math.isinf(value)):
            raise ValidationError(f"{name} must be a finite number")

        if inclusive:
            if value < min_val or value > max_val:
                raise ValidationError(f"{name} must be in range [{min_val}, {max_val}], got {value}")
        else:
            if value <= min_val or value >= max_val:
                raise ValidationError(f"{name} must be in range ({min_val}, {max_val}), got {value}")

        return value

    @staticmethod
    def validate_coordinates(
        latitude: float,
        longitude: float,
        altitude: Optional[float] = None
    ) -> Tuple[float, float, Optional[float]]:
        """
        Validate geographic coordinates for satellite tracking.

        Args:
            latitude: Latitude in degrees (-90 to 90)
            longitude: Longitude in degrees (-180 to 180)
            altitude: Optional altitude in meters

        Returns:
            Tuple of validated coordinates

        Raises:
            ValidationError: If coordinates are invalid
        """
        latitude = BoundaryValidator.validate_range(
            latitude, "latitude", -90.0, 90.0, inclusive=True
        )

        longitude = BoundaryValidator.validate_range(
            longitude, "longitude", -180.0, 180.0, inclusive=True
        )

        if altitude is not None:
            altitude = BoundaryValidator.validate_range(
                altitude, "altitude", -1000.0, 100000000.0, inclusive=True
            )

        return latitude, longitude, altitude

    @staticmethod
    def validate_time_interval(
        start_time: float,
        end_time: float,
        min_duration: float = 0.0,
        max_duration: Optional[float] = None
    ) -> Tuple[float, float]:
        """
        Validate time interval constraints.

        Args:
            start_time: Start time (seconds since epoch)
            end_time: End time (seconds since epoch)
            min_duration: Minimum required duration
            max_duration: Maximum allowed duration

        Returns:
            Tuple of validated times

        Raises:
            ValidationError: If time interval is invalid
        """
        start_time = BoundaryValidator.validate_positive_float(
            start_time, "start_time", min_value=0.0
        )

        end_time = BoundaryValidator.validate_positive_float(
            end_time, "end_time", min_value=0.0
        )

        if end_time <= start_time:
            raise ValidationError("end_time must be greater than start_time")

        duration = end_time - start_time

        if duration < min_duration:
            raise ValidationError(f"Duration {duration:.3f}s must be >= {min_duration:.3f}s")

        if max_duration is not None and duration > max_duration:
            raise ValidationError(f"Duration {duration:.3f}s must be <= {max_duration:.3f}s")

        return start_time, end_time

    @staticmethod
    def validate_message_size(size_bytes: int) -> int:
        """
        Validate message size for space communication.

        Args:
            size_bytes: Message size in bytes

        Returns:
            int: Validated size

        Raises:
            ValidationError: If size exceeds limits
        """
        return BoundaryValidator.validate_positive_integer(
            size_bytes,
            "message_size",
            min_value=1,
            max_value=BoundaryValidator.MAX_MESSAGE_SIZE_BYTES
        )

    @staticmethod
    def validate_snr(snr_db: float) -> float:
        """
        Validate Signal-to-Noise Ratio.

        Args:
            snr_db: SNR in decibels

        Returns:
            float: Validated SNR

        Raises:
            ValidationError: If SNR is unrealistic
        """
        return BoundaryValidator.validate_range(
            snr_db, "snr_db", -50.0, 100.0, inclusive=True
        )

    @staticmethod
    def validate_network_address(address: str) -> str:
        """
        Validate network address format.

        Args:
            address: Network address (IP or hostname)

        Returns:
            str: Validated address

        Raises:
            ValidationError: If address format is invalid
        """
        address = BoundaryValidator.validate_string(
            address, "address", min_length=1, max_length=253
        )

        # Basic validation for IPv4/IPv6 or hostname
        ipv4_pattern = r'^(\d{1,3}\.){3}\d{1,3}$'
        hostname_pattern = r'^[a-zA-Z0-9][a-zA-Z0-9\-\.]*[a-zA-Z0-9]$'

        if not (re.match(ipv4_pattern, address) or re.match(hostname_pattern, address)):
            raise ValidationError(f"Invalid network address format: {address}")

        return address


class SafetyLimits:
    """
    Safety limits and constraints for space communication systems.

    Defines operational boundaries to prevent equipment damage
    and ensure mission safety.
    """

    # Power safety limits
    MAX_TRANSMIT_POWER_WATTS = 5000  # 5 kW safe maximum
    MAX_ANTENNA_GAIN_DB = 60         # 60 dB maximum antenna gain

    # Frequency safety limits
    SAFE_FREQUENCY_RANGES = [
        (1.0e9, 2.0e9),    # L-band
        (2.0e9, 4.0e9),    # S-band
        (4.0e9, 8.0e9),    # C-band
        (8.0e9, 12.0e9),   # X-band
        (12.0e9, 18.0e9),  # Ku-band
        (18.0e9, 27.0e9),  # K-band
        (27.0e9, 40.0e9),  # Ka-band
    ]

    @staticmethod
    def check_power_safety(power_watts: float, gain_db: float) -> bool:
        """
        Check if power levels are within safety limits.

        Args:
            power_watts: Transmitter power in watts
            gain_db: Antenna gain in dB

        Returns:
            bool: True if within safety limits
        """
        if power_watts > SafetyLimits.MAX_TRANSMIT_POWER_WATTS:
            return False

        if gain_db > SafetyLimits.MAX_ANTENNA_GAIN_DB:
            return False

        # Check effective radiated power
        erp_watts = power_watts * (10 ** (gain_db / 10))
        if erp_watts > 50000:  # 50 kW ERP limit
            return False

        return True

    @staticmethod
    def check_frequency_safety(frequency_hz: float) -> bool:
        """
        Check if frequency is in safe operational range.

        Args:
            frequency_hz: Frequency in Hz

        Returns:
            bool: True if frequency is in safe range
        """
        for min_freq, max_freq in SafetyLimits.SAFE_FREQUENCY_RANGES:
            if min_freq <= frequency_hz <= max_freq:
                return True
        return False


__all__ = [
    'BoundaryValidator',
    'ValidationError',
    'SafetyLimits'
]
