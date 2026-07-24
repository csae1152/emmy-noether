//! `noether-link` — inter-module topology and telegate cost modeling
//! for distributed / modular quantum architectures.
//!
//! # Scope (v0.1 — near-field / chiplet regime only)
//!
//! This crate deliberately targets the **near-field, microwave-linked
//! chiplet regime** first, not long-haul photonic interconnects. In that
//! regime, inter-module coupling can be modeled topologically as an
//! extension of the local SWAP-chain graph the router already uses —
//! see `topology::ModuleGraph`. The photonic / heralded-entanglement
//! regime (distillation scheduling, non-deterministic link generation)
//! is intentionally out of scope for v0.1; `telegate::LinkKind::Photonic`
//! exists as a placeholder so the type system doesn't need to change
//! later, but its cost model is a stub.
//!
//! # Module layout
//!
//! - [`topology`] — module/link graph: which physical qubits live in
//!   which module, and which inter-module couplers exist.
//! - [`telegate`] — cost model for a single remote (inter-module)
//!   operation: entanglement generation/distribution + local ops +
//!   classical communication, per the standard DQC telegate decomposition.
//! - [`noise`] — entanglement-link fidelity decay, reusing
//!   `noether-noise`'s fractional process machinery with link-specific
//!   parameters instead of qubit-calibration-derived ones.
//! - [`cost`] — glue layer: turns telegate + noise into a single scalar
//!   (or Pareto-tuple) edge weight the router's path search can consume.
//!
//! # Integration seam with `noether-router`
//!
//! Two options, deliberately not decided here:
//!
//! 1. **Tight coupling**: `noether-router::emmy` imports `noether-link`
//!    directly and adds `CostModel::telegate_cost(&self, edge: LinkEdge)`
//!    as a new branch alongside the existing SWAP-cost branch.
//! 2. **Loose coupling**: `noether-link` exposes a `CostModel` trait
//!    (see [`cost::EdgeCost`]) that `noether-router` depends on as an
//!    abstraction, so the router doesn't need to know link internals.
//!
//! Option 2 keeps `noether-router`'s existing single-module IP
//! (`emmy.rs`) unmodified and testable in isolation — recommended
//! unless there's a concrete reason routing decisions need visibility
//! into telegate internals (e.g. joint SWAP/telegate optimization).

pub mod cost;
pub mod noise;
pub mod telegate;
pub mod topology;

pub use cost::EdgeCost;
pub use telegate::{LinkKind, TelegateCost};
pub use topology::{ModuleGraph, ModuleId, PhysicalQubitId};
