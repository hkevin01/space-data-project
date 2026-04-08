//! Integration tests for the frequency band simulation crate.
//!
//! Tests cover the core public API of `frequency_band_simulation`:
//! - `FrequencyBand::get_standard_bands()` coverage and field correctness
//! - `FrequencyBand::simulate_transmission()` — physics properties
//! - `score_bands_for_conditions()` — ranking logic in clear sky vs storm
//! - `TransmissionResult` — invariants on computed output fields
//! - Determinism — identical inputs yield identical outputs

use frequency_band_simulation::{
    score_bands_for_conditions, BandScore, BandType, EnvironmentalConditions, FrequencyBand,
    TransmissionParameters,
};

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn clear_sky() -> EnvironmentalConditions {
    EnvironmentalConditions {
        rain_rate_mm_hour: 0.0,
        cloud_cover_percent: 5.0,
        atmospheric_pressure_mb: 1013.0,
        temperature_celsius: 20.0,
        humidity_percent: 40.0,
        ionospheric_activity: 0.1,
        solar_activity: 0.1,
    }
}

fn tropical_storm() -> EnvironmentalConditions {
    EnvironmentalConditions {
        rain_rate_mm_hour: 100.0,
        cloud_cover_percent: 100.0,
        atmospheric_pressure_mb: 990.0,
        temperature_celsius: 28.0,
        humidity_percent: 95.0,
        ionospheric_activity: 0.3,
        solar_activity: 0.3,
    }
}

fn leo_params() -> TransmissionParameters {
    TransmissionParameters {
        distance_km: 550.0,
        data_size_mb: 100.0,
        required_data_rate_mbps: 10.0,
        elevation_angle_degrees: 45.0,
        transmit_power_watts: 50.0,
        antenna_diameter_meters: 2.0,
    }
}

// ─── Standard Band Catalogue Tests ────────────────────────────────────────────

/// `get_standard_bands()` must return exactly 5 standard bands.
#[test]
fn test_get_standard_bands_returns_five() {
    assert_eq!(FrequencyBand::get_standard_bands().len(), 5);
}

/// Every band must have a bandwidth > 0 GHz.
#[test]
fn test_all_bands_have_positive_bandwidth() {
    for band in FrequencyBand::get_standard_bands() {
        let bw = band.frequency_range.max_ghz - band.frequency_range.min_ghz;
        assert!(
            bw > 0.0,
            "{:?}: bandwidth must be > 0, got {:.2} GHz",
            band.name, bw
        );
    }
}

/// Centre frequency must be within [min, max] range for every band.
#[test]
fn test_band_centre_within_range() {
    for band in FrequencyBand::get_standard_bands() {
        let lo = band.frequency_range.min_ghz;
        let hi = band.frequency_range.max_ghz;
        let center = (lo + hi) / 2.0;
        assert!(center >= lo && center <= hi,
            "{:?}: centre {:.2} not in [{:.2}, {:.2}]", band.name, center, lo, hi);
    }
}

/// Ka-Band must support the highest max data rate of all standard bands.
#[test]
fn test_ka_band_has_highest_max_data_rate() {
    let bands = FrequencyBand::get_standard_bands();
    let ka = bands
        .iter()
        .find(|b| b.name == BandType::KaBand)
        .expect("Ka-Band must be in standard catalogue");
    for other in &bands {
        if other.name != BandType::KaBand {
            assert!(
                ka.characteristics.max_data_rate_mbps >= other.characteristics.max_data_rate_mbps,
                "Ka-Band ({:.0} Mbps) must have >= rate of {:?} ({:.0} Mbps)",
                ka.characteristics.max_data_rate_mbps,
                other.name,
                other.characteristics.max_data_rate_mbps
            );
        }
    }
}

/// UHF-Band must have the lowest max data rate.
#[test]
fn test_uhf_band_has_lowest_max_data_rate() {
    let bands = FrequencyBand::get_standard_bands();
    let uhf = bands
        .iter()
        .find(|b| b.name == BandType::UHFBand)
        .expect("UHF-Band must be in standard catalogue");
    for other in &bands {
        if other.name != BandType::UHFBand {
            assert!(
                other.characteristics.max_data_rate_mbps >= uhf.characteristics.max_data_rate_mbps,
                "UHF ({:.0} Mbps) must be <= {:?} ({:.0} Mbps)",
                uhf.characteristics.max_data_rate_mbps,
                other.name,
                other.characteristics.max_data_rate_mbps
            );
        }
    }
}

/// UHF-Band frequency range must be the lowest numerically (< 3 GHz).
#[test]
fn test_uhf_band_occupies_lowest_frequency_range() {
    let bands = FrequencyBand::get_standard_bands();
    let uhf = bands.iter().find(|b| b.name == BandType::UHFBand).unwrap();
    assert!(uhf.frequency_range.min_ghz < 3.0,
        "UHF min must be below 3 GHz, got {:.2}", uhf.frequency_range.min_ghz);
}

// ─── simulate_transmission() Physics Tests ────────────────────────────────────

/// Actual data rate in clear sky must never exceed the band's maximum.
#[test]
fn test_actual_data_rate_does_not_exceed_max() {
    let params = leo_params();
    let env = clear_sky();
    for band in FrequencyBand::get_standard_bands() {
        let result = band.simulate_transmission(&params, &env);
        assert!(
            result.actual_data_rate_mbps <= band.characteristics.max_data_rate_mbps + 1e-9,
            "{:?}: actual {:.2} Mbps > max {:.2} Mbps",
            band.name,
            result.actual_data_rate_mbps,
            band.characteristics.max_data_rate_mbps
        );
    }
}

/// Actual data rate must be non-negative.
#[test]
fn test_actual_data_rate_non_negative() {
    for (env_name, env) in [("clear", clear_sky()), ("storm", tropical_storm())] {
        for band in FrequencyBand::get_standard_bands() {
            let result = band.simulate_transmission(&leo_params(), &env);
            assert!(
                result.actual_data_rate_mbps >= 0.0,
                "{:?} in {}: negative data rate {:.2}",
                band.name, env_name, result.actual_data_rate_mbps
            );
        }
    }
}

/// Increasing distance must increase total latency.
#[test]
fn test_higher_distance_increases_latency() {
    let env = clear_sky();
    for band in FrequencyBand::get_standard_bands() {
        let res_leo = band.simulate_transmission(&leo_params(), &env);
        let geo_params = TransmissionParameters {
            distance_km: 36_000.0,
            ..leo_params()
        };
        let res_geo = band.simulate_transmission(&geo_params, &env);
        assert!(
            res_geo.total_latency_ms > res_leo.total_latency_ms,
            "{:?}: GEO latency {:.2} must exceed LEO latency {:.2}",
            band.name, res_geo.total_latency_ms, res_leo.total_latency_ms
        );
    }
}

/// More transmit power should keep success rate equal or better (monotonic).
#[test]
fn test_higher_tx_power_improves_or_maintains_snr() {
    let env = clear_sky();
    for band in FrequencyBand::get_standard_bands() {
        let params_lo = TransmissionParameters { transmit_power_watts: 1.0, ..leo_params() };
        let params_hi = TransmissionParameters { transmit_power_watts: 10_000.0, ..leo_params() };
        let res_lo = band.simulate_transmission(&params_lo, &env);
        let res_hi = band.simulate_transmission(&params_hi, &env);
        assert!(
            res_hi.signal_to_noise_ratio_db >= res_lo.signal_to_noise_ratio_db,
            "{:?}: higher power must improve SNR: {:.2} >= {:.2}",
            band.name, res_hi.signal_to_noise_ratio_db, res_lo.signal_to_noise_ratio_db
        );
    }
}

/// Path loss must be positive for any non-zero distance.
#[test]
fn test_path_loss_is_positive() {
    let env = clear_sky();
    for band in FrequencyBand::get_standard_bands() {
        let result = band.simulate_transmission(&leo_params(), &env);
        assert!(result.path_loss_db > 0.0, "{:?}: path loss must be > 0", band.name);
    }
}

// ─── score_bands_for_conditions() Tests ───────────────────────────────────────

/// In clear sky, Ka-Band must rank highest (maximum bandwidth).
///
/// - **Requirement**: `score_bands_for_conditions` postcondition (from FN-SIM-001 docstring):
///   "Under clear-sky conditions, Ka-Band should score highest."
#[test]
fn test_clear_sky_ka_band_scores_highest() {
    let bands = FrequencyBand::get_standard_bands();
    let scores = score_bands_for_conditions(&bands, &leo_params(), &clear_sky());
    assert!(!scores.is_empty());
    assert_eq!(
        scores[0].band, BandType::KaBand,
        "Ka-Band must win in clear sky; best was {:?} (score {:.4})",
        scores[0].band, scores[0].composite_score
    );
}

/// In tropical storm (100 mm/hr rain), UHF must rank highest (rain immunity).
///
/// - **Requirement**: `score_bands_for_conditions` postcondition (from FN-SIM-001 docstring):
///   "Under tropical-storm conditions (rain ≥ 100 mm/h), UHF should score highest."
#[test]
fn test_storm_uhf_band_scores_highest() {
    let bands = FrequencyBand::get_standard_bands();
    let scores = score_bands_for_conditions(&bands, &leo_params(), &tropical_storm());
    assert!(!scores.is_empty());
    assert_eq!(
        scores[0].band, BandType::UHFBand,
        "UHF must win in 100 mm/hr storm; best was {:?} (score {:.4})",
        scores[0].band, scores[0].composite_score
    );
}

/// Score count must equal the number of input bands.
///
/// - **Requirement**: FN-SIM-001 postcondition: `result.len() == bands.len()`.
#[test]
fn test_score_count_equals_input_count() {
    let bands = FrequencyBand::get_standard_bands();
    let scores = score_bands_for_conditions(&bands, &leo_params(), &clear_sky());
    assert_eq!(scores.len(), bands.len(),
        "score_bands_for_conditions must return one score per input band");
}

/// Scores must be sorted strictly descending by composite_score.
#[test]
fn test_scores_are_sorted_descending() {
    let bands = FrequencyBand::get_standard_bands();
    let scores = score_bands_for_conditions(&bands, &leo_params(), &clear_sky());
    for win in scores.windows(2) {
        assert!(win[0].composite_score >= win[1].composite_score,
            "Scores not sorted descending: {:.4} < {:.4}", win[0].composite_score, win[1].composite_score);
    }
}

/// Composite scores must all be in [0.0, 1.0].
#[test]
fn test_composite_scores_in_unit_range() {
    let bands = FrequencyBand::get_standard_bands();
    for env in [clear_sky(), tropical_storm()] {
        let scores = score_bands_for_conditions(&bands, &leo_params(), &env);
        for s in &scores {
            assert!(s.composite_score >= 0.0 && s.composite_score <= 1.0,
                "{:?}: composite score {:.4} not in [0, 1]", s.band, s.composite_score);
        }
    }
}

/// Results must be deterministic — same inputs always produce the same output.
#[test]
fn test_scoring_is_deterministic() {
    let bands = FrequencyBand::get_standard_bands();
    let first = score_bands_for_conditions(&bands, &leo_params(), &clear_sky());
    let second = score_bands_for_conditions(&bands, &leo_params(), &clear_sky());
    for (a, b) in first.iter().zip(second.iter()) {
        assert_eq!(a.band, b.band);
        assert_eq!(a.composite_score, b.composite_score,
            "Results must be identical across repeated calls (deterministic)");
    }
}

/// Achievable data rates must be non-negative in all environments.
#[test]
fn test_achievable_rates_non_negative_all_envs() {
    let bands = FrequencyBand::get_standard_bands();
    for env in [clear_sky(), tropical_storm()] {
        for score in score_bands_for_conditions(&bands, &leo_params(), &env) {
            assert!(score.achievable_rate_mbps >= 0.0,
                "{:?}: achievable rate must be >= 0", score.band);
        }
    }
}

/// In clear sky with very lenient requirement, all bands must meet the requirement.
#[test]
fn test_all_bands_meet_lenient_requirement_in_clear_sky() {
    let bands = FrequencyBand::get_standard_bands();
    let lenient = TransmissionParameters {
        required_data_rate_mbps: 0.001, // near-zero requirement
        ..leo_params()
    };
    let scores = score_bands_for_conditions(&bands, &lenient, &clear_sky());
    for s in &scores {
        assert!(s.meets_requirement,
            "{:?} must meet a 0.001 Mbps requirement in clear sky", s.band);
    }
}

/// GEO-orbit must yield lower composite scores than LEO for the same best band.
#[test]
fn test_geo_scores_lower_than_leo() {
    let bands = FrequencyBand::get_standard_bands();
    let geo_params = TransmissionParameters { distance_km: 36_000.0, ..leo_params() };
    let leo_scores = score_bands_for_conditions(&bands, &leo_params(), &clear_sky());
    let geo_scores = score_bands_for_conditions(&bands, &geo_params, &clear_sky());
    assert!(
        leo_scores[0].composite_score > geo_scores[0].composite_score,
        "LEO best score ({:.4}) must exceed GEO best ({:.4})",
        leo_scores[0].composite_score, geo_scores[0].composite_score
    );
}
