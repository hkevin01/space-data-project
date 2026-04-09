//! Advanced RF Techniques Module
//!
//! This module models six advanced radio-frequency techniques used in
//! mission-critical satellite and ground-station communication systems:
//!
//! 1. **Beamforming**            — phased-array spatial steering and nulling
//! 2. **MIMO**                   — spatial multiplexing for capacity scaling
//! 3. **DSSS**                   — direct-sequence spread spectrum (anti-jam / LPI)
//! 4. **FHSS**                   — frequency-hopping spread spectrum (anti-intercept)
//! 5. **Adaptive Modulation & Coding (AMC)** — link-adaptive waveform selection
//! 6. **Polarization Diversity** — dual-polarization isolation and combining
//!
//! # Requirements Traceability
//! - REQ-FN-009: Advanced RF Signal Processing
//! - REQ-FN-010: Anti-Jam / LPI/LPD Communication
//! - REQ-PF-003: Link Capacity Optimisation
//! - REQ-SE-002: Interference Mitigation

use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────────────────────
// 1. BEAMFORMING
// ─────────────────────────────────────────────────────────────────────────────

/// Type of beamforming algorithm employed by the phased array.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BeamformingMode {
    /// Fixed phase-shift weights set at design time (analog beamforming).
    Analog,
    /// Weights computed in the digital baseband after ADC (digital beamforming).
    Digital,
    /// Weights continuously adapted to maximise SINR (adaptive / MVDR).
    Adaptive,
}

/// Configuration of a phased-array antenna for beamforming.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhasedArrayConfig {
    /// Number of antenna elements in the array.
    pub num_elements: usize,
    /// Element spacing as a fraction of wavelength (λ). 0.5 = λ/2 spacing.
    pub element_spacing_lambda: f64,
    /// Operating carrier frequency in GHz.
    pub carrier_freq_ghz: f64,
    /// Beamforming algorithm selection.
    pub mode: BeamformingMode,
    /// Maximum transmit power distributed across all elements (watts).
    pub total_power_watts: f64,
}

/// Result of a beamforming simulation for a single target direction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeamformingResult {
    /// Achieved array gain in dBi toward the target.
    pub array_gain_dbi: f64,
    /// Half-power beamwidth in degrees (3 dB).
    pub hpbw_degrees: f64,
    /// Deepest null depth achieved toward the interference direction (dB below main lobe).
    pub null_depth_db: f64,
    /// Effective interference rejection ratio in dB.
    pub interference_rejection_db: f64,
    /// EIRP (effective isotropic radiated power) in dBW toward target.
    pub eirp_dbw: f64,
}

impl PhasedArrayConfig {
    /// Simulate beamforming toward `target_angle_deg` with an optional interference
    /// source at `interference_angle_deg`.
    ///
    /// Returns antenna gain, beamwidth, and null depth achievable by this array.
    ///
    /// # Array gain formula
    /// For a uniform linear array (ULA) with *N* isotropic elements at λ/2 spacing,
    /// the maximum array gain is 10 · log₁₀(N) dBi.  The half-power beamwidth
    /// approximates to 0.886 · λ / (N · d) radians.
    pub fn simulate_beamforming(
        &self,
        target_angle_deg: f64,
        interference_angle_deg: Option<f64>,
    ) -> BeamformingResult {
        let n = self.num_elements as f64;

        // Array gain: N elements coherently add E-fields → N² power gain / N aperture → N.
        let array_gain_linear = n;
        let array_gain_dbi = 10.0 * array_gain_linear.log10();

        // HPBW for ULA: θ₃dB ≈ 0.886 / (N · d/λ)  [radians] → degrees
        let hpbw_degrees = (0.886 / (n * self.element_spacing_lambda)).to_degrees();

        // Null steering depth depends on mode.
        let (null_depth_db, interference_rejection_db) =
            match (self.mode, interference_angle_deg) {
                (BeamformingMode::Analog, Some(int_angle)) => {
                    let sep = (target_angle_deg - int_angle).abs();
                    // Analog: null depth limited by quantisation; ~20–30 dB typical.
                    let depth = if sep > hpbw_degrees { 25.0 } else { 12.0 };
                    (depth, depth - 3.0)
                }
                (BeamformingMode::Digital, Some(int_angle)) => {
                    let sep = (target_angle_deg - int_angle).abs();
                    // Digital: per-element weights → up to ~40 dB nulls.
                    let depth = if sep > hpbw_degrees / 2.0 { 40.0 } else { 20.0 };
                    (depth, depth - 2.0)
                }
                (BeamformingMode::Adaptive, Some(int_angle)) => {
                    let sep = (target_angle_deg - int_angle).abs();
                    // Adaptive (MVDR/LCMV): theoretical depth limited by element errors;
                    // ≥ 50 dB practical null depth when angular separation > beamwidth/4.
                    let depth = if sep > hpbw_degrees / 4.0 { 55.0 } else { 30.0 };
                    (depth, depth - 1.5)
                }
                _ => (0.0, 0.0), // No interference source specified
            };

        // EIRP: P_tx_dBW + array_gain_dBi
        let tx_power_dbw = 10.0 * self.total_power_watts.log10();
        let eirp_dbw = tx_power_dbw + array_gain_dbi;

        BeamformingResult {
            array_gain_dbi,
            hpbw_degrees,
            null_depth_db,
            interference_rejection_db,
            eirp_dbw,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. MIMO (Multiple Input Multiple Output)
// ─────────────────────────────────────────────────────────────────────────────

/// MIMO operating regime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MimoMode {
    /// Independent data streams sent over each TX–RX path (spatial multiplexing).
    SpatialMultiplexing,
    /// Same data on every TX/RX path for maximum reliability (diversity combining).
    TransmitDiversity,
    /// Massive MIMO: large TX array with multi-user zeroforcing precoding.
    MassiveMimo,
}

/// MIMO system configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MimoConfig {
    /// Number of transmit antennas.
    pub num_tx: usize,
    /// Number of receive antennas.
    pub num_rx: usize,
    /// MIMO operating mode.
    pub mode: MimoMode,
    /// Channel signal-to-noise ratio at the receiver (dB).
    pub channel_snr_db: f64,
    /// Available channel bandwidth in MHz.
    pub bandwidth_mhz: f64,
}

/// Results of a MIMO link simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MimoResult {
    /// Number of independent spatial streams (≤ min(Tx, Rx)).
    pub spatial_streams: usize,
    /// Theoretical channel capacity in Mbps (Shannon with MIMO multiplexing).
    pub channel_capacity_mbps: f64,
    /// Capacity gain versus a SISO link with the same total power and bandwidth.
    pub capacity_gain_vs_siso: f64,
    /// Achieved diversity order (relevant in transmit-diversity mode).
    pub diversity_order: usize,
    /// Estimated post-combining SNR at the receiver (dB).
    pub effective_snr_db: f64,
}

impl MimoConfig {
    /// Simulate MIMO channel capacity using the Shannon–Foschini formula.
    ///
    /// For spatial multiplexing with *N_s* streams each allocated 1/N_s of the
    /// total SNR:  C = N_s · B · log₂(1 + SNR/N_s)
    pub fn simulate(&self) -> MimoResult {
        let n_tx = self.num_tx as f64;
        let n_rx = self.num_rx as f64;
        let snr_linear = 10.0_f64.powf(self.channel_snr_db / 10.0);

        let (spatial_streams, capacity_mbps, diversity_order, effective_snr_db) = match self.mode {
            MimoMode::SpatialMultiplexing => {
                let ns = self.num_tx.min(self.num_rx);
                let ns_f = ns as f64;
                // Each stream receives SNR / Ns (equal power split).
                let per_stream_snr = snr_linear / ns_f;
                let capacity = ns_f * self.bandwidth_mhz * (1.0 + per_stream_snr).log2();
                // Effective SNR after MRC combining across ns streams.
                let eff_snr_db = self.channel_snr_db; // SNR maintained, capacity scales.
                (ns, capacity, 1, eff_snr_db)
            }
            MimoMode::TransmitDiversity => {
                // Alamouti / STBC: diversity order = Tx × Rx.
                let d_order = self.num_tx * self.num_rx;
                // SNR gain from combining: ~10 · log10(Tx · Rx) dB in ideal channel.
                let snr_gain_db = 10.0 * (n_tx * n_rx).log10();
                let eff_snr_db = self.channel_snr_db + snr_gain_db;
                let eff_snr_lin = 10.0_f64.powf(eff_snr_db / 10.0);
                let capacity = self.bandwidth_mhz * (1.0 + eff_snr_lin).log2();
                (1, capacity, d_order, eff_snr_db)
            }
            MimoMode::MassiveMimo => {
                // Massive MIMO (N_tx >> N_rx): near-orthogonal channels via ZF precoding.
                // Capacity scales as N_rx · B · log2(1 + N_tx · SNR / N_rx).
                let effective_snr_per_user = snr_linear * n_tx / n_rx;
                let capacity =
                    n_rx * self.bandwidth_mhz * (1.0 + effective_snr_per_user).log2();
                let eff_snr_db =
                    10.0 * effective_snr_per_user.log10();
                (self.num_rx, capacity, self.num_tx, eff_snr_db)
            }
        };

        // SISO baseline for gain comparison.
        let siso_capacity = self.bandwidth_mhz * (1.0 + snr_linear).log2();
        let gain = if siso_capacity > 0.0 {
            capacity_mbps / siso_capacity
        } else {
            1.0
        };

        MimoResult {
            spatial_streams,
            channel_capacity_mbps: capacity_mbps,
            capacity_gain_vs_siso: gain,
            diversity_order,
            effective_snr_db,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. DSSS — Direct-Sequence Spread Spectrum
// ─────────────────────────────────────────────────────────────────────────────

/// DSSS configuration parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DsssConfig {
    /// Information bit rate in kbps.
    pub data_rate_kbps: f64,
    /// Chip rate (PN code rate) in Mcps (mega chips per second).
    pub chip_rate_mcps: f64,
    /// Received signal-to-noise ratio before despreading (dB).
    pub input_snr_db: f64,
    /// Jammer-to-signal ratio at the receiver (dB). Set to 0 for no jamming.
    pub jammer_to_signal_db: f64,
}

/// Results of a DSSS link analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DsssResult {
    /// Processing gain: chip_rate / data_rate (linear then converted to dB).
    pub processing_gain_db: f64,
    /// SNR after despreading (input_snr + processing_gain, dB).
    pub post_despread_snr_db: f64,
    /// Effective jamming margin: processing_gain − required_eb_n0, dB.
    pub jamming_margin_db: f64,
    /// Whether the link survives the modelled jamming environment.
    pub jam_resistant: bool,
    /// Low-probability-of-intercept spectral density (W/Hz), normalised to 1 W TX.
    pub spectral_density_w_per_hz: f64,
}

/// Minimum Eb/N0 required for BER < 10⁻⁵ with BPSK (≈9.6 dB).
const BPSK_MIN_EB_N0_DB: f64 = 9.6;

impl DsssConfig {
    /// Compute DSSS processing gain, post-despread SNR, and jamming margin.
    pub fn analyse(&self) -> DsssResult {
        // Processing gain Gp = chip_rate / data_rate.
        let chip_rate_kbps = self.chip_rate_mcps * 1_000.0;
        let gp_linear = chip_rate_kbps / self.data_rate_kbps;
        let processing_gain_db = 10.0 * gp_linear.log10();

        // Post-despread SNR.
        let post_despread_snr_db = self.input_snr_db + processing_gain_db;

        // Jamming margin: how much jammer power the link can tolerate.
        let jamming_margin_db = processing_gain_db - BPSK_MIN_EB_N0_DB;

        // Effective jammer impact: J/S − processing_gain (negative → jammer suppressed).
        let effective_jammer_db = self.jammer_to_signal_db - processing_gain_db;
        let jam_resistant = effective_jammer_db < 0.0;

        // LPI spectral density: 1 W spread over chip_rate bandwidth.
        let bandwidth_hz = self.chip_rate_mcps * 1e6;
        let spectral_density_w_per_hz = 1.0 / bandwidth_hz;

        DsssResult {
            processing_gain_db,
            post_despread_snr_db,
            jamming_margin_db,
            jam_resistant,
            spectral_density_w_per_hz,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. FHSS — Frequency-Hopping Spread Spectrum
// ─────────────────────────────────────────────────────────────────────────────

/// FHSS hopping rate classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HoppingRate {
    /// Multiple hops per data symbol — harder to follow, better anti-jam.
    Fast,
    /// One or more symbols per hop — simpler hardware, moderate resilience.
    Slow,
}

/// FHSS system configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FhssConfig {
    /// Total number of available frequency channels.
    pub num_channels: usize,
    /// Bandwidth of each individual channel in MHz.
    pub channel_bandwidth_mhz: f64,
    /// Hop rate: number of frequency changes per second.
    pub hop_rate_hz: f64,
    /// Slow or fast hopping classification.
    pub hopping_rate: HoppingRate,
    /// Fraction of channels permanently jammed (0.0–1.0).
    pub jammed_channel_fraction: f64,
}

/// Results of an FHSS anti-jam / spread analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FhssResult {
    /// Total spread bandwidth (num_channels × channel_bandwidth_mhz), MHz.
    pub total_spread_bandwidth_mhz: f64,
    /// Processing gain in dB (total_bw / channel_bw → log scale).
    pub processing_gain_db: f64,
    /// Probability that a given hop lands on a jammed channel.
    pub jam_hit_probability: f64,
    /// Average throughput degradation factor due to jamming (0.0 = no data, 1.0 = full rate).
    pub throughput_factor: f64,
    /// Estimated resistance rating: "High" / "Moderate" / "Low".
    pub anti_jam_rating: &'static str,
}

impl FhssConfig {
    /// Analyse FHSS spread bandwidth, processing gain, and jamming resilience.
    pub fn analyse(&self) -> FhssResult {
        let total_bw = self.num_channels as f64 * self.channel_bandwidth_mhz;
        let pg_linear = self.num_channels as f64;
        let processing_gain_db = 10.0 * pg_linear.log10();

        let jam_hit_probability = self.jammed_channel_fraction.clamp(0.0, 1.0);

        // Expected fraction of hops NOT jammed.
        let throughput_factor = match self.hopping_rate {
            HoppingRate::Fast => {
                // Fast hopping: each symbol spread; partial channel jamming still allows symbol recovery.
                (1.0 - jam_hit_probability).powf(1.5)
            }
            HoppingRate::Slow => {
                // Slow hopping: entire burst lost if hop lands on jammed channel.
                1.0 - jam_hit_probability
            }
        };

        let anti_jam_rating = if processing_gain_db >= 20.0 && jam_hit_probability < 0.1 {
            "High"
        } else if processing_gain_db >= 10.0 && jam_hit_probability < 0.3 {
            "Moderate"
        } else {
            "Low"
        };

        FhssResult {
            total_spread_bandwidth_mhz: total_bw,
            processing_gain_db,
            jam_hit_probability,
            throughput_factor,
            anti_jam_rating,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. Adaptive Modulation and Coding (AMC / ACM)
// ─────────────────────────────────────────────────────────────────────────────

/// Supported modulation schemes ranked by spectral efficiency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ModulationScheme {
    Bpsk,   // 1 bit/symbol
    Qpsk,   // 2 bits/symbol
    Qam16,  // 4 bits/symbol
    Qam64,  // 6 bits/symbol
    Qam256, // 8 bits/symbol
}

impl ModulationScheme {
    /// Spectral efficiency in bits per symbol.
    pub fn bits_per_symbol(self) -> f64 {
        match self {
            ModulationScheme::Bpsk => 1.0,
            ModulationScheme::Qpsk => 2.0,
            ModulationScheme::Qam16 => 4.0,
            ModulationScheme::Qam64 => 6.0,
            ModulationScheme::Qam256 => 8.0,
        }
    }

    /// Minimum Eb/N0 threshold for BER < 10⁻⁶ (approximated from standard tables).
    pub fn min_eb_n0_db(self) -> f64 {
        match self {
            ModulationScheme::Bpsk => 10.5,
            ModulationScheme::Qpsk => 10.5, // same as BPSK per symbol, same BER curve
            ModulationScheme::Qam16 => 14.5,
            ModulationScheme::Qam64 => 18.8,
            ModulationScheme::Qam256 => 24.0,
        }
    }
}

/// Coding rate options for Forward Error Correction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CodingRate {
    Rate1_2, // 1/2 — robust, 3 dB coding gain over uncoded
    Rate2_3, // 2/3
    Rate3_4, // 3/4
    Rate5_6, // 5/6
    Rate7_8, // 7/8 — near-uncoded, maximum spectral efficiency
}

impl CodingRate {
    /// Fraction of payload bits to total coded bits.
    pub fn ratio(self) -> f64 {
        match self {
            CodingRate::Rate1_2 => 0.5,
            CodingRate::Rate2_3 => 2.0 / 3.0,
            CodingRate::Rate3_4 => 0.75,
            CodingRate::Rate5_6 => 5.0 / 6.0,
            CodingRate::Rate7_8 => 7.0 / 8.0,
        }
    }

    /// Approximate Eb/N0 coding gain (dB) relative to uncoded BPSK.
    pub fn coding_gain_db(self) -> f64 {
        match self {
            CodingRate::Rate1_2 => 3.5,
            CodingRate::Rate2_3 => 2.5,
            CodingRate::Rate3_4 => 2.0,
            CodingRate::Rate5_6 => 1.0,
            CodingRate::Rate7_8 => 0.5,
        }
    }
}

/// Input conditions for the AMC algorithm.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmcConditions {
    /// Measured link SNR (dB).
    pub link_snr_db: f64,
    /// Available channel bandwidth in MHz.
    pub bandwidth_mhz: f64,
    /// Whether the link operates under a latency constraint favouring robustness.
    pub latency_sensitive: bool,
}

/// AMC selection and achievable throughput result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmcResult {
    /// Selected modulation scheme.
    pub modulation: ModulationScheme,
    /// Selected FEC coding rate.
    pub coding_rate: CodingRate,
    /// Achievable information throughput in Mbps.
    pub throughput_mbps: f64,
    /// Spectral efficiency in bits/s/Hz.
    pub spectral_efficiency: f64,
    /// Estimated link margin above the required Eb/N0 (dB). Negative = outage.
    pub link_margin_db: f64,
}

/// Select the best (modulation, coding rate) pair for the current link SNR.
///
/// The algorithm walks from the highest-order MCS downward and picks the first
/// combination whose required Eb/N0 (after coding gain) is met by the link SNR
/// with at least 2 dB of margin.
pub fn select_amc(conditions: &AmcConditions) -> AmcResult {
    const MARGIN_DB: f64 = 2.0;

    let schemes = [
        ModulationScheme::Qam256,
        ModulationScheme::Qam64,
        ModulationScheme::Qam16,
        ModulationScheme::Qpsk,
        ModulationScheme::Bpsk,
    ];
    let rates = [
        CodingRate::Rate7_8,
        CodingRate::Rate5_6,
        CodingRate::Rate3_4,
        CodingRate::Rate2_3,
        CodingRate::Rate1_2,
    ];

    // Start with minimum if no better option fits.
    let mut best_mod = ModulationScheme::Bpsk;
    let mut best_rate = CodingRate::Rate1_2;

    'outer: for &mod_scheme in &schemes {
        for &code_rate in &rates {
            // Effective Eb/N0 requirement = modulation threshold − coding gain.
            let required_eb_n0 = mod_scheme.min_eb_n0_db() - code_rate.coding_gain_db();
            if conditions.latency_sensitive && mod_scheme > ModulationScheme::Qam16 {
                continue; // Safety: high-order QAM adds re-try latency
            }
            if conditions.link_snr_db >= required_eb_n0 + MARGIN_DB {
                best_mod = mod_scheme;
                best_rate = code_rate;
                break 'outer;
            }
        }
    }

    // Shannon-limited throughput after FEC overhead.
    let symbol_rate_msps = conditions.bandwidth_mhz; // Assume Nyquist: 1 sym/s per Hz
    let raw_rate_mbps = symbol_rate_msps * best_mod.bits_per_symbol();
    let throughput_mbps = raw_rate_mbps * best_rate.ratio();
    let spectral_efficiency = throughput_mbps / conditions.bandwidth_mhz;

    let required_eb_n0 = best_mod.min_eb_n0_db() - best_rate.coding_gain_db();
    let link_margin_db = conditions.link_snr_db - required_eb_n0;

    AmcResult {
        modulation: best_mod,
        coding_rate: best_rate,
        throughput_mbps,
        spectral_efficiency,
        link_margin_db,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 6. Polarization Diversity
// ─────────────────────────────────────────────────────────────────────────────

/// Antenna polarization types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Polarization {
    LinearHorizontal,
    LinearVertical,
    RightHandCircular,  // RHCP — standard for most spacecraft downlinks
    LeftHandCircular,   // LHCP
    DualLinear,         // Simultaneous H + V (polarization reuse)
    DualCircular,       // Simultaneous RHCP + LHCP (full polarization diversity)
}

/// Configuration for a polarization diversity link.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolarizationConfig {
    /// Transmit polarization.
    pub tx_polarization: Polarization,
    /// Receive polarization.
    pub rx_polarization: Polarization,
    /// Cross-polarization discrimination (XPD) of the antenna (dB).
    /// Typical values: 25–35 dB for high-quality feed systems.
    pub xpd_db: f64,
    /// Polarization rotation angle encountered along the path (degrees).
    /// Faraday rotation is frequency and ionosphere dependent.
    pub faraday_rotation_deg: f64,
}

/// Results of a polarization diversity analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolarizationResult {
    /// Polarization isolation achieved between two channels (dB).
    pub channel_isolation_db: f64,
    /// Signal loss due to polarization mismatch (dB, 0 = perfect alignment).
    pub polarization_loss_db: f64,
    /// Capacity multiplier: 2.0 for dual-polarization reuse, 1.0 otherwise.
    pub capacity_multiplier: f64,
    /// Whether the Faraday rotation warrants a polarization tracking correction.
    pub faraday_correction_needed: bool,
}

impl PolarizationConfig {
    /// Evaluate polarization isolation, mismatch loss, and capacity multiplier.
    ///
    /// Dual-polarization (DualLinear or DualCircular) doubles channel capacity
    /// by transmitting independent streams on orthogonal polarizations, provided
    /// the cross-polarization discrimination (XPD) is sufficient (≥ 20 dB).
    pub fn analyse(&self) -> PolarizationResult {
        // Polarization mismatch loss: co-pol is 0 dB; cross-pol = XPD attenuation.
        let polarization_loss_db = match (&self.tx_polarization, &self.rx_polarization) {
            (Polarization::LinearHorizontal, Polarization::LinearHorizontal)
            | (Polarization::LinearVertical, Polarization::LinearVertical)
            | (Polarization::RightHandCircular, Polarization::RightHandCircular)
            | (Polarization::LeftHandCircular, Polarization::LeftHandCircular)
            | (Polarization::DualLinear, Polarization::DualLinear)
            | (Polarization::DualCircular, Polarization::DualCircular) => {
                // Co-polarized: Add Faraday rotation loss (cos² of rotation angle).
                let rotation_rad = self.faraday_rotation_deg.to_radians();
                let mismatch_loss = -20.0 * rotation_rad.cos().abs().log10();
                mismatch_loss
            }
            (Polarization::LinearHorizontal, Polarization::LinearVertical)
            | (Polarization::LinearVertical, Polarization::LinearHorizontal)
            | (Polarization::RightHandCircular, Polarization::LeftHandCircular)
            | (Polarization::LeftHandCircular, Polarization::RightHandCircular) => {
                // Cross-polarized: 3 dB loss + XPD suppression insufficient.
                3.0 + (30.0 - self.xpd_db).max(0.0)
            }
            _ => {
                // Mixed: partial mismatch (~1–3 dB).
                1.5
            }
        };

        // Channel isolation equals the antenna XPD in the co-pol case.
        let channel_isolation_db = self.xpd_db;

        // Capacity multiplier: dual-pol only beneficial when XPD > 20 dB.
        let capacity_multiplier = match self.tx_polarization {
            Polarization::DualLinear | Polarization::DualCircular if self.xpd_db >= 20.0 => 2.0,
            _ => 1.0,
        };

        // Faraday rotation > 45° introduces measurable alignment loss.
        let faraday_correction_needed = self.faraday_rotation_deg.abs() > 45.0;

        PolarizationResult {
            channel_isolation_db,
            polarization_loss_db,
            capacity_multiplier,
            faraday_correction_needed,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Convenience: combined advanced-RF link summary
// ─────────────────────────────────────────────────────────────────────────────

/// Combined snapshot of all advanced RF techniques applied to a single link.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedRfLinkSummary {
    pub beamforming: BeamformingResult,
    pub mimo: MimoResult,
    pub dsss: DsssResult,
    pub fhss: FhssResult,
    pub amc: AmcResult,
    pub polarization: PolarizationResult,
}

/// Run all six advanced RF analyses with representative LEO science-downlink
/// parameters and return the combined summary.
///
/// # Example parameters used
/// - 500 km LEO link, 40 W TX, X-Band (10 GHz), 4×4 MIMO, 1023-chip DSSS,
///   128-channel FHSS, dual-circular polarization, 30 dB SNR.
pub fn run_advanced_rf_demo() -> AdvancedRfLinkSummary {
    // 1. Beamforming: 16-element ULA, adaptive mode, 30° target, 60° jammer.
    let phased_array = PhasedArrayConfig {
        num_elements: 16,
        element_spacing_lambda: 0.5,
        carrier_freq_ghz: 10.0, // X-Band
        mode: BeamformingMode::Adaptive,
        total_power_watts: 40.0,
    };
    let beamforming = phased_array.simulate_beamforming(30.0, Some(60.0));

    // 2. MIMO: 4×4 spatial multiplexing, 30 dB SNR, 500 MHz bandwidth.
    let mimo_cfg = MimoConfig {
        num_tx: 4,
        num_rx: 4,
        mode: MimoMode::SpatialMultiplexing,
        channel_snr_db: 30.0,
        bandwidth_mhz: 500.0,
    };
    let mimo = mimo_cfg.simulate();

    // 3. DSSS: 1023-chip Gold code, 10 kbps data, 10.23 Mcps chip rate.
    let dsss_cfg = DsssConfig {
        data_rate_kbps: 10.0,
        chip_rate_mcps: 10.23,
        input_snr_db: -10.0, // Operating below noise floor (spread spectrum advantage)
        jammer_to_signal_db: 20.0, // 20 dB jammer
    };
    let dsss = dsss_cfg.analyse();

    // 4. FHSS: 128-channel fast hopping, 5% channels jammed.
    let fhss_cfg = FhssConfig {
        num_channels: 128,
        channel_bandwidth_mhz: 1.0,
        hop_rate_hz: 1000.0,
        hopping_rate: HoppingRate::Fast,
        jammed_channel_fraction: 0.05,
    };
    let fhss = fhss_cfg.analyse();

    // 5. AMC: 30 dB link SNR, 500 MHz bandwidth, not latency-sensitive.
    let amc_conditions = AmcConditions {
        link_snr_db: 30.0,
        bandwidth_mhz: 500.0,
        latency_sensitive: false,
    };
    let amc = select_amc(&amc_conditions);

    // 6. Polarization: RHCP TX/RX, 30 dB XPD, 20° Faraday rotation.
    let pol_cfg = PolarizationConfig {
        tx_polarization: Polarization::DualCircular,
        rx_polarization: Polarization::DualCircular,
        xpd_db: 30.0,
        faraday_rotation_deg: 20.0,
    };
    let polarization = pol_cfg.analyse();

    AdvancedRfLinkSummary {
        beamforming,
        mimo,
        dsss,
        fhss,
        amc,
        polarization,
    }
}
