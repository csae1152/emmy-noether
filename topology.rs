//! Module and inter-module link topology.
//!
//! Design choice: rather than inventing a parallel graph type, this
//! models the multi-module system as a single `petgraph` graph whose
//! nodes are physical qubits (same as the router's existing local
//! graph) and whose edges carry an [`EdgeClass`] discriminant. This
//! means the router's existing shortest-path code *can* traverse
//! inter-module edges without modification, as long as it asks
//! `EdgeClass` for a cost instead of assuming a uniform SWAP cost.
//!
//! If that turns out to be too invasive to retrofit into `emmy.rs`,
//! the fallback is to keep two separate graphs (`local: UnGraph<..>`
//! per module + a small `inter_module: UnGraph<..>` of module-to-module
//! couplers) and compose paths across the boundary explicitly. Left as
//! an open question — see README.md.

use petgraph::graph::{NodeIndex, UnGraph};
use std::collections::HashMap;

/// Identifies a module (a chiplet, a trapped-ion trap segment, etc.).
/// Opaque on purpose — the router shouldn't need to know what a module
/// *is* physically, only that operations crossing module boundaries
/// are costed differently.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ModuleId(pub u32);

/// A physical qubit, tagged with the module it lives in. Distinct from
/// whatever qubit-indexing type `noether-core` already uses for
/// single-module circuits — conversion is expected to happen at the
/// integration seam, not inside this crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PhysicalQubitId {
    pub module: ModuleId,
    pub local_index: u32,
}

/// Discriminates local (intra-module) couplings from inter-module
/// links. Local edges are expected to reuse whatever cost the router
/// already computes for SWAPs; this crate only adds new information
/// for the `InterModule` case.
#[derive(Debug, Clone, PartialEq)]
pub enum EdgeClass {
    /// Ordinary intra-module coupling — no telegate involved. Carried
    /// here mainly so a single graph type can represent both edge
    /// kinds; the router's existing cost logic should still own this.
    Local,
    /// A physical coupler connecting two modules. `kind` distinguishes
    /// near-field (chiplet, microwave/capacitive) from far-link
    /// (photonic) couplers — see [`crate::telegate::LinkKind`].
    InterModule(crate::telegate::LinkKind),
}

pub struct ModuleGraph {
    graph: UnGraph<PhysicalQubitId, EdgeClass>,
    index_of: HashMap<PhysicalQubitId, NodeIndex>,
}

impl ModuleGraph {
    pub fn new() -> Self {
        Self {
            graph: UnGraph::new_undirected(),
            index_of: HashMap::new(),
        }
    }

    pub fn add_qubit(&mut self, qubit: PhysicalQubitId) -> NodeIndex {
        *self
            .index_of
            .entry(qubit)
            .or_insert_with(|| self.graph.add_node(qubit))
    }

    pub fn add_local_edge(&mut self, a: PhysicalQubitId, b: PhysicalQubitId) {
        let (ia, ib) = (self.add_qubit(a), self.add_qubit(b));
        self.graph.add_edge(ia, ib, EdgeClass::Local);
    }

    /// Registers a physical inter-module coupler. Note this is the
    /// *coupler*, not a telegate cost — cost is computed on demand by
    /// [`crate::cost`] from [`crate::telegate::TelegateCost`] plus
    /// current entanglement-fidelity state, since both are
    /// time-varying (calibration drift, duty-cycle heating, etc.) in
    /// a way a static edge weight can't capture well.
    pub fn add_inter_module_link(
        &mut self,
        a: PhysicalQubitId,
        b: PhysicalQubitId,
        kind: crate::telegate::LinkKind,
    ) {
        assert_ne!(
            a.module, b.module,
            "add_inter_module_link called with two qubits in the same module; \
             use add_local_edge instead"
        );
        let (ia, ib) = (self.add_qubit(a), self.add_qubit(b));
        self.graph.add_edge(ia, ib, EdgeClass::InterModule(kind));
    }

    /// All inter-module couplers touching `module` — useful for the
    /// partitioner (step "1. Circuit-Partitionierung" from the earlier
    /// discussion) to know which modules a given module can telegate
    /// to directly vs. only via multi-hop entanglement swapping.
    pub fn links_of(&self, module: ModuleId) -> Vec<(PhysicalQubitId, PhysicalQubitId)> {
        use petgraph::visit::EdgeRef;
        self.graph
            .edge_references()
            .filter_map(|e| {
                let (a, b) = (self.graph[e.source()], self.graph[e.target()]);
                match e.weight() {
                    EdgeClass::InterModule(_) if a.module == module || b.module == module => {
                        Some((a, b))
                    }
                    _ => None,
                }
            })
            .collect()
    }

    pub fn inner(&self) -> &UnGraph<PhysicalQubitId, EdgeClass> {
        &self.graph
    }
}

impl Default for ModuleGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telegate::LinkKind;

    #[test]
    fn rejects_same_module_inter_link() {
        let mut g = ModuleGraph::new();
        let a = PhysicalQubitId { module: ModuleId(0), local_index: 0 };
        let b = PhysicalQubitId { module: ModuleId(0), local_index: 1 };
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            g.add_inter_module_link(a, b, LinkKind::NearField);
        }));
        assert!(result.is_err());
    }

    #[test]
    fn links_of_filters_by_module() {
        let mut g = ModuleGraph::new();
        let a = PhysicalQubitId { module: ModuleId(0), local_index: 0 };
        let b = PhysicalQubitId { module: ModuleId(1), local_index: 0 };
        g.add_inter_module_link(a, b, LinkKind::NearField);
        assert_eq!(g.links_of(ModuleId(0)).len(), 1);
        assert_eq!(g.links_of(ModuleId(2)).len(), 0);
    }
}
