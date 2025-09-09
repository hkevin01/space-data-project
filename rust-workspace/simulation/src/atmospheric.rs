//! Atmospheric Effects Simulation Module
//!
//! This module simulates various atmospheric phenomena that affect
//! satellite-ground communication, including rain fade, atmospheric
//! absorption, scintillation, and ionospheric effects.

use rand::Rng;
use serde::{Deserialize, Serialize};

/// Atmospheric conditions that affect signal propagation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtmosphericConditions {
    pub altitude_km: f64,
    pub rain_rate_mm_hour: f64,
    pub cloud_density: f64,       // 0.0 to 1.0
    pub water_vapor_density: f64, // g/m³
    pub temperature_celsius: f64,
    pub pressure_mb: f64,
    pub wind_speed_ms: f64,
    pub humidity_percent: f64,
}

/// Ionospheric conditions affecting lower frequency bands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IonosphericConditions {
    pub total_electron_content: f64, // TEC in electrons/m²
    pub solar_flux_index: f64,       // Solar radio flux at 10.7 cm
    pub geomagnetic_index: f64,      // Kp index (0-9)
    pub scintillation_index: f64,    // S4 index (0-1)
    pub time_of_day: f64,            // Hours (0-24)
    pub season: Season,
    pub solar_cycle_phase: f64, // Solar cycle position (0-1)
}

/// Seasonal variations
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

/// Rain fade models and calculations
pub struct RainFadeModel;

impl RainFadeModel {
    /// Calculate rain attenuation using ITU-R P.618 model
    pub fn calculate_rain_attenuation(
        frequency_ghz: f64,
        rain_rate_mm_hour: f64,
        path_length_km: f64,
        elevation_angle_deg: f64,
    ) -> f64 {
        if rain_rate_mm_hour <= 0.0 {
            return 0.0;
        }

        // ITU-R P.838 coefficients for rain attenuation
        let (k_h, k_v, alpha_h, alpha_v) = Self::get_itu_coefficients(frequency_ghz);

        // Use horizontal polarization coefficients (conservative estimate)
        let k = k_h;
        let alpha = alpha_h;

        // Specific attenuation (dB/km)
        let gamma_r = k * rain_rate_mm_hour.powf(alpha);

        // Effective path length through rain cell
        let effective_path = Self::calculate_effective_path_length(
            path_length_km,
            elevation_angle_deg,
            rain_rate_mm_hour,
        );

        gamma_r * effective_path
    }

    /// Get ITU-R P.838 coefficients for different frequencies
    fn get_itu_coefficients(frequency_ghz: f64) -> (f64, f64, f64, f64) {
        // Simplified coefficient lookup
        match frequency_ghz {
            f if f <= 1.0 => (0.0000387, 0.0000352, 0.912, 0.880),
            f if f <= 2.0 => (0.000154, 0.000138, 0.963, 0.923),
            f if f <= 4.0 => (0.000650, 0.000591, 1.121, 1.075),
            f if f <= 6.0 => (0.00175, 0.00155, 1.308, 1.265),
            f if f <= 7.0 => (0.00301, 0.00265, 1.332, 1.312),
            f if f <= 8.0 => (0.00454, 0.00395, 1.327, 1.310),
            f if f <= 10.0 => (0.0101, 0.00887, 1.276, 1.264),
            f if f <= 12.0 => (0.0188, 0.0168, 1.217, 1.200),
            f if f <= 15.0 => (0.0367, 0.0335, 1.154, 1.128),
            f if f <= 20.0 => (0.0751, 0.0691, 1.099, 1.065),
            f if f <= 25.0 => (0.124, 0.113, 1.061, 1.030),
            f if f <= 30.0 => (0.187, 0.167, 1.021, 1.000),
            f if f <= 35.0 => (0.263, 0.233, 0.979, 0.963),
            f if f <= 40.0 => (0.350, 0.310, 0.939, 0.929),
            f if f <= 45.0 => (0.442, 0.393, 0.903, 0.897),
            f if f <= 50.0 => (0.536, 0.479, 0.873, 0.868),
            _ => (0.707, 0.642, 0.826, 0.824),
        }
    }

    /// Calculate effective path length through rain cell
    fn calculate_effective_path_length(
        slant_path_km: f64,
        elevation_angle_deg: f64,
        rain_rate_mm_hour: f64,
    ) -> f64 {
        // Rain height model (simplified)
        let rain_height_km = if rain_rate_mm_hour < 10.0 { 3.0 } else { 4.0 };

        let elevation_rad = elevation_angle_deg.to_radians();
        let path_through_rain = rain_height_km / elevation_rad.sin();

        // Horizontal reduction factor
        let horizontal_reduction = 1.0 / (1.0 + path_through_rain / 35.0);

        (path_through_rain * horizontal_reduction).min(slant_path_km)
    }

    /// Generate realistic rain rate scenarios
    pub fn generate_rain_scenarios() -> Vec<(String, f64)> {
        vec![
            ("Clear Sky".to_string(), 0.0),
            ("Light Drizzle".to_string(), 0.5),
            ("Light Rain".to_string(), 2.0),
            ("Moderate Rain".to_string(), 5.0),
            ("Heavy Rain".to_string(), 10.0),
            ("Very Heavy Rain".to_string(), 20.0),
            ("Extreme Rain".to_string(), 50.0),
            ("Tropical Storm".to_string(), 100.0),
        ]
    }
}

/// Atmospheric absorption model
pub struct AtmosphericAbsorption;

impl AtmosphericAbsorption {
    /// Calculate gaseous absorption using ITU-R P.676
    pub fn calculate_gaseous_absorption(
        frequency_ghz: f64,
        altitude_km: f64,
        conditions: &AtmosphericConditions,
    ) -> f64 {
        let oxygen_absorption = Self::calculate_oxygen_absorption(
            frequency_ghz,
            conditions.pressure_mb,
            conditions.temperature_celsius,
        );

        let water_vapor_absorption = Self::calculate_water_vapor_absorption(
            frequency_ghz,
            conditions.water_vapor_density,
            conditions.temperature_celsius,
        );

        // Path length through atmosphere (simplified)
        let path_length_km = if altitude_km > 0.0 {
            altitude_km / 0.8 // Approximate scale height
        } else {
            10.0 // Default atmospheric path
        };

        (oxygen_absorption + water_vapor_absorption) * path_length_km
    }

    /// Calculate oxygen absorption
    fn calculate_oxygen_absorption(
        frequency_ghz: f64,
        pressure_mb: f64,
        temperature_celsius: f64,
    ) -> f64 {
        let temperature_k = temperature_celsius + 273.15;
        let pressure_kpa = pressure_mb * 0.1;

        // Simplified oxygen absorption model
        // Main absorption lines at 60 GHz and 118 GHz
        let absorption_60 =
            Self::line_absorption(frequency_ghz, 60.0, 15.0, pressure_kpa, temperature_k);
        let absorption_118 =
            Self::line_absorption(frequency_ghz, 118.75, 1.0, pressure_kpa, temperature_k);

        (absorption_60 + absorption_118) * pressure_kpa / 101.325
    }

    /// Calculate water vapor absorption
    fn calculate_water_vapor_absorption(
        frequency_ghz: f64,
        water_vapor_density: f64,
        temperature_celsius: f64,
    ) -> f64 {
        let temperature_k = temperature_celsius + 273.15;

        // Main water vapor absorption lines
        let absorption_22 = Self::line_absorption(
            frequency_ghz,
            22.235,
            2.0,
            water_vapor_density,
            temperature_k,
        );
        let absorption_183 = Self::line_absorption(
            frequency_ghz,
            183.31,
            1.0,
            water_vapor_density,
            temperature_k,
        );
        let absorption_325 = Self::line_absorption(
            frequency_ghz,
            325.15,
            0.5,
            water_vapor_density,
            temperature_k,
        );

        (absorption_22 + absorption_183 + absorption_325) * water_vapor_density / 10.0
    }

    /// Calculate line absorption using Lorentzian line shape
    fn line_absorption(
        frequency_ghz: f64,
        line_center_ghz: f64,
        line_strength: f64,
        density: f64,
        temperature_k: f64,
    ) -> f64 {
        let line_width: f64 = 0.1; // Simplified line width
        let frequency_diff = frequency_ghz - line_center_ghz;

        let lorentzian = line_width.powi(2) / (frequency_diff.powi(2) + line_width.powi(2));

        line_strength * lorentzian * density * (300.0 / temperature_k).sqrt()
    }
}

/// Ionospheric effects model
pub struct IonosphericModel;

impl IonosphericModel {
    /// Calculate ionospheric scintillation
    pub fn calculate_scintillation(
        frequency_ghz: f64,
        conditions: &IonosphericConditions,
        elevation_angle_deg: f64,
    ) -> f64 {
        // Scintillation is frequency dependent: proportional to f^(-1.5)
        let frequency_factor = (frequency_ghz / 1.0).powf(-1.5);

        // Elevation angle dependence
        let elevation_factor = 1.0 / elevation_angle_deg.to_radians().sin().sqrt();

        // Time of day variation (peak around midnight)
        let time_factor =
            1.0 + 0.5 * (2.0 * std::f64::consts::PI * conditions.time_of_day / 24.0).cos();

        // Solar activity influence
        let solar_factor = 1.0 + conditions.solar_flux_index / 200.0;

        // Geomagnetic activity
        let geomagnetic_factor = 1.0 + conditions.geomagnetic_index / 9.0;

        conditions.scintillation_index
            * frequency_factor
            * elevation_factor
            * time_factor
            * solar_factor
            * geomagnetic_factor
    }

    /// Calculate ionospheric group delay
    pub fn calculate_group_delay(
        frequency_ghz: f64,
        conditions: &IonosphericConditions,
        elevation_angle_deg: f64,
    ) -> f64 {
        // Group delay is proportional to TEC/f^2
        let frequency_hz = frequency_ghz * 1e9;
        let slant_tec = conditions.total_electron_content / elevation_angle_deg.to_radians().sin();

        // Constant for group delay calculation
        let k = 40.3; // m³/s²

        k * slant_tec / frequency_hz.powi(2) * 1000.0 // Convert to ms
    }

    /// Calculate Faraday rotation
    pub fn calculate_faraday_rotation(
        frequency_ghz: f64,
        conditions: &IonosphericConditions,
        elevation_angle_deg: f64,
    ) -> f64 {
        // Faraday rotation is proportional to TEC/f^2
        let frequency_hz = frequency_ghz * 1e9;
        let slant_tec = conditions.total_electron_content / elevation_angle_deg.to_radians().sin();

        // Earth's magnetic field strength (simplified)
        let magnetic_field_tesla = 50e-6; // Typical value

        // Faraday rotation constant
        let k_f = 2.36e4; // rad⋅T⋅m²⋅s⋅Hz⋅el⁻¹

        k_f * magnetic_field_tesla * slant_tec / frequency_hz.powi(2)
    }

    /// Generate realistic ionospheric conditions
    pub fn generate_ionospheric_scenarios() -> Vec<(String, IonosphericConditions)> {
        vec![
            (
                "Quiet Ionosphere".to_string(),
                IonosphericConditions {
                    total_electron_content: 10e16,
                    solar_flux_index: 70.0,
                    geomagnetic_index: 1.0,
                    scintillation_index: 0.1,
                    time_of_day: 12.0,
                    season: Season::Spring,
                    solar_cycle_phase: 0.5,
                },
            ),
            (
                "Moderate Activity".to_string(),
                IonosphericConditions {
                    total_electron_content: 50e16,
                    solar_flux_index: 120.0,
                    geomagnetic_index: 3.0,
                    scintillation_index: 0.3,
                    time_of_day: 22.0,
                    season: Season::Summer,
                    solar_cycle_phase: 0.8,
                },
            ),
            (
                "High Activity".to_string(),
                IonosphericConditions {
                    total_electron_content: 100e16,
                    solar_flux_index: 200.0,
                    geomagnetic_index: 6.0,
                    scintillation_index: 0.6,
                    time_of_day: 2.0,
                    season: Season::Autumn,
                    solar_cycle_phase: 1.0,
                },
            ),
            (
                "Severe Storm".to_string(),
                IonosphericConditions {
                    total_electron_content: 200e16,
                    solar_flux_index: 300.0,
                    geomagnetic_index: 8.0,
                    scintillation_index: 0.9,
                    time_of_day: 0.0,
                    season: Season::Winter,
                    solar_cycle_phase: 0.9,
                },
            ),
        ]
    }
}

/// Scintillation effects simulation
pub struct ScintillationModel;

impl ScintillationModel {
    /// Generate scintillation time series
    pub fn generate_scintillation_series(
        duration_seconds: f64,
        sampling_rate_hz: f64,
        s4_index: f64,
    ) -> Vec<f64> {
        let num_samples = (duration_seconds * sampling_rate_hz) as usize;
        let mut rng = rand::thread_rng();
        let mut series = Vec::with_capacity(num_samples);

        // Generate correlated noise for realistic scintillation
        let mut previous_value = 0.0;
        let correlation_factor = 0.9; // Temporal correlation

        for _ in 0..num_samples {
            let random_component = rng.gen_range(-1.0..1.0);
            let current_value =
                correlation_factor * previous_value + (1.0 - correlation_factor) * random_component;

            // Scale by S4 index and convert to amplitude scintillation
            let amplitude_scintillation = 1.0 + s4_index * current_value;
            series.push(amplitude_scintillation.max(0.1)); // Prevent negative values

            previous_value = current_value;
        }

        series
    }

    /// Calculate scintillation statistics
    pub fn calculate_statistics(scintillation_series: &[f64]) -> ScintillationStatistics {
        let mean = scintillation_series.iter().sum::<f64>() / scintillation_series.len() as f64;

        let variance = scintillation_series
            .iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>()
            / scintillation_series.len() as f64;

        let std_dev = variance.sqrt();

        let min_value = scintillation_series
            .iter()
            .fold(f64::INFINITY, |a, &b| a.min(b));
        let max_value = scintillation_series
            .iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        // Calculate fade probability (amplitude < 0.5)
        let fade_count = scintillation_series.iter().filter(|&&x| x < 0.5).count();
        let fade_probability = fade_count as f64 / scintillation_series.len() as f64;

        ScintillationStatistics {
            mean,
            std_dev,
            min_value,
            max_value,
            fade_probability,
        }
    }
}

/// Statistical measures of scintillation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScintillationStatistics {
    pub mean: f64,
    pub std_dev: f64,
    pub min_value: f64,
    pub max_value: f64,
    pub fade_probability: f64,
}

/// Weather pattern generator for simulation
pub struct WeatherPatternGenerator;

impl WeatherPatternGenerator {
    /// Generate realistic weather scenarios
    pub fn generate_weather_scenarios() -> Vec<(String, AtmosphericConditions)> {
        vec![
            (
                "Clear Sky".to_string(),
                AtmosphericConditions {
                    altitude_km: 10.0,
                    rain_rate_mm_hour: 0.0,
                    cloud_density: 0.0,
                    water_vapor_density: 5.0,
                    temperature_celsius: 20.0,
                    pressure_mb: 1013.25,
                    wind_speed_ms: 2.0,
                    humidity_percent: 50.0,
                },
            ),
            (
                "Partly Cloudy".to_string(),
                AtmosphericConditions {
                    altitude_km: 10.0,
                    rain_rate_mm_hour: 0.0,
                    cloud_density: 0.3,
                    water_vapor_density: 8.0,
                    temperature_celsius: 18.0,
                    pressure_mb: 1010.0,
                    wind_speed_ms: 5.0,
                    humidity_percent: 65.0,
                },
            ),
            (
                "Overcast".to_string(),
                AtmosphericConditions {
                    altitude_km: 10.0,
                    rain_rate_mm_hour: 1.0,
                    cloud_density: 0.8,
                    water_vapor_density: 12.0,
                    temperature_celsius: 15.0,
                    pressure_mb: 1005.0,
                    wind_speed_ms: 8.0,
                    humidity_percent: 80.0,
                },
            ),
            (
                "Thunderstorm".to_string(),
                AtmosphericConditions {
                    altitude_km: 15.0,
                    rain_rate_mm_hour: 25.0,
                    cloud_density: 1.0,
                    water_vapor_density: 20.0,
                    temperature_celsius: 12.0,
                    pressure_mb: 995.0,
                    wind_speed_ms: 15.0,
                    humidity_percent: 95.0,
                },
            ),
        ]
    }

    /// Generate time-varying weather conditions
    pub fn generate_dynamic_weather(
        duration_hours: f64,
        time_step_minutes: f64,
    ) -> Vec<AtmosphericConditions> {
        let num_steps = (duration_hours * 60.0 / time_step_minutes) as usize;
        let mut weather_series = Vec::with_capacity(num_steps);
        let mut rng = rand::thread_rng();

        // Start with baseline conditions
        let mut current_conditions = AtmosphericConditions {
            altitude_km: 10.0,
            rain_rate_mm_hour: 0.0,
            cloud_density: 0.2,
            water_vapor_density: 8.0,
            temperature_celsius: 20.0,
            pressure_mb: 1013.25,
            wind_speed_ms: 3.0,
            humidity_percent: 60.0,
        };

        for _ in 0..num_steps {
            // Add random variations
            current_conditions.rain_rate_mm_hour =
                (current_conditions.rain_rate_mm_hour + rng.gen_range(-0.5..0.5)).max(0.0);

            current_conditions.cloud_density =
                (current_conditions.cloud_density + rng.gen_range(-0.05..0.05)).clamp(0.0, 1.0);

            current_conditions.temperature_celsius += rng.gen_range(-0.5..0.5);
            current_conditions.humidity_percent =
                (current_conditions.humidity_percent + rng.gen_range(-2.0..2.0)).clamp(0.0, 100.0);

            weather_series.push(current_conditions.clone());
        }

        weather_series
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_rain_fade_calculation() {
        let attenuation = RainFadeModel::calculate_rain_attenuation(
            20.0,   // 20 GHz (K-band)
            10.0,   // 10 mm/hour rain rate
            1000.0, // 1000 km path
            45.0,   // 45 degree elevation
        );

        assert!(attenuation > 0.0);
        assert!(attenuation < 100.0); // Reasonable range
    }

    #[test]
    fn test_atmospheric_absorption() {
        let conditions = AtmosphericConditions {
            altitude_km: 10.0,
            rain_rate_mm_hour: 0.0,
            cloud_density: 0.0,
            water_vapor_density: 10.0,
            temperature_celsius: 20.0,
            pressure_mb: 1013.25,
            wind_speed_ms: 5.0,
            humidity_percent: 60.0,
        };

        let absorption = AtmosphericAbsorption::calculate_gaseous_absorption(
            22.0, // Near water vapor line
            10.0,
            &conditions,
        );

        assert!(absorption > 0.0);
    }

    #[test]
    fn test_scintillation_generation() {
        let series = ScintillationModel::generate_scintillation_series(
            10.0, // 10 seconds
            10.0, // 10 Hz sampling
            0.3,  // S4 index
        );

        assert_eq!(series.len(), 100);
        assert!(series.iter().all(|&x| x > 0.0));

        let stats = ScintillationModel::calculate_statistics(&series);
        assert!(stats.mean > 0.0);
        assert!(stats.std_dev > 0.0);
    }
}
