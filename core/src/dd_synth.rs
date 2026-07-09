//! Synthese einer Dynamical-Decoupling (DD) Pulssequenz, deren Timing sich am
//! geschaetzten (lokalen) Hurst-Exponenten H orientiert.
//!
//! Physikalische Motivation (siehe README/Quellen): Fuer 1/f-artiges Rauschen
//! koennen Entkopplungspulse, die LANGSAMER als die schnellste Bad-Zeitskala
//! sind, die Dekohaerenzrate drastisch reduzieren -- Entkopplung muss bei
//! long-memory-Rauschen also nicht zwingend so schnell wie moeglich sein
//! (Shiokawa & Lidar, quant-ph/0211081). Ein groesseres H (staerkeres
//! Langzeitgedaechtnis / persistenteres Rauschen) spricht daher heuristisch
//! fuer WEITER auseinanderliegende, aber praeziser platzierte Pulse statt
//! eines dichten, starren CPMG-Rasters.
//!
//! WICHTIG (ehrlicher Hinweis): Die konkrete Abbildung H -> Pulsabstand ist
//! hier eine EINFACHE, PLAUSIBLE HEURISTIK fuer das Proof-of-Concept -- keine
//! aus einer vollstaendigen Filterfunktions-/Chi-Funktion-Rechnung (wie in der
//! DD-Literatur ueblich) hergeleitete optimale Sequenz. Fuer produktiven
//! Einsatz muesste die Sequenz gegen eine echte Filterfunktionsanalyse und
//! Kalibrierungsdaten der Zielhardware optimiert/validiert werden.

/// Erzeugt Zeitstempel (in denselben Einheiten wie `total_time`, z.B. Mikrosekunden)
/// fuer eine DD-Pulssequenz im Intervall (0, total_time), basierend auf dem
/// geschaetzten Hurst-Exponenten `h` und einer hardwarebedingten Mindestpulsluecke.
///
/// - `h <= 0.6`: naeherungsweise weisses/kurzreichweitiges Rauschen -> dichteres,
///   nahezu gleichverteiltes CPMG-artiges Raster (mehr, schnellere Pulse).
/// - `h > 0.6`: persistentes/langreichweitiges (1/f-artiges) Rauschen -> weniger,
///   aber weiter auseinanderliegende Pulse, gemaess dem Slow-Pulse-Befund fuer 1/f-Baeder.
pub fn synthesize_dd_sequence(
    h: f64,
    total_time: f64,
    min_pulse_gap: f64,
    base_pulse_count: usize,
) -> Vec<f64> {
    assert!(total_time > 0.0);
    assert!(min_pulse_gap > 0.0);
    assert!(base_pulse_count >= 1);

    // Spacing-Multiplikator waechst mit H oberhalb von 0.5 (persistenteres Rauschen
    // -> "langsamere" Entkopplung gemaess Shiokawa/Lidar), begrenzt auf ein
    // plausibles Intervall, um entartete Sequenzen zu vermeiden.
    let persistence = (h - 0.5).max(0.0);
    let spacing_multiplier = 1.0 + 2.5 * persistence; // H=0.5 -> 1.0x, H=0.9 -> ~2.0x

    // Zielanzahl Pulse: bei starkem Gedaechtnis (hohes H) tendenziell weniger,
    // dafuer praeziser platzierte Pulse als bei weissem Rauschen.
    let reduction = (1.0 - 0.5 * persistence).clamp(0.4, 1.0);
    let target_pulses = ((base_pulse_count as f64) * reduction).round().max(1.0) as usize;

    // gleichmaessiger Grundabstand, mit Multiplikator skaliert, aber durch
    // Hardware-Mindestabstand und Gesamtzeit begrenzt
    let naive_gap = total_time / (target_pulses as f64 + 1.0);
    let gap = (naive_gap * spacing_multiplier).max(min_pulse_gap);

    let mut timestamps = Vec::new();
    let mut t = gap;
    while t < total_time && timestamps.len() < target_pulses.max(1) * 4 {
        timestamps.push(t);
        t += gap;
    }
    timestamps
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn white_noise_case_yields_denser_sequence_than_persistent_case() {
        let white = synthesize_dd_sequence(0.5, 100.0, 0.5, 8);
        let persistent = synthesize_dd_sequence(0.9, 100.0, 0.5, 8);
        assert!(
            persistent.len() <= white.len(),
            "erwartet: persistentes Rauschen (H=0.9) fuehrt zu <= Pulsen als weisses (H=0.5); white={} persistent={}",
            white.len(),
            persistent.len()
        );
        if persistent.len() >= 2 {
            let gap_p = persistent[1] - persistent[0];
            let gap_w = white[1] - white[0];
            assert!(gap_p > gap_w, "erwartet groesseren Pulsabstand bei H=0.9");
        }
    }

    #[test]
    fn respects_minimum_pulse_gap() {
        let seq = synthesize_dd_sequence(0.95, 10.0, 2.0, 20);
        for w in seq.windows(2) {
            assert!(w[1] - w[0] >= 2.0 - 1e-9);
        }
    }

    #[test]
    fn all_timestamps_within_total_time() {
        let seq = synthesize_dd_sequence(0.7, 50.0, 0.3, 10);
        for &t in &seq {
            assert!(t > 0.0 && t < 50.0);
        }
    }
}
