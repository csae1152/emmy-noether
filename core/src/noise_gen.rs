//! Erzeugt synthetisches, spektral geformtes ("farbiges") Gauss-Rauschen mit
//! ungefaehr vorgegebenem Hurst-Exponenten H, um die DFA-Schaetzung (hurst.rs)
//! zu testen und zu validieren.
//!
//! Methode: spektrale Synthese ("Voss/Mandelbrot-Filterung"). Wir erzeugen
//! weisses Gauss-Rauschen, transformieren es in den Frequenzraum (FFT), formen
//! das Amplitudenspektrum gemaess |X(f)| ~ 1/f^((H-0.5)) (der theoretische
//! Zusammenhang zwischen dem PSD-Exponenten fraktionalen Gauss-Rauschens und
//! dessen Hurst-Exponent H: PSD ~ 1/f^(2H-1)) und transformieren zurueck.
//!
//! WICHTIG: Das ist eine praktische Approximation, kein exakter fGn-Generator
//! (z.B. Davies-Harte). Fuer ein Proof-of-Concept zur Validierung der
//! Hurst-Schaetzung in hurst.rs ist das aber ausreichend und ueblich.

use rustfft::{num_complex::Complex, FftPlanner};

pub fn generate_colored_noise(n: usize, target_hurst: f64, seed: u64) -> Vec<f64> {
    use rand::SeedableRng;
    use rand_distr::{Distribution, Normal};

    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    let normal = Normal::new(0.0, 1.0).unwrap();

    // 1. weisses Gauss-Rauschen im Zeitbereich
    let mut buffer: Vec<Complex<f64>> = (0..n)
        .map(|_| Complex::new(normal.sample(&mut rng), 0.0))
        .collect();

    // 2. FFT in den Frequenzraum
    let mut planner = FftPlanner::<f64>::new();
    let fft = planner.plan_fft_forward(n);
    fft.process(&mut buffer);

    // 3. Amplitudenspektrum gemaess Ziel-H formen: Filter ~ 1/f^(H - 0.5)
    let filter_exponent = target_hurst - 0.5;
    for (k, bin) in buffer.iter_mut().enumerate() {
        let freq_index = if k <= n / 2 { k } else { n - k };
        let f = (freq_index.max(1)) as f64; // f=0 (DC) wird unten separat behandelt
        let scale = f.powf(-filter_exponent);
        *bin *= scale;
    }
    buffer[0] = Complex::new(0.0, 0.0); // DC-Komponente entfernen (kein Drift)

    // 4. inverse FFT zurueck in den Zeitbereich
    let ifft = planner.plan_fft_inverse(n);
    ifft.process(&mut buffer);

    let norm = 1.0 / (n as f64).sqrt();
    let series: Vec<f64> = buffer.iter().map(|c| c.re * norm).collect();

    // Normalisierung auf Einheitsvarianz, damit target_hurst unabhaengig von n vergleichbar bleibt
    let mean = series.iter().sum::<f64>() / series.len() as f64;
    let var = series.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / series.len() as f64;
    let std = var.sqrt().max(1e-12);
    series.iter().map(|x| (x - mean) / std).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_correct_length() {
        let s = generate_colored_noise(1024, 0.7, 1);
        assert_eq!(s.len(), 1024);
    }

    #[test]
    fn has_zero_mean_and_unit_variance() {
        let s = generate_colored_noise(2048, 0.6, 2);
        let mean = s.iter().sum::<f64>() / s.len() as f64;
        let var = s.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / s.len() as f64;
        assert!(mean.abs() < 1e-6);
        assert!((var - 1.0).abs() < 1e-6);
    }
}
