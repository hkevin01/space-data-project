"""
Main entry point for Space Data Communication Analysis System.

This module provides the main application entry point with comprehensive
initialization, configuration management, and graceful shutdown capabilities
for NASA-compliant space communication operations.

Author: Space Data Communication Team
Version: 1.0.0
NASA Compliance: CCSDS Blue Book Standards
"""

import asyncio
import argparse
import logging
import signal
import sys
import time
import os
from pathlib import Path
from typing import Dict, Any, Optional
import yaml

# Configure logging first
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
    datefmt='%Y-%m-%d %H:%M:%S UTC'
)
logger = logging.getLogger(__name__)

# Import application modules
try:
    from .messaging.priority_scheduler import MessageScheduler, Message, MessagePriority, TimeConstraints
    from .fault_tolerance.ldpc_error_correction import LDPCEncoder, CodeParameters
    from .utils.health_check import detailed_health_check
except ImportError as e:
    logger.error(f"Failed to import application modules: {e}")
    sys.exit(1)


class SpaceCommApplication:
    """Main application class for space communication system.
    
    Manages the lifecycle of all system components including message scheduling,
    error correction, monitoring, and graceful shutdown procedures.
    """
    
    def __init__(self, config: Dict[str, Any]):
        """Initialize the application with configuration.
        
        Args:
            config: Application configuration dictionary
        """
        self.config = config
        self.is_running = False
        self.shutdown_event = asyncio.Event()
        
        # Core components
        self.message_scheduler: Optional[MessageScheduler] = None
        self.ldpc_encoder: Optional[LDPCEncoder] = None
        
        # Monitoring
        self.health_check_interval = config.get("health_check_interval", 30.0)
        self.performance_logging_interval = config.get("performance_logging_interval", 60.0)
        
        # Shutdown handling
        self.cleanup_tasks = []
        
        logger.info("Space Communication Application initialized")
    
    async def initialize_components(self) -> None:
        """Initialize all system components."""
        try:
            logger.info("Initializing system components...")
            
            # Initialize message scheduler
            scheduler_config = self.config.get("message_scheduler", {})
            self.message_scheduler = MessageScheduler(
                max_bandwidth=scheduler_config.get("max_bandwidth", 50000),
                max_queue_size=scheduler_config.get("max_queue_size", 10000),
                enable_adaptive_scheduling=scheduler_config.get("enable_adaptive_scheduling", True),
                enable_performance_monitoring=scheduler_config.get("enable_performance_monitoring", True),
                memory_limit_mb=scheduler_config.get("memory_limit_mb", 512)
            )
            
            # Initialize LDPC encoder
            ldpc_config = self.config.get("ldpc_encoder", {})
            code_params = CodeParameters(
                code_rate=ldpc_config.get("code_rate", 0.5),
                block_length=ldpc_config.get("block_length", 1024),
                max_iterations=ldpc_config.get("max_iterations", 50)
            )
            
            self.ldpc_encoder = LDPCEncoder(
                code_parameters=code_params,
                enable_adaptive_mode=ldpc_config.get("enable_adaptive_mode", True),
                enable_performance_tracking=ldpc_config.get("enable_performance_tracking", True),
                memory_limit_mb=ldpc_config.get("memory_limit_mb", 256)
            )
            
            logger.info("All system components initialized successfully")
            
        except Exception as e:
            logger.error(f"Failed to initialize components: {e}")
            raise
    
    async def start_services(self) -> None:
        """Start all application services."""
        try:
            logger.info("Starting application services...")
            
            # Start message scheduler
            if self.message_scheduler:
                await self.message_scheduler.start_processing()
            
            # Start monitoring tasks
            if self.config.get("enable_health_monitoring", True):
                health_task = asyncio.create_task(self._health_monitoring_loop())
                self.cleanup_tasks.append(health_task)
            
            if self.config.get("enable_performance_logging", True):
                perf_task = asyncio.create_task(self._performance_logging_loop())
                self.cleanup_tasks.append(perf_task)
            
            self.is_running = True
            logger.info("All services started successfully")
            
        except Exception as e:
            logger.error(f"Failed to start services: {e}")
            raise
    
    async def _health_monitoring_loop(self) -> None:
        """Background health monitoring loop."""
        logger.info("Started health monitoring loop")
        
        while self.is_running and not self.shutdown_event.is_set():
            try:
                # Perform health check
                health_status = await detailed_health_check()
                
                # Log health status
                overall_status = health_status.get("overall_status", "unknown")
                if overall_status == "critical":
                    logger.error("System health check: CRITICAL")
                elif overall_status == "warning":
                    logger.warning("System health check: WARNING")
                else:
                    logger.debug("System health check: HEALTHY")
                
                # Wait for next check
                await asyncio.sleep(self.health_check_interval)
                
            except asyncio.CancelledError:
                logger.info("Health monitoring loop cancelled")
                break
            except Exception as e:
                logger.error(f"Error in health monitoring loop: {e}")
                await asyncio.sleep(10.0)  # Back off on errors
        
        logger.info("Health monitoring loop stopped")
    
    async def _performance_logging_loop(self) -> None:
        """Background performance logging loop."""
        logger.info("Started performance logging loop")
        
        while self.is_running and not self.shutdown_event.is_set():
            try:
                # Log scheduler performance
                if self.message_scheduler and self.message_scheduler.enable_performance_monitoring:
                    metrics = self.message_scheduler.metrics.get_metrics_summary()
                    logger.info(f"Scheduler metrics: {metrics}")
                
                # Log LDPC performance
                if self.ldpc_encoder and self.ldpc_encoder.enable_performance_tracking:
                    ldpc_metrics = self.ldpc_encoder.get_performance_summary()
                    logger.info(f"LDPC metrics: {ldpc_metrics}")
                
                # Wait for next log
                await asyncio.sleep(self.performance_logging_interval)
                
            except asyncio.CancelledError:
                logger.info("Performance logging loop cancelled")
                break
            except Exception as e:
                logger.error(f"Error in performance logging loop: {e}")
                await asyncio.sleep(30.0)  # Back off on errors
        
        logger.info("Performance logging loop stopped")
    
    async def demo_message_processing(self) -> None:
        """Demonstrate message processing capabilities."""
        if not self.message_scheduler:
            logger.warning("Message scheduler not initialized")
            return
        
        logger.info("Starting message processing demonstration...")
        
        try:
            # Create demo messages
            demo_messages = [
                Message(
                    message_id="critical_001",
                    content="Emergency system alert",
                    priority=MessagePriority.CRITICAL,
                    bandwidth_required=1000,
                    processing_time_estimate=0.001,
                    time_constraints=TimeConstraints(max_latency_ms=1.0)
                ),
                Message(
                    message_id="high_001", 
                    content="Navigation update",
                    priority=MessagePriority.HIGH,
                    bandwidth_required=500,
                    processing_time_estimate=0.005,
                    time_constraints=TimeConstraints(max_latency_ms=10.0)
                ),
                Message(
                    message_id="medium_001",
                    content="Science data packet",
                    priority=MessagePriority.MEDIUM,
                    bandwidth_required=2000,
                    processing_time_estimate=0.010,
                    time_constraints=TimeConstraints(max_latency_ms=50.0)
                ),
                Message(
                    message_id="low_001",
                    content="Housekeeping telemetry",
                    priority=MessagePriority.LOW,
                    bandwidth_required=100,
                    processing_time_estimate=0.020,
                    time_constraints=TimeConstraints(max_latency_ms=1000.0)
                )
            ]
            
            # Add messages to scheduler
            for message in demo_messages:
                success = await self.message_scheduler.add_message(message)
                if success:
                    logger.info(f"Added demo message: {message.message_id}")
                else:
                    logger.warning(f"Failed to add demo message: {message.message_id}")
            
            # Let messages process for a while
            await asyncio.sleep(5.0)
            
            # Check queue status
            status = await self.message_scheduler.get_queue_status()
            logger.info(f"Queue status after demo: {status}")
            
        except Exception as e:
            logger.error(f"Error in demo message processing: {e}")
    
    async def demo_error_correction(self) -> None:
        """Demonstrate error correction capabilities."""
        if not self.ldpc_encoder:
            logger.warning("LDPC encoder not initialized")
            return
        
        logger.info("Starting error correction demonstration...")
        
        try:
            # Create test data
            test_data = b"Hello, Space Communication System! This is a test message for LDPC encoding."
            logger.info(f"Original data: {test_data}")
            
            # Encode data
            encoded_data, metadata = await self.ldpc_encoder.encode(test_data)
            logger.info(f"Encoded data length: {len(encoded_data)} bits")
            
            # Simulate channel errors
            corrupted_data = await self.ldpc_encoder.simulate_channel_errors(
                encoded_data, 
                error_rate=0.05,  # 5% error rate
                burst_probability=0.1,
                burst_length=3
            )
            
            # Decode corrupted data
            result = await self.ldpc_encoder.decode(corrupted_data, metadata)
            
            if result.success:
                # Convert back to original format
                decoded_bytes = bytes(np.packbits(result.corrected_data))
                # Trim to original length
                original_length = len(test_data)
                decoded_bytes = decoded_bytes[:original_length]
                
                logger.info(f"Decoded data: {decoded_bytes}")
                logger.info(f"Decoding successful: {result.success}")
                logger.info(f"Iterations used: {result.iterations_used}")
                logger.info(f"Bit error rate: {result.bit_error_rate:.4f}")
                logger.info(f"Decoding time: {result.decoding_time_ms:.2f}ms")
                
                # Verify correctness
                if decoded_bytes == test_data:
                    logger.info("✅ Error correction successful - data recovered perfectly!")
                else:
                    logger.warning("⚠️ Error correction partial - some errors remain")
            else:
                logger.error("❌ Error correction failed")
            
        except Exception as e:
            logger.error(f"Error in demo error correction: {e}")
    
    async def run_main_loop(self) -> None:
        """Run the main application loop."""
        logger.info("Starting main application loop...")
        
        try:
            # Run demonstrations
            await self.demo_message_processing()
            await self.demo_error_correction()
            
            # Main loop - wait for shutdown signal
            logger.info("Application running - waiting for shutdown signal...")
            await self.shutdown_event.wait()
            
        except KeyboardInterrupt:
            logger.info("Received keyboard interrupt")
        except Exception as e:
            logger.error(f"Error in main loop: {e}")
        finally:
            logger.info("Main loop ended")
    
    async def graceful_shutdown(self) -> None:
        """Perform graceful shutdown of all components."""
        logger.info("Starting graceful shutdown...")
        
        # Signal shutdown
        self.is_running = False
        self.shutdown_event.set()
        
        try:
            # Shutdown message scheduler
            if self.message_scheduler:
                await self.message_scheduler.graceful_shutdown(timeout_seconds=30.0)
            
            # Cancel monitoring tasks
            for task in self.cleanup_tasks:
                if not task.done():
                    task.cancel()
            
            # Wait for tasks to complete
            if self.cleanup_tasks:
                await asyncio.gather(*self.cleanup_tasks, return_exceptions=True)
            
            logger.info("Graceful shutdown completed")
            
        except Exception as e:
            logger.error(f"Error during shutdown: {e}")


def load_configuration(config_path: Optional[str] = None) -> Dict[str, Any]:
    """Load application configuration from file or use defaults.
    
    Args:
        config_path: Path to configuration file
        
    Returns:
        Configuration dictionary
    """
    default_config = {
        "message_scheduler": {
            "max_bandwidth": 50000,
            "max_queue_size": 10000,
            "enable_adaptive_scheduling": True,
            "enable_performance_monitoring": True,
            "memory_limit_mb": 512
        },
        "ldpc_encoder": {
            "code_rate": 0.5,
            "block_length": 1024,
            "max_iterations": 50,
            "enable_adaptive_mode": True,
            "enable_performance_tracking": True,
            "memory_limit_mb": 256
        },
        "enable_health_monitoring": True,
        "enable_performance_logging": True,
        "health_check_interval": 30.0,
        "performance_logging_interval": 60.0,
        "log_level": "INFO"
    }
    
    if config_path and Path(config_path).exists():
        try:
            with open(config_path, 'r') as f:
                file_config = yaml.safe_load(f)
                # Merge configurations (file config overrides defaults)
                default_config.update(file_config)
                logger.info(f"Loaded configuration from {config_path}")
        except Exception as e:
            logger.warning(f"Failed to load config file {config_path}: {e}")
            logger.info("Using default configuration")
    else:
        logger.info("Using default configuration")
    
    return default_config


def setup_signal_handlers(app: SpaceCommApplication) -> None:
    """Setup signal handlers for graceful shutdown."""
    def signal_handler(signum, frame):
        logger.info(f"Received signal {signum}")
        # Set the shutdown event
        asyncio.create_task(app.graceful_shutdown())
    
    # Register signal handlers
    signal.signal(signal.SIGTERM, signal_handler)
    signal.signal(signal.SIGINT, signal_handler)


async def main(config_path: Optional[str] = None, mode: str = "demo") -> None:
    """Main application entry point.
    
    Args:
        config_path: Path to configuration file
        mode: Application mode (demo, production, test)
    """
    logger.info(f"Starting Space Data Communication System in {mode} mode")
    
    try:
        # Load configuration
        config = load_configuration(config_path)
        
        # Set logging level
        log_level = config.get("log_level", "INFO")
        logging.getLogger().setLevel(getattr(logging, log_level))
        
        # Create application instance
        app = SpaceCommApplication(config)
        
        # Setup signal handlers
        setup_signal_handlers(app)
        
        # Initialize and start
        await app.initialize_components()
        await app.start_services()
        
        # Run main loop
        await app.run_main_loop()
        
    except Exception as e:
        logger.error(f"Application failed: {e}")
        sys.exit(1)
    
    logger.info("Space Data Communication System shutdown complete")


def cli_main() -> None:
    """Command line interface entry point."""
    parser = argparse.ArgumentParser(
        description="NASA-compliant Space Data Communication Analysis System"
    )
    parser.add_argument(
        "--config", 
        type=str, 
        help="Path to configuration file"
    )
    parser.add_argument(
        "--mode",
        choices=["demo", "production", "test"],
        default="demo",
        help="Application mode"
    )
    parser.add_argument(
        "--log-level",
        choices=["DEBUG", "INFO", "WARNING", "ERROR"],
        default="INFO",
        help="Logging level"
    )
    
    args = parser.parse_args()
    
    # Set logging level from command line
    logging.getLogger().setLevel(getattr(logging, args.log_level))
    
    # Run application
    try:
        asyncio.run(main(config_path=args.config, mode=args.mode))
    except KeyboardInterrupt:
        logger.info("Application interrupted by user")
    except Exception as e:
        logger.error(f"Application error: {e}")
        sys.exit(1)


if __name__ == "__main__":
    cli_main()
