"""Space Data Communication Analysis Project - Source Package.

This package implements a comprehensive NASA CCSDS-compliant space communication
system with priority-based messaging, fault tolerance, and advanced security features.

Modules:
    messaging: Priority-based message scheduling and processing
    fault_tolerance: Error correction and redundancy mechanisms  
    security: Cryptography and authentication systems
    bands: Communication frequency band analysis
    utils: Utility functions and helpers
    visualization: Data visualization components
    monitoring: System monitoring and metrics collection

Author: Space Data Communication Team
Version: 1.0.0
License: MIT
NASA Compliance: CCSDS Blue Book Standards
"""

__version__ = "1.0.0"
__author__ = "Space Data Communication Team"
__email__ = "space-data-team@example.com"
__license__ = "MIT"

# Package-level imports for convenience
from .messaging.priority_scheduler import (
    MessageScheduler,
    Message, 
    MessagePriority,
    MessageStatus,
    CommunicationBand,
    TimeConstraints
)

__all__ = [
    "MessageScheduler",
    "Message",
    "MessagePriority", 
    "MessageStatus",
    "CommunicationBand",
    "TimeConstraints"
]
