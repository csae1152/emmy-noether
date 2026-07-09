//! emmy-noether-core
//! =====================
//! Proof-of-Concept-Bibliothek: fraktale/langzeit-korrelierte Rauschmodellierung
//! (inspiriert von "memory multi-fractional Brownian motion", mmfBm) fuer
//! noise-aware Passes in Quantencompilern (Routing/Scheduling, Dynamical
//! Decoupling).
//!
//! Ablauf, wie er in einer echten Transpiler-Pipeline genutzt wuerde:
//!
//! 1. Kalibrierungsdaten (z.B. Ramsey-/Echo-Zerfallskurven pro Qubit ueber die
//!    Zeit) werden erfasst -> Vec<f64>.
//! 2. `hurst::estimate_local_hurst(...)` schaetzt daraus einen zeitabhaengigen
//!    (lokalen) Hurst-Exponenten H(t) je Qubit.
//! 3. `dd_synth::synthesize_dd_sequence(...)` erzeugt daraus eine an das
//!    aktuelle Rauschregime angepasste Dynamical-Decoupling-Sequenz.
//! 4. `cost::schedule_cost(...)` liefert eine rauschbewusste Kostenfunktion,
//!    die ein Router/Scheduler (z.B. als zusaetzliches Kriterium neben
//!    Gate-Anzahl) beim Vergleich von Schedule-Kandidaten heranziehen kann.
//!
//! WICHTIG: Dies ist explizit ein Forschungs-/Proof-of-Concept-Stand, kein
//! validiertes Produkt. Siehe README.md fuer Einordnung, Grenzen und
//! naechste Schritte (Validierung mit echten Kalibrierungsdaten / Kooperation
//! mit Rauschcharakterisierungs-Experten).

pub mod cost;
pub mod dd_synth;
pub mod hurst;
pub mod noise_gen;

pub use cost::{predicted_fidelity, schedule_cost, IdleSlot};
pub use dd_synth::synthesize_dd_sequence;
pub use hurst::{estimate_hurst_dfa, estimate_local_hurst};
pub use noise_gen::generate_colored_noise;

/// High-Level-Convenience-Funktion: aus einer rohen Kalibrierungszeitreihe direkt
/// eine DD-Sequenz fuer ein gegebenes Zeitfenster ableiten.
///
/// `calibration_series`: z.B. eine Ramsey-Zerfallsmessung ueber die Zeit.
/// `total_time`, `min_pulse_gap`, `base_pulse_count`: siehe `dd_synth::synthesize_dd_sequence`.
pub fn dd_sequence_from_calibration(
    calibration_series: &[f64],
    dfa_min: usize,
    dfa_max: usize,
    total_time: f64,
    min_pulse_gap: f64,
    base_pulse_count: usize,
) -> Option<Vec<f64>> {
    let h = hurst::estimate_hurst_dfa(calibration_series, dfa_min, dfa_max)?;
    Some(dd_synth::synthesize_dd_sequence(
        h,
        total_time,
        min_pulse_gap,
        base_pulse_count,
    ))
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn end_to_end_pipeline_runs() {
        let series = generate_colored_noise(4096, 0.85, 123);
        let seq = dd_sequence_from_calibration(&series, 8, 512, 100.0, 0.5, 8);
        assert!(seq.is_some());
        let seq = seq.unwrap();
        assert!(!seq.is_empty());
    }
}
