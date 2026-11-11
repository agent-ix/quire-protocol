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
    control_operation, delivery, queue_capacity, ControlOperation, Delivery, ProtocolView,
    QueueCapacity,
};
use qsl_semantics::check::{ChannelOverflow, CheckedOutstanding, LossSelector};
use quire_contract_model::{CheckedNodeId, CheckedPackageV2, CheckedTerm, LiteralKind};

use crate::link::{argument, node, reference, LinkRefusal};

/// A replicated role's lifetime (QSpec FR-465 "Role"). The `until` predicate
/// is the checked node the role names; it is never evaluated here.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Lifetime {
    /// `lifetime workflow`.
    Workflow,
    /// `lifetime scope`.
    Scope,
    /// `lifetime until(p)`, by the checked node holding `p`.
    Until(CheckedNodeId),
}

/// What a replicated role declares beyond its model.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Replication {
    /// The population the instances are drawn from, as authored.
    pub population: Box<str>,
    /// The authored `max` live-instance bound, or `None` when the checked
    /// node carries none. No default is substituted.
    pub max: Option<u64>,
    /// The binder node the lifetime predicate reads.
    pub binder: CheckedNodeId,
    /// The authored lifetime.
    pub lifetime: Lifetime,
}

/// One authored role. A static role has no [`Replication`]; a replicated role
/// is one declaration, never expanded into instances.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleDeclaration {
    /// The checked `role` node.
    pub node: CheckedNodeId,
    /// The replication group, absent for a static role.
    pub replication: Option<Replication>,
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
    /// The authored overflow policy, or `None` when the queue declares none.
    pub overflow: Option<ChannelOverflow>,
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
    /// The authored iteration maximum, or `None` when the loop has none.
    pub maximum: Option<u64>,
    /// The authored invariant, by its checked node. Not interpreted here.
    pub invariant: Option<CheckedNodeId>,
    /// The authored variant, by its checked node. Not interpreted here.
    pub variant: Option<CheckedNodeId>,
}

/// Every role, channel, join and loop declaration of one linked protocol, in
/// the protocol view's order.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Declarations {
    roles: Vec<RoleDeclaration>,
    channels: Vec<ChannelDeclaration>,
    joins: Vec<JoinDeclaration>,
    loops: Vec<LoopDeclaration>,
}

impl Declarations {
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
        let mut declarations = Self::default();
        for id in view.roles().iter().copied() {
            let group = member_of_node(package, id, "replication")?;
            declarations.roles.push(RoleDeclaration {
                node: id.clone(),
                replication: replication(id, group)?,
            });
        }
        for id in view.channels().iter().copied() {
            declarations.channels.push(channel(package, view, id)?);
        }
        for id in view.controls().iter().copied() {
            let control = node(package, id)
                .ok_or_else(|| LinkRefusal::UnreadableNode { node: id.clone() })?;
            match control_operation(control) {
                Ok(ControlOperation::Parallel) => declarations.joins.push(join(package, id)?),
                Ok(ControlOperation::Repeat) => declarations.loops.push(repeat(package, id)?),
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

/// Read a role's `replication` group. An empty group is a static role.
fn replication(
    role: &CheckedNodeId,
    group: &CheckedTerm,
) -> Result<Option<Replication>, LinkRefusal> {
    let group = members(group).ok_or_else(|| malformed(role, "replication"))?;
    if group.is_empty() {
        return Ok(None);
    }
    let bad = || malformed(role, "replication");
    let population = named(group, "population").and_then(text).ok_or_else(bad)?;
    let max = named(group, "max")
        .map(|term| integer(term).ok_or_else(bad))
        .transpose()?;
    let binder = named(group, "binder")
        .and_then(reference)
        .ok_or_else(bad)?
        .clone();
    let lifetime = match named(group, "lifetime").and_then(text) {
        Some("workflow") => Lifetime::Workflow,
        Some("scope") => Lifetime::Scope,
        Some("until") => Lifetime::Until(
            named(group, "until")
                .and_then(reference)
                .ok_or_else(bad)?
                .clone(),
        ),
        _ => return Err(bad()),
    };
    Ok(Some(Replication {
        population: population.into(),
        max,
        binder,
        lifetime,
    }))
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

/// The overflow policy a queue authors, `None` when it authors none.
fn overflow(
    package: &CheckedPackageV2,
    queue: &CheckedNodeId,
) -> Result<Option<ChannelOverflow>, LinkRefusal> {
    let bad = || malformed(queue, "overflow");
    let Some(policy) = argument(package, queue, "overflow") else {
        return Ok(None);
    };
    match text(policy).ok_or_else(bad)? {
        "block" => Ok(Some(ChannelOverflow::Block)),
        "reject" => Ok(Some(ChannelOverflow::Reject)),
        "loss" => {
            let selector = argument(package, queue, "loss")
                .and_then(members)
                .and_then(|selector| match selector {
                    [only] => text(only),
                    _ => None,
                })
                .ok_or_else(bad)?;
            match selector {
                "head" => Ok(Some(ChannelOverflow::Loss(LossSelector::Head))),
                "tail" => Ok(Some(ChannelOverflow::Loss(LossSelector::Tail))),
                "new" => Ok(Some(ChannelOverflow::Loss(LossSelector::New))),
                _ => Err(bad()),
            }
        }
        _ => Err(bad()),
    }
}

fn join(package: &CheckedPackageV2, id: &CheckedNodeId) -> Result<JoinDeclaration, LinkRefusal> {
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
        policy,
        joined,
        outstanding,
    })
}

fn repeat(package: &CheckedPackageV2, id: &CheckedNodeId) -> Result<LoopDeclaration, LinkRefusal> {
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
        maximum,
        invariant,
        variant,
    })
}
