# Contributing to Space Data Communication Analysis Project

Thank you for your interest in contributing to our space data communication analysis project! This document provides guidelines for contributing to ensure consistency and quality across the codebase.

## Code of Conduct

By participating in this project, you agree to abide by our Code of Conduct. Please read the full text to understand what actions will and will not be tolerated.

## Getting Started

### Prerequisites

- Python 3.10 or higher
- Docker and Docker Compose
- Git
- Basic understanding of space communication systems and NASA standards

### Development Environment Setup

1. Fork the repository
2. Clone your fork: `git clone https://github.com/your-username/space-data-project.git`
3. Set up development environment:
   ```bash
   cd space-data-project
   pip install -r requirements.txt
   pip install -r requirements-dev.txt
   ```
4. Run tests to verify setup: `pytest tests/`

## Development Workflow

### Branching Strategy

- `main`: Production-ready code
- `develop`: Integration branch for features
- `feature/*`: Feature development branches
- `hotfix/*`: Critical bug fixes
- `release/*`: Release preparation branches

### Making Changes

1. Create a feature branch from `develop`:
   ```bash
   git checkout develop
   git pull origin develop
   git checkout -b feature/your-feature-name
   ```

2. Make your changes following the coding standards
3. Add or update tests for your changes
4. Run the test suite: `pytest tests/`
5. Update documentation as needed
6. Commit your changes with descriptive messages

### Commit Message Format

Follow the conventional commits specification:

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

Examples:
- `feat(messaging): add priority-based message scheduler`
- `fix(security): resolve encryption key rotation issue`
- `docs(api): update authentication documentation`

## Coding Standards

### Python Style Guide

- Follow PEP 8 style guidelines
- Use type hints for all function signatures
- Maximum line length: 120 characters
- Use meaningful variable and function names
- Follow naming conventions:
  - Variables and functions: `snake_case`
  - Classes: `PascalCase`
  - Constants: `UPPER_CASE`
  - Private methods: `_leading_underscore`

### Code Quality Requirements

- Maintain test coverage above 95%
- All public functions must have docstrings
- Use type hints consistently
- Handle errors gracefully with appropriate logging
- Follow NASA coding standards for space applications

### Example Code Structure

```python
"""Module for priority-based message scheduling.

This module implements NASA CCSDS-compliant message scheduling
with support for high-frequency communication requirements.
"""

import asyncio
import logging
from typing import Dict, List, Optional, Union
from enum import Enum

logger = logging.getLogger(__name__)


class MessagePriority(Enum):
    """Message priority levels for space communication."""
    
    HIGH = "high"
    MEDIUM = "medium"
    LOW = "low"


class MessageScheduler:
    """Priority-based message scheduler for space communication.
    
    Implements adaptive scheduling with guaranteed latency requirements
    for different priority levels according to NASA standards.
    
    Attributes:
        max_bandwidth: Maximum available bandwidth in Hz
        priority_queues: Separate queues for each priority level
    """
    
    def __init__(self, max_bandwidth: int = 10000) -> None:
        """Initialize the message scheduler.
        
        Args:
            max_bandwidth: Maximum bandwidth allocation in Hz
            
        Raises:
            ValueError: If max_bandwidth is not positive
        """
        if max_bandwidth <= 0:
            raise ValueError("max_bandwidth must be positive")
            
        self.max_bandwidth = max_bandwidth
        self.priority_queues: Dict[str, List] = {
            priority.value: [] for priority in MessagePriority
        }
        logger.info(f"MessageScheduler initialized with {max_bandwidth}Hz bandwidth")
    
    async def add_message(
        self, 
        message: str, 
        priority: MessagePriority, 
        bandwidth_required: int
    ) -> bool:
        """Add a message to the appropriate priority queue.
        
        Args:
            message: Message content
            priority: Message priority level
            bandwidth_required: Required bandwidth in Hz
            
        Returns:
            True if message was successfully queued, False otherwise
            
        Raises:
            ValueError: If bandwidth_required is negative
        """
        # Implementation here...
        pass
```

## Testing Guidelines

### Test Structure

- Unit tests in `tests/unit/`
- Integration tests in `tests/integration/`
- Performance tests in `tests/performance/`
- End-to-end tests in `tests/e2e/`

### Test Requirements

- All new code must have corresponding tests
- Tests should cover both success and failure cases
- Use meaningful test names that describe the scenario
- Mock external dependencies appropriately
- Include performance benchmarks for critical paths

### Example Test

```python
"""Tests for priority-based message scheduler."""

import pytest
import asyncio
from unittest.mock import Mock, patch

from src.messaging.priority_scheduler import MessageScheduler, MessagePriority


class TestMessageScheduler:
    """Test suite for MessageScheduler class."""
    
    @pytest.fixture
    def scheduler(self) -> MessageScheduler:
        """Create a MessageScheduler instance for testing."""
        return MessageScheduler(max_bandwidth=1000)
    
    def test_init_valid_bandwidth(self) -> None:
        """Test scheduler initialization with valid bandwidth."""
        scheduler = MessageScheduler(max_bandwidth=5000)
        assert scheduler.max_bandwidth == 5000
        assert len(scheduler.priority_queues) == 3
    
    def test_init_invalid_bandwidth(self) -> None:
        """Test scheduler initialization with invalid bandwidth raises ValueError."""
        with pytest.raises(ValueError, match="max_bandwidth must be positive"):
            MessageScheduler(max_bandwidth=0)
    
    @pytest.mark.asyncio
    async def test_add_high_priority_message(self, scheduler: MessageScheduler) -> None:
        """Test adding high priority message to scheduler."""
        result = await scheduler.add_message(
            "Critical telemetry", 
            MessagePriority.HIGH, 
            100
        )
        assert result is True
        assert len(scheduler.priority_queues["high"]) == 1
```

## Documentation Standards

### Code Documentation

- All public APIs must have docstrings
- Use Google-style docstrings
- Include type information in docstrings
- Provide usage examples for complex functions
- Document any NASA-specific requirements or constraints

### Project Documentation

- Update README.md for user-facing changes
- Update API documentation for interface changes
- Include NASA compliance information
- Provide installation and usage examples
- Update project plan for significant features

## NASA Standards Compliance

### Requirements

- All communication protocols must comply with CCSDS standards
- Follow NASA requirements documentation format
- Implement proper error handling and logging
- Include fault tolerance mechanisms
- Maintain security standards for space applications

### Documentation Requirements

- Requirements traceability matrix
- Design documentation following NASA-STD-DESIGN
- Interface control documents for external systems
- Test procedures and results documentation

## Performance Guidelines

### Performance Requirements

- Message processing: 1000Hz for high-priority messages
- Latency: <50ms for real-time analysis
- Memory usage: <80% of available system memory
- CPU utilization: <70% under normal load

### Optimization Guidelines

- Profile code using appropriate tools
- Optimize critical paths identified through benchmarking
- Use appropriate data structures for performance
- Consider memory allocation patterns
- Implement caching where beneficial

## Security Considerations

### Security Requirements

- All sensitive data must be encrypted
- Implement proper authentication and authorization
- Follow secure coding practices
- Regular security audits and penetration testing
- Compliance with space application security standards

### Secure Coding Practices

- Validate all inputs
- Use parameterized queries for database operations
- Implement proper error handling without information leakage
- Use secure random number generation
- Regular dependency updates for security patches

## Pull Request Process

1. Ensure your branch is up to date with the target branch
2. Run the full test suite and ensure all tests pass
3. Update documentation as needed
4. Create a pull request with a clear description
5. Request review from appropriate team members
6. Address feedback and update as needed
7. Merge after approval from maintainers

## Review Criteria

- Code follows project standards and guidelines
- All tests pass and coverage requirements are met
- Documentation is updated and accurate
- NASA compliance requirements are satisfied
- Security considerations are addressed
- Performance impact is acceptable

## Questions and Support

If you have questions about contributing:

- Check existing issues and discussions
- Review the project documentation
- Ask questions in GitHub discussions
- Contact maintainers for complex issues

Thank you for contributing to our space data communication analysis project!
