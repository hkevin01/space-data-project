# Multi-stage Dockerfile for Space Data Communication Analysis Project
# Optimized for production deployment with security and performance considerations

# Stage 1: Base image with system dependencies
FROM python:3.11-slim as base

# Set environment variables for production
ENV PYTHONUNBUFFERED=1 \
    PYTHONDONTWRITEBYTECODE=1 \
    PIP_NO_CACHE_DIR=1 \
    PIP_DISABLE_PIP_VERSION_CHECK=1 \
    POETRY_VENV_IN_PROJECT=1 \
    POETRY_NO_INTERACTION=1 \
    POETRY_CACHE_DIR=/tmp/poetry_cache

# Create non-root user for security
RUN groupadd --gid 1000 spaceuser && \
    useradd --uid 1000 --gid 1000 --create-home --shell /bin/bash spaceuser

# Install system dependencies required for space communication libraries
RUN apt-get update && apt-get install -y \
    build-essential \
    gcc \
    g++ \
    gfortran \
    libopenblas-dev \
    liblapack-dev \
    libfftw3-dev \
    libssl-dev \
    libcurl4-openssl-dev \
    pkg-config \
    git \
    curl \
    wget \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Stage 2: Development dependencies (for multi-stage builds)
FROM base as development

# Set working directory
WORKDIR /app

# Copy requirements files
COPY requirements.txt requirements-dev.txt ./

# Install Python dependencies in virtual environment
RUN python -m venv /opt/venv
ENV PATH="/opt/venv/bin:$PATH"

# Upgrade pip and install dependencies
RUN pip install --upgrade pip setuptools wheel && \
    pip install -r requirements.txt && \
    pip install -r requirements-dev.txt

# Copy application code
COPY . .

# Change ownership to non-root user
RUN chown -R spaceuser:spaceuser /app /opt/venv

# Switch to non-root user
USER spaceuser

# Expose port for development server
EXPOSE 8000

# Development command
CMD ["python", "-m", "src.main"]

# Stage 3: Production image
FROM base as production

# Set working directory
WORKDIR /app

# Copy requirements file
COPY requirements.txt ./

# Install Python dependencies in virtual environment
RUN python -m venv /opt/venv
ENV PATH="/opt/venv/bin:$PATH"

# Install only production dependencies
RUN pip install --upgrade pip setuptools wheel && \
    pip install -r requirements.txt && \
    pip cache purge

# Copy application source code
COPY src/ ./src/
COPY scripts/ ./scripts/
COPY data/ ./data/
COPY assets/ ./assets/

# Create necessary directories
RUN mkdir -p /app/logs /app/data/processed /app/data/temp && \
    chown -R spaceuser:spaceuser /app /opt/venv

# Switch to non-root user for security
USER spaceuser

# Health check for container monitoring
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
    CMD python -c "import src.utils.health_check; src.utils.health_check.check()" || exit 1

# Expose ports
EXPOSE 8000 9090 3000

# Set default environment variables
ENV SPACE_COMM_ENV=production \
    SPACE_COMM_LOG_LEVEL=INFO \
    SPACE_COMM_METRICS_PORT=9090 \
    SPACE_COMM_DASHBOARD_PORT=3000

# Labels for container metadata
LABEL maintainer="Space Data Team" \
      description="NASA-compliant space data communication analysis system" \
      version="1.0.0" \
      nasa.compliance="CCSDS" \
      security.scan="enabled"

# Production command
CMD ["python", "-m", "src.main", "--mode", "production"]

# Stage 4: Testing image (for CI/CD)
FROM development as testing

# Install additional testing dependencies
RUN pip install pytest-xvfb pytest-qt

# Copy test configuration
COPY pytest.ini .coveragerc ./
COPY tests/ ./tests/

# Run tests during build (optional, can be disabled)
RUN python -m pytest tests/ --cov=src --cov-report=term-missing --cov-fail-under=95

# Testing command
CMD ["python", "-m", "pytest", "tests/", "-v", "--cov=src"]

# Stage 5: Security scanning image
FROM production as security

# Switch back to root for security scanning tools
USER root

# Install security scanning tools
RUN pip install bandit safety semgrep

# Run security scans
RUN bandit -r src/ -f json -o /tmp/bandit-report.json && \
    safety check --json --output /tmp/safety-report.json && \
    semgrep --config=auto src/ --json --output=/tmp/semgrep-report.json

# Copy security reports
RUN mkdir -p /app/security-reports && \
    cp /tmp/*-report.json /app/security-reports/ && \
    chown -R spaceuser:spaceuser /app/security-reports

# Switch back to non-root user
USER spaceuser

# Security scanning command
CMD ["sh", "-c", "echo 'Security scan completed. Reports available in /app/security-reports/'"]

# Stage 6: Monitoring image (with additional monitoring tools)
FROM production as monitoring

# Install monitoring dependencies
USER root
RUN pip install prometheus-client grafana-api psutil

# Copy monitoring configuration
COPY monitoring/ ./monitoring/

# Create monitoring user
RUN chown -R spaceuser:spaceuser /app/monitoring

USER spaceuser

# Monitoring command
CMD ["python", "-m", "src.monitoring.server"]

# Default target is production
FROM production
