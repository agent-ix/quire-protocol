// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-044: the linked finite protocol representation of one `protocol_clause`
//! of an admitted [`CheckedPackageV2`].
//!
//! The lists come from QSL's [`protocol_view`] and the causal relation from
//! QSL's [`causal_relation`]; this module owns no table of nodes and builds no
//! causal edge. A read the package cannot answer is the owner's
//! [`ViewRefusal`], and no partial representation accompanies it.
//!
//! The retired wire types do not resolve through this crate.
//!
//! ```compile_fail,E0432
//! use quire_protocol::link::Package;
//! ```
//!
//! ```compile_fail,E0432
//! use quire_protocol::link::Handle;
//! ```
//!
//! ```compile_fail,E0432
//! use quire_protocol::link::Declaration;
//! ```
//!
//! ```compile_fail,E0432
//! use quire_protocol::wire::Package;
//! ```

use qsl_package::protocol_view::{
    causal_relation, protocol_view, CausalEdge, ProtocolView, Scheduling, ViewRefusal,
};
use quire_contract_model::{CheckedNodeId, CheckedPackageV2, CheckedProtocolActivationRow};

/// The nodes and causal edges of one protocol clause.
#[derive(Clone, Debug)]
pub struct LinkedProtocol {
    clause: CheckedNodeId,
    roles: Vec<CheckedNodeId>,
    channels: Vec<CheckedNodeId>,
    queues: Vec<CheckedNodeId>,
    controls: Vec<CheckedNodeId>,
    compensations: Vec<CheckedNodeId>,
    requirements: Vec<CheckedNodeId>,
    run: CheckedNodeId,
    finish: CheckedNodeId,
    terminal: Option<CheckedNodeId>,
    scheduling: Option<Scheduling>,
    activation: Vec<CheckedProtocolActivationRow>,
    edges: Vec<CausalEdge>,
}

fn owned(ids: &[&CheckedNodeId]) -> Vec<CheckedNodeId> {
    ids.iter().map(|id| (*id).clone()).collect()
}

impl LinkedProtocol {
    fn of(view: &ProtocolView<'_>, edges: &[CausalEdge]) -> Self {
        Self {
            clause: view.clause().clone(),
            roles: owned(view.roles()),
            channels: owned(view.channels()),
            queues: owned(view.queues()),
            controls: owned(view.controls()),
            compensations: owned(view.compensations()),
            requirements: owned(view.requirements()),
            run: view.run().clone(),
            finish: view.finish().clone(),
            terminal: view.terminal().cloned(),
            scheduling: view.scheduling(),
            activation: view.activation().to_vec(),
            edges: edges.to_vec(),
        }
    }

    /// The `protocol_clause` node.
    pub fn clause(&self) -> &CheckedNodeId {
        &self.clause
    }

    /// The role nodes, in the view's order.
    pub fn roles(&self) -> &[CheckedNodeId] {
        &self.roles
    }

    /// The channel nodes, in the view's order.
    pub fn channels(&self) -> &[CheckedNodeId] {
        &self.channels
    }

    /// The queue nodes, in the view's order.
    pub fn queues(&self) -> &[CheckedNodeId] {
        &self.queues
    }

    /// The control nodes in layout order.
    pub fn controls(&self) -> &[CheckedNodeId] {
        &self.controls
    }

    /// The compensation nodes, in the view's order.
    pub fn compensations(&self) -> &[CheckedNodeId] {
        &self.compensations
    }

    /// The required temporal clause nodes.
    pub fn requirements(&self) -> &[CheckedNodeId] {
        &self.requirements
    }

    /// The root `run` control.
    pub fn run(&self) -> &CheckedNodeId {
        &self.run
    }

    /// The `finish` obligation.
    pub fn finish(&self) -> &CheckedNodeId {
        &self.finish
    }

    /// The `terminal` obligation, when the clause has one.
    pub fn terminal(&self) -> Option<&CheckedNodeId> {
        self.terminal.as_ref()
    }

    /// The `scheduling` member, when written.
    pub fn scheduling(&self) -> Option<Scheduling> {
        self.scheduling
    }

    /// The activation rows of the clause.
    pub fn activation(&self) -> &[CheckedProtocolActivationRow] {
        &self.activation
    }

    /// The causal edges, exactly as QSL's `causal_relation` returns them.
    pub fn edges(&self) -> &[CausalEdge] {
        &self.edges
    }
}

/// Link the protocol clause `protocol` of `package`.
///
/// # Errors
///
/// The owner's [`ViewRefusal`]: `UnknownProtocol` for an absent node or a
/// node that is not a `protocol` node, `NotAProtocolClause` for a
/// `protocol_reference` carrier, and the member-naming refusals for a node
/// that lacks a member its layout states. No representation is returned with
/// a refusal.
pub fn link(
    package: &CheckedPackageV2,
    protocol: &CheckedNodeId,
) -> Result<LinkedProtocol, ViewRefusal> {
    let view = protocol_view(package, protocol)?;
    let relation = causal_relation(package, protocol)?;
    Ok(LinkedProtocol::of(&view, relation.edges()))
}
