# GitHub Copilot Configuration for Space Data Project

## Project Context
This project implements a NASA-compliant space communication system with priority-based messaging, fault tolerance, and advanced security features.

## Code Preferences
- Use type hints for all function signatures
- Follow PEP 8 style guidelines
- Include comprehensive docstrings with NASA standards
- Implement robust error handling and boundary condition checks
- Add performance monitoring and time measurement
- Include memory management and crash prevention

## Domain-Specific Knowledge
- CCSDS (Consultative Committee for Space Data Systems) protocols
- Space communication bands (K-band, Ka-band, X-band, S-band, UHF)
- Priority-based message scheduling
- LDPC (Low-Density Parity Check) error correction
- Post-quantum cryptography
- Satellite communication patterns
- Defense and GPS integration

## Common Patterns
- Async/await for high-performance communication
- Graceful degradation under fault conditions
- Time-critical operations with microsecond precision
- Memory-efficient data structures for large telemetry datasets
- Security-first design with encryption by default

## Preferred Libraries
- asyncio for asynchronous programming
- numpy/scipy for signal processing
- cryptography for security features
- prometheus_client for metrics
- pytest for testing
- typing for type annotations
