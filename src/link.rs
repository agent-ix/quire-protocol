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
    causal_relation, control_operation, delivery, locus, protocol_view, temporal_binding,
    CausalEdge, ControlOperation, Delivery, ProtocolView, Scheduling, ViewRefusal,
};
use quire_contract_model::{
    CheckedNodeId, CheckedPackageV2, CheckedProtocolActivationRow, CheckedSemanticNodeV2,
    CheckedSourceMapEntry, CheckedTerm, LiteralKind,
};
use serde_json::Value;

/// A consumer-owned refusal produced while linking an admitted protocol.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LinkRefusal {
    /// QSL could not answer a typed read. The owner's refusal is preserved.
    View(ViewRefusal),
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
    /// Two channel nodes with the same name state different delivery premises.
    ConflictingChannelPremise {
        /// The later channel node in the protocol view.
        channel: CheckedNodeId,
        /// The earlier channel node with the same name.
        other: CheckedNodeId,
        /// Source-map entries available for both channel nodes.
        locus: Vec<CheckedSourceMapEntry>,
    },
}

fn argument<'a>(node: &'a CheckedSemanticNodeV2, name: &str) -> Option<&'a CheckedTerm> {
    let CheckedTerm::Application { arguments, .. } = &node.body else {
        return None;
    };
    arguments.iter().find_map(|argument| match argument {
        CheckedTerm::Binding { name: bound, value } if &**bound == name => Some(value.as_ref()),
        _ => None,
    })
}

fn reference(term: &CheckedTerm) -> Option<&CheckedNodeId> {
    match term {
        CheckedTerm::Reference { target } => Some(target),
        _ => None,
    }
}

fn text(term: &CheckedTerm) -> Option<&str> {
    match term {
        CheckedTerm::Literal {
            value_kind: LiteralKind::Text,
            value: Value::String(value),
            ..
        } => Some(value),
        _ => None,
    }
}

fn integer(term: &CheckedTerm) -> Option<u64> {
    match term {
        CheckedTerm::Literal {
            value_kind: LiteralKind::Integer,
            value: Value::String(value),
            ..
        } if !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()) => {
            value.parse().ok()
        }
        _ => None,
    }
}

fn source_locus(package: &CheckedPackageV2, node: &CheckedNodeId) -> Vec<CheckedSourceMapEntry> {
    locus(package.source_map(), node)
        .map(|entries| entries.into_iter().cloned().collect())
        .unwrap_or_default()
}

fn has_zero_repeat_maximum(node: &CheckedSemanticNodeV2) -> bool {
    control_operation(node) == Ok(ControlOperation::Repeat)
        && argument(node, "maximum").and_then(integer) == Some(0)
}

fn same_channel_premises(
    left: &CheckedSemanticNodeV2,
    right: &CheckedSemanticNodeV2,
) -> Result<bool, ViewRefusal> {
    let left_delivery = delivery(left)?;
    let right_delivery = delivery(right)?;
    let same = ["ordering", "capacity", "overflow"]
        .into_iter()
        .all(|name| argument(left, name) == argument(right, name));
    Ok(same && left_delivery == right_delivery)
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
/// An owner [`ViewRefusal`] wrapped in [`LinkRefusal::View`] for an absent
/// node or a node that is not a `protocol` node, `NotAProtocolClause` for a
/// `protocol_reference` carrier, and the member-naming refusals for a node
/// that lacks a member its layout states. Consumer refusals identify zero
/// progress repeats, duplicate authorities, and conflicting channel premises.
/// No representation is returned with a refusal.
pub fn link(
    package: &CheckedPackageV2,
    protocol: &CheckedNodeId,
) -> Result<LinkedProtocol, LinkRefusal> {
    let view = protocol_view(package, protocol).map_err(LinkRefusal::View)?;
    for id in view.controls() {
        let node = package
            .node(id)
            .ok_or_else(|| LinkRefusal::View(ViewRefusal::UnknownProtocol { node: id.clone() }))?;
        if has_zero_repeat_maximum(node) {
            return Err(LinkRefusal::ZeroProgressIteration {
                control: id.clone(),
                locus: source_locus(package, id),
            });
        }
    }

    let mut authorities: Vec<(&CheckedNodeId, &CheckedNodeId)> = Vec::new();
    for id in view.roles() {
        let node = package
            .node(id)
            .ok_or_else(|| LinkRefusal::View(ViewRefusal::UnknownProtocol { node: id.clone() }))?;
        let authority = argument(node, "model").and_then(reference).ok_or_else(|| {
            LinkRefusal::View(ViewRefusal::MissingMember {
                node: id.clone(),
                member: "model",
            })
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

    let mut channels: Vec<(&CheckedNodeId, &str, &CheckedSemanticNodeV2)> = Vec::new();
    for id in view.channels() {
        let node = package
            .node(id)
            .ok_or_else(|| LinkRefusal::View(ViewRefusal::UnknownProtocol { node: id.clone() }))?;
        let name = argument(node, "name").and_then(text).ok_or_else(|| {
            LinkRefusal::View(ViewRefusal::MissingMember {
                node: id.clone(),
                member: "name",
            })
        })?;
        for (other_id, other_name, other_node) in &channels {
            if name == *other_name
                && !same_channel_premises(other_node, node).map_err(LinkRefusal::View)?
            {
                let mut entries = source_locus(package, id);
                entries.extend(source_locus(package, other_id));
                return Err(LinkRefusal::ConflictingChannelPremise {
                    channel: id.clone(),
                    other: (*other_id).clone(),
                    locus: entries,
                });
            }
        }
        channels.push((id, name, node));
    }

    if let Some(rows) = package.temporal_bindings() {
        for id in view.requirements() {
            temporal_binding(rows, id).map_err(LinkRefusal::View)?;
        }
    } else if let Some(id) = view.requirements().first() {
        return Err(LinkRefusal::View(ViewRefusal::MissingMember {
            node: id.clone(),
            member: "temporal_bindings",
        }));
    }

    let relation = causal_relation(package, protocol).map_err(LinkRefusal::View)?;
    Ok(LinkedProtocol::of(&view, relation.edges()))
}
