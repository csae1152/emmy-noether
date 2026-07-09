//! Rauschbewusste Kostenfunktion, die anstelle einer reinen Gate-Count-Heuristik
//! (wie sie z.B. SABRE/LightSABRE fuer Routing-Entscheidungen nutzen) eine
//! geschaetzte Fidelity basierend auf dem lokalen Rauschmodell liefert.
//!
//! Physikalischer Hintergrund: Mehrere aktuelle Arbeiten zu langreichweitig
//! korreliertem (mmfBm-artigem) Rauschen beobachten gestreckt-exponentiellen
//! Kohaerenzzerfall mit dynamisch veraenderlichen Exponenten, statt des
//! einfachen exponentiellen Zerfalls, den man bei Markov'schem Rauschen erwarten
//! wuerde. Diese Kostenfunktion bildet das nach:
//!
//! ```text
//! Fidelity(t) ~= exp( -(t / tau_phi)^p(H) )
//! ```
//!
//! mit einem vom lokalen Hurst-Exponenten H abhaengigen Streckungsexponenten p(H).
//!
//! WICHTIG (ehrlicher Hinweis): Die konkrete Form p(H) = 2H ist eine
//! VEREINFACHENDE Modellannahme fuer dieses Proof-of-Concept, keine aus den
//! zitierten Arbeiten direkt uebernommene Formel. Tau_phi und die tatsaechliche
//! H-Abhaengigkeit muessten aus echten Kalibrierungsdaten (Ramsey-/Echo-Kurven)
//! der Zielhardware gefittet werden, bevor man dem Ergebnis in einem echten
//! Compiler-Pass vertraut.

/// Geschaetzte Fidelity nach Idle-/Wartezeit `t`, gegeben eine (aus Kalibrierung
/// gefittete) Dephasierungszeit `tau_phi` und den lokalen Hurst-Exponenten `h`.
pub fn predicted_fidelity(idle_time: f64, tau_phi: f64, h: f64) -> f64 {
    if idle_time <= 0.0 || tau_phi <= 0.0 {
        return 1.0;
    }
    let stretch_exponent = (2.0 * h).clamp(0.5, 2.5);
    (-(idle_time / tau_phi).powf(stretch_exponent)).exp()
}

/// Ein einzelner Schedule-Slot: welches Qubit ist fuer wie lange "idle"
/// (wartet auf ein anderes Gate im Circuit), waehrend eine Routing-/Scheduling-
/// Entscheidung getroffen wird.
#[derive(Debug, Clone)]
pub struct IdleSlot {
    pub qubit: usize,
    pub idle_time: f64,
    pub tau_phi: f64,
    pub local_hurst: f64,
}

/// Aggregierte, rauschbewusste Kosten eines Schedule-Kandidaten: Summe der
/// erwarteten Infidelity ueber alle Idle-Slots. Ein Router/Scheduler kann
/// mehrere Kandidaten (z.B. verschiedene SWAP-Reihenfolgen) damit vergleichen
/// und den Kandidaten mit der geringsten Gesamtkosten waehlen -- als Ergaenzung
/// zur reinen Gate-/SWAP-Anzahl.
pub fn schedule_cost(slots: &[IdleSlot]) -> f64 {
    slots
        .iter()
        .map(|s| 1.0 - predicted_fidelity(s.idle_time, s.tau_phi, s.local_hurst))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn longer_idle_time_costs_more() {
        let f_short = predicted_fidelity(1.0, 50.0, 0.7);
        let f_long = predicted_fidelity(20.0, 50.0, 0.7);
        assert!(f_long < f_short);
    }

    #[test]
    fn schedule_cost_sums_correctly() {
        let slots = vec![
            IdleSlot { qubit: 0, idle_time: 5.0, tau_phi: 40.0, local_hurst: 0.6 },
            IdleSlot { qubit: 1, idle_time: 15.0, tau_phi: 40.0, local_hurst: 0.8 },
        ];
        let cost = schedule_cost(&slots);
        let expected = (1.0 - predicted_fidelity(5.0, 40.0, 0.6))
            + (1.0 - predicted_fidelity(15.0, 40.0, 0.8));
        assert!((cost - expected).abs() < 1e-12);
    }

    #[test]
    fn zero_idle_time_has_no_cost() {
        assert_eq!(predicted_fidelity(0.0, 40.0, 0.7), 1.0);
    }
}
