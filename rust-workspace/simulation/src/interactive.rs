//! Interactive Frequency Band Simulation Demo
//!
//! A user-friendly interactive demonstration of frequency band characteristics
//! and their performance under various conditions.

mod atmospheric;
mod bands;
mod interference;

use bands::{BandType, EnvironmentalConditions, FrequencyBand, TransmissionParameters};
use std::collections::HashMap;
use std::io::{self, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    display_welcome();

    loop {
        display_main_menu();

        match get_user_choice()? {
            1 => run_quick_demo()?,
            2 => run_band_selection_demo()?,
            3 => run_weather_comparison()?,
            4 => run_distance_analysis()?,
            5 => run_mission_scenario_planner()?,
            6 => run_real_time_simulation()?,
            7 => display_band_characteristics(),
            8 => run_educational_mode()?,
            0 => {
                println!("👋 Thanks for using the Frequency Band Simulator!");
                break;
            }
            _ => println!("❌ Invalid choice. Please select 0-8."),
        }

        wait_for_enter();
    }

    Ok(())
}

fn display_welcome() {
    println!("🚀 Interactive Frequency Band Simulation Demo");
    println!("{}", "=".repeat(60));
    println!("Welcome to the comprehensive satellite communication");
    println!("frequency band simulator. This tool demonstrates how");
    println!("different frequency bands perform under various");
    println!("atmospheric and operational conditions.");
    println!();
}

fn display_main_menu() {
    println!("\n📋 Main Menu - Choose your exploration:");
    println!("  1. 🎯 Quick Demo - See all bands in action");
    println!("  2. 🔍 Band Selection - Choose specific bands to test");
    println!("  3. 🌦️  Weather Impact - Compare performance in different weather");
    println!("  4. 📏 Distance Analysis - See how range affects performance");
    println!("  5. 🛰️  Mission Planner - Plan your satellite mission");
    println!("  6. ⏰ Real-time Simulation - Watch dynamic conditions");
    println!("  7. 📚 Band Characteristics - Learn about each band");
    println!("  8. 🎓 Educational Mode - Guided learning experience");
    println!("  0. 🚪 Exit");
    print!("\nYour choice: ");
    io::stdout().flush().unwrap();
}

fn get_user_choice() -> Result<u32, Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().parse().unwrap_or(999))
}

fn run_quick_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🎯 Quick Demo - All Bands Performance");
    println!("{}", "=".repeat(50));

    let bands = FrequencyBand::get_standard_bands();

    // Standard mission parameters
    let params = TransmissionParameters {
        distance_km: 1200.0,
        data_size_mb: 100.0,
        required_data_rate_mbps: 200.0,
        elevation_angle_degrees: 35.0,
        transmit_power_watts: 100.0,
        antenna_diameter_meters: 3.0,
    };

    println!("📡 Mission Parameters:");
    println!("  • Distance to satellite: {} km", params.distance_km);
    println!("  • Data to transmit: {} MB", params.data_size_mb);
    println!(
        "  • Required data rate: {} Mbps",
        params.required_data_rate_mbps
    );
    println!(
        "  • Satellite elevation: {}°",
        params.elevation_angle_degrees
    );

    // Test three scenarios
    let scenarios = vec![
        (
            "☀️ Clear Sky",
            EnvironmentalConditions {
                rain_rate_mm_hour: 0.0,
                cloud_cover_percent: 10.0,
                atmospheric_pressure_mb: 1015.0,
                temperature_celsius: 22.0,
                humidity_percent: 45.0,
                ionospheric_activity: 0.1,
                solar_activity: 0.1,
            },
        ),
        (
            "🌧️ Moderate Rain",
            EnvironmentalConditions {
                rain_rate_mm_hour: 5.0,
                cloud_cover_percent: 90.0,
                atmospheric_pressure_mb: 1008.0,
                temperature_celsius: 16.0,
                humidity_percent: 85.0,
                ionospheric_activity: 0.2,
                solar_activity: 0.2,
            },
        ),
        (
            "⛈️ Heavy Storm",
            EnvironmentalConditions {
                rain_rate_mm_hour: 25.0,
                cloud_cover_percent: 100.0,
                atmospheric_pressure_mb: 995.0,
                temperature_celsius: 12.0,
                humidity_percent: 98.0,
                ionospheric_activity: 0.4,
                solar_activity: 0.3,
            },
        ),
    ];

    for (scenario_name, environment) in scenarios {
        println!("\n{}", scenario_name);
        println!("{}", "-".repeat(30));

        for band in &bands {
            let result = band.simulate_transmission(&params, &environment);

            let status = if result.success {
                if result.transmission_efficiency > 0.8 {
                    "🟢 EXCELLENT"
                } else if result.transmission_efficiency > 0.6 {
                    "🟡 GOOD"
                } else if result.transmission_efficiency > 0.4 {
                    "🟠 FAIR"
                } else {
                    "🔴 POOR"
                }
            } else {
                "❌ FAILED"
            };

            println!(
                "  {:<10} {} ({:.0} Mbps, {:.1}% eff.)",
                format!("{}", band.name),
                status,
                result.actual_data_rate_mbps,
                result.transmission_efficiency * 100.0
            );
        }
    }

    println!("\n💡 Key Insights:");
    println!("  • Ka-Band offers highest data rates but is weather-sensitive");
    println!("  • S-Band is most reliable but has limited bandwidth");
    println!("  • K-Band provides good balance of speed and reliability");
    println!("  • UHF-Band is excellent for emergency communications");

    Ok(())
}

fn run_band_selection_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔍 Band Selection Demo");
    println!("{}", "=".repeat(40));

    let bands = FrequencyBand::get_standard_bands();

    println!("Available frequency bands:");
    for (i, band) in bands.iter().enumerate() {
        println!(
            "  {}. {} ({:.1} - {:.1} GHz)",
            i + 1,
            band.name,
            band.frequency_range.min_ghz,
            band.frequency_range.max_ghz
        );
    }

    print!("\nSelect bands to compare (e.g., 1,3,5 or 'all'): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    let selected_bands: Vec<&FrequencyBand> = if input == "all" {
        bands.iter().collect()
    } else {
        input
            .split(',')
            .filter_map(|s| s.trim().parse::<usize>().ok())
            .filter_map(|i| {
                if i > 0 && i <= bands.len() {
                    Some(&bands[i - 1])
                } else {
                    None
                }
            })
            .collect()
    };

    if selected_bands.is_empty() {
        println!("❌ No valid bands selected.");
        return Ok(());
    }

    // Get mission parameters
    let distance = get_parameter("Distance (km)", 1000.0)?;
    let data_size = get_parameter("Data size (MB)", 100.0)?;
    let rain_rate = get_parameter("Rain rate (mm/hour)", 0.0)?;
    let elevation = get_parameter("Elevation angle (degrees)", 30.0)?;

    let params = TransmissionParameters {
        distance_km: distance,
        data_size_mb: data_size,
        required_data_rate_mbps: 100.0,
        elevation_angle_degrees: elevation,
        transmit_power_watts: 100.0,
        antenna_diameter_meters: 3.0,
    };

    let environment = EnvironmentalConditions {
        rain_rate_mm_hour: rain_rate,
        cloud_cover_percent: rain_rate * 3.0,
        atmospheric_pressure_mb: 1013.25,
        temperature_celsius: 20.0,
        humidity_percent: 60.0,
        ionospheric_activity: 0.2,
        solar_activity: 0.1,
    };

    println!("\n📊 Comparison Results:");
    println!(
        "{:<12} {:>10} {:>12} {:>10} {:>12} {:>10}",
        "Band", "Success", "Data Rate", "Latency", "Power", "Weather"
    );
    println!(
        "{:<12} {:>10} {:>12} {:>10} {:>12} {:>10}",
        "", "", "(Mbps)", "(ms)", "(W)", "Impact"
    );
    println!("{}", "-".repeat(75));

    for band in selected_bands {
        let result = band.simulate_transmission(&params, &environment);

        println!(
            "{:<12} {:>10} {:>12.0} {:>10.1} {:>12.1} {:>10.1}%",
            format!("{}", band.name),
            if result.success { "✅" } else { "❌" },
            result.actual_data_rate_mbps,
            result.total_latency_ms,
            result.power_consumption_watts,
            result.weather_impact_factor * 100.0
        );
    }

    Ok(())
}

fn run_weather_comparison() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🌦️ Weather Impact Comparison");
    println!("{}", "=".repeat(45));

    let bands = FrequencyBand::get_standard_bands();

    let params = TransmissionParameters {
        distance_km: 1000.0,
        data_size_mb: 100.0,
        required_data_rate_mbps: 200.0,
        elevation_angle_degrees: 30.0,
        transmit_power_watts: 100.0,
        antenna_diameter_meters: 3.0,
    };

    // Different weather conditions
    let weather_conditions = vec![
        ("☀️ Clear Sky", 0.0, 0.0, 20.0, 50.0),
        ("⛅ Partly Cloudy", 0.0, 30.0, 18.0, 65.0),
        ("☁️ Overcast", 1.0, 90.0, 15.0, 80.0),
        ("🌦️ Light Rain", 3.0, 100.0, 14.0, 90.0),
        ("🌧️ Moderate Rain", 8.0, 100.0, 12.0, 95.0),
        ("⛈️ Heavy Rain", 20.0, 100.0, 10.0, 99.0),
        ("🌪️ Severe Storm", 50.0, 100.0, 8.0, 100.0),
    ];

    println!("Performance across weather conditions:\n");

    // Show efficiency table
    println!(
        "{:<15} {:>8} {:>8} {:>8} {:>8} {:>8}",
        "Weather", "K-Band", "Ka-Band", "S-Band", "X-Band", "UHF-Band"
    );
    println!("{}", "-".repeat(65));

    for (weather_name, rain_rate, cloud_cover, temp, humidity) in weather_conditions {
        let environment = EnvironmentalConditions {
            rain_rate_mm_hour: rain_rate,
            cloud_cover_percent: cloud_cover,
            atmospheric_pressure_mb: 1013.25,
            temperature_celsius: temp,
            humidity_percent: humidity,
            ionospheric_activity: 0.2,
            solar_activity: 0.1,
        };

        print!("{:<15}", weather_name);

        for band in &bands {
            let result = band.simulate_transmission(&params, &environment);
            let efficiency = (result.transmission_efficiency * 100.0) as i32;

            let color = if efficiency >= 80 {
                "🟢"
            } else if efficiency >= 60 {
                "🟡"
            } else if efficiency >= 40 {
                "🟠"
            } else {
                "🔴"
            };

            print!(" {:>6}%{}", efficiency, color);
        }
        println!();
    }

    println!("\n📈 Weather Sensitivity Ranking (most to least sensitive):");
    println!("  1. Ka-Band - Extremely sensitive to rain");
    println!("  2. K-Band - High sensitivity to precipitation");
    println!("  3. X-Band - Moderate weather sensitivity");
    println!("  4. S-Band - Low weather impact");
    println!("  5. UHF-Band - Minimal weather effects");

    Ok(())
}

fn run_distance_analysis() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📏 Distance Analysis");
    println!("{}", "=".repeat(35));

    let bands = FrequencyBand::get_standard_bands();
    let distances = vec![200.0, 500.0, 1000.0, 2000.0, 5000.0, 10000.0];

    println!("Performance vs distance (clear sky conditions):\n");

    println!(
        "{:<12} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8}",
        "Band", "200km", "500km", "1000km", "2000km", "5000km", "10000km"
    );
    println!("{}", "-".repeat(75));

    for band in &bands {
        print!("{:<12}", format!("{}", band.name));

        for &distance in &distances {
            let params = TransmissionParameters {
                distance_km: distance,
                data_size_mb: 100.0,
                required_data_rate_mbps: 100.0,
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

            let result = band.simulate_transmission(&params, &environment);
            let rate = result.actual_data_rate_mbps as i32;

            print!(" {:>7}M", rate);
        }
        println!();
    }

    println!("\n💡 Distance Insights:");
    println!("  • All bands follow inverse square law for path loss");
    println!("  • Higher frequencies have greater path loss");
    println!("  • Ka-Band maintains high rates even at long distances");
    println!("  • UHF-Band is most resilient to distance");

    Ok(())
}

fn run_mission_scenario_planner() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🛰️ Mission Scenario Planner");
    println!("{}", "=".repeat(40));

    println!("Let's plan your satellite communication mission!\n");

    // Mission type selection
    println!("Select mission type:");
    println!("  1. 📡 Earth Observation (high data volume)");
    println!("  2. 🗺️  Navigation/GPS (reliable positioning)");
    println!("  3. 📞 Communications (voice/data relay)");
    println!("  4. 🛡️  Military/Defense (secure, reliable)");
    println!("  5. 🚀 Deep Space (extreme distance)");

    let mission_type = get_user_choice()?;

    let (mission_name, params, requirements) = match mission_type {
        1 => (
            "Earth Observation",
            TransmissionParameters {
                distance_km: 800.0,
                data_size_mb: 1000.0,
                required_data_rate_mbps: 500.0,
                elevation_angle_degrees: 45.0,
                transmit_power_watts: 150.0,
                antenna_diameter_meters: 4.0,
            },
            "High data rate, weather resilience important",
        ),

        2 => (
            "Navigation/GPS",
            TransmissionParameters {
                distance_km: 20000.0,
                data_size_mb: 1.0,
                required_data_rate_mbps: 1.0,
                elevation_angle_degrees: 60.0,
                transmit_power_watts: 50.0,
                antenna_diameter_meters: 1.0,
            },
            "Global coverage, low power, high reliability",
        ),

        3 => (
            "Communications",
            TransmissionParameters {
                distance_km: 36000.0,
                data_size_mb: 100.0,
                required_data_rate_mbps: 100.0,
                elevation_angle_degrees: 30.0,
                transmit_power_watts: 200.0,
                antenna_diameter_meters: 3.5,
            },
            "Consistent quality, moderate data rates",
        ),

        4 => (
            "Military/Defense",
            TransmissionParameters {
                distance_km: 1500.0,
                data_size_mb: 200.0,
                required_data_rate_mbps: 200.0,
                elevation_angle_degrees: 25.0,
                transmit_power_watts: 250.0,
                antenna_diameter_meters: 5.0,
            },
            "Anti-jamming, secure, all-weather",
        ),

        5 => (
            "Deep Space",
            TransmissionParameters {
                distance_km: 150000000.0, // 150 million km
                data_size_mb: 50.0,
                required_data_rate_mbps: 0.1,
                elevation_angle_degrees: 45.0,
                transmit_power_watts: 400.0,
                antenna_diameter_meters: 70.0,
            },
            "Maximum sensitivity, very long range",
        ),

        _ => {
            println!("❌ Invalid mission type.");
            return Ok(());
        }
    };

    println!("\n🎯 Mission: {}", mission_name);
    println!("Requirements: {}", requirements);
    println!("Distance: {:.0} km", params.distance_km);
    println!("Data volume: {:.0} MB", params.data_size_mb);
    println!("Required rate: {:.1} Mbps", params.required_data_rate_mbps);

    // Test environmental conditions
    let environments = vec![
        (
            "Optimal",
            EnvironmentalConditions {
                rain_rate_mm_hour: 0.0,
                cloud_cover_percent: 0.0,
                atmospheric_pressure_mb: 1013.25,
                temperature_celsius: 20.0,
                humidity_percent: 30.0,
                ionospheric_activity: 0.05,
                solar_activity: 0.05,
            },
        ),
        (
            "Typical",
            EnvironmentalConditions {
                rain_rate_mm_hour: 2.0,
                cloud_cover_percent: 40.0,
                atmospheric_pressure_mb: 1010.0,
                temperature_celsius: 15.0,
                humidity_percent: 70.0,
                ionospheric_activity: 0.2,
                solar_activity: 0.2,
            },
        ),
        (
            "Adverse",
            EnvironmentalConditions {
                rain_rate_mm_hour: 15.0,
                cloud_cover_percent: 100.0,
                atmospheric_pressure_mb: 1000.0,
                temperature_celsius: 10.0,
                humidity_percent: 95.0,
                ionospheric_activity: 0.6,
                solar_activity: 0.5,
            },
        ),
    ];

    println!("\n📊 Band Suitability Analysis:");
    println!(
        "{:<12} {:>10} {:>10} {:>10} {:>15}",
        "Band", "Optimal", "Typical", "Adverse", "Recommendation"
    );
    println!("{}", "-".repeat(70));

    let bands = FrequencyBand::get_standard_bands();
    let mut recommendations = Vec::new();

    for band in &bands {
        print!("{:<12}", format!("{}", band.name));

        let mut scores = Vec::new();
        for (_, environment) in &environments {
            let result = band.simulate_transmission(&params, environment);
            let score = if result.success {
                (result.transmission_efficiency * 100.0) as i32
            } else {
                0
            };
            scores.push(score);
            print!(" {:>9}%", score);
        }

        // Calculate overall suitability
        let avg_score = scores.iter().sum::<i32>() as f32 / scores.len() as f32;
        let min_score = *scores.iter().min().unwrap();

        let recommendation = if avg_score >= 80.0 && min_score >= 60 {
            "🟢 EXCELLENT"
        } else if avg_score >= 60.0 && min_score >= 40 {
            "🟡 GOOD"
        } else if avg_score >= 40.0 {
            "🟠 FAIR"
        } else {
            "🔴 POOR"
        };

        recommendations.push((band.name, avg_score, recommendation));
        println!(" {:>15}", recommendation);
    }

    // Sort recommendations by score
    recommendations.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    println!("\n🏆 Recommended Bands for {} Mission:", mission_name);
    for (i, (band_name, score, status)) in recommendations.iter().take(3).enumerate() {
        println!(
            "  {}. {} - {:.0}% avg performance {}",
            i + 1,
            band_name,
            score,
            status
        );
    }

    Ok(())
}

fn run_real_time_simulation() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n⏰ Real-time Simulation");
    println!("{}", "=".repeat(35));
    println!("Watch how band performance changes with dynamic weather!");
    println!("Press Ctrl+C to stop the simulation.\n");

    let bands = FrequencyBand::get_standard_bands();

    let params = TransmissionParameters {
        distance_km: 1000.0,
        data_size_mb: 100.0,
        required_data_rate_mbps: 200.0,
        elevation_angle_degrees: 35.0,
        transmit_power_watts: 100.0,
        antenna_diameter_meters: 3.0,
    };

    let mut time = 0.0;
    let time_step = 0.5; // hours

    println!(
        "{:<6} {:>10} {:>10} {:>10} {:>10} {:>10} {:>10}",
        "Time", "Weather", "K-Band", "Ka-Band", "S-Band", "X-Band", "UHF-Band"
    );
    println!("{}", "-".repeat(75));

    for step in 0..20 {
        // 10 hours simulation
        time += time_step;

        // Simulate changing weather (sine wave pattern with random variations)
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let base_rain = 5.0
            * (time * 2.0 * std::f64::consts::PI / 24.0)
                .sin()
                .max(0.0_f64);
        let rain_rate = base_rain + rng.gen_range(-2.0..2.0).max(0.0_f64);

        let weather_desc = if rain_rate < 0.5 {
            "Clear"
        } else if rain_rate < 3.0 {
            "Light"
        } else if rain_rate < 8.0 {
            "Moderate"
        } else {
            "Heavy"
        };

        let environment = EnvironmentalConditions {
            rain_rate_mm_hour: rain_rate,
            cloud_cover_percent: rain_rate * 8.0,
            atmospheric_pressure_mb: 1013.25 - rain_rate,
            temperature_celsius: 20.0 - rain_rate * 0.5,
            humidity_percent: 50.0 + rain_rate * 3.0,
            ionospheric_activity: 0.1 + rng.gen_range(0.0..0.3),
            solar_activity: 0.1 + rng.gen_range(0.0..0.2),
        };

        print!("{:>5.1}h {:>10}", time, weather_desc);

        for band in &bands {
            let result = band.simulate_transmission(&params, &environment);
            let rate = (result.actual_data_rate_mbps / 10.0) as i32; // Scale for display
            print!(" {:>9}M", rate);
        }
        println!();

        // Sleep to create real-time effect
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    println!("\n💡 Observations:");
    println!("  • Weather patterns significantly affect higher frequency bands");
    println!("  • S-Band and UHF-Band maintain consistent performance");
    println!("  • Ka-Band shows highest variability with weather changes");

    Ok(())
}

fn display_band_characteristics() {
    println!("\n📚 Frequency Band Characteristics");
    println!("{}", "=".repeat(50));

    let bands = FrequencyBand::get_standard_bands();

    for band in &bands {
        println!("\n🔸 {}", band.name);
        println!(
            "   Frequency: {:.1} - {:.1} GHz",
            band.frequency_range.min_ghz, band.frequency_range.max_ghz
        );

        let purpose = match band.name {
            BandType::KBand => "High-speed data transmission",
            BandType::KaBand => "Ultra-high bandwidth applications",
            BandType::SBand => "Telemetry, tracking, and command (TT&C)",
            BandType::XBand => "Medium-speed data and radar",
            BandType::UHFBand => "Emergency and backup communications",
            _ => "General communications",
        };
        println!("   Purpose: {}", purpose);

        println!(
            "   Max Data Rate: {:.0} Mbps",
            band.characteristics.max_data_rate_mbps
        );
        println!(
            "   Rain Sensitivity: {:.0}%",
            band.limitations.rain_fade_susceptibility * 100.0
        );
        println!(
            "   Weather Dependence: {:.0}%",
            band.limitations.weather_dependence * 100.0
        );

        let pros_cons = match band.name {
            BandType::KBand => (
                vec![
                    "Good data rates",
                    "Moderate weather resistance",
                    "Proven technology",
                ],
                vec!["Rain fade effects", "Atmospheric absorption"],
            ),
            BandType::KaBand => (
                vec![
                    "Highest data rates",
                    "Compact antennas",
                    "Efficient spectrum use",
                ],
                vec![
                    "Very sensitive to rain",
                    "Requires precise pointing",
                    "Cloud attenuation",
                ],
            ),
            BandType::SBand => (
                vec!["Weather resistant", "Reliable", "Global coverage"],
                vec![
                    "Limited bandwidth",
                    "Interference issues",
                    "Large antennas needed",
                ],
            ),
            BandType::XBand => (
                vec![
                    "Good balance",
                    "Established infrastructure",
                    "Moderate data rates",
                ],
                vec!["Some weather sensitivity", "Spectrum congestion"],
            ),
            BandType::UHFBand => (
                vec!["Excellent reliability", "Low power", "All-weather"],
                vec!["Very limited bandwidth", "Large antennas", "Interference"],
            ),
            _ => (vec![], vec![]),
        };

        println!("   Advantages: {}", pros_cons.0.join(", "));
        println!("   Challenges: {}", pros_cons.1.join(", "));
    }
}

fn run_educational_mode() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🎓 Educational Mode - Learn About Frequency Bands");
    println!("{}", "=".repeat(55));

    println!("Welcome to the guided learning experience!");
    println!("We'll explore how different factors affect satellite communication.\n");

    // Lesson 1: Path Loss
    println!("📖 Lesson 1: Path Loss and Distance");
    println!("{}", "-".repeat(40));
    println!("Radio waves weaken as they travel through space following the");
    println!("inverse square law: doubling distance reduces signal by 6 dB.\n");

    let params = TransmissionParameters {
        distance_km: 1000.0,
        data_size_mb: 100.0,
        required_data_rate_mbps: 100.0,
        elevation_angle_degrees: 45.0,
        transmit_power_watts: 100.0,
        antenna_diameter_meters: 3.0,
    };

    let clear_env = EnvironmentalConditions {
        rain_rate_mm_hour: 0.0,
        cloud_cover_percent: 0.0,
        atmospheric_pressure_mb: 1013.25,
        temperature_celsius: 20.0,
        humidity_percent: 50.0,
        ionospheric_activity: 0.1,
        solar_activity: 0.1,
    };

    println!("Let's see how S-Band (reliable) performs at different distances:");
    let s_band = &FrequencyBand::get_standard_bands()[2]; // S-Band

    for &distance in &[500.0, 1000.0, 2000.0, 4000.0] {
        let mut test_params = params.clone();
        test_params.distance_km = distance;
        let result = s_band.simulate_transmission(&test_params, &clear_env);
        println!(
            "  {:.0} km: {:.0} Mbps",
            distance, result.actual_data_rate_mbps
        );
    }

    wait_for_enter();

    // Lesson 2: Frequency Effects
    println!("\n📖 Lesson 2: Frequency and Atmospheric Effects");
    println!("{}", "-".repeat(45));
    println!("Higher frequencies offer more bandwidth but are more susceptible");
    println!("to atmospheric effects like rain fade.\n");

    println!("Rain fade comparison (10 mm/hour rain):");
    let rainy_env = EnvironmentalConditions {
        rain_rate_mm_hour: 10.0,
        cloud_cover_percent: 100.0,
        atmospheric_pressure_mb: 1005.0,
        temperature_celsius: 15.0,
        humidity_percent: 90.0,
        ionospheric_activity: 0.2,
        solar_activity: 0.1,
    };

    let bands = FrequencyBand::get_standard_bands();
    for band in &bands {
        let clear_result = band.simulate_transmission(&params, &clear_env);
        let rain_result = band.simulate_transmission(&params, &rainy_env);
        let degradation = ((clear_result.actual_data_rate_mbps - rain_result.actual_data_rate_mbps)
            / clear_result.actual_data_rate_mbps
            * 100.0) as i32;

        println!("  {}: {}% degradation", band.name, degradation);
    }

    wait_for_enter();

    // Lesson 3: Trade-offs
    println!("\n📖 Lesson 3: Engineering Trade-offs");
    println!("{}", "-".repeat(35));
    println!("Satellite communication involves balancing multiple factors:");
    println!("• Data rate vs. reliability");
    println!("• Power consumption vs. performance");
    println!("• Weather sensitivity vs. bandwidth\n");

    println!("Mission type recommendations:");
    let scenarios = vec![
        (
            "Emergency/Disaster Relief",
            "UHF/S-Band",
            "Reliability over speed",
        ),
        (
            "Scientific Data Download",
            "Ka/K-Band",
            "High data rates when weather permits",
        ),
        (
            "Global Communications",
            "X/S-Band",
            "Good balance of performance and reliability",
        ),
        (
            "Military Operations",
            "Multiple Bands",
            "Redundancy and anti-jamming",
        ),
    ];

    for (mission, recommended, rationale) in scenarios {
        println!("  🎯 {}: Use {}", mission, recommended);
        println!("     Rationale: {}", rationale);
    }

    println!("\n🎉 Congratulations! You've completed the educational tour.");
    println!("You now understand the key principles of satellite frequency band selection!");

    Ok(())
}

fn get_parameter(prompt: &str, default: f64) -> Result<f64, Box<dyn std::error::Error>> {
    print!("{} [{}]: ", prompt, default);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if input.trim().is_empty() {
        Ok(default)
    } else {
        Ok(input.trim().parse().unwrap_or(default))
    }
}

fn wait_for_enter() {
    print!("\nPress Enter to continue...");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
}
