"""Fault tolerance module for space communication systems.

This module provides comprehensive fault tolerance mechanisms including
error correction, redundancy systems, and graceful degradation capabilities
designed for NASA space mission requirements.
"""

from .ldpc_error_correction import (
    LDPCEncoder,
    ErrorCorrectionMode,
    ChannelCondition,
    CodeParameters,
    DecodingResult,
    PerformanceTracker
)

__all__ = [
    "LDPCEncoder",
    "ErrorCorrectionMode", 
    "ChannelCondition",
    "CodeParameters",
    "DecodingResult",
    "PerformanceTracker"
]
