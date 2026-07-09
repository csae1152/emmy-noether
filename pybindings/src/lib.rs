//! PyO3-Bindings fuer emmy-noether-core.
//!
//! Baubar mit `maturin build --release` (im pybindings/-Verzeichnis), erzeugt
//! ein Python-Modul `emmy_noether`, das z.B. in einem custom Qiskit
//! TransformationPass importiert werden kann (siehe examples/qiskit_pass.py).

use pyo3::prelude::*;

/// Schaetzt einen (zeitvariablen) lokalen Hurst-Exponenten aus einer
/// Kalibrierungszeitreihe (z.B. Ramsey-/Echo-Zerfallsmessung).
///
/// Gibt eine Liste von (Fenster-Mittelpunkt-Index, geschaetztes H) zurueck.
#[pyfunction]
fn estimate_local_hurst(
    series: Vec<f64>,
    window_len: usize,
    step: usize,
    dfa_min: usize,
    dfa_max: usize,
) -> PyResult<Vec<(usize, f64)>> {
    Ok(emmy_noether_core::estimate_local_hurst(
        &series, window_len, step, dfa_min, dfa_max,
    ))
}

/// Schaetzt einen einzelnen globalen Hurst-Exponenten via DFA.
#[pyfunction]
fn estimate_hurst(series: Vec<f64>, dfa_min: usize, dfa_max: usize) -> PyResult<Option<f64>> {
    Ok(emmy_noether_core::estimate_hurst_dfa(&series, dfa_min, dfa_max))
}

/// Erzeugt eine an den geschaetzten Hurst-Exponenten angepasste
/// Dynamical-Decoupling-Pulssequenz (Zeitstempel innerhalb (0, total_time)).
#[pyfunction]
fn synthesize_dd_sequence(
    h: f64,
    total_time: f64,
    min_pulse_gap: f64,
    base_pulse_count: usize,
) -> PyResult<Vec<f64>> {
    Ok(emmy_noether_core::synthesize_dd_sequence(
        h,
        total_time,
        min_pulse_gap,
        base_pulse_count,
    ))
}

/// Direkter Weg von rohen Kalibrierungsdaten zu einer DD-Sequenz.
#[pyfunction]
fn dd_sequence_from_calibration(
    calibration_series: Vec<f64>,
    dfa_min: usize,
    dfa_max: usize,
    total_time: f64,
    min_pulse_gap: f64,
    base_pulse_count: usize,
) -> PyResult<Option<Vec<f64>>> {
    Ok(emmy_noether_core::dd_sequence_from_calibration(
        &calibration_series,
        dfa_min,
        dfa_max,
        total_time,
        min_pulse_gap,
        base_pulse_count,
    ))
}

/// Rauschbewusste Fidelity-Schaetzung fuer eine gegebene Idle-/Wartezeit --
/// nutzbar als zusaetzliches Kriterium in einer Routing-/Scheduling-Kostenfunktion.
#[pyfunction]
fn predicted_fidelity(idle_time: f64, tau_phi: f64, h: f64) -> PyResult<f64> {
    Ok(emmy_noether_core::predicted_fidelity(idle_time, tau_phi, h))
}

/// Python-Modul-Definition.
#[pymodule]
fn emmy_noether(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(estimate_local_hurst, m)?)?;
    m.add_function(wrap_pyfunction!(estimate_hurst, m)?)?;
    m.add_function(wrap_pyfunction!(synthesize_dd_sequence, m)?)?;
    m.add_function(wrap_pyfunction!(dd_sequence_from_calibration, m)?)?;
    m.add_function(wrap_pyfunction!(predicted_fidelity, m)?)?;
    Ok(())
}
