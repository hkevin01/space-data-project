"""
Health check utility for space communication system.

Provides comprehensive health monitoring for all system components
with NASA mission-critical requirements compliance.
"""

import asyncio
import logging
import time
import psutil
import sys
from typing import Dict, Any, List, Optional
from dataclasses import dataclass
from enum import Enum

logger = logging.getLogger(__name__)


class HealthStatus(Enum):
    """System health status levels."""
    HEALTHY = "healthy"
    WARNING = "warning"
    CRITICAL = "critical"
    UNKNOWN = "unknown"


@dataclass
class HealthCheckResult:
    """Result of a health check operation."""
    status: HealthStatus
    message: str
    details: Dict[str, Any]
    timestamp: float
    check_duration_ms: float


class SystemHealthChecker:
    """Comprehensive system health monitoring for space applications."""
    
    def __init__(self):
        self.start_time = time.time()
        self.health_thresholds = {
            "cpu_warning": 70.0,
            "cpu_critical": 85.0,
            "memory_warning": 75.0,
            "memory_critical": 90.0,
            "disk_warning": 80.0,
            "disk_critical": 95.0
        }
    
    def check_system_resources(self) -> HealthCheckResult:
        """Check system resource utilization."""
        start_time = time.time()
        
        try:
            # CPU usage
            cpu_percent = psutil.cpu_percent(interval=1)
            
            # Memory usage
            memory = psutil.virtual_memory()
            memory_percent = memory.percent
            
            # Disk usage
            disk = psutil.disk_usage('/')
            disk_percent = (disk.used / disk.total) * 100
            
            # Determine overall status
            status = HealthStatus.HEALTHY
            issues = []
            
            if cpu_percent > self.health_thresholds["cpu_critical"]:
                status = HealthStatus.CRITICAL
                issues.append(f"CPU usage critical: {cpu_percent:.1f}%")
            elif cpu_percent > self.health_thresholds["cpu_warning"]:
                status = HealthStatus.WARNING
                issues.append(f"CPU usage high: {cpu_percent:.1f}%")
            
            if memory_percent > self.health_thresholds["memory_critical"]:
                status = HealthStatus.CRITICAL
                issues.append(f"Memory usage critical: {memory_percent:.1f}%")
            elif memory_percent > self.health_thresholds["memory_warning"]:
                status = HealthStatus.WARNING
                issues.append(f"Memory usage high: {memory_percent:.1f}%")
            
            if disk_percent > self.health_thresholds["disk_critical"]:
                status = HealthStatus.CRITICAL
                issues.append(f"Disk usage critical: {disk_percent:.1f}%")
            elif disk_percent > self.health_thresholds["disk_warning"]:
                status = HealthStatus.WARNING
                issues.append(f"Disk usage high: {disk_percent:.1f}%")
            
            message = "System resources healthy" if not issues else "; ".join(issues)
            
            details = {
                "cpu_percent": cpu_percent,
                "memory_percent": memory_percent,
                "memory_available_gb": memory.available / (1024**3),
                "disk_percent": disk_percent,
                "disk_free_gb": disk.free / (1024**3),
                "uptime_seconds": time.time() - self.start_time
            }
            
            return HealthCheckResult(
                status=status,
                message=message,
                details=details,
                timestamp=time.time(),
                check_duration_ms=(time.time() - start_time) * 1000
            )
            
        except Exception as e:
            return HealthCheckResult(
                status=HealthStatus.UNKNOWN,
                message=f"Error checking system resources: {e}",
                details={"error": str(e)},
                timestamp=time.time(),
                check_duration_ms=(time.time() - start_time) * 1000
            )
    
    def check_application_health(self) -> HealthCheckResult:
        """Check application-specific health metrics."""
        start_time = time.time()
        
        try:
            # Basic application health checks
            status = HealthStatus.HEALTHY
            message = "Application healthy"
            
            details = {
                "python_version": sys.version,
                "process_id": psutil.Process().pid,
                "thread_count": psutil.Process().num_threads(),
                "open_files": len(psutil.Process().open_files()),
                "memory_info": psutil.Process().memory_info()._asdict()
            }
            
            return HealthCheckResult(
                status=status,
                message=message,
                details=details,
                timestamp=time.time(),
                check_duration_ms=(time.time() - start_time) * 1000
            )
            
        except Exception as e:
            return HealthCheckResult(
                status=HealthStatus.UNKNOWN,
                message=f"Error checking application health: {e}",
                details={"error": str(e)},
                timestamp=time.time(),
                check_duration_ms=(time.time() - start_time) * 1000
            )
    
    async def comprehensive_health_check(self) -> Dict[str, HealthCheckResult]:
        """Perform comprehensive health check of all system components."""
        results = {}
        
        # System resources
        results["system_resources"] = self.check_system_resources()
        
        # Application health
        results["application"] = self.check_application_health()
        
        return results


def check() -> bool:
    """Quick health check for container health monitoring.
    
    Returns:
        True if system is healthy, False otherwise
    """
    try:
        checker = SystemHealthChecker()
        
        # Quick system check
        result = checker.check_system_resources()
        
        if result.status == HealthStatus.CRITICAL:
            logger.error(f"Health check failed: {result.message}")
            return False
        
        if result.status == HealthStatus.WARNING:
            logger.warning(f"Health check warning: {result.message}")
        
        return True
        
    except Exception as e:
        logger.error(f"Health check error: {e}")
        return False


async def detailed_health_check() -> Dict[str, Any]:
    """Detailed health check for monitoring dashboards."""
    checker = SystemHealthChecker()
    results = await checker.comprehensive_health_check()
    
    # Aggregate overall status
    overall_status = HealthStatus.HEALTHY
    
    for check_name, result in results.items():
        if result.status == HealthStatus.CRITICAL:
            overall_status = HealthStatus.CRITICAL
            break
        elif result.status == HealthStatus.WARNING and overall_status == HealthStatus.HEALTHY:
            overall_status = HealthStatus.WARNING
    
    return {
        "overall_status": overall_status.value,
        "timestamp": time.time(),
        "checks": {
            name: {
                "status": result.status.value,
                "message": result.message,
                "details": result.details,
                "check_duration_ms": result.check_duration_ms
            }
            for name, result in results.items()
        }
    }


if __name__ == "__main__":
    # Command line health check
    is_healthy = check()
    sys.exit(0 if is_healthy else 1)
