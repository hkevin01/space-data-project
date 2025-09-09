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
