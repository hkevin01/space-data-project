use frequency_band_simulation::*;

/// Main entry point for frequency band simulation
fn main() {
    // Run a comprehensive demonstration of the frequency band simulation
    println!("===== SPACE FREQUENCY BAND SIMULATION =====");
    println!("Simulating satellite-ground communications across different bands\n");
    
    // Create a basic transmission scenario
    let ka_band = FrequencyBand::ka_band();
    let s_band = FrequencyBand::s_band();
    
    println!("Testing {} vs {}", ka_band.name, s_band.name);
    
    // Test under clear conditions
    println!("\n--- Clear Weather Conditions ---");
    let clear_result_ka = simulate_transmission(
        &ka_band,
        1000.0,   // 1000W transmit power
        0.7,      // 70cm dish efficiency
        100000.0, // 100km distance
        0.0,      // No rain
        25.0,     // 25°C
        60.0,     // 60% humidity
        1013.25   // Standard atmospheric pressure
    );
    
    let clear_result_s = simulate_transmission(
        &s_band,
        1000.0,
        0.7,
        100000.0,
        0.0,
        25.0,
        60.0,
        1013.25
    );
    
    println!("Ka-Band: SNR={:.1} dB, Data Rate={:.1} Mbps", 
             clear_result_ka.snr_db, clear_result_ka.data_rate_mbps);
    println!("S-Band:  SNR={:.1} dB, Data Rate={:.1} Mbps", 
             clear_result_s.snr_db, clear_result_s.data_rate_mbps);
    
    // Test under stormy conditions
    println!("\n--- Stormy Weather Conditions ---");
    let storm_result_ka = simulate_transmission(
        &ka_band,
        1000.0,
        0.7,
        100000.0,
        15.0,     // Heavy rain
        15.0,     // Cold temperature
        85.0,     // High humidity
        1008.0    // Low pressure
    );
    
    let storm_result_s = simulate_transmission(
        &s_band,
        1000.0,
        0.7,
        100000.0,
        15.0,
        15.0,
        85.0,
        1008.0
    );
    
    println!("Ka-Band: SNR={:.1} dB, Data Rate={:.1} Mbps (Rain fade: {:.1} dB)", 
             storm_result_ka.snr_db, storm_result_ka.data_rate_mbps, storm_result_ka.rain_fade_db);
    println!("S-Band:  SNR={:.1} dB, Data Rate={:.1} Mbps (Rain fade: {:.1} dB)", 
             storm_result_s.snr_db, storm_result_s.data_rate_mbps, storm_result_s.rain_fade_db);
    
    println!("\n================================================================");
    println!("Simulation completed successfully!");
}
    println!("🚀 Frequency Band Simulation Demonstration");
    println!("{}", "=".repeat(60));

    let bands = FrequencyBand::get_standard_bands();
    
    // Standard transmission parameters
    let params = TransmissionParameters {
        distance_km: 1000.0,
        data_size_mb: 100.0,
        required_data_rate_mbps: 100.0,
        elevation_angle_degrees: 30.0,
        transmit_power_watts: 100.0,
        antenna_diameter_meters: 3.0,
    };

    println!("\n📡 Transmission Parameters:");
    println!("  Distance: {} km", params.distance_km);
    println!("  Data Size: {} MB", params.data_size_mb);
    println!("  Required Data Rate: {} Mbps", params.required_data_rate_mbps);
    println!("  Elevation Angle: {}°", params.elevation_angle_degrees);
    println!("  Transmit Power: {} W", params.transmit_power_watts);

    // Test different weather conditions
    let weather_scenarios = vec![
        ("Clear Sky", EnvironmentalConditions {
            rain_rate_mm_hour: 0.0,
            cloud_cover_percent: 0.0,
            atmospheric_pressure_mb: 1013.25,
            temperature_celsius: 20.0,
            humidity_percent: 50.0,
            ionospheric_activity: 0.1,
            solar_activity: 0.1,
        }),
        ("Light Rain", EnvironmentalConditions {
            rain_rate_mm_hour: 2.0,
            cloud_cover_percent: 80.0,
            atmospheric_pressure_mb: 1010.0,
            temperature_celsius: 15.0,
            humidity_percent: 85.0,
            ionospheric_activity: 0.2,
            solar_activity: 0.1,
        }),
        ("Heavy Rain", EnvironmentalConditions {
            rain_rate_mm_hour: 15.0,
            cloud_cover_percent: 100.0,
            atmospheric_pressure_mb: 1005.0,
            temperature_celsius: 12.0,
            humidity_percent: 95.0,
            ionospheric_activity: 0.3,
            solar_activity: 0.2,
        }),
    ];

    for (weather_name, environment) in weather_scenarios {
        println!("\n🌦️  Weather Condition: {}", weather_name);
        println!("{}", "-".repeat(40));
        
        for band in &bands {
            let result = band.simulate_transmission(&params, &environment);
            println!("{}", result);
        }
    }

    println!("\n📊 Band Characteristics Summary:");
    println!("{}", "-".repeat(80));
    println!("{:<12} {:>12} {:>12} {:>12} {:>15}", 
             "Band", "Max Rate", "Frequency", "Rain Fade", "Weather Dep.");
    println!("{:<12} {:>12} {:>12} {:>12} {:>15}", 
             "", "(Mbps)", "(GHz)", "Suscept.", "Factor");
    println!("{}", "-".repeat(80));
    
    for band in &bands {
        println!("{:<12} {:>12.0} {:>12.1} {:>12.1} {:>15.1}", 
                 format!("{}", band.name),
                 band.characteristics.max_data_rate_mbps,
                 band.frequency_range.center_ghz,
                 band.limitations.rain_fade_susceptibility,
                 band.limitations.weather_dependence);
    }

    Ok(())
}

/// Run batch simulation over time period
fn run_batch_simulation(duration_hours: f64) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Running Batch Simulation for {:.1} hours", duration_hours);
    
    let bands = FrequencyBand::get_standard_bands();
    let time_step_minutes = 10.0;
    let num_steps = (duration_hours * 60.0 / time_step_minutes) as usize;
    
    // Generate dynamic weather conditions
    let weather_series = WeatherPatternGenerator::generate_dynamic_weather(
        duration_hours, 
        time_step_minutes
    );
    
    let params = TransmissionParameters {
        distance_km: 800.0,
        data_size_mb: 50.0,
        required_data_rate_mbps: 200.0,
        elevation_angle_degrees: 45.0,
        transmit_power_watts: 150.0,
        antenna_diameter_meters: 4.0,
    };

    // CSV output preparation
    let mut csv_data = Vec::new();
    let header = "Time(h),Band,Success,DataRate(Mbps),Latency(ms),SignalStrength(dB),ErrorRate,PowerConsumption(W),WeatherImpact,Efficiency(%)";
    csv_data.push(header.to_string());

    for (step, weather_conditions) in weather_series.iter().enumerate().take(num_steps) {
        let time_hours = step as f64 * time_step_minutes / 60.0;
        
        // Convert atmospheric conditions to environmental conditions
        let environment = EnvironmentalConditions {
            rain_rate_mm_hour: weather_conditions.rain_rate_mm_hour,
            cloud_cover_percent: weather_conditions.cloud_density * 100.0,
            atmospheric_pressure_mb: weather_conditions.pressure_mb,
            temperature_celsius: weather_conditions.temperature_celsius,
            humidity_percent: weather_conditions.humidity_percent,
            ionospheric_activity: 0.2,
            solar_activity: 0.1,
        };
        
        for band in &bands {
            let result = band.simulate_transmission(&params, &environment);
            
            let csv_line = format!(
                "{:.2},{},{},{:.2},{:.2},{:.1},{:.2e},{:.2},{:.3},{:.1}",
                time_hours,
                band.name,
                result.success,
                result.actual_data_rate_mbps,
                result.total_latency_ms,
                result.signal_strength_db,
                result.error_rate,
                result.power_consumption_watts,
                result.weather_impact_factor,
                result.transmission_efficiency * 100.0
            );
            csv_data.push(csv_line);
        }
        
        if step % 10 == 0 {
            print!("Progress: {:.1}%\r", (step as f64 / num_steps as f64) * 100.0);
            io::stdout().flush().unwrap();
        }
    }
    
    // Write CSV file
    let filename = format!("band_simulation_{:.0}h.csv", duration_hours);
    std::fs::write(&filename, csv_data.join("\n"))?;
    println!("\n✅ Results written to: {}", filename);
    
    Ok(())
}

/// Run interactive mode for exploring parameters
fn run_interactive_mode() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎮 Interactive Frequency Band Simulation");
    println!("{}", "=".repeat(50));
    
    let bands = FrequencyBand::get_standard_bands();
    
    loop {
        println!("\nAvailable Commands:");
        println!("  1. Simulate transmission");
        println!("  2. Compare bands");
        println!("  3. Weather impact analysis");
        println!("  4. Interference scenario");
        println!("  5. Rain fade analysis");
        println!("  q. Quit");
        
        print!("\nEnter choice: ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        match input.trim() {
            "1" => interactive_transmission_simulation(&bands)?,
            "2" => interactive_band_comparison(&bands)?,
            "3" => interactive_weather_analysis(&bands)?,
            "4" => interactive_interference_analysis()?,
            "5" => interactive_rain_fade_analysis()?,
            "q" | "quit" => break,
            _ => println!("Invalid choice, please try again."),
        }
    }
    
    Ok(())
}

/// Interactive transmission simulation
fn interactive_transmission_simulation(bands: &[FrequencyBand]) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📡 Transmission Simulation");
    
    // Get user parameters
    let distance = get_user_input("Distance (km)", "1000")?;
    let data_size = get_user_input("Data size (MB)", "100")?;
    let required_rate = get_user_input("Required data rate (Mbps)", "100")?;
    let elevation = get_user_input("Elevation angle (degrees)", "30")?;
    let power = get_user_input("Transmit power (W)", "100")?;
    let rain_rate = get_user_input("Rain rate (mm/hour)", "0")?;
    
    let params = TransmissionParameters {
        distance_km: distance,
        data_size_mb: data_size,
        required_data_rate_mbps: required_rate,
        elevation_angle_degrees: elevation,
        transmit_power_watts: power,
        antenna_diameter_meters: 3.0,
    };
    
    let environment = EnvironmentalConditions {
        rain_rate_mm_hour: rain_rate,
        cloud_cover_percent: if rain_rate > 0.0 { 80.0 } else { 20.0 },
        atmospheric_pressure_mb: 1013.25,
        temperature_celsius: 20.0,
        humidity_percent: 60.0,
        ionospheric_activity: 0.2,
        solar_activity: 0.1,
    };
    
    println!("\nSimulation Results:");
    println!("{}", "-".repeat(80));
    
    for band in bands {
        let result = band.simulate_transmission(&params, &environment);
        println!("{}", result);
    }
    
    Ok(())
}

/// Interactive band comparison
fn interactive_band_comparison(bands: &[FrequencyBand]) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📊 Band Comparison Analysis");
    
    let params = TransmissionParameters {
        distance_km: 1000.0,
        data_size_mb: 100.0,
        required_data_rate_mbps: 500.0,
        elevation_angle_degrees: 30.0,
        transmit_power_watts: 100.0,
        antenna_diameter_meters: 3.0,
    };
    
    // Test across different rain rates
    let rain_rates = vec![0.0, 1.0, 5.0, 10.0, 25.0, 50.0];
    
    println!("\nPerformance vs Rain Rate:");
    println!("{:<12} {:>8} {:>12} {:>12} {:>12}", "Band", "Rain(mm/h)", "Success", "Rate(Mbps)", "Efficiency(%)");
    println!("{}", "-".repeat(60));
    
    for &rain_rate in &rain_rates {
        let environment = EnvironmentalConditions {
            rain_rate_mm_hour: rain_rate,
            cloud_cover_percent: rain_rate * 2.0,
            atmospheric_pressure_mb: 1013.25,
            temperature_celsius: 15.0,
            humidity_percent: 70.0 + rain_rate,
            ionospheric_activity: 0.2,
            solar_activity: 0.1,
        };
        
        for band in bands {
            let result = band.simulate_transmission(&params, &environment);
            println!("{:<12} {:>8.1} {:>12} {:>12.0} {:>12.1}", 
                     format!("{}", band.name),
                     rain_rate,
                     if result.success { "✅" } else { "❌" },
                     result.actual_data_rate_mbps,
                     result.transmission_efficiency * 100.0);
        }
        println!();
    }
    
    Ok(())
}

/// Interactive weather analysis
fn interactive_weather_analysis(bands: &[FrequencyBand]) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🌦️  Weather Impact Analysis");
    
    let weather_scenarios = atmospheric::WeatherPatternGenerator::generate_weather_scenarios();
    
    let params = TransmissionParameters {
        distance_km: 800.0,
        data_size_mb: 200.0,
        required_data_rate_mbps: 300.0,
        elevation_angle_degrees: 45.0,
        transmit_power_watts: 120.0,
        antenna_diameter_meters: 3.5,
    };
    
    for (weather_name, atmospheric_conditions) in weather_scenarios {
        println!("\n--- {} ---", weather_name);
        
        let environment = EnvironmentalConditions {
            rain_rate_mm_hour: atmospheric_conditions.rain_rate_mm_hour,
            cloud_cover_percent: atmospheric_conditions.cloud_density * 100.0,
            atmospheric_pressure_mb: atmospheric_conditions.pressure_mb,
            temperature_celsius: atmospheric_conditions.temperature_celsius,
            humidity_percent: atmospheric_conditions.humidity_percent,
            ionospheric_activity: 0.2,
            solar_activity: 0.1,
        };
        
        for band in bands {
            let result = band.simulate_transmission(&params, &environment);
            if result.weather_impact_factor > 0.1 {
                println!("  {}: Weather impact {:.1}%, Efficiency {:.1}%", 
                         band.name, 
                         result.weather_impact_factor * 100.0,
                         result.transmission_efficiency * 100.0);
            }
        }
    }
    
    Ok(())
}

/// Interactive interference analysis
fn interactive_interference_analysis() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📻 Interference Analysis");
    
    let jamming_scenarios = JammingSimulator::generate_jamming_scenarios();
    
    for (scenario_name, interference_source) in jamming_scenarios {
        println!("\n--- {} ---", scenario_name);
        println!("  Power: {:.1} dBm", interference_source.power_dbm);
        println!("  Frequency: {:.1} GHz", interference_source.frequency_ghz);
        println!("  Bandwidth: {:.1} MHz", interference_source.bandwidth_mhz);
        
        // Calculate interference impact
        match interference_source.location {
            interference::SourceLocation::Terrestrial { distance_km, azimuth_deg } => {
                println!("  Location: {:.1} km at {:.0}° azimuth", distance_km, azimuth_deg);
                
                // Simulate jamming effectiveness
                let jammer_effectiveness = match interference_source.source_type {
                    interference::InterferenceType::Jamming => {
                        JammingSimulator::simulate_barrage_jamming(
                            interference_source.power_dbm,
                            interference_source.bandwidth_mhz,
                            100.0, // Assumed signal bandwidth
                            distance_km,
                        )
                    },
                    _ => -999.0,
                };
                
                if jammer_effectiveness > -999.0 {
                    println!("  Received Power: {:.1} dBm", jammer_effectiveness);
                    if jammer_effectiveness > -80.0 {
                        println!("  Impact: 🔴 HIGH - Communication severely degraded");
                    } else if jammer_effectiveness > -100.0 {
                        println!("  Impact: 🟡 MODERATE - Some degradation expected");
                    } else {
                        println!("  Impact: 🟢 LOW - Minimal impact");
                    }
                }
            },
            _ => println!("  Location: Other"),
        }
    }
    
    Ok(())
}

/// Interactive rain fade analysis
fn interactive_rain_fade_analysis() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🌧️  Rain Fade Analysis");
    
    let bands = FrequencyBand::get_standard_bands();
    let rain_scenarios = RainFadeModel::generate_rain_scenarios();
    
    println!("\nRain fade attenuation (dB) for 1000 km path at 30° elevation:");
    println!("{:<20} {:>10} {:>10} {:>10} {:>10} {:>10}", 
             "Rain Condition", "K-Band", "Ka-Band", "S-Band", "X-Band", "UHF-Band");
    println!("{}", "-".repeat(80));
    
    for (rain_name, rain_rate) in rain_scenarios {
        print!("{:<20}", rain_name);
        
        for band in &bands {
            let attenuation = RainFadeModel::calculate_rain_attenuation(
                band.frequency_range.center_ghz,
                rain_rate,
                1000.0, // path length
                30.0,   // elevation angle
            );
            print!(" {:>10.1}", attenuation);
        }
        println!();
    }
    
    Ok(())
}

/// Run stress test with extreme conditions
fn run_stress_test() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔥 Running Stress Test - Extreme Conditions");
    
    let bands = FrequencyBand::get_standard_bands();
    
    let extreme_conditions = vec![
        ("Extreme Rain", EnvironmentalConditions {
            rain_rate_mm_hour: 100.0,
            cloud_cover_percent: 100.0,
            atmospheric_pressure_mb: 980.0,
            temperature_celsius: 5.0,
            humidity_percent: 100.0,
            ionospheric_activity: 0.9,
            solar_activity: 0.9,
        }),
        ("Solar Storm", EnvironmentalConditions {
            rain_rate_mm_hour: 0.0,
            cloud_cover_percent: 0.0,
            atmospheric_pressure_mb: 1013.25,
            temperature_celsius: 20.0,
            humidity_percent: 50.0,
            ionospheric_activity: 1.0,
            solar_activity: 1.0,
        }),
        ("Perfect Conditions", EnvironmentalConditions {
            rain_rate_mm_hour: 0.0,
            cloud_cover_percent: 0.0,
            atmospheric_pressure_mb: 1013.25,
            temperature_celsius: 20.0,
            humidity_percent: 30.0,
            ionospheric_activity: 0.0,
            solar_activity: 0.0,
        }),
    ];
    
    let stress_params = TransmissionParameters {
        distance_km: 2000.0, // Very long distance
        data_size_mb: 1000.0, // Large data transfer
        required_data_rate_mbps: 1000.0, // High data rate requirement
        elevation_angle_degrees: 10.0, // Low elevation (worst case)
        transmit_power_watts: 50.0, // Limited power
        antenna_diameter_meters: 2.0, // Smaller antenna
    };
    
    for (condition_name, environment) in extreme_conditions {
        println!("\n🌪️  Condition: {}", condition_name);
        println!("{}", "-".repeat(50));
        
        for band in &bands {
            let result = band.simulate_transmission(&stress_params, &environment);
            
            let status_icon = if result.success {
                if result.transmission_efficiency > 0.8 { "🟢" }
                else if result.transmission_efficiency > 0.5 { "🟡" }
                else { "🟠" }
            } else { "🔴" };
            
            println!("{} {}: {:.0} Mbps, {:.1}% efficiency", 
                     status_icon,
                     band.name,
                     result.actual_data_rate_mbps,
                     result.transmission_efficiency * 100.0);
        }
    }
    
    Ok(())
}

/// Run comprehensive band comparison
fn run_band_comparison() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Comprehensive Band Comparison");
    
    let bands = FrequencyBand::get_standard_bands();
    
    // Test matrix: different distances and weather conditions
    let distances = vec![500.0, 1000.0, 2000.0, 5000.0];
    let weather_conditions = vec![
        ("Clear", 0.0, 0.0),
        ("Light Rain", 2.0, 50.0),
        ("Heavy Rain", 15.0, 100.0),
    ];
    
    for (weather_name, rain_rate, cloud_cover) in weather_conditions {
        println!("\n🌦️  Weather: {}", weather_name);
        println!("{:<12} {:>8} {:>12} {:>12} {:>12}", "Band", "Dist(km)", "Success", "Rate(Mbps)", "Power(W)");
        println!("{}", "-".repeat(60));
        
        for &distance in &distances {
            let params = TransmissionParameters {
                distance_km: distance,
                data_size_mb: 100.0,
                required_data_rate_mbps: 200.0,
                elevation_angle_degrees: 30.0,
                transmit_power_watts: 100.0,
                antenna_diameter_meters: 3.0,
            };
            
            let environment = EnvironmentalConditions {
                rain_rate_mm_hour: rain_rate,
                cloud_cover_percent: cloud_cover,
                atmospheric_pressure_mb: 1013.25,
                temperature_celsius: 15.0,
                humidity_percent: 70.0,
                ionospheric_activity: 0.2,
                solar_activity: 0.1,
            };
            
            for band in &bands {
                let result = band.simulate_transmission(&params, &environment);
                println!("{:<12} {:>8.0} {:>12} {:>12.0} {:>12.1}", 
                         format!("{}", band.name),
                         distance,
                         if result.success { "✅" } else { "❌" },
                         result.actual_data_rate_mbps,
                         result.power_consumption_watts);
            }
            println!();
        }
    }
    
    Ok(())
}

/// Run weather impact analysis
fn run_weather_impact_analysis() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌦️  Weather Impact Analysis");
    
    let bands = FrequencyBand::get_standard_bands();
    let rain_rates = vec![0.0, 1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 100.0];
    
    let params = TransmissionParameters {
        distance_km: 1000.0,
        data_size_mb: 100.0,
        required_data_rate_mbps: 500.0,
        elevation_angle_degrees: 30.0,
        transmit_power_watts: 100.0,
        antenna_diameter_meters: 3.0,
    };
    
    println!("\nTransmission Success Rate vs Rain Rate:");
    println!("{:<12} {:>8} {:>10} {:>10} {:>10} {:>10} {:>10} {:>10} {:>10}", 
             "Band", "0mm/h", "1mm/h", "2mm/h", "5mm/h", "10mm/h", "20mm/h", "50mm/h", "100mm/h");
    println!("{}", "-".repeat(90));
    
    for band in &bands {
        print!("{:<12}", format!("{}", band.name));
        
        for &rain_rate in &rain_rates {
            let environment = EnvironmentalConditions {
                rain_rate_mm_hour: rain_rate,
                cloud_cover_percent: rain_rate * 2.0,
                atmospheric_pressure_mb: 1013.25,
                temperature_celsius: 15.0,
                humidity_percent: 60.0 + rain_rate,
                ionospheric_activity: 0.2,
                solar_activity: 0.1,
            };
            
            let result = band.simulate_transmission(&params, &environment);
            let efficiency_percent = (result.transmission_efficiency * 100.0) as i32;
            
            print!(" {:>10}%", efficiency_percent);
        }
        println!();
    }
    
    Ok(())
}

/// Run interference analysis
fn run_interference_analysis() -> Result<(), Box<dyn std::error::Error>> {
    println!("📻 Interference Analysis");
    
    let interference_scenarios = InterferenceAnalyzer::generate_interference_scenarios();
    
    for (i, scenario) in interference_scenarios.iter().enumerate() {
        println!("\n--- Interference Scenario {} ---", i + 1);
        
        let total_interference = InterferenceAnalyzer::calculate_total_interference(
            scenario,
            10.0,  // 10 GHz receiver
            100.0, // 100 MHz bandwidth
            (40.0, -74.0), // New York coordinates
        );
        
        println!("Total Interference: {:.1} dBm", total_interference);
        
        // Calculate SIR for different signal levels
        let signal_levels = vec![-60.0, -70.0, -80.0, -90.0];
        
        for signal_level in signal_levels {
            let sir = InterferenceAnalyzer::calculate_sir(
                signal_level,
                scenario,
                10.0,
                100.0,
                (40.0, -74.0),
            );
            
            println!("Signal: {:.0} dBm, SIR: {:.1} dB", signal_level, sir);
        }
    }
    
    Ok(())
}

/// Helper function to get user input with default value
fn get_user_input(prompt: &str, default: &str) -> Result<f64, Box<dyn std::error::Error>> {
    print!("{} [{}]: ", prompt, default);
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    let trimmed = input.trim();
    if trimmed.is_empty() {
        Ok(default.parse()?)
    } else {
        Ok(trimmed.parse()?)
    }
}
