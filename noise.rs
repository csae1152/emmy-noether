//! Entanglement-link fidelity decay.
//!
//! **Assumption flagged, not verified**: this module assumes
//! `noether-noise` exposes something like a `FractionalProcess` type
//! parameterized by a (possibly time-varying) Hurst exponent, since
//! that's how it was described for qubit calibration drift. The exact
//! type/trait names below are guesses — treat this file as the shape
//! of the integration, not a drop-in. Swap in the real import once
//! you're at a keyboard with the actual `noether-noise` API in front
//! of you.
//!
//! # Why reuse the fractional model here at all
//!
//! Naive DQC noise models treat entanglement fidelity as a fixed
//! per-link constant (see the `phys.org` / npj-Quantum-Information
//! results discussed earlier — even state-of-the-art work mostly
//! reports a single threshold number, not a decay process). That's a
//! reasonable simplification for a single generation event, but
//! routing decisions chain multiple telegates over the *same* physical
//! coupler under time pressure — repeated rapid use is exactly the
//! regime where non-Markovian, memory-carrying noise (duty-cycle
//! heating, quasi-static drift between calibration cycles) shows up
//! and a plain exponential-decay model underestimates degradation.
//! That's the same argument that motivated the fractional model for
//! qubit-level dynamical decoupling — this module is a bet that it
//! transfers, not a proven result. Worth validating against real
//! coupler data before it drives routing decisions.

use std::time::Duration;

/// Placeholder for whatever `noether-noise` actually calls its
/// fractional-process handle. Kept as a trait here so this crate can
/// compile and be tested independently of the exact upstream type —
/// replace with a direct dependency once wired up for real.
pub trait FractionalDecayModel {
    /// Hurst exponent at a given point in the link's duty cycle.
    /// H > 0.5 → persistent (memory-reinforcing) noise, H < 0.5 →
    /// anti-persistent. Time-varying to capture e.g. thermal buildup
    /// over a burst of consecutive telegates.
    fn hurst_exponent(&self, elapsed_since_last_use: Duration) -> f64;
}

/// Entanglement fidelity as a function of how recently and how often
/// a given coupler has been used — the link-level analogue of
/// `noether-noise`'s per-qubit calibration-driven decoherence model.
pub struct LinkNoiseModel<M: FractionalDecayModel> {
    base_fidelity: f64,
    decay: M,
}

impl<M: FractionalDecayModel> LinkNoiseModel<M> {
    pub fn new(base_fidelity: f64, decay: M) -> Self {
        Self { base_fidelity, decay }
    }

    /// Estimated fidelity for the *next* telegate over this coupler,
    /// given how long it's been since the last one. This is the
    /// number [`crate::cost::EdgeCost`] should call into — it's
    /// deliberately not cached/memoized here, since "elapsed since
    /// last use" only makes sense in the context of a specific
    /// schedule the router is currently evaluating.
    ///
    /// Placeholder combination rule (linear discount by Hurst-scaled
    /// factor) — this is the part most likely to need replacing with
    /// whatever fractional-Brownian-motion-to-fidelity mapping
    /// `noether-noise::noise_gen` already uses for qubits, rather than
    /// inventing a second, inconsistent one here.
    pub fn fidelity_estimate(&self, elapsed_since_last_use: Duration) -> f64 {
        let h = self.decay.hurst_exponent(elapsed_since_last_use);
        let recency_penalty = (1.0 - elapsed_since_last_use.as_secs_f64().min(1.0)) * (h - 0.5).abs();
        (self.base_fidelity - recency_penalty).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct ConstantHurst(f64);
    impl FractionalDecayModel for ConstantHurst {
        fn hurst_exponent(&self, _elapsed: Duration) -> f64 {
            self.0
        }
    }

    #[test]
    fn fidelity_stays_in_bounds() {
        let model = LinkNoiseModel::new(0.995, ConstantHurst(0.7));
        let f = model.fidelity_estimate(Duration::from_millis(10));
        assert!((0.0..=1.0).contains(&f));
    }
}
