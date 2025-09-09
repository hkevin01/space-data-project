//! Frequency Band Simulation Module
//!
//! A comprehensive simulation framework for satellite-ground communication
//! across different frequency bands with realistic atmospheric effects.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Types of frequency bands
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BandType {
    KBand,
    KaBand,
    SBand,
    XBand,
    UHFBand,
}

impl std::fmt::Display for BandType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BandType::KBand => write!(f, "K-Band"),
            BandType::KaBand => write!(f, "Ka-Band"),
            BandType::SBand => write!(f, "S-Band"),
            BandType::XBand => write!(f, "X-Band"),
            BandType::UHFBand => write!(f, "UHF-Band"),
        }
    }
}

/// Frequency range for a band
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrequencyRange {
    pub min_ghz: f64,
    pub max_ghz: f64,
}

/// Band characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandCharacteristics {
    pub max_data_rate_mbps: f64,
    pub power_efficiency: f64,
    pub antenna_gain_dbi: f64,
    pub noise_temperature_k: f64,
}

/// Environmental conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentalConditions {
    pub rain_rate_mm_hour: f64,
    pub cloud_cover_percent: f64,
    pub atmospheric_pressure_mb: f64,
    pub temperature_celsius: f64,
    pub humidity_percent: f64,
    pub ionospheric_activity: f64,
    pub solar_activity: f64,
}

/// Transmission parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransmissionParameters {
    pub distance_km: f64,
    pub data_size_mb: f64,
    pub required_data_rate_mbps: f64,
    pub elevation_angle_degrees: f64,
    pub transmit_power_watts: f64,
    pub antenna_diameter_meters: f64,
}

/// Results of transmission simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransmissionResult {
    pub success: bool,
    pub actual_data_rate_mbps: f64,
    pub total_latency_ms: f64,
    pub power_consumption_watts: f64,
    pub transmission_efficiency: f64,
    pub weather_impact_factor: f64,
    pub signal_to_noise_ratio_db: f64,
    pub path_loss_db: f64,
}

/// Frequency band definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrequencyBand {
    pub name: BandType,
    pub frequency_range: FrequencyRange,
    pub characteristics: BandCharacteristics,
}

impl FrequencyBand {
    /// Get standard space communication frequency bands
    pub fn get_standard_bands() -> Vec<FrequencyBand> {
        vec![
            // K-Band (20-30 GHz)
            FrequencyBand {
                name: BandType::KBand,
                frequency_range: FrequencyRange {
                    min_ghz: 20.0,
                    max_ghz: 30.0,
                },
                characteristics: BandCharacteristics {
                    max_data_rate_mbps: 1000.0,
                    power_efficiency: 0.75,
                    antenna_gain_dbi: 45.0,
                    noise_temperature_k: 500.0,
                },
            },
            // Ka-Band (26.5-40 GHz)
            FrequencyBand {
                name: BandType::KaBand,
                frequency_range: FrequencyRange {
                    min_ghz: 26.5,
                    max_ghz: 40.0,
                },
                characteristics: BandCharacteristics {
                    max_data_rate_mbps: 2000.0,
                    power_efficiency: 0.8,
                    antenna_gain_dbi: 50.0,
                    noise_temperature_k: 600.0,
                },
            },
            // S-Band (2-4 GHz)
            FrequencyBand {
                name: BandType::SBand,
                frequency_range: FrequencyRange {
                    min_ghz: 2.0,
                    max_ghz: 4.0,
                },
                characteristics: BandCharacteristics {
                    max_data_rate_mbps: 100.0,
                    power_efficiency: 0.6,
                    antenna_gain_dbi: 25.0,
                    noise_temperature_k: 300.0,
                },
            },
            // X-Band (8-12 GHz)
            FrequencyBand {
                name: BandType::XBand,
                frequency_range: FrequencyRange {
                    min_ghz: 8.0,
                    max_ghz: 12.0,
                },
                characteristics: BandCharacteristics {
                    max_data_rate_mbps: 500.0,
                    power_efficiency: 0.7,
                    antenna_gain_dbi: 35.0,
                    noise_temperature_k: 400.0,
                },
            },
            // UHF-Band (0.3-3 GHz)
            FrequencyBand {
                name: BandType::UHFBand,
                frequency_range: FrequencyRange {
                    min_ghz: 0.3,
                    max_ghz: 3.0,
                },
                characteristics: BandCharacteristics {
                    max_data_rate_mbps: 10.0,
                    power_efficiency: 0.5,
                    antenna_gain_dbi: 15.0,
                    noise_temperature_k: 200.0,
                },
            },
        ]
    }

    /// Simulate transmission for this frequency band
    pub fn simulate_transmission(
        &self,
        params: &TransmissionParameters,
        environment: &EnvironmentalConditions,
    ) -> TransmissionResult {
        // Calculate center frequency
        let center_freq_ghz = (self.frequency_range.min_ghz + self.frequency_range.max_ghz) / 2.0;

        // Calculate path loss using Friis equation
        let path_loss_db = 20.0
            * (4.0 * std::f64::consts::PI * params.distance_km * 1000.0 * center_freq_ghz * 1e9
                / 299792458.0)
                .log10();

        // Calculate weather impact
        let weather_impact = self.calculate_weather_impact(center_freq_ghz, environment);

        // Calculate atmospheric attenuation
        let atmospheric_loss_db = self.calculate_atmospheric_loss(center_freq_ghz, environment);

        // Total loss
        let total_loss_db = path_loss_db + atmospheric_loss_db + weather_impact;

        // Calculate received power
        let tx_power_dbm = 10.0 * params.transmit_power_watts.log10() + 30.0;
        let rx_power_dbm = tx_power_dbm + self.characteristics.antenna_gain_dbi - total_loss_db;

        // Calculate SNR
        let noise_power_dbm =
            10.0 * (1.38e-23 * self.characteristics.noise_temperature_k * 1e6).log10() + 30.0;
        let snr_db = rx_power_dbm - noise_power_dbm;

        // Calculate achievable data rate based on Shannon-Hartley theorem
        let bandwidth_mhz = (self.frequency_range.max_ghz - self.frequency_range.min_ghz) * 1000.0;
        let achievable_rate = bandwidth_mhz * (1.0 + 10.0_f64.powf(snr_db / 10.0)).log2();

        // Apply band limitations
        let actual_data_rate = achievable_rate.min(self.characteristics.max_data_rate_mbps);

        // Calculate efficiency
        let efficiency = if achievable_rate > 0.0 {
            (actual_data_rate / self.characteristics.max_data_rate_mbps).min(1.0)
        } else {
            0.0
        };

        // Check if transmission is successful
        let success = snr_db > 10.0 && actual_data_rate >= params.required_data_rate_mbps;

        // Calculate transmission time and latency
        let transmission_time_ms = if actual_data_rate > 0.0 {
            params.data_size_mb * 8.0 / actual_data_rate
        } else {
            f64::INFINITY
        };

        let propagation_delay_ms = params.distance_km / 299.792458; // Speed of light
        let total_latency = transmission_time_ms + propagation_delay_ms;

        // Calculate power consumption
        let power_consumption = params.transmit_power_watts / self.characteristics.power_efficiency;

        TransmissionResult {
            success,
            actual_data_rate_mbps: actual_data_rate,
            total_latency_ms: total_latency,
            power_consumption_watts: power_consumption,
            transmission_efficiency: efficiency,
            weather_impact_factor: weather_impact / 100.0,
            signal_to_noise_ratio_db: snr_db,
            path_loss_db,
        }
    }

    fn calculate_weather_impact(
        &self,
        frequency_ghz: f64,
        environment: &EnvironmentalConditions,
    ) -> f64 {
        let rain_attenuation = match self.name {
            BandType::KaBand => environment.rain_rate_mm_hour * 2.5, // Very sensitive
            BandType::KBand => environment.rain_rate_mm_hour * 1.8,  // High sensitivity
            BandType::XBand => environment.rain_rate_mm_hour * 0.8,  // Moderate sensitivity
            BandType::SBand => environment.rain_rate_mm_hour * 0.2,  // Low sensitivity
            BandType::UHFBand => environment.rain_rate_mm_hour * 0.05, // Very low sensitivity
        };

        let cloud_attenuation = environment.cloud_cover_percent * frequency_ghz * 0.001;

        rain_attenuation + cloud_attenuation
    }

    fn calculate_atmospheric_loss(
        &self,
        frequency_ghz: f64,
        environment: &EnvironmentalConditions,
    ) -> f64 {
        // Simplified atmospheric loss model
        let oxygen_absorption = if frequency_ghz > 15.0 {
            (frequency_ghz - 15.0) * 0.01
        } else {
            0.0
        };

        let water_vapor_absorption = environment.humidity_percent * frequency_ghz * 0.0001;

        oxygen_absorption + water_vapor_absorption
    }
}

pub fn run_basic_demo() {
    println!("=== Frequency Band Simulation Demo ===");

    let bands = FrequencyBand::get_standard_bands();

    let params = TransmissionParameters {
        distance_km: 1000.0,
        data_size_mb: 100.0,
        required_data_rate_mbps: 200.0,
        elevation_angle_degrees: 35.0,
        transmit_power_watts: 100.0,
        antenna_diameter_meters: 3.0,
    };

    let clear_weather = EnvironmentalConditions {
        rain_rate_mm_hour: 0.0,
        cloud_cover_percent: 10.0,
        atmospheric_pressure_mb: 1015.0,
        temperature_celsius: 22.0,
        humidity_percent: 45.0,
        ionospheric_activity: 0.1,
        solar_activity: 0.1,
    };

    let stormy_weather = EnvironmentalConditions {
        rain_rate_mm_hour: 25.0,
        cloud_cover_percent: 100.0,
        atmospheric_pressure_mb: 995.0,
        temperature_celsius: 12.0,
        humidity_percent: 98.0,
        ionospheric_activity: 0.4,
        solar_activity: 0.3,
    };

    println!("\nClear Weather Performance:");
    println!(
        "{:<12} {:>15} {:>12} {:>10}",
        "Band", "Data Rate (Mbps)", "SNR (dB)", "Success"
    );
    for band in &bands {
        let result = band.simulate_transmission(&params, &clear_weather);
        println!(
            "{:<12} {:>15.1} {:>12.1} {:>10}",
            band.name,
            result.actual_data_rate_mbps,
            result.signal_to_noise_ratio_db,
            if result.success { "Yes" } else { "No" }
        );
    }

    println!("\nStormy Weather Performance:");
    println!(
        "{:<12} {:>15} {:>12} {:>10}",
        "Band", "Data Rate (Mbps)", "SNR (dB)", "Success"
    );
    for band in &bands {
        let result = band.simulate_transmission(&params, &stormy_weather);
        println!(
            "{:<12} {:>15.1} {:>12.1} {:>10}",
            band.name,
            result.actual_data_rate_mbps,
            result.signal_to_noise_ratio_db,
            if result.success { "Yes" } else { "No" }
        );
    }

    println!("\nWeather Impact Analysis:");
    println!("{:<12} {:>20}", "Band", "Performance Degradation");
    for band in &bands {
        let clear_result = band.simulate_transmission(&params, &clear_weather);
        let storm_result = band.simulate_transmission(&params, &stormy_weather);

        let degradation = if clear_result.actual_data_rate_mbps > 0.0 {
            ((clear_result.actual_data_rate_mbps - storm_result.actual_data_rate_mbps)
                / clear_result.actual_data_rate_mbps
                * 100.0)
                .max(0.0)
        } else {
            0.0
        };

        println!("{:<12} {:>18.1}%", band.name, degradation);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_band_creation() {
        let bands = FrequencyBand::get_standard_bands();
        assert_eq!(bands.len(), 5);

        let ka_band = &bands[1];
        assert_eq!(ka_band.name, BandType::KaBand);
        assert_eq!(ka_band.frequency_range.min_ghz, 26.5);
    }

    #[test]
    fn test_clear_weather_simulation() {
        let s_band = &FrequencyBand::get_standard_bands()[2];

        let params = TransmissionParameters {
            distance_km: 500.0,
            data_size_mb: 50.0,
            required_data_rate_mbps: 50.0,
            elevation_angle_degrees: 45.0,
            transmit_power_watts: 100.0,
            antenna_diameter_meters: 3.0,
        };

        let clear_conditions = EnvironmentalConditions {
            rain_rate_mm_hour: 0.0,
            cloud_cover_percent: 0.0,
            atmospheric_pressure_mb: 1013.25,
            temperature_celsius: 20.0,
            humidity_percent: 50.0,
            ionospheric_activity: 0.1,
            solar_activity: 0.1,
        };

        let result = s_band.simulate_transmission(&params, &clear_conditions);
        assert!(result.success);
        assert!(result.actual_data_rate_mbps > 0.0);
        assert!(result.signal_to_noise_ratio_db > 10.0);
    }

    #[test]
    fn test_weather_impact() {
        let ka_band = &FrequencyBand::get_standard_bands()[1];

        let params = TransmissionParameters {
            distance_km: 1000.0,
            data_size_mb: 100.0,
            required_data_rate_mbps: 200.0,
            elevation_angle_degrees: 30.0,
            transmit_power_watts: 150.0,
            antenna_diameter_meters: 4.0,
        };

        let clear = EnvironmentalConditions {
            rain_rate_mm_hour: 0.0,
            cloud_cover_percent: 0.0,
            atmospheric_pressure_mb: 1013.25,
            temperature_celsius: 20.0,
            humidity_percent: 50.0,
            ionospheric_activity: 0.1,
            solar_activity: 0.1,
        };

        let rainy = EnvironmentalConditions {
            rain_rate_mm_hour: 15.0,
            cloud_cover_percent: 100.0,
            atmospheric_pressure_mb: 1000.0,
            temperature_celsius: 15.0,
            humidity_percent: 90.0,
            ionospheric_activity: 0.3,
            solar_activity: 0.2,
        };

        let clear_result = ka_band.simulate_transmission(&params, &clear);
        let rainy_result = ka_band.simulate_transmission(&params, &rainy);

        // Ka-Band should be significantly affected by rain
        assert!(clear_result.actual_data_rate_mbps > rainy_result.actual_data_rate_mbps);
        assert!(clear_result.signal_to_noise_ratio_db > rainy_result.signal_to_noise_ratio_db);
    }
}
