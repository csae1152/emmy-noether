//! Detrended Fluctuation Analysis (DFA) zur Schaetzung des (lokalen) Hurst-Exponenten
//! einer Zeitreihe (z.B. Ramsey-/Echo-Zerfallssignal aus Kalibrierungsmessungen).
//!
//! DFA ist eine etablierte, robuste Methode zur Charakterisierung von
//! Langzeitkorrelationen in nicht-stationaren Zeitreihen (Peng et al. 1994) und
//! ist die Standardmethode, um den Hurst-Exponenten H von fraktionaler
//! Brownscher Bewegung / 1/f^beta-artigem Rauschen aus Messdaten zu schaetzen,
//! ohne Stationaritaet vorauszusetzen -- genau die Eigenschaft, die man fuer ein
//! mmfBm-Modell (memory multi-fractional Brownian motion) mit zeitabhaengigem
//! H(t) braucht.
//!
//! WICHTIG (ehrlicher Hinweis): Dies ist eine vereinfachte Referenzimplementierung
//! fuer ein Proof-of-Concept. Fuer produktive Nutzung braucht es Validierung
//! gegen echte Kalibrierungsdaten und ggf. robustere Schaetzer (z.B. Wavelet-basiert,
//! Whittle-Schaetzer) sowie eine Fehlerabschaetzung des geschaetzten H.

/// Schaetzt einen einzelnen (globalen) Hurst-Exponenten fuer eine Zeitreihe mittels DFA.
///
/// `series`: die (bereits vom Mittelwert befreite, aber nicht notwendig stationaere) Zeitreihe.
/// `min_window`, `max_window`: Fenstergroessen (in Samples) fuer die Fluktuationsanalyse.
pub fn estimate_hurst_dfa(series: &[f64], min_window: usize, max_window: usize) -> Option<f64> {
    if series.len() < max_window * 4 || min_window < 4 || max_window <= min_window {
        return None;
    }

    // Schritt 1: integrierte (kumulierte) Reihe der Abweichung vom Mittelwert
    let mean = series.iter().sum::<f64>() / series.len() as f64;
    let mut profile = Vec::with_capacity(series.len());
    let mut acc = 0.0;
    for &x in series {
        acc += x - mean;
        profile.push(acc);
    }

    // Schritt 2: fuer verschiedene Fenstergroessen n die mittlere quadratische
    // Fluktuation nach lokaler linearer Entrendung berechnen -> F(n)
    let mut log_n = Vec::new();
    let mut log_f = Vec::new();

    let mut n = min_window;
    while n <= max_window {
        let num_windows = profile.len() / n;
        if num_windows < 2 {
            n = next_window(n);
            continue;
        }
        let mut fluct_sq_sum = 0.0;
        for w in 0..num_windows {
            let seg = &profile[w * n..(w + 1) * n];
            let (slope, intercept) = linear_fit(seg);
            let mut ss = 0.0;
            for (i, &y) in seg.iter().enumerate() {
                let trend = intercept + slope * i as f64;
                let d = y - trend;
                ss += d * d;
            }
            fluct_sq_sum += ss / n as f64;
        }
        let f_n = (fluct_sq_sum / num_windows as f64).sqrt();
        if f_n > 0.0 {
            log_n.push((n as f64).ln());
            log_f.push(f_n.ln());
        }
        n = next_window(n);
    }

    if log_n.len() < 2 {
        return None;
    }

    // Schritt 3: H = Steigung von log F(n) gegen log n (lineare Regression)
    let (slope, _intercept) = linear_fit_xy(&log_n, &log_f);
    Some(slope)
}

/// Schaetzt einen zeitabhaengigen (lokalen) Hurst-Exponenten H(t) via gleitendem Fenster.
/// Gibt Paare (Fenster-Mittelpunkt-Index, geschaetztes H) zurueck.
///
/// Dies ist die praktische Naeherung an das in der Literatur beschriebene
/// "memory multi-fractional Brownian motion" (mmfBm) Modell mit zeitvariablem H(t):
/// ein sich langsam veraenderndes H wird durch DFA auf ueberlappenden Fenstern
/// approximiert, statt ein einzelnes globales H fuer die ganze Messreihe anzunehmen.
pub fn estimate_local_hurst(
    series: &[f64],
    window_len: usize,
    step: usize,
    dfa_min: usize,
    dfa_max: usize,
) -> Vec<(usize, f64)> {
    let mut out = Vec::new();
    if series.len() < window_len {
        return out;
    }
    let mut start = 0;
    while start + window_len <= series.len() {
        let seg = &series[start..start + window_len];
        if let Some(h) = estimate_hurst_dfa(seg, dfa_min, dfa_max) {
            let center = start + window_len / 2;
            out.push((center, h.clamp(0.05, 1.2)));
        }
        start += step;
    }
    out
}

fn next_window(n: usize) -> usize {
    // moderates geometrisches Wachstum der Fenstergroessen (log-aequidistant)
    ((n as f64) * 1.3).ceil() as usize + 1
}

fn linear_fit(seg: &[f64]) -> (f64, f64) {
    let xs: Vec<f64> = (0..seg.len()).map(|i| i as f64).collect();
    linear_fit_xy(&xs, seg)
}

fn linear_fit_xy(xs: &[f64], ys: &[f64]) -> (f64, f64) {
    let n = xs.len() as f64;
    let mean_x = xs.iter().sum::<f64>() / n;
    let mean_y = ys.iter().sum::<f64>() / n;
    let mut num = 0.0;
    let mut den = 0.0;
    for (x, y) in xs.iter().zip(ys.iter()) {
        num += (x - mean_x) * (y - mean_y);
        den += (x - mean_x) * (x - mean_x);
    }
    let slope = if den.abs() > 1e-12 { num / den } else { 0.0 };
    let intercept = mean_y - slope * mean_x;
    (slope, intercept)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::noise_gen::generate_colored_noise;

    #[test]
    fn white_noise_has_hurst_near_half() {
        let series = generate_colored_noise(4096, 0.5, 42);
        let h = estimate_hurst_dfa(&series, 8, 512).expect("estimation failed");
        // weisses Rauschen -> H ~ 0.5; DFA-Schaetzer hat Varianz, daher grosszuegige Toleranz
        assert!((h - 0.5).abs() < 0.25, "H={h} zu weit von 0.5 entfernt");
    }

    #[test]
    fn persistent_noise_has_higher_hurst_than_white() {
        let white = generate_colored_noise(4096, 0.5, 7);
        let persistent = generate_colored_noise(4096, 0.9, 7);
        let h_white = estimate_hurst_dfa(&white, 8, 512).unwrap();
        let h_pers = estimate_hurst_dfa(&persistent, 8, 512).unwrap();
        assert!(
            h_pers > h_white,
            "erwartet: persistentes Rauschen (H_ziel=0.9) hat groesseres geschaetztes H als weisses (H_ziel=0.5); h_white={h_white} h_pers={h_pers}"
        );
    }

    #[test]
    fn local_hurst_returns_expected_number_of_windows() {
        let series = generate_colored_noise(2048, 0.7, 1);
        let local = estimate_local_hurst(&series, 512, 256, 8, 128);
        assert!(!local.is_empty());
        for (_, h) in &local {
            assert!(*h > 0.0 && *h < 1.2);
        }
    }
}
