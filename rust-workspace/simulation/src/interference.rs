//! Interference and Noise Simulation Module
//!
//! This module simulates various sources of interference and noise that affect
//! satellite communication systems, including thermal noise, co-channel interference,
//! adjacent channel interference, and intentional jamming.

use rand::Rng;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

/// Types of interference sources
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InterferenceType {
    ThermalNoise,
    CoChannelInterference,
    AdjacentChannelInterference,
    IntermodulationDistortion,
    AtmosphericNoise,
    GalacticNoise,
    ManMadeNoise,
    Jamming,
    MultiPath,
    PhaseNoise,
}

/// Interference source characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterferenceSource {
    pub source_type: InterferenceType,
    pub power_dbm: f64,
    pub frequency_ghz: f64,
    pub bandwidth_mhz: f64,
    pub location: SourceLocation,
    pub temporal_pattern: TemporalPattern,
    pub spectral_characteristics: SpectralCharacteristics,
}

/// Location of interference source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SourceLocation {
    Terrestrial {
        distance_km: f64,
        azimuth_deg: f64,
    },
    Satellite {
        orbital_slot_deg: f64,
        inclination_deg: f64,
    },
    Atmospheric {
        altitude_km: f64,
    },
    Galactic,
}

/// Temporal behavior patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemporalPattern {
    Continuous,
    Intermittent {
        duty_cycle: f64,
        period_seconds: f64,
    },
    Burst {
        burst_duration_ms: f64,
        inter_burst_interval_s: f64,
    },
    Random {
        mean_duration_s: f64,
        occurrence_rate_hz: f64,
    },
}

/// Spectral characteristics of interference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpectralCharacteristics {
    pub spectral_shape: SpectralShape,
    pub peak_frequency_ghz: f64,
    pub bandwidth_3db_mhz: f64,
    pub out_of_band_rejection_db: f64,
    pub phase_noise_profile: PhaseNoiseProfile,
}

/// Spectral shape models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpectralShape {
    Gaussian,
    Rectangular,
    RaisedCosine { rolloff_factor: f64 },
    Chirp { chirp_rate_hz_s: f64 },
    MultiTone { tone_frequencies: Vec<f64> },
}

/// Phase noise characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseNoiseProfile {
    pub close_in_db_hz: f64,          // Phase noise at 100 Hz offset
    pub white_noise_floor_db_hz: f64, // White noise floor
    pub flicker_corner_hz: f64,       // 1/f noise corner frequency
}

/// Comprehensive noise and interference calculator
pub struct NoiseCalculator;

impl NoiseCalculator {
    /// Calculate thermal noise power
    pub fn calculate_thermal_noise(
        temperature_k: f64,
        bandwidth_hz: f64,
        noise_figure_db: f64,
    ) -> f64 {
        const BOLTZMANN_CONSTANT: f64 = 1.380649e-23; // J/K

        // Thermal noise power in watts
        let thermal_noise_watts = BOLTZMANN_CONSTANT * temperature_k * bandwidth_hz;

        // Convert to dBm
        let thermal_noise_dbm = 10.0 * (thermal_noise_watts * 1000.0).log10();

        // Add noise figure
        thermal_noise_dbm + noise_figure_db
    }

    /// Calculate atmospheric noise temperature
    pub fn calculate_atmospheric_noise_temperature(
        frequency_ghz: f64,
        elevation_angle_deg: f64,
        weather_conditions: &crate::atmospheric::AtmosphericConditions,
    ) -> f64 {
        // Base atmospheric temperature
        let base_temp_k = 290.0;

        // Frequency dependence (higher frequencies have lower atmospheric noise)
        let frequency_factor = 1.0 / (1.0 + frequency_ghz / 10.0);

        // Elevation angle dependence (lower angles see more atmosphere)
        let elevation_factor = 1.0 / elevation_angle_deg.to_radians().sin();

        // Weather impact (rain and clouds increase noise temperature)
        let weather_factor = 1.0
            + weather_conditions.rain_rate_mm_hour / 10.0
            + weather_conditions.cloud_density * 0.2;

        base_temp_k * frequency_factor * elevation_factor * weather_factor
    }

    /// Calculate galactic noise temperature
    pub fn calculate_galactic_noise_temperature(
        frequency_ghz: f64,
        galactic_latitude_deg: f64,
    ) -> f64 {
        // Galactic noise follows approximately T ∝ f^(-2.5)
        let frequency_mhz = frequency_ghz * 1000.0;
        let base_temp = 2.725 * (frequency_mhz / 150.0).powf(-2.5);

        // Galactic latitude dependence (minimum at poles, maximum at galactic center)
        let latitude_factor = 1.0 + 10.0 * (-galactic_latitude_deg.abs() / 30.0).exp();

        base_temp * latitude_factor
    }

    /// Calculate co-channel interference
    pub fn calculate_co_channel_interference(
        interferer_power_dbm: f64,
        path_loss_db: f64,
        antenna_discrimination_db: f64,
    ) -> f64 {
        interferer_power_dbm - path_loss_db - antenna_discrimination_db
    }

    /// Calculate adjacent channel interference
    pub fn calculate_adjacent_channel_interference(
        interferer_power_dbm: f64,
        frequency_separation_mhz: f64,
        filter_selectivity_db_mhz: f64,
    ) -> f64 {
        let spectral_separation_loss = frequency_separation_mhz * filter_selectivity_db_mhz;
        interferer_power_dbm - spectral_separation_loss
    }
}

/// Jamming simulation models
pub struct JammingSimulator;

impl JammingSimulator {
    /// Simulate barrage jamming (wideband noise)
    pub fn simulate_barrage_jamming(
        jammer_power_dbm: f64,
        jammer_bandwidth_mhz: f64,
        signal_bandwidth_mhz: f64,
        range_km: f64,
    ) -> f64 {
        // Free space path loss
        let path_loss_db = 20.0 * (4.0 * PI * range_km * 1000.0 / 299_792_458.0).log10();

        // Spectral power density reduction
        let spectral_density_reduction =
            10.0 * (jammer_bandwidth_mhz / signal_bandwidth_mhz).log10();

        jammer_power_dbm - path_loss_db - spectral_density_reduction
    }

    /// Simulate spot jamming (narrowband, targeted)
    pub fn simulate_spot_jamming(
        jammer_power_dbm: f64,
        frequency_accuracy_hz: f64,
        signal_bandwidth_hz: f64,
        range_km: f64,
    ) -> f64 {
        // Free space path loss
        let path_loss_db = 20.0 * (4.0 * PI * range_km * 1000.0 / 299_792_458.0).log10();

        // Frequency accuracy factor (perfect spot jamming vs. offset)
        let frequency_accuracy_factor = if frequency_accuracy_hz < signal_bandwidth_hz / 2.0 {
            0.0 // Perfect targeting
        } else {
            10.0 * (frequency_accuracy_hz / signal_bandwidth_hz).log10()
        };

        jammer_power_dbm - path_loss_db - frequency_accuracy_factor
    }

    /// Simulate sweep jamming
    pub fn simulate_sweep_jamming(
        jammer_power_dbm: f64,
        sweep_bandwidth_mhz: f64,
        sweep_rate_mhz_s: f64,
        signal_bandwidth_mhz: f64,
        range_km: f64,
    ) -> f64 {
        // Free space path loss
        let path_loss_db = 20.0 * (4.0 * PI * range_km * 1000.0 / 299_792_458.0).log10();

        // Dwell time on target frequency
        let dwell_time_s = signal_bandwidth_mhz / sweep_rate_mhz_s;

        // Duty cycle (fraction of time jamming is effective)
        let duty_cycle = (signal_bandwidth_mhz / sweep_bandwidth_mhz).min(1.0);

        // Time-averaged jamming power
        let time_averaged_reduction = 10.0 * duty_cycle.log10();

        jammer_power_dbm - path_loss_db + time_averaged_reduction
    }

    /// Generate jamming scenarios
    pub fn generate_jamming_scenarios() -> Vec<(String, InterferenceSource)> {
        vec![
            (
                "Barrage Jammer".to_string(),
                InterferenceSource {
                    source_type: InterferenceType::Jamming,
                    power_dbm: 40.0,
                    frequency_ghz: 10.0,
                    bandwidth_mhz: 1000.0,
                    location: SourceLocation::Terrestrial {
                        distance_km: 500.0,
                        azimuth_deg: 45.0,
                    },
                    temporal_pattern: TemporalPattern::Continuous,
                    spectral_characteristics: SpectralCharacteristics {
                        spectral_shape: SpectralShape::Gaussian,
                        peak_frequency_ghz: 10.0,
                        bandwidth_3db_mhz: 1000.0,
                        out_of_band_rejection_db: 20.0,
                        phase_noise_profile: PhaseNoiseProfile {
                            close_in_db_hz: -80.0,
                            white_noise_floor_db_hz: -140.0,
                            flicker_corner_hz: 10000.0,
                        },
                    },
                },
            ),
            (
                "Spot Jammer".to_string(),
                InterferenceSource {
                    source_type: InterferenceType::Jamming,
                    power_dbm: 50.0,
                    frequency_ghz: 12.0,
                    bandwidth_mhz: 10.0,
                    location: SourceLocation::Terrestrial {
                        distance_km: 200.0,
                        azimuth_deg: 90.0,
                    },
                    temporal_pattern: TemporalPattern::Intermittent {
                        duty_cycle: 0.8,
                        period_seconds: 10.0,
                    },
                    spectral_characteristics: SpectralCharacteristics {
                        spectral_shape: SpectralShape::Rectangular,
                        peak_frequency_ghz: 12.0,
                        bandwidth_3db_mhz: 10.0,
                        out_of_band_rejection_db: 60.0,
                        phase_noise_profile: PhaseNoiseProfile {
                            close_in_db_hz: -90.0,
                            white_noise_floor_db_hz: -150.0,
                            flicker_corner_hz: 1000.0,
                        },
                    },
                },
            ),
            (
                "Sweep Jammer".to_string(),
                InterferenceSource {
                    source_type: InterferenceType::Jamming,
                    power_dbm: 45.0,
                    frequency_ghz: 8.0,
                    bandwidth_mhz: 2000.0,
                    location: SourceLocation::Terrestrial {
                        distance_km: 800.0,
                        azimuth_deg: 180.0,
                    },
                    temporal_pattern: TemporalPattern::Continuous,
                    spectral_characteristics: SpectralCharacteristics {
                        spectral_shape: SpectralShape::Chirp {
                            chirp_rate_hz_s: 1e6,
                        },
                        peak_frequency_ghz: 8.0,
                        bandwidth_3db_mhz: 2000.0,
                        out_of_band_rejection_db: 30.0,
                        phase_noise_profile: PhaseNoiseProfile {
                            close_in_db_hz: -85.0,
                            white_noise_floor_db_hz: -145.0,
                            flicker_corner_hz: 5000.0,
                        },
                    },
                },
            ),
        ]
    }
}

/// Multi-path propagation simulator
pub struct MultiPathSimulator;

impl MultiPathSimulator {
    /// Calculate multi-path fading characteristics
    pub fn calculate_multipath_fading(
        frequency_ghz: f64,
        path_differences_m: &[f64],
        path_powers_db: &[f64],
    ) -> MultiPathResult {
        assert_eq!(path_differences_m.len(), path_powers_db.len());

        let wavelength_m = 299_792_458.0 / (frequency_ghz * 1e9);

        // Calculate complex amplitude sum
        let mut real_sum = 0.0;
        let mut imag_sum = 0.0;

        for (i, (&path_diff, &power_db)) in path_differences_m
            .iter()
            .zip(path_powers_db.iter())
            .enumerate()
        {
            let amplitude = 10.0_f64.powf(power_db / 20.0);
            let phase = 2.0 * PI * path_diff / wavelength_m;

            real_sum += amplitude * phase.cos();
            imag_sum += amplitude * phase.sin();
        }

        let resultant_amplitude = (real_sum.powi(2) + imag_sum.powi(2)).sqrt();
        let resultant_power_db = 20.0 * resultant_amplitude.log10();
        let phase_shift_deg = imag_sum.atan2(real_sum).to_degrees();

        // Calculate fade depth and enhancement
        let direct_power_db = path_powers_db[0]; // Assume first path is direct
        let fade_depth_db = direct_power_db - resultant_power_db;

        MultiPathResult {
            resultant_power_db,
            fade_depth_db,
            phase_shift_deg,
            amplitude_variation: resultant_amplitude,
        }
    }

    /// Generate realistic multi-path scenarios
    pub fn generate_multipath_scenarios() -> Vec<(String, Vec<f64>, Vec<f64>)> {
        vec![
            (
                "Urban Environment".to_string(),
                vec![0.0, 50.0, 150.0, 300.0],
                vec![0.0, -6.0, -12.0, -18.0],
            ),
            (
                "Mountainous Terrain".to_string(),
                vec![0.0, 200.0, 800.0],
                vec![0.0, -3.0, -10.0],
            ),
            (
                "Ocean Reflection".to_string(),
                vec![0.0, 100.0],
                vec![0.0, -4.0],
            ),
            (
                "Building Reflection".to_string(),
                vec![0.0, 25.0, 75.0, 125.0, 200.0],
                vec![0.0, -8.0, -15.0, -20.0, -25.0],
            ),
        ]
    }
}

/// Result of multi-path calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiPathResult {
    pub resultant_power_db: f64,
    pub fade_depth_db: f64,
    pub phase_shift_deg: f64,
    pub amplitude_variation: f64,
}

/// Phase noise simulator
pub struct PhaseNoiseSimulator;

impl PhaseNoiseSimulator {
    /// Generate phase noise time series
    pub fn generate_phase_noise_series(
        duration_seconds: f64,
        sampling_rate_hz: f64,
        phase_noise_profile: &PhaseNoiseProfile,
    ) -> Vec<f64> {
        let num_samples = (duration_seconds * sampling_rate_hz) as usize;
        let mut rng = rand::thread_rng();
        let mut phase_series = Vec::with_capacity(num_samples);

        // Generate colored noise based on phase noise profile
        let mut accumulated_phase = 0.0;

        for i in 0..num_samples {
            let frequency_offset = (i as f64) * sampling_rate_hz / (num_samples as f64);

            // Calculate phase noise power spectral density
            let phase_noise_psd = if frequency_offset < phase_noise_profile.flicker_corner_hz {
                // 1/f noise region
                phase_noise_profile.close_in_db_hz - 10.0 * frequency_offset.log10()
            } else {
                // White noise floor
                phase_noise_profile.white_noise_floor_db_hz
            };

            // Convert PSD to time domain noise
            let noise_power = 10.0_f64.powf(phase_noise_psd / 10.0);
            let phase_increment = rng.gen_range(-1.0..1.0) * noise_power.sqrt();

            accumulated_phase += phase_increment;
            phase_series.push(accumulated_phase);
        }

        phase_series
    }

    /// Calculate phase noise impact on signal quality
    pub fn calculate_phase_noise_impact(
        phase_noise_rms_degrees: f64,
        modulation_type: ModulationType,
    ) -> f64 {
        let phase_noise_rms_radians = phase_noise_rms_degrees.to_radians();

        // Calculate SNR degradation due to phase noise
        match modulation_type {
            ModulationType::BPSK => {
                // For BPSK, phase noise mainly affects timing
                -20.0 * phase_noise_rms_radians.log10()
            }
            ModulationType::QPSK => {
                // For QPSK, phase noise affects both I and Q channels
                -10.0 * (phase_noise_rms_radians.powi(2) / 2.0).log10()
            }
            ModulationType::PSK8 => {
                // Higher order PSK is more sensitive to phase noise
                -10.0 * (phase_noise_rms_radians.powi(2) / 0.5).log10()
            }
            ModulationType::QAM16 => {
                // QAM is sensitive to both phase and amplitude noise
                -10.0 * (phase_noise_rms_radians.powi(2) / 0.25).log10()
            }
            ModulationType::QAM64 => {
                // Higher order QAM is very sensitive
                -10.0 * (phase_noise_rms_radians.powi(2) / 0.1).log10()
            }
        }
    }
}

/// Modulation types for phase noise analysis
#[derive(Debug, Clone, Copy)]
pub enum ModulationType {
    BPSK,
    QPSK,
    PSK8,
    QAM16,
    QAM64,
}

/// Comprehensive interference analyzer
pub struct InterferenceAnalyzer;

impl InterferenceAnalyzer {
    /// Calculate total interference power
    pub fn calculate_total_interference(
        interference_sources: &[InterferenceSource],
        receiver_frequency_ghz: f64,
        receiver_bandwidth_mhz: f64,
        receiver_location: (f64, f64), // (lat, lon)
    ) -> f64 {
        let mut total_interference_linear = 0.0;

        for source in interference_sources {
            let interference_power = self::calculate_source_interference(
                source,
                receiver_frequency_ghz,
                receiver_bandwidth_mhz,
                receiver_location,
            );

            // Convert dB to linear and sum
            total_interference_linear += 10.0_f64.powf(interference_power / 10.0);
        }

        // Convert back to dB
        10.0 * total_interference_linear.log10()
    }

    /// Calculate signal-to-interference ratio
    pub fn calculate_sir(
        signal_power_dbm: f64,
        interference_sources: &[InterferenceSource],
        receiver_frequency_ghz: f64,
        receiver_bandwidth_mhz: f64,
        receiver_location: (f64, f64),
    ) -> f64 {
        let total_interference = Self::calculate_total_interference(
            interference_sources,
            receiver_frequency_ghz,
            receiver_bandwidth_mhz,
            receiver_location,
        );

        signal_power_dbm - total_interference
    }

    /// Generate interference scenarios for testing
    pub fn generate_interference_scenarios() -> Vec<Vec<InterferenceSource>> {
        vec![
            // Low interference scenario
            vec![InterferenceSource {
                source_type: InterferenceType::ThermalNoise,
                power_dbm: -110.0,
                frequency_ghz: 10.0,
                bandwidth_mhz: 100.0,
                location: SourceLocation::Atmospheric { altitude_km: 50.0 },
                temporal_pattern: TemporalPattern::Continuous,
                spectral_characteristics: SpectralCharacteristics {
                    spectral_shape: SpectralShape::Gaussian,
                    peak_frequency_ghz: 10.0,
                    bandwidth_3db_mhz: 100.0,
                    out_of_band_rejection_db: 40.0,
                    phase_noise_profile: PhaseNoiseProfile {
                        close_in_db_hz: -120.0,
                        white_noise_floor_db_hz: -160.0,
                        flicker_corner_hz: 100.0,
                    },
                },
            }],
            // High interference scenario
            JammingSimulator::generate_jamming_scenarios()
                .into_iter()
                .map(|(_, source)| source)
                .collect(),
            // Mixed interference scenario
            vec![
                // Add various interference sources...
            ],
        ]
    }
}

/// Calculate interference from a specific source
fn calculate_source_interference(
    source: &InterferenceSource,
    receiver_frequency_ghz: f64,
    receiver_bandwidth_mhz: f64,
    receiver_location: (f64, f64),
) -> f64 {
    // Calculate path loss based on source location
    let path_loss_db = match &source.location {
        SourceLocation::Terrestrial { distance_km, .. } => {
            20.0 * (4.0 * PI * distance_km * 1000.0 / 299_792_458.0).log10()
        }
        SourceLocation::Satellite { .. } => {
            // Typical satellite path loss
            200.0
        }
        SourceLocation::Atmospheric { altitude_km } => {
            // Atmospheric path calculation
            20.0 * (altitude_km * 1000.0).log10()
        }
        SourceLocation::Galactic => {
            // Galactic sources are very weak
            300.0
        }
    };

    // Calculate spectral overlap
    let frequency_separation = (source.frequency_ghz - receiver_frequency_ghz).abs() * 1000.0; // MHz
    let spectral_overlap = if frequency_separation < receiver_bandwidth_mhz / 2.0 {
        0.0 // Full overlap
    } else {
        // Calculate roll-off based on spectral shape
        match source.spectral_characteristics.spectral_shape {
            SpectralShape::Gaussian => {
                let sigma = source.bandwidth_mhz / 2.35; // Convert FWHM to sigma
                -10.0 * (frequency_separation / sigma).powi(2) / 2.0
            }
            SpectralShape::Rectangular => {
                if frequency_separation > source.bandwidth_mhz / 2.0 {
                    -source.spectral_characteristics.out_of_band_rejection_db
                } else {
                    0.0
                }
            }
            _ => -20.0 * frequency_separation.log10(), // Default roll-off
        }
    };

    source.power_dbm - path_loss_db + spectral_overlap
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_thermal_noise_calculation() {
        let noise_power = NoiseCalculator::calculate_thermal_noise(
            290.0, // Room temperature
            1e6,   // 1 MHz bandwidth
            3.0,   // 3 dB noise figure
        );

        // Should be around -111 dBm for 1 MHz at room temperature
        assert!((noise_power - (-111.0)).abs() < 5.0);
    }

    #[test]
    fn test_jamming_simulation() {
        let jammer_power = JammingSimulator::simulate_barrage_jamming(
            40.0,   // 40 dBm jammer power
            1000.0, // 1 GHz jammer bandwidth
            100.0,  // 100 MHz signal bandwidth
            100.0,  // 100 km range
        );

        assert!(jammer_power < 40.0); // Should be less than transmitted power
        assert!(jammer_power > -50.0); // Should be detectable
    }

    #[test]
    fn test_multipath_calculation() {
        let path_differences = vec![0.0, 100.0]; // Direct and one reflection
        let path_powers = vec![0.0, -6.0]; // Reflection 6 dB weaker

        let result = MultiPathSimulator::calculate_multipath_fading(
            10.0, // 10 GHz
            &path_differences,
            &path_powers,
        );

        assert!(result.fade_depth_db.abs() < 20.0); // Reasonable fade depth
    }

    #[test]
    fn test_phase_noise_generation() {
        let profile = PhaseNoiseProfile {
            close_in_db_hz: -80.0,
            white_noise_floor_db_hz: -140.0,
            flicker_corner_hz: 1000.0,
        };

        let phase_series = PhaseNoiseSimulator::generate_phase_noise_series(
            1.0,    // 1 second
            1000.0, // 1 kHz sampling
            &profile,
        );

        assert_eq!(phase_series.len(), 1000);
    }
}
