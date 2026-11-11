// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-044: the linked finite protocol representation of one `protocol_clause`
//! of an admitted [`CheckedPackageV2`].
//!
//! The lists come from QSL's [`protocol_view`] and the causal relation from
//! QSL's [`causal_relation`]; this module owns no table of nodes and builds no
//! causal edge. A read QSL cannot answer is QSL's own [`ViewRefusal`], carried
//! unchanged; a node or member the linker itself reads from the package and
//! cannot find is the linker's own refusal. No partial representation
//! accompanies either.
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

use crate::declarations::Declarations;
use qsl_package::protocol_view::{
    causal_relation, control_operation, delivery, locus, protocol_view, temporal_binding,
    CausalEdge, ControlOperation, ProtocolView, Scheduling, ViewRefusal,
};
use quire_contract_model::{
    CheckedNodeId, CheckedPackageV2, CheckedProtocolActivationRow, CheckedSemanticNodeV2,
    CheckedSourceMapEntry, CheckedTerm, LiteralKind, TemporalBinding,
};

/// A consumer-owned refusal produced while linking an admitted protocol.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LinkRefusal {
    /// QSL could not answer a typed read. The owner's refusal is preserved
    /// exactly as QSL returned it; the linker never constructs one.
    View(ViewRefusal),
    /// A node listed by the protocol view is absent from the package graph.
    UnreadableNode {
        /// The listed node the linker could not read.
        node: CheckedNodeId,
    },
    /// A node lacks a member the linker reads from its checked content.
    MissingMember {
        /// The node read.
        node: CheckedNodeId,
        /// The member the linker needed.
        member: &'static str,
    },
    /// A declaration member the linker reads is present but not a value the
    /// QSL layout admits for it.
    MalformedMember {
        /// The node read.
        node: CheckedNodeId,
        /// The member that did not read.
        member: &'static str,
    },
    /// A repeat control has a maximum of zero and therefore cannot progress.
    ZeroProgressIteration {
        /// The repeat control.
        control: CheckedNodeId,
        /// The available source-map entries for the control.
        locus: Vec<CheckedSourceMapEntry>,
    },
    /// Two role nodes name the same model authority.
    DuplicateRoleAuthority {
        /// The later role node in the protocol view.
        role: CheckedNodeId,
        /// The shared authority model node.
        authority: CheckedNodeId,
        /// The available source-map entries for the later role.
        locus: Vec<CheckedSourceMapEntry>,
    },
    /// Two channel nodes with the same route and payload state conflicting premises.
    ConflictingChannelPremise {
        /// The later channel node in the protocol view.
        channel: CheckedNodeId,
        /// The earlier channel node with the same route and payload.
        other: CheckedNodeId,
        /// Source-map entries available for both channel nodes.
        locus: Vec<CheckedSourceMapEntry>,
    },
}

pub(crate) fn node<'a>(
    package: &'a CheckedPackageV2,
    id: &CheckedNodeId,
) -> Option<&'a CheckedSemanticNodeV2> {
    package
        .graph()
        .nodes
        .iter()
        .find(|node| &node.node_id == id)
}

pub(crate) fn argument<'a>(
    package: &'a CheckedPackageV2,
    node: &CheckedNodeId,
    name: &str,
) -> Option<&'a CheckedTerm> {
    let CheckedTerm::Application { arguments, .. } = package.term(node)? else {
        return None;
    };
    arguments.iter().find_map(|argument| match argument {
        CheckedTerm::Binding { name: bound, value } if &**bound == name => Some(value.as_ref()),
        _ => None,
    })
}

pub(crate) fn reference(term: &CheckedTerm) -> Option<&CheckedNodeId> {
    match term {
        CheckedTerm::Reference { target } => Some(target),
        _ => None,
    }
}

fn integer(term: &CheckedTerm) -> Option<u64> {
    let CheckedTerm::Aggregate { members } = term else {
        return None;
    };
    let [CheckedTerm::Literal {
        value_kind: LiteralKind::Integer,
        value,
        ..
    }] = members.as_slice()
    else {
        return None;
    };
    // QSL encodes an integer literal as its canonical decimal string only.
    match value.as_str() {
        Some(value) if !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()) => {
            value.parse().ok()
        }
        _ => None,
    }
}

pub(crate) fn source_locus(
    package: &CheckedPackageV2,
    node: &CheckedNodeId,
) -> Vec<CheckedSourceMapEntry> {
    locus(package.source_map(), node)
        .map(|entries| entries.into_iter().cloned().collect())
        .unwrap_or_default()
}

fn has_zero_repeat_maximum(package: &CheckedPackageV2, node: &CheckedSemanticNodeV2) -> bool {
    control_operation(node) == Ok(ControlOperation::Repeat)
        && argument(package, &node.node_id, "maximum").and_then(integer) == Some(0)
}

fn same_channel_premises(
    package: &CheckedPackageV2,
    left: &CheckedSemanticNodeV2,
    right: &CheckedSemanticNodeV2,
) -> Result<bool, ViewRefusal> {
    let left_delivery = delivery(left)?;
    let right_delivery = delivery(right)?;
    let same_channel = ["from", "to", "carries"].into_iter().all(|name| {
        argument(package, &left.node_id, name) == argument(package, &right.node_id, name)
    });
    let same_premise = argument(package, &left.node_id, "ordering")
        == argument(package, &right.node_id, "ordering")
        && left_delivery == right_delivery;
    Ok(!same_channel || same_premise)
}

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
    requirement_terms: Vec<CheckedTerm>,
    temporal_bindings: Vec<TemporalBinding>,
    run: CheckedNodeId,
    finish: CheckedNodeId,
    terminal: Option<CheckedNodeId>,
    scheduling: Option<Scheduling>,
    activation: Vec<CheckedProtocolActivationRow>,
    edges: Vec<CausalEdge>,
    declarations: Declarations,
}

fn owned(ids: &[&CheckedNodeId]) -> Vec<CheckedNodeId> {
    ids.iter().map(|id| (*id).clone()).collect()
}

impl LinkedProtocol {
    fn of(
        view: &ProtocolView<'_>,
        edges: &[CausalEdge],
        requirement_terms: Vec<CheckedTerm>,
        temporal_bindings: Vec<TemporalBinding>,
        declarations: Declarations,
    ) -> Self {
        Self {
            clause: view.clause().clone(),
            roles: owned(view.roles()),
            channels: owned(view.channels()),
            queues: owned(view.queues()),
            controls: owned(view.controls()),
            compensations: owned(view.compensations()),
            requirements: owned(view.requirements()),
            requirement_terms,
            temporal_bindings,
            run: view.run().clone(),
            finish: view.finish().clone(),
            terminal: view.terminal().cloned(),
            scheduling: view.scheduling(),
            activation: view.activation().to_vec(),
            edges: edges.to_vec(),
            declarations,
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

    /// The admitted typed bodies of the required temporal clauses.
    pub fn requirement_terms(&self) -> &[CheckedTerm] {
        &self.requirement_terms
    }

    /// The selected clock bindings for required temporal clauses, in view order.
    pub fn temporal_bindings(&self) -> &[TemporalBinding] {
        &self.temporal_bindings
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

    /// The authored role, channel, join and loop declarations (FR-050 to
    /// FR-052).
    pub fn declarations(&self) -> &Declarations {
        &self.declarations
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
/// QSL's own [`ViewRefusal`], unchanged inside [`LinkRefusal::View`], for
/// every read QSL answers with a refusal: an absent or non-protocol node, a
/// `protocol_reference` carrier, a node lacking a member its layout states.
/// The linker's own refusals identify a listed node or member it cannot read
/// ([`LinkRefusal::UnreadableNode`], [`LinkRefusal::MissingMember`]),
/// zero-progress repeats, duplicate authorities, and conflicting channel
/// premises. No representation is returned with a refusal.
pub fn link(
    package: &CheckedPackageV2,
    protocol: &CheckedNodeId,
) -> Result<LinkedProtocol, LinkRefusal> {
    let view = protocol_view(package, protocol).map_err(LinkRefusal::View)?;
    for id in view.controls().iter().copied() {
        let node =
            node(package, id).ok_or_else(|| LinkRefusal::UnreadableNode { node: id.clone() })?;
        if has_zero_repeat_maximum(package, node) {
            return Err(LinkRefusal::ZeroProgressIteration {
                control: id.clone(),
                locus: source_locus(package, id),
            });
        }
    }

    let mut authorities: Vec<(&CheckedNodeId, &CheckedNodeId)> = Vec::new();
    for id in view.roles().iter().copied() {
        node(package, id).ok_or_else(|| LinkRefusal::UnreadableNode { node: id.clone() })?;
        let authority = argument(package, id, "model")
            .and_then(reference)
            .ok_or_else(|| LinkRefusal::MissingMember {
                node: id.clone(),
                member: "model",
            })?;
        if authorities
            .iter()
            .any(|(_, previous)| *previous == authority)
        {
            return Err(LinkRefusal::DuplicateRoleAuthority {
                role: id.clone(),
                authority: authority.clone(),
                locus: source_locus(package, id),
            });
        }
        authorities.push((id, authority));
    }

    let mut channels: Vec<(&CheckedNodeId, &CheckedSemanticNodeV2)> = Vec::new();
    for id in view.channels().iter().copied() {
        let node =
            node(package, id).ok_or_else(|| LinkRefusal::UnreadableNode { node: id.clone() })?;
        for (other_id, other_node) in &channels {
            if !same_channel_premises(package, other_node, node).map_err(LinkRefusal::View)? {
                let mut entries = source_locus(package, id);
                entries.extend(source_locus(package, other_id));
                return Err(LinkRefusal::ConflictingChannelPremise {
                    channel: id.clone(),
                    other: (*other_id).clone(),
                    locus: entries,
                });
            }
        }
        channels.push((id, node));
    }

    let mut requirement_terms = Vec::with_capacity(view.requirements().len());
    let mut temporal_bindings = Vec::with_capacity(view.requirements().len());
    for id in view.requirements().iter().copied() {
        let term = package.term(id).ok_or_else(|| LinkRefusal::MissingMember {
            node: id.clone(),
            member: "body",
        })?;
        let rows = package
            .temporal_bindings()
            .ok_or_else(|| LinkRefusal::MissingMember {
                node: id.clone(),
                member: "temporal_bindings",
            })?;
        let binding = temporal_binding(rows, id).map_err(LinkRefusal::View)?;
        requirement_terms.push(term.clone());
        temporal_bindings.push(binding.clone());
    }

    let declarations = Declarations::read(package, &view)?;
    let relation = causal_relation(package, protocol).map_err(LinkRefusal::View)?;
    Ok(LinkedProtocol::of(
        &view,
        relation.edges(),
        requirement_terms,
        temporal_bindings,
        declarations,
    ))
}
