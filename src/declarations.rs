// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-050, FR-051, FR-052: the authored role, channel, join and loop
//! declarations of one linked `protocol_clause`.
//!
//! Every value is read from the checked node's own content, through QSL's
//! `delivery` and `queue_capacity` accessors wherever QSL publishes one. The
//! module keeps no table of its own, mints no identity (a role instance
//! identity exists only where an observation names it), supplies no default
//! for an absent bound, overflow policy or `outstanding` rule, and evaluates
//! nothing: a symbolic capacity stays its authored name, a `predicate` join
//! stays a reference to its function node, and a loop's invariant and variant
//! stay references to their checked nodes.

use qsl_package::protocol_view::{
    await_anchor, control_operation, delivery, queue_capacity, ControlOperation, Delivery,
    ProtocolView, QueueCapacity,
};
use qsl_package::{
    protocol_occurrence_schemas_from_read, ProtocolOccurrenceSchema, RoleReplication,
};
use qsl_semantics::check::wire_id::WireGraph;
use qsl_semantics::check::{ChannelOverflow, CheckedOutstanding, LossSelector};
use quire_contract_model::{CheckedNodeId, CheckedPackageV2, CheckedTerm, LiteralKind};
use quire_exact::NodeKey;

use crate::link::{argument, node, reference, LinkRefusal};

/// One authored role. The replication is QSL's own `RoleReplication`, read
/// through its occurrence schema: a static role is [`RoleReplication::Static`]
/// and a replicated role is one declaration with its population, `max` and
/// lifetime, never expanded into instances. QSL's `max` is required, so no
/// default is ever substituted for an absent one.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleDeclaration {
    /// The checked `role` node.
    pub node: CheckedNodeId,
    /// QSL's structural key of the node.
    pub key: NodeKey,
    /// Static, or replicated with population, maximum and lifetime.
    pub replication: RoleReplication,
}

impl RoleDeclaration {
    /// The authored `max` of a replicated role; `None` for a static role,
    /// which has no population bound. A maximum beyond `u64` saturates: no
    /// history counts that many instances.
    #[must_use]
    pub fn max(&self) -> Option<u64> {
        match &self.replication {
            RoleReplication::Static => None,
            RoleReplication::Replicated { max, .. } => Some(max.to_u64().unwrap_or(u64::MAX)),
        }
    }
}

/// A channel's authored ordering.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Ordering {
    /// `ordering fifo by (binder) { key }`: order is judged per key.
    Fifo {
        /// The message binder node.
        binder: CheckedNodeId,
        /// The key node.
        key: CheckedNodeId,
    },
    /// `ordering unordered`.
    Unordered,
}

/// A queue's authored capacity. A symbolic capacity is never resolved.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Capacity {
    /// A written integer capacity.
    Literal(u64),
    /// `symbolic(name)`, by its authored name.
    Symbolic(Box<str>),
}

/// One authored channel with its queue.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChannelDeclaration {
    /// The checked `channel` node.
    pub node: CheckedNodeId,
    /// The checked `queue` node of the channel.
    pub queue: CheckedNodeId,
    /// The delivery premise, as QSL's `delivery` accessor reads it.
    pub delivery: Delivery,
    /// The authored ordering.
    pub ordering: Ordering,
    /// The authored capacity, as QSL's `queue_capacity` accessor reads it.
    pub capacity: Capacity,
    /// The authored overflow policy; QSL's grammar requires one.
    pub overflow: ChannelOverflow,
}

/// A join's authored policy (QSpec FR-171).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JoinPolicy {
    /// `join all`.
    All,
    /// `join any`.
    Any,
    /// `join quorum(n)`.
    Quorum(u64),
    /// `join predicate(f)`, by the function node; never evaluated here.
    Predicate(CheckedNodeId),
}

/// One authored `parallel` join, keyed to the control identity the checked
/// package assigns.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JoinDeclaration {
    /// The `parallel` control node.
    pub control: CheckedNodeId,
    /// QSL's structural key of the control.
    pub key: NodeKey,
    /// The authored policy.
    pub policy: JoinPolicy,
    /// The labels of the joined branches, ascending as the package lists them.
    pub joined: Vec<Box<str>>,
    /// The authored `outstanding` rule, or `None` when the join has none.
    pub outstanding: Option<CheckedOutstanding>,
}

impl JoinDeclaration {
    /// The completed-branch count the policy requires, when it is a count:
    /// every joined branch for `all`, one for `any`, `n` for `quorum(n)`, and
    /// `None` for `predicate`, which is not evaluated.
    #[must_use]
    pub fn threshold(&self) -> Option<u64> {
        match &self.policy {
            JoinPolicy::All => u64::try_from(self.joined.len()).ok(),
            JoinPolicy::Any => Some(1),
            JoinPolicy::Quorum(count) => Some(*count),
            JoinPolicy::Predicate(_) => None,
        }
    }
}

/// One authored `repeat`, keyed to its control identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoopDeclaration {
    /// The `repeat` control node.
    pub control: CheckedNodeId,
    /// QSL's structural key of the control.
    pub key: NodeKey,
    /// The authored iteration maximum, or `None` when the loop has none.
    pub maximum: Option<u64>,
    /// The authored invariant, by its checked node. Not interpreted here.
    pub invariant: Option<CheckedNodeId>,
    /// The authored variant, by its checked node. Not interpreted here.
    pub variant: Option<CheckedNodeId>,
}

/// One authored `await`: the closed window `within [lower, upper]` that QSL
/// publishes on the checked node, in the units of the named clock and counted
/// from the anchor instant of the awaited `after` event. Never interpreted
/// here beyond its bounds.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AwaitDeclaration {
    /// The `await` control node.
    pub control: CheckedNodeId,
    /// QSL's structural key of the control.
    pub key: NodeKey,
    /// The authored lower bound of the window.
    pub lower: u64,
    /// The authored upper bound of the window, inclusive.
    pub upper: u64,
    /// The authored clock name.
    pub clock: Box<str>,
    /// The `event` control or `compensation` template the await follows
    /// (the target of QSL's published `after = {kind, target}`), read from
    /// QSL's checked await accessor.
    pub anchor: CheckedNodeId,
}

/// Every role, channel, join and loop declaration of one linked protocol, in
/// the protocol view's order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Declarations {
    roles: Vec<RoleDeclaration>,
    channels: Vec<ChannelDeclaration>,
    joins: Vec<JoinDeclaration>,
    loops: Vec<LoopDeclaration>,
    awaits: Vec<AwaitDeclaration>,
    schema: ProtocolOccurrenceSchema,
    clause: CheckedNodeId,
}

impl Declarations {
    /// QSL's occurrence schema of the clause: its roles, controls and repeat
    /// dimensions, which key every join and loop occurrence.
    #[must_use]
    pub fn schema(&self) -> &ProtocolOccurrenceSchema {
        &self.schema
    }

    /// The role declarations.
    #[must_use]
    pub fn roles(&self) -> &[RoleDeclaration] {
        &self.roles
    }

    /// The channel declarations.
    #[must_use]
    pub fn channels(&self) -> &[ChannelDeclaration] {
        &self.channels
    }

    /// The join declarations.
    #[must_use]
    pub fn joins(&self) -> &[JoinDeclaration] {
        &self.joins
    }

    /// The loop declarations.
    #[must_use]
    pub fn loops(&self) -> &[LoopDeclaration] {
        &self.loops
    }

    /// The `await` declarations.
    #[must_use]
    pub fn awaits(&self) -> &[AwaitDeclaration] {
        &self.awaits
    }

    /// The `await` declaration at control `id`.
    #[must_use]
    pub fn await_at(&self, id: &CheckedNodeId) -> Option<&AwaitDeclaration> {
        self.awaits
            .iter()
            .find(|declaration| &declaration.control == id)
    }

    /// The name of the checked package these declarations were read from.
    #[must_use]
    pub fn package(&self) -> &str {
        &self.clause.package
    }

    /// Whether `id` is the protocol clause or a node it declares: a role,
    /// channel, queue, join, loop or `await` control. A reference to any
    /// other node is not a reference into this checked package's protocol.
    #[must_use]
    pub fn declares(&self, id: &CheckedNodeId) -> bool {
        &self.clause == id
            || self.role(id).is_some()
            || self
                .channels
                .iter()
                .any(|channel| &channel.node == id || &channel.queue == id)
            || self.join(id).is_some()
            || self.loop_at(id).is_some()
            || self.await_at(id).is_some()
    }

    /// The role declaration at `id`.
    #[must_use]
    pub fn role(&self, id: &CheckedNodeId) -> Option<&RoleDeclaration> {
        self.roles.iter().find(|role| &role.node == id)
    }

    /// The channel declaration at `id`.
    #[must_use]
    pub fn channel(&self, id: &CheckedNodeId) -> Option<&ChannelDeclaration> {
        self.channels.iter().find(|channel| &channel.node == id)
    }

    /// The join declaration at control `id`.
    #[must_use]
    pub fn join(&self, id: &CheckedNodeId) -> Option<&JoinDeclaration> {
        self.joins.iter().find(|join| &join.control == id)
    }

    /// The loop declaration at control `id`.
    #[must_use]
    pub fn loop_at(&self, id: &CheckedNodeId) -> Option<&LoopDeclaration> {
        self.loops
            .iter()
            .find(|declaration| &declaration.control == id)
    }

    /// Read every declaration of `view` from `package`.
    pub(crate) fn read(
        package: &CheckedPackageV2,
        view: &ProtocolView<'_>,
    ) -> Result<Self, LinkRefusal> {
        let clause = view.clause();
        let unkeyed = |node: &CheckedNodeId| LinkRefusal::UnkeyedNode { node: node.clone() };
        let graph = WireGraph::new(package.resolved_graph().nodes.iter().map(|n| &n.node_id))
            .map_err(|_| unkeyed(clause))?;
        let key_of = |id: &CheckedNodeId| graph.key(id).map_err(|_| unkeyed(id));
        let clause_key = key_of(clause)?;
        let schema = protocol_occurrence_schemas_from_read(package)
            .map_err(LinkRefusal::Occurrence)?
            .into_iter()
            .map(|occurrence| occurrence.schema)
            .find(|schema| schema.workflow == clause_key)
            .ok_or_else(|| LinkRefusal::MissingMember {
                node: clause.clone(),
                member: "occurrence schema",
            })?;
        let mut declarations = Self {
            roles: Vec::new(),
            channels: Vec::new(),
            joins: Vec::new(),
            loops: Vec::new(),
            awaits: Vec::new(),
            schema,
            clause: clause.clone(),
        };
        let schema = declarations.schema.clone();
        for id in view.roles().iter().copied() {
            let key = key_of(id)?;
            let slot = schema
                .roles
                .iter()
                .find(|slot| slot.key == key)
                .ok_or_else(|| LinkRefusal::MissingMember {
                    node: id.clone(),
                    member: "role slot",
                })?;
            declarations.roles.push(RoleDeclaration {
                node: id.clone(),
                key,
                replication: slot.replication.clone(),
            });
        }
        for id in view.channels().iter().copied() {
            declarations.channels.push(channel(package, view, id)?);
        }
        for id in view.controls().iter().copied() {
            let control = node(package, id)
                .ok_or_else(|| LinkRefusal::UnreadableNode { node: id.clone() })?;
            match control_operation(control) {
                Ok(ControlOperation::Parallel) => {
                    declarations.joins.push(join(package, id, key_of(id)?)?);
                }
                Ok(ControlOperation::Repeat) => {
                    declarations.loops.push(repeat(package, id, key_of(id)?)?);
                }
                Ok(ControlOperation::Await) => {
                    declarations
                        .awaits
                        .push(await_control(package, id, key_of(id)?)?);
                }
                Ok(_) | Err(_) => {}
            }
        }
        Ok(declarations)
    }
}

fn member_of_node<'p>(
    package: &'p CheckedPackageV2,
    id: &CheckedNodeId,
    member: &'static str,
) -> Result<&'p CheckedTerm, LinkRefusal> {
    argument(package, id, member).ok_or_else(|| LinkRefusal::MissingMember {
        node: id.clone(),
        member,
    })
}

fn members(term: &CheckedTerm) -> Option<&[CheckedTerm]> {
    match term {
        CheckedTerm::Aggregate { members } => Some(members),
        _ => None,
    }
}

fn named<'t>(members: &'t [CheckedTerm], wanted: &str) -> Option<&'t CheckedTerm> {
    members.iter().find_map(|term| match term {
        CheckedTerm::Binding { name, value } if &**name == wanted => Some(value.as_ref()),
        _ => None,
    })
}

fn text(term: &CheckedTerm) -> Option<&str> {
    match term {
        CheckedTerm::Literal {
            value_kind: LiteralKind::Text,
            value,
            ..
        } => value.as_str(),
        _ => None,
    }
}

fn integer(term: &CheckedTerm) -> Option<u64> {
    match term {
        CheckedTerm::Literal {
            value_kind: LiteralKind::Integer,
            value,
            ..
        } => value
            .as_str()
            .filter(|digits| !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()))
            .and_then(|digits| digits.parse().ok()),
        _ => None,
    }
}

fn malformed(node: &CheckedNodeId, member: &'static str) -> LinkRefusal {
    LinkRefusal::MalformedMember {
        node: node.clone(),
        member,
    }
}

fn ordering(channel: &CheckedNodeId, group: &CheckedTerm) -> Result<Ordering, LinkRefusal> {
    let bad = || malformed(channel, "ordering");
    let group = members(group).ok_or_else(bad)?;
    match named(group, "kind").and_then(text) {
        Some("unordered") => Ok(Ordering::Unordered),
        Some("fifo") => Ok(Ordering::Fifo {
            binder: named(group, "binder")
                .and_then(reference)
                .ok_or_else(bad)?
                .clone(),
            key: named(group, "key")
                .and_then(reference)
                .ok_or_else(bad)?
                .clone(),
        }),
        _ => Err(bad()),
    }
}

fn channel(
    package: &CheckedPackageV2,
    view: &ProtocolView<'_>,
    id: &CheckedNodeId,
) -> Result<ChannelDeclaration, LinkRefusal> {
    let channel_node =
        node(package, id).ok_or_else(|| LinkRefusal::UnreadableNode { node: id.clone() })?;
    let ordering = ordering(id, member_of_node(package, id, "ordering")?)?;
    let delivery = delivery(channel_node).map_err(LinkRefusal::View)?;
    let queue = view
        .queues()
        .iter()
        .copied()
        .find(|queue| argument(package, queue, "channel").and_then(reference) == Some(id))
        .ok_or_else(|| LinkRefusal::MissingMember {
            node: id.clone(),
            member: "queue",
        })?;
    let queue_node = node(package, queue).ok_or_else(|| LinkRefusal::UnreadableNode {
        node: queue.clone(),
    })?;
    let capacity = match queue_capacity(queue_node).map_err(LinkRefusal::View)? {
        QueueCapacity::Literal(value) => Capacity::Literal(value),
        QueueCapacity::Symbolic(name) => Capacity::Symbolic(name.into()),
    };
    Ok(ChannelDeclaration {
        node: id.clone(),
        queue: queue.clone(),
        delivery,
        ordering,
        capacity,
        overflow: overflow(package, queue)?,
    })
}

/// The overflow policy a queue authors; QSL's grammar requires one.
fn overflow(
    package: &CheckedPackageV2,
    queue: &CheckedNodeId,
) -> Result<ChannelOverflow, LinkRefusal> {
    let bad = || malformed(queue, "overflow");
    let policy = member_of_node(package, queue, "overflow")?;
    match text(policy).ok_or_else(bad)? {
        "block" => Ok(ChannelOverflow::Block),
        "reject" => Ok(ChannelOverflow::Reject),
        "loss" => {
            let selector = argument(package, queue, "loss")
                .and_then(members)
                .and_then(|selector| match selector {
                    [only] => text(only),
                    _ => None,
                })
                .ok_or_else(bad)?;
            match selector {
                "head" => Ok(ChannelOverflow::Loss(LossSelector::Head)),
                "tail" => Ok(ChannelOverflow::Loss(LossSelector::Tail)),
                "new" => Ok(ChannelOverflow::Loss(LossSelector::New)),
                _ => Err(bad()),
            }
        }
        _ => Err(bad()),
    }
}

fn join(
    package: &CheckedPackageV2,
    id: &CheckedNodeId,
    key: NodeKey,
) -> Result<JoinDeclaration, LinkRefusal> {
    let bad = |member| malformed(id, member);
    let group = members(member_of_node(package, id, "join")?).ok_or_else(|| bad("join"))?;
    let policy = match named(group, "policy").and_then(text) {
        Some("all") => JoinPolicy::All,
        Some("any") => JoinPolicy::Any,
        Some("quorum") => JoinPolicy::Quorum(
            named(group, "count")
                .and_then(integer)
                .ok_or_else(|| bad("join"))?,
        ),
        Some("predicate") => JoinPolicy::Predicate(
            named(group, "predicate")
                .and_then(reference)
                .ok_or_else(|| bad("join"))?
                .clone(),
        ),
        _ => return Err(bad("join")),
    };
    let joined = members(member_of_node(package, id, "joined")?)
        .ok_or_else(|| bad("joined"))?
        .iter()
        .map(|label| text(label).map(Box::from).ok_or_else(|| bad("joined")))
        .collect::<Result<Vec<Box<str>>, _>>()?;
    let outstanding = match members(member_of_node(package, id, "outstanding")?)
        .ok_or_else(|| bad("outstanding"))?
    {
        [] => None,
        [rule] => match text(rule) {
            Some("continue") => Some(CheckedOutstanding::Continue),
            Some("cancel") => Some(CheckedOutstanding::Cancel),
            _ => return Err(bad("outstanding")),
        },
        _ => return Err(bad("outstanding")),
    };
    Ok(JoinDeclaration {
        control: id.clone(),
        key,
        policy,
        joined,
        outstanding,
    })
}

fn repeat(
    package: &CheckedPackageV2,
    id: &CheckedNodeId,
    key: NodeKey,
) -> Result<LoopDeclaration, LinkRefusal> {
    let bad = |member| malformed(id, member);
    let maximum =
        match members(member_of_node(package, id, "maximum")?).ok_or_else(|| bad("maximum"))? {
            [] => None,
            [count] => Some(integer(count).ok_or_else(|| bad("maximum"))?),
            _ => return Err(bad("maximum")),
        };
    let proof = members(member_of_node(package, id, "proof")?).ok_or_else(|| bad("proof"))?;
    let (invariant, variant) = if proof.is_empty() {
        (None, None)
    } else {
        let part = |name| {
            named(proof, name)
                .and_then(reference)
                .cloned()
                .ok_or_else(|| bad("proof"))
        };
        (Some(part("invariant")?), Some(part("variant")?))
    };
    Ok(LoopDeclaration {
        control: id.clone(),
        key,
        maximum,
        invariant,
        variant,
    })
}

fn await_control(
    package: &CheckedPackageV2,
    id: &CheckedNodeId,
    key: NodeKey,
) -> Result<AwaitDeclaration, LinkRefusal> {
    let bad = |member| malformed(id, member);
    let within = members(member_of_node(package, id, "within")?).ok_or_else(|| bad("within"))?;
    let bound = |name| {
        named(within, name)
            .and_then(integer)
            .ok_or_else(|| bad("within"))
    };
    let (lower, upper) = (bound("lower")?, bound("upper")?);
    let clock = text(member_of_node(package, id, "clock")?)
        .map(Box::from)
        .ok_or_else(|| bad("clock"))?;
    let checked =
        node(package, id).ok_or_else(|| LinkRefusal::UnreadableNode { node: id.clone() })?;
    let anchor = await_anchor(checked).map_err(LinkRefusal::View)?;
    Ok(AwaitDeclaration {
        control: id.clone(),
        key,
        lower,
        upper,
        clock,
        anchor: anchor.target.clone(),
    })
}
