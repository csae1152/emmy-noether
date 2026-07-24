//! Cost model for a single telegate (remote inter-module two-qubit
//! operation), following the standard DQC decomposition:
//!
//!   telegate = entanglement generation/distribution
//!            + local operations (on both sides)
//!            + classical communication (to complete the protocol)
//!
//! Each component is modeled separately because they have genuinely
//! different scaling behavior and different places a compiler can
//! optimize them: entanglement generation is the dominant *time* cost
//! and the dominant *fidelity* cost; local ops are cheap and already
//! covered by the router's existing single-module cost model; classical
//! communication is usually latency-bound, not fidelity-bound, but
//! matters for scheduling (it serializes dependent telegates).

use std::time::Duration;

/// Distinguishes the physical regime of an inter-module coupler.
/// See the crate-level docs for why v0.1 only implements `NearField`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkKind {
    /// Microwave/capacitive chiplet-to-chiplet coupler. Deterministic,
    /// short-distance — cost is dominated by coupler gate fidelity and
    /// duty-cycle heating, closer in character to an ordinary two-qubit
    /// gate than to a networking problem.
    NearField,
    /// Heralded photonic entanglement over longer distances.
    /// Non-deterministic generation (success probability < 1, requires
    /// retries), distillation may be needed to reach usable fidelity.
    /// **Not implemented in v0.1** — see [`TelegateCost::for_link`].
    Photonic,
}

/// The three cost components of a single telegate, kept separate
/// rather than pre-summed so callers can apply different objectives
/// (e.g. minimize fidelity loss subject to a latency budget, rather
/// than a single weighted scalar) without recomputing anything.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TelegateCost {
    /// Expected wall-clock time to generate and distribute usable
    /// entanglement across this link. For `NearField`, effectively
    /// the coupler gate duration; deterministic in v0.1 (no retry
    /// modeling — see open question in README.md).
    pub entanglement_time: Duration,
    /// Fidelity of the distributed entangled pair *before* any local
    /// correction — i.e. the raw resource quality the router's
    /// downstream cost function should weight against local-op noise
    /// when deciding whether a telegate or a longer local reroute is
    /// cheaper.
    pub entanglement_fidelity: f64,
    /// Fixed classical round-trip overhead to complete the telegate
    /// protocol (correction signaling). Matters for *scheduling*
    /// (serializes dependent operations) more than for the router's
    /// per-edge cost, but included here so `cost::EdgeCost` doesn't
    /// need a second data source.
    pub classical_overhead: Duration,
}

impl TelegateCost {
    /// Placeholder near-field cost model. The two constants below are
    /// NOT calibrated — they're structural placeholders so the type
    /// signature and integration points are correct from day one.
    /// Replace with a calibration-data-driven estimate (analogous to
    /// how `noether-noise::hurst` derives its Hurst exponent from
    /// per-qubit calibration, not from a hardcoded constant) before
    /// this feeds into any real routing decision.
    ///
    /// The fidelity term is where the fractional-noise reuse matters
    /// most: near-field couplers used in rapid succession (as routing
    /// will do, since telegates sit on the hot path of any multi-hop
    /// remote operation) plausibly exhibit the same kind of
    /// non-Markovian, memory-carrying degradation your qubit-level
    /// dynamical-decoupling model already targets — see
    /// [`crate::noise`].
    pub fn for_link(kind: LinkKind) -> Self {
        match kind {
            LinkKind::NearField => TelegateCost {
                entanglement_time: Duration::from_nanos(200),
                entanglement_fidelity: 0.995,
                classical_overhead: Duration::from_nanos(50),
            },
            LinkKind::Photonic => {
                // Deliberately unimplemented rather than silently wrong.
                // A real model needs: generation success probability,
                // retry-until-success expected time, distillation
                // rounds vs. target fidelity. That's a distinct enough
                // problem (queueing/retry, not just decay) that it
                // probably wants its own module, not a variant here.
                unimplemented!(
                    "Photonic telegate cost is out of scope for noether-link v0.1 \
                     — see crate-level docs. Tracked as a follow-on, not stubbed \
                     with fake numbers to avoid a silently-wrong default."
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn near_field_cost_is_well_formed() {
        let c = TelegateCost::for_link(LinkKind::NearField);
        assert!(c.entanglement_fidelity > 0.0 && c.entanglement_fidelity <= 1.0);
    }

    #[test]
    #[should_panic]
    fn photonic_is_intentionally_unimplemented() {
        let _ = TelegateCost::for_link(LinkKind::Photonic);
    }
}
