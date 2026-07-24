//! Turns [`crate::telegate::TelegateCost`] + [`crate::noise`] into a
//! single edge-cost value the router's path search can consume,
//! without requiring `noether-router` to know anything about telegate
//! internals — see "Integration seam" in `lib.rs`.

use crate::telegate::TelegateCost;
use std::time::Duration;

/// Deliberately a trait, not a concrete struct: `noether-router`
/// should depend on this trait (loose coupling, option 2 from the
/// crate-level docs), and this crate provides one implementation.
/// If you later want joint SWAP/telegate optimization in `emmy.rs`,
/// having this as a trait means the router can also implement it
/// directly for tight coupling without noether-link needing to change.
pub trait EdgeCost {
    /// A single scalar suitable for shortest-path search. Combining
    /// time and fidelity into one number is a real modeling choice,
    /// not a formality — see `ScalarCost` below for the default and
    /// why it's probably wrong for your actual objective.
    fn scalar_cost(&self) -> f64;
}

/// Weights for combining telegate time and fidelity loss into a
/// single scalar. **Not calibrated** — these need to come from
/// whatever the router's existing SWAP-cost objective already weighs
/// time vs. fidelity by, so telegate costs and SWAP costs are
/// comparable on the same path. Mixing two independently-chosen
/// scales here would silently bias the router toward or against
/// telegates regardless of their true relative cost.
#[derive(Debug, Clone, Copy)]
pub struct CostWeights {
    pub time_weight: f64,
    pub infidelity_weight: f64,
}

pub struct ScalarCost {
    pub telegate: TelegateCost,
    pub weights: CostWeights,
}

impl EdgeCost for ScalarCost {
    fn scalar_cost(&self) -> f64 {
        let time_s = self.telegate.entanglement_time.as_secs_f64()
            + self.telegate.classical_overhead.as_secs_f64();
        let infidelity = 1.0 - self.telegate.entanglement_fidelity;
        self.weights.time_weight * time_s + self.weights.infidelity_weight * infidelity
    }
}

/// Alternative to `ScalarCost` worth considering before committing to
/// a single weighted sum: expose the raw (time, infidelity) pair and
/// let the router's search operate over a Pareto frontier, or apply a
/// hard fidelity budget (reject any path whose cumulative infidelity
/// exceeds a threshold) rather than trading fidelity for speed
/// linearly. Not implemented here — flagging it because "just take a
/// weighted sum" is the easy default and it's worth deciding
/// deliberately rather than by default.
pub struct ParetoCost {
    pub time: Duration,
    pub infidelity: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telegate::LinkKind;

    #[test]
    fn scalar_cost_increases_with_infidelity_weight() {
        let telegate = TelegateCost::for_link(LinkKind::NearField);
        let low = ScalarCost {
            telegate,
            weights: CostWeights { time_weight: 1.0, infidelity_weight: 0.0 },
        };
        let high = ScalarCost {
            telegate,
            weights: CostWeights { time_weight: 1.0, infidelity_weight: 1000.0 },
        };
        assert!(high.scalar_cost() > low.scalar_cost());
    }
}
