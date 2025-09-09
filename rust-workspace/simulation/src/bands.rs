//! Frequency Band Simulation Module
//!
//! This module provides realistic simulation of different frequency bands used in
//! satellite-ground communication systems, including atmospheric effects, interference,
//! and band-specific characteristics.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a frequency band with its characteristics and capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrequencyBand {
    pub name: BandType,
    pub frequency_range: FrequencyRange,
    pub purpose: BandPurpose,
    pub characteristics: BandCharacteristics,
    pub limitations: BandLimitations,
}

/// Types of frequency bands supported in the simulation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BandType {
    KBand,
    KaBand,
    SBand,
    XBand,
    UHFBand,
    LBand,
    CBand,
}

/// Frequency range for a band (in GHz)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrequencyRange {
    pub min_ghz: f64,
    pub max_ghz: f64,
    pub center_ghz: f64,
}

/// Primary purpose and use cases for each band
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BandPurpose {
    HighSpeedData,
    UltraHighBandwidth,
    TelemetryTrackingCommand,
    MediumSpeedData,
    EmergencyBackup,
    Navigation,
    CommercialSatellite,
}

/// Performance characteristics of a frequency band
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandCharacteristics {
    pub max_data_rate_mbps: f64,
    pub typical_data_rate_mbps: f64,
    pub base_latency_ms: f64,
    pub power_efficiency: f64, // Watts per Mbps
    pub antenna_gain_db: f64,
    pub beam_width_degrees: f64,
    pub penetration_capability: f64, // 0.0 to 1.0
}

/// Limitations and challenges for each band
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandLimitations {
    pub atmospheric_attenuation_db_km: f64,
    pub rain_fade_susceptibility: f64,   // 0.0 to 1.0
    pub ionospheric_effects: f64,        // 0.0 to 1.0
    pub interference_level: f64,         // 0.0 to 1.0
    pub pointing_accuracy_required: f64, // degrees
    pub weather_dependence: f64,         // 0.0 to 1.0
}

/// Result of a band transmission simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransmissionResult {
    pub band_type: BandType,
    pub success: bool,
    pub actual_data_rate_mbps: f64,
    pub total_latency_ms: f64,
    pub signal_strength_db: f64,
    pub error_rate: f64,
    pub power_consumption_watts: f64,
    pub weather_impact_factor: f64,
    pub interference_level: f64,
    pub transmission_efficiency: f64,
}

/// Environmental conditions affecting transmission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentalConditions {
    pub rain_rate_mm_hour: f64,
    pub cloud_cover_percent: f64,
    pub atmospheric_pressure_mb: f64,
    pub temperature_celsius: f64,
    pub humidity_percent: f64,
    pub ionospheric_activity: f64, // 0.0 to 1.0
    pub solar_activity: f64,       // 0.0 to 1.0
}

/// Transmission parameters for simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransmissionParameters {
    pub distance_km: f64,
    pub data_size_mb: f64,
    pub required_data_rate_mbps: f64,
    pub elevation_angle_degrees: f64,
    pub transmit_power_watts: f64,
    pub antenna_diameter_meters: f64,
}

impl FrequencyBand {
    /// Create a new frequency band with specified characteristics
    pub fn new(
        name: BandType,
        frequency_range: FrequencyRange,
        purpose: BandPurpose,
        characteristics: BandCharacteristics,
        limitations: BandLimitations,
    ) -> Self {
        Self {
            name,
            frequency_range,
            purpose,
            characteristics,
            limitations,
        }
    }

    /// Get predefined frequency band configurations
    pub fn get_standard_bands() -> Vec<FrequencyBand> {
        vec![
            // K-Band (20-30 GHz)
            FrequencyBand::new(
                BandType::KBand,
                FrequencyRange {
                    min_ghz: 20.0,
                    max_ghz: 30.0,
                    center_ghz: 25.0,
                },
                BandPurpose::HighSpeedData,
                BandCharacteristics {
                    max_data_rate_mbps: 10_000.0,
                    typical_data_rate_mbps: 2_000.0,
                    base_latency_ms: 1.0,
                    power_efficiency: 0.1,
                    antenna_gain_db: 45.0,
                    beam_width_degrees: 1.5,
                    penetration_capability: 0.3,
                },
                BandLimitations {
                    atmospheric_attenuation_db_km: 0.5,
                    rain_fade_susceptibility: 0.8,
                    ionospheric_effects: 0.1,
                    interference_level: 0.3,
                    pointing_accuracy_required: 0.1,
                    weather_dependence: 0.7,
                },
            ),
            // Ka-Band (26.5-40 GHz)
            FrequencyBand::new(
                BandType::KaBand,
                FrequencyRange {
                    min_ghz: 26.5,
                    max_ghz: 40.0,
                    center_ghz: 33.25,
                },
                BandPurpose::UltraHighBandwidth,
                BandCharacteristics {
                    max_data_rate_mbps: 50_000.0,
                    typical_data_rate_mbps: 10_000.0,
                    base_latency_ms: 0.5,
                    power_efficiency: 0.15,
                    antenna_gain_db: 50.0,
                    beam_width_degrees: 1.0,
                    penetration_capability: 0.2,
                },
                BandLimitations {
                    atmospheric_attenuation_db_km: 0.8,
                    rain_fade_susceptibility: 0.95,
                    ionospheric_effects: 0.05,
                    interference_level: 0.2,
                    pointing_accuracy_required: 0.05,
                    weather_dependence: 0.9,
                },
            ),
            // S-Band (2-4 GHz)
            FrequencyBand::new(
                BandType::SBand,
                FrequencyRange {
                    min_ghz: 2.0,
                    max_ghz: 4.0,
                    center_ghz: 3.0,
                },
                BandPurpose::TelemetryTrackingCommand,
                BandCharacteristics {
                    max_data_rate_mbps: 100.0,
                    typical_data_rate_mbps: 20.0,
                    base_latency_ms: 10.0,
                    power_efficiency: 0.05,
                    antenna_gain_db: 25.0,
                    beam_width_degrees: 10.0,
                    penetration_capability: 0.9,
                },
                BandLimitations {
                    atmospheric_attenuation_db_km: 0.1,
                    rain_fade_susceptibility: 0.2,
                    ionospheric_effects: 0.7,
                    interference_level: 0.6,
                    pointing_accuracy_required: 1.0,
                    weather_dependence: 0.2,
                },
            ),
            // X-Band (8-12 GHz)
            FrequencyBand::new(
                BandType::XBand,
                FrequencyRange {
                    min_ghz: 8.0,
                    max_ghz: 12.0,
                    center_ghz: 10.0,
                },
                BandPurpose::MediumSpeedData,
                BandCharacteristics {
                    max_data_rate_mbps: 1_000.0,
                    typical_data_rate_mbps: 300.0,
                    base_latency_ms: 5.0,
                    power_efficiency: 0.08,
                    antenna_gain_db: 35.0,
                    beam_width_degrees: 3.0,
                    penetration_capability: 0.6,
                },
                BandLimitations {
                    atmospheric_attenuation_db_km: 0.2,
                    rain_fade_susceptibility: 0.4,
                    ionospheric_effects: 0.3,
                    interference_level: 0.5,
                    pointing_accuracy_required: 0.3,
                    weather_dependence: 0.4,
                },
            ),
            // UHF-Band (300 MHz - 3 GHz)
            FrequencyBand::new(
                BandType::UHFBand,
                FrequencyRange {
                    min_ghz: 0.3,
                    max_ghz: 3.0,
                    center_ghz: 1.65,
                },
                BandPurpose::EmergencyBackup,
                BandCharacteristics {
                    max_data_rate_mbps: 10.0,
                    typical_data_rate_mbps: 2.0,
                    base_latency_ms: 50.0,
                    power_efficiency: 0.02,
                    antenna_gain_db: 15.0,
                    beam_width_degrees: 30.0,
                    penetration_capability: 0.95,
                },
                BandLimitations {
                    atmospheric_attenuation_db_km: 0.05,
                    rain_fade_susceptibility: 0.1,
                    ionospheric_effects: 0.9,
                    interference_level: 0.8,
                    pointing_accuracy_required: 5.0,
                    weather_dependence: 0.1,
                },
            ),
        ]
    }

    /// Simulate transmission with given parameters and environmental conditions
    pub fn simulate_transmission(
        &self,
        params: &TransmissionParameters,
        environment: &EnvironmentalConditions,
    ) -> TransmissionResult {
        // Calculate path loss
        let path_loss_db =
            self.calculate_path_loss(params.distance_km, params.elevation_angle_degrees);

        // Calculate atmospheric attenuation
        let atmospheric_loss_db =
            self.calculate_atmospheric_attenuation(params.distance_km, environment);

        // Calculate total signal loss
        let total_loss_db = path_loss_db + atmospheric_loss_db;

        // Calculate received signal strength
        let signal_strength_db = params.transmit_power_watts.log10() * 10.0
            + self.characteristics.antenna_gain_db
            - total_loss_db;

        // Calculate achievable data rate based on signal quality
        let snr_db = signal_strength_db - self.calculate_noise_floor();
        let achievable_data_rate = self.calculate_data_rate_from_snr(snr_db);

        // Calculate actual data rate (limited by requirements and band capacity)
        let actual_data_rate = achievable_data_rate
            .min(params.required_data_rate_mbps)
            .min(self.characteristics.max_data_rate_mbps);

        // Calculate transmission time and latency
        let transmission_time_ms = (params.data_size_mb * 8.0) / actual_data_rate * 1000.0;
        let propagation_delay_ms = params.distance_km / 300.0; // Speed of light approximation
        let total_latency =
            self.characteristics.base_latency_ms + propagation_delay_ms + transmission_time_ms;

        // Calculate error rate based on signal quality and band characteristics
        let base_error_rate = 1e-9; // Base BER
        let snr_degradation = (-snr_db / 10.0).exp();
        let error_rate = base_error_rate * snr_degradation;

        // Calculate power consumption
        let power_consumption = actual_data_rate * self.characteristics.power_efficiency;

        // Calculate weather impact
        let weather_impact = self.calculate_weather_impact(environment);

        // Calculate interference level
        let interference = self.calculate_interference_level(environment);

        // Calculate transmission efficiency
        let efficiency = (actual_data_rate / self.characteristics.max_data_rate_mbps)
            * (1.0 - error_rate)
            * (1.0 - weather_impact);

        // Determine if transmission is successful
        let success = signal_strength_db > -80.0 // Minimum receivable signal
            && error_rate < 1e-3                 // Maximum acceptable error rate
            && actual_data_rate > 0.1; // Minimum data rate

        TransmissionResult {
            band_type: self.name,
            success,
            actual_data_rate_mbps: actual_data_rate,
            total_latency_ms: total_latency,
            signal_strength_db,
            error_rate,
            power_consumption_watts: power_consumption,
            weather_impact_factor: weather_impact,
            interference_level: interference,
            transmission_efficiency: efficiency,
        }
    }

    /// Calculate path loss using Friis transmission equation
    fn calculate_path_loss(&self, distance_km: f64, elevation_angle_degrees: f64) -> f64 {
        let distance_m = distance_km * 1000.0;
        let frequency_hz = self.frequency_range.center_ghz * 1e9;
        let wavelength_m = 299_792_458.0 / frequency_hz;

        // Basic path loss
        let path_loss = 20.0 * (4.0 * std::f64::consts::PI * distance_m / wavelength_m).log10();

        // Elevation angle correction (lower angles have more atmospheric loss)
        let elevation_factor = 1.0 / elevation_angle_degrees.to_radians().sin();
        let elevation_loss = 10.0 * elevation_factor.log10();

        path_loss + elevation_loss
    }

    /// Calculate atmospheric attenuation based on weather conditions
    fn calculate_atmospheric_attenuation(
        &self,
        distance_km: f64,
        environment: &EnvironmentalConditions,
    ) -> f64 {
        let base_attenuation = self.limitations.atmospheric_attenuation_db_km * distance_km;

        // Rain fade calculation (ITU-R P.618 model simplified)
        let rain_attenuation = self.calculate_rain_fade(environment.rain_rate_mm_hour);

        // Cloud attenuation
        let cloud_attenuation = environment.cloud_cover_percent / 100.0 * 0.1 * distance_km;

        // Atmospheric absorption (oxygen and water vapor)
        let absorption = self.calculate_atmospheric_absorption(
            environment.humidity_percent,
            environment.atmospheric_pressure_mb,
        ) * distance_km;

        base_attenuation + rain_attenuation + cloud_attenuation + absorption
    }

    /// Calculate rain fade using simplified ITU model
    fn calculate_rain_fade(&self, rain_rate_mm_hour: f64) -> f64 {
        if rain_rate_mm_hour <= 0.0 {
            return 0.0;
        }

        let frequency_ghz = self.frequency_range.center_ghz;

        // Simplified rain attenuation model
        let k = match frequency_ghz {
            f if f < 10.0 => 0.0001,
            f if f < 20.0 => 0.01,
            f if f < 30.0 => 0.1,
            _ => 0.5,
        };

        let alpha = match frequency_ghz {
            f if f < 10.0 => 0.8,
            f if f < 20.0 => 1.0,
            f if f < 30.0 => 1.2,
            _ => 1.5,
        };

        k * rain_rate_mm_hour.powf(alpha) * self.limitations.rain_fade_susceptibility
    }

    /// Calculate atmospheric absorption
    fn calculate_atmospheric_absorption(&self, humidity_percent: f64, pressure_mb: f64) -> f64 {
        let frequency_ghz = self.frequency_range.center_ghz;

        // Water vapor absorption peaks around 22 GHz
        let water_vapor_peak = 22.0;
        let water_vapor_absorption = humidity_percent / 100.0
            * (-((frequency_ghz - water_vapor_peak) / 5.0).powi(2)).exp()
            * 0.1;

        // Oxygen absorption peak around 60 GHz
        let oxygen_peak = 60.0;
        let oxygen_absorption =
            pressure_mb / 1013.25 * (-((frequency_ghz - oxygen_peak) / 10.0).powi(2)).exp() * 0.05;

        water_vapor_absorption + oxygen_absorption
    }

    /// Calculate noise floor for the band
    fn calculate_noise_floor(&self) -> f64 {
        // Thermal noise calculation: kTB
        let boltzmann_constant = 1.38e-23; // J/K
        let temperature_k = 290.0; // Standard noise temperature
        let bandwidth_hz = (self.frequency_range.max_ghz - self.frequency_range.min_ghz) * 1e9;

        let thermal_noise_watts = boltzmann_constant * temperature_k * bandwidth_hz;
        let thermal_noise_dbm = 10.0 * (thermal_noise_watts * 1000.0).log10();

        // Add system noise figure
        let noise_figure_db = match self.name {
            BandType::KaBand | BandType::KBand => 3.0,
            BandType::XBand => 2.5,
            BandType::SBand => 2.0,
            BandType::UHFBand => 1.5,
            _ => 2.0,
        };

        thermal_noise_dbm + noise_figure_db
    }

    /// Calculate achievable data rate from signal-to-noise ratio
    fn calculate_data_rate_from_snr(&self, snr_db: f64) -> f64 {
        let snr_linear = 10.0_f64.powf(snr_db / 10.0);

        // Shannon capacity: C = B * log2(1 + SNR)
        let bandwidth_hz = (self.frequency_range.max_ghz - self.frequency_range.min_ghz) * 1e9;
        let capacity_bps = bandwidth_hz * (1.0 + snr_linear).log2();
        let capacity_mbps = capacity_bps / 1e6;

        // Apply practical efficiency factor (typically 70-90% of Shannon limit)
        let efficiency_factor = 0.8;
        capacity_mbps * efficiency_factor
    }

    /// Calculate weather impact factor
    fn calculate_weather_impact(&self, environment: &EnvironmentalConditions) -> f64 {
        let rain_impact =
            environment.rain_rate_mm_hour / 50.0 * self.limitations.rain_fade_susceptibility;
        let cloud_impact = environment.cloud_cover_percent / 100.0 * 0.1;
        let humidity_impact = environment.humidity_percent / 100.0 * 0.05;

        (rain_impact + cloud_impact + humidity_impact).min(1.0)
            * self.limitations.weather_dependence
    }

    /// Calculate interference level
    fn calculate_interference_level(&self, environment: &EnvironmentalConditions) -> f64 {
        let base_interference = self.limitations.interference_level;
        let ionospheric_interference =
            environment.ionospheric_activity * self.limitations.ionospheric_effects;
        let solar_interference = environment.solar_activity * 0.1;

        (base_interference + ionospheric_interference + solar_interference).min(1.0)
    }
}

impl fmt::Display for BandType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BandType::KBand => write!(f, "K-Band"),
            BandType::KaBand => write!(f, "Ka-Band"),
            BandType::SBand => write!(f, "S-Band"),
            BandType::XBand => write!(f, "X-Band"),
            BandType::UHFBand => write!(f, "UHF-Band"),
            BandType::LBand => write!(f, "L-Band"),
            BandType::CBand => write!(f, "C-Band"),
        }
    }
}

impl fmt::Display for TransmissionResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Band: {} | Success: {} | Data Rate: {:.2} Mbps | Latency: {:.2} ms | Signal: {:.1} dB | Efficiency: {:.1}%",
            self.band_type,
            if self.success { "✅" } else { "❌" },
            self.actual_data_rate_mbps,
            self.total_latency_ms,
            self.signal_strength_db,
            self.transmission_efficiency * 100.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_band_creation() {
        let bands = FrequencyBand::get_standard_bands();
        assert_eq!(bands.len(), 5);

        let k_band = &bands[0];
        assert_eq!(k_band.name, BandType::KBand);
        assert_eq!(k_band.frequency_range.center_ghz, 25.0);
    }

    #[test]
    fn test_transmission_simulation() {
        let bands = FrequencyBand::get_standard_bands();
        let s_band = &bands[2]; // S-Band

        let params = TransmissionParameters {
            distance_km: 1000.0,
            data_size_mb: 100.0,
            required_data_rate_mbps: 10.0,
            elevation_angle_degrees: 45.0,
            transmit_power_watts: 100.0,
            antenna_diameter_meters: 3.0,
        };

        let environment = EnvironmentalConditions {
            rain_rate_mm_hour: 0.0,
            cloud_cover_percent: 0.0,
            atmospheric_pressure_mb: 1013.25,
            temperature_celsius: 20.0,
            humidity_percent: 50.0,
            ionospheric_activity: 0.1,
            solar_activity: 0.1,
        };

        let result = s_band.simulate_transmission(&params, &environment);
        assert!(result.success);
        assert!(result.actual_data_rate_mbps > 0.0);
        assert!(result.total_latency_ms > 0.0);
    }

    #[test]
    fn test_rain_fade_calculation() {
        let bands = FrequencyBand::get_standard_bands();
        let ka_band = &bands[1]; // Ka-Band (most susceptible to rain)
        let s_band = &bands[2]; // S-Band (least susceptible)

        let ka_rain_fade = ka_band.calculate_rain_fade(10.0); // 10 mm/hour
        let s_rain_fade = s_band.calculate_rain_fade(10.0);

        assert!(ka_rain_fade > s_rain_fade);
        assert!(ka_rain_fade > 0.0);
    }
}
