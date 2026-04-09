use frequency_band_simulation::advanced_rf;
use frequency_band_simulation::*;

/// Main entry point for frequency band simulation
fn main() {
    println!("===== SPACE FREQUENCY BAND SIMULATION =====");
    println!("Simulating satellite-ground communications across different bands\n");

    // Run the basic demonstration from the library
    run_basic_demo();

    println!("\n================================================================");
    println!("===== ADVANCED RF TECHNIQUES DEMONSTRATION =====\n");

    let summary = advanced_rf::run_advanced_rf_demo();

    // --- Beamforming ---
    println!("1. BEAMFORMING (16-element ULA, Adaptive, X-Band, 40 W)");
    println!("   Array Gain     : {:.1} dBi", summary.beamforming.array_gain_dbi);
    println!("   HPBW           : {:.2}°", summary.beamforming.hpbw_degrees);
    println!("   Null Depth     : {:.1} dB", summary.beamforming.null_depth_db);
    println!("   EIRP           : {:.1} dBW\n", summary.beamforming.eirp_dbw);

    // --- MIMO ---
    println!("2. MIMO (4×4 Spatial Multiplexing, 30 dB SNR, 500 MHz)");
    println!("   Spatial Streams: {}", summary.mimo.spatial_streams);
    println!("   Capacity       : {:.0} Mbps", summary.mimo.channel_capacity_mbps);
    println!("   Gain vs SISO   : {:.1}×\n", summary.mimo.capacity_gain_vs_siso);

    // --- DSSS ---
    println!("3. DSSS (1023-chip Gold code, 10 kbps data, 10.23 Mcps)");
    println!("   Processing Gain: {:.1} dB", summary.dsss.processing_gain_db);
    println!("   Post-despread SNR: {:.1} dB", summary.dsss.post_despread_snr_db);
    println!("   Jamming Margin : {:.1} dB", summary.dsss.jamming_margin_db);
    println!("   Jam Resistant  : {}\n", summary.dsss.jam_resistant);

    // --- FHSS ---
    println!("4. FHSS (128 channels, fast hopping, 5% jammed)");
    println!("   Spread BW      : {:.0} MHz", summary.fhss.total_spread_bandwidth_mhz);
    println!("   Processing Gain: {:.1} dB", summary.fhss.processing_gain_db);
    println!("   Throughput     : {:.0}%", summary.fhss.throughput_factor * 100.0);
    println!("   Anti-Jam Rating: {}\n", summary.fhss.anti_jam_rating);

    // --- AMC ---
    println!("5. ADAPTIVE MODULATION & CODING (30 dB SNR, 500 MHz)");
    println!("   Modulation     : {:?}", summary.amc.modulation);
    println!("   Coding Rate    : {:?}", summary.amc.coding_rate);
    println!("   Throughput     : {:.0} Mbps", summary.amc.throughput_mbps);
    println!("   Spectral Eff.  : {:.2} bps/Hz", summary.amc.spectral_efficiency);
    println!("   Link Margin    : {:.1} dB\n", summary.amc.link_margin_db);

    // --- Polarization Diversity ---
    println!("6. POLARIZATION DIVERSITY (Dual-Circular, 30 dB XPD, 20° Faraday)");
    println!("   Channel Isolation: {:.1} dB", summary.polarization.channel_isolation_db);
    println!("   Pol Loss       : {:.2} dB", summary.polarization.polarization_loss_db);
    println!("   Capacity ×     : {:.1}×", summary.polarization.capacity_multiplier);
    println!("   Faraday Correct: {}\n", summary.polarization.faraday_correction_needed);

    println!("================================================================");
    println!("Simulation completed successfully!");
}
