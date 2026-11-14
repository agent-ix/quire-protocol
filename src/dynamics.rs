// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-053 to FR-056: one admitted observation history judged against the
//! authored role lifetimes and population bounds, queue capacities, overflow
//! policies, delivery premises and key orders, join settlements and loop
//! maxima of a linked protocol.
//!
//! This is an assessment of one history, not an exploration. QSL owns the step
//! semantics (ADR-027); nothing here steps a protocol, evaluates a `until`
//! predicate, a key expression or a join predicate, or infers a bound, an end
//! or an exit the history does not show. A violation is reported against the
//! exact declaration and never repairs the history (FR-027), and no
//! assessment reads or writes a closure or progress axis (FR-028).
//!
//! Every fact carries an `ObservationRef`: the observation's position in the
//! admitted history's own order, and its identity. The assessments sort facts
//! by that reference before reading them, so the order in which a caller
//! presents the facts changes no value (FR-026).
//!
//! The fact types are this crate's input to the assessment. They key every
//! subject by the identities of [`crate::ids`] and the checked node ids of
//! the linked protocol, and carry no value the library derives: a message's
//! `fifo` key is the exact bytes the admitted observation gives it.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use qsl_eval::simulation::protocol::Unsupported;
use qsl_package::protocol_view::Delivery;
use qsl_package::{
    OccurrenceKeyError, ProtocolOccurrenceSchema, RoleLifetime, RoleReplication,
    WorkflowInstanceIdentity,
};
use qsl_semantics::check::{ChannelOverflow, CheckedOutstanding, LossSelector};
use quire_contract_model::CheckedNodeId;
use quire_exact::{Integer, NodeKey};

use crate::declarations::{
    Capacity, ChannelDeclaration, Declarations, JoinPolicy, Ordering, RoleDeclaration,
};
use crate::disposition::{AssessmentUnsupported, Construct, UnsupportedCause};
use crate::ids::{MessageId, RoleInstanceId, WorkflowInstanceId};

/// Why a history cannot be assessed against a linked protocol. The history
/// names a subject the protocol does not declare; nothing is guessed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HistoryRefusal {
    /// A fact names a node the linked protocol does not declare as the kind
    /// of subject the fact is about.
    UndeclaredSubject {
        /// The node the fact named.
        node: CheckedNodeId,
    },
    /// A spawn, retire or end fact names a static role, which QSL never
    /// spawns: only its activity is observable.
    StaticRoleLifecycle {
        /// The static role.
        role: CheckedNodeId,
    },
    /// A join or loop fact's workflow identity is not text QSL can key an
    /// occurrence by (non-UTF-8).
    WorkflowNotText {
        /// The workflow identity bytes.
        workflow: Box<[u8]>,
    },
    /// QSL's occurrence key refused the fact's workflow, control or repeat
    /// ordinals.
    Occurrence(OccurrenceKeyError),
    /// A join fact names a branch label the join does not list.
    UndeclaredBranch {
        /// The join control.
        join: CheckedNodeId,
        /// The label the fact named.
        branch: Box<str>,
    },
}

/// One observation's place in the admitted history.
///
/// The derived order is the history's order: by position, then by identity,
/// so two observations at one position still order the same way every time.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ObservationRef {
    /// The observation's position in the admitted history's order.
    pub position: u64,
    /// The observation's identity bytes, as the producer gave them.
    pub identity: Box<[u8]>,
}

/// Validate a join or loop fact's occurrence against QSL's schema: the
/// control, its owner, the workflow identity and one repeat ordinal for each
/// enclosing `repeat` (QSL FR-384 "Runtime key binding"). Occurrences that
/// differ in any of these never merge.
pub(crate) fn bind_occurrence(
    declarations: &Declarations,
    control: &NodeKey,
    workflow: &WorkflowInstanceId,
    repeat_ordinals: &[Integer],
) -> Result<(), HistoryRefusal> {
    let schema: &ProtocolOccurrenceSchema = declarations.schema();
    let identity = std::str::from_utf8(workflow.identity_bytes()).map_err(|_| {
        HistoryRefusal::WorkflowNotText {
            workflow: workflow.identity_bytes().into(),
        }
    })?;
    let identity = WorkflowInstanceIdentity::new(identity).map_err(HistoryRefusal::Occurrence)?;
    let owner = schema
        .controls
        .iter()
        .find(|entry| &entry.key == control)
        .map(|entry| entry.owner.clone())
        .ok_or_else(|| {
            HistoryRefusal::Occurrence(OccurrenceKeyError::UnknownNode {
                node: control.clone(),
            })
        })?;
    schema
        .bind(identity, control, &owner, repeat_ordinals)
        .map(|_| ())
        .map_err(HistoryRefusal::Occurrence)
}

// ---------------------------------------------------------------------------
// FR-053: role lifetime and population
// ---------------------------------------------------------------------------

/// What one role instance did at one observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RoleEvent {
    /// The instance was spawned.
    Spawn,
    /// The instance took a step, including a compensation attempt of a
    /// template its role owns.
    Act,
    /// The instance was retired: the end of an `until(p)` lifetime.
    Retire,
    /// The observation that ends the workflow instance holding the role: the
    /// end of a `workflow` lifetime.
    WorkflowEnd,
    /// The step QSL ADR-027 RR-4 ends a `scope` lifetime by: no running thread
    /// can reach a node the role owns, the effect of a template it owns or an
    /// open registration of one.
    ScopeEnd,
}

/// One observed fact about a role instance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleFact {
    /// The observation.
    pub at: ObservationRef,
    /// The checked `role` node.
    pub role: CheckedNodeId,
    /// The workflow instance the role instance belongs to. A role's `max`
    /// bounds the live instances of one workflow instance, and a workflow
    /// end ends only that workflow's instances.
    pub workflow: WorkflowInstanceId,
    /// The instance identity the observation names. Never minted here.
    pub instance: RoleInstanceId,
    /// What the instance did.
    pub event: RoleEvent,
}

/// A lifetime violation, naming the instance and the observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LifetimeViolation {
    /// The instance acted before its observed spawn, or with none observed.
    BeforeSpawn {
        /// The instance.
        instance: RoleInstanceId,
        /// The observation that acted.
        at: ObservationRef,
    },
    /// The instance acted after the observed end of its lifetime.
    AfterEnd {
        /// The instance.
        instance: RoleInstanceId,
        /// The observation that acted.
        at: ObservationRef,
    },
    /// The instance was retired at or before the observation that spawned it.
    RetireBeforeSpawn {
        /// The instance.
        instance: RoleInstanceId,
        /// The retiring observation.
        at: ObservationRef,
    },
}

/// A population violation, naming the role, the bound and the observed count.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PopulationViolation {
    /// The checked `role` node.
    pub role: CheckedNodeId,
    /// The workflow instance whose live count exceeded the bound.
    pub workflow: WorkflowInstanceId,
    /// The authored bound.
    pub bound: u64,
    /// The greatest number of simultaneously live instances observed.
    pub observed: u64,
}

/// One role's assessment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleAssessment {
    /// The checked `role` node.
    pub role: CheckedNodeId,
    /// The workflow instance assessed: one assessment for each role and
    /// workflow instance the history names.
    pub workflow: WorkflowInstanceId,
    /// Each observed instance and whether all its observed activity lies
    /// inside its authored lifetime.
    pub instances: BTreeMap<RoleInstanceId, bool>,
    /// The greatest number of simultaneously live instances observed.
    pub max_live: u64,
    /// The authored population bound, or `None` when the role authors none.
    pub bound: Option<u64>,
    /// Whether `max_live` stayed within `bound`; `None` when there is no bound.
    pub within_bound: Option<bool>,
    /// Every lifetime violation, in the history's order.
    pub lifetime_violations: Vec<LifetimeViolation>,
    /// The population violation, when the bound was exceeded.
    pub population_violation: Option<PopulationViolation>,
}

/// Assess `facts` against the role declarations of `declarations`.
///
/// # Errors
///
/// [`HistoryRefusal::UndeclaredSubject`] when a fact names a node that is not
/// a declared role.
pub fn assess_roles(
    declarations: &Declarations,
    facts: &[RoleFact],
) -> Result<Vec<RoleAssessment>, HistoryRefusal> {
    let mut by_role: BTreeMap<(usize, &WorkflowInstanceId), Vec<&RoleFact>> = BTreeMap::new();
    for fact in facts {
        let index = declarations
            .roles()
            .iter()
            .position(|role| role.node == fact.role)
            .ok_or_else(|| HistoryRefusal::UndeclaredSubject {
                node: fact.role.clone(),
            })?;
        let is_static = matches!(
            declarations.roles()[index].replication,
            RoleReplication::Static
        );
        if is_static && fact.event != RoleEvent::Act {
            return Err(HistoryRefusal::StaticRoleLifecycle {
                role: fact.role.clone(),
            });
        }
        by_role
            .entry((index, &fact.workflow))
            .or_default()
            .push(fact);
    }
    Ok(by_role
        .into_iter()
        .map(|((index, workflow), mut role_facts)| {
            role_facts.sort_by(|left, right| left.at.cmp(&right.at));
            assess_role(&declarations.roles()[index], workflow, &role_facts)
        })
        .collect())
}

fn ends_lifetime(lifetime: Option<RoleLifetime>, event: &RoleEvent) -> bool {
    matches!(
        (lifetime, event),
        (Some(RoleLifetime::Workflow), RoleEvent::WorkflowEnd)
            | (Some(RoleLifetime::Scope), RoleEvent::ScopeEnd)
            | (Some(RoleLifetime::Until), RoleEvent::Retire)
    )
}

fn assess_role(
    role: &RoleDeclaration,
    workflow: &WorkflowInstanceId,
    facts: &[&RoleFact],
) -> RoleAssessment {
    let lifetime = match &role.replication {
        RoleReplication::Static => None,
        RoleReplication::Replicated { lifetime, .. } => Some(*lifetime),
    };
    let bound = role.max();
    let mut spawned: BTreeMap<&RoleInstanceId, &ObservationRef> = BTreeMap::new();
    let mut ended: BTreeMap<&RoleInstanceId, &ObservationRef> = BTreeMap::new();
    let mut instances: BTreeMap<RoleInstanceId, bool> = BTreeMap::new();
    let mut violations = Vec::new();
    let mut live: BTreeSet<&RoleInstanceId> = BTreeSet::new();
    let mut max_live = 0u64;
    for fact in facts {
        instances.entry(fact.instance.clone()).or_insert(true);
        let mut violated = None;
        match &fact.event {
            RoleEvent::Spawn => {
                spawned.entry(&fact.instance).or_insert(&fact.at);
                live.insert(&fact.instance);
                max_live = max_live.max(u64::try_from(live.len()).unwrap_or(u64::MAX));
            }
            // A static role is never spawned: its instance is the workflow's,
            // so its activity has no spawn to precede.
            RoleEvent::Act if lifetime.is_none() => {}
            RoleEvent::Act => {
                if !spawned.contains_key(&fact.instance) {
                    violated = Some(LifetimeViolation::BeforeSpawn {
                        instance: fact.instance.clone(),
                        at: fact.at.clone(),
                    });
                } else if ended.contains_key(&fact.instance) {
                    violated = Some(LifetimeViolation::AfterEnd {
                        instance: fact.instance.clone(),
                        at: fact.at.clone(),
                    });
                }
            }
            RoleEvent::Retire => {
                if !spawned.contains_key(&fact.instance) {
                    violated = Some(LifetimeViolation::RetireBeforeSpawn {
                        instance: fact.instance.clone(),
                        at: fact.at.clone(),
                    });
                }
            }
            RoleEvent::WorkflowEnd | RoleEvent::ScopeEnd => {}
        }
        if ends_lifetime(lifetime, &fact.event) {
            ended.entry(&fact.instance).or_insert(&fact.at);
            live.remove(&fact.instance);
        }
        if let Some(violation) = violated {
            instances.insert(fact.instance.clone(), false);
            violations.push(violation);
        }
    }
    let population_violation = bound.and_then(|bound| {
        (max_live > bound).then(|| PopulationViolation {
            role: role.node.clone(),
            workflow: workflow.clone(),
            bound,
            observed: max_live,
        })
    });
    RoleAssessment {
        role: role.node.clone(),
        workflow: workflow.clone(),
        instances,
        max_live,
        bound,
        within_bound: bound.map(|bound| max_live <= bound),
        lifetime_violations: violations,
        population_violation,
    }
}

// ---------------------------------------------------------------------------
// FR-054: queue capacity, overflow, order and delivery premise
// ---------------------------------------------------------------------------

/// What an overflow event observed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OverflowOutcome {
    /// The send blocked.
    Blocked,
    /// The send was rejected.
    Rejected,
    /// A message was lost: the head, the tail or the new message.
    Lost {
        /// Which message the loss took.
        selector: LossSelector,
        /// The message lost.
        dropped: MessageId,
    },
}

/// What happened to a message on a channel.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ChannelEvent {
    /// The message entered the queue. `key` is the exact bytes of the message's
    /// ordering key as the admitted observation gives them.
    Enqueue {
        /// The message.
        message: MessageId,
        /// Its ordering key bytes.
        key: Box<[u8]>,
    },
    /// The message was delivered and left the queue.
    Deliver {
        /// The message.
        message: MessageId,
    },
    /// The queue was full when `message` was sent, with this outcome.
    Overflow {
        /// The message whose send met the full queue.
        message: MessageId,
        /// The observed outcome.
        outcome: OverflowOutcome,
    },
    /// The message was lost outside an overflow.
    Lose {
        /// The message.
        message: MessageId,
    },
    /// The message was delivered more than once.
    Duplicate {
        /// The message.
        message: MessageId,
    },
}

/// One observed fact about a channel.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChannelFact {
    /// The observation.
    pub at: ObservationRef,
    /// The checked `channel` node.
    pub channel: CheckedNodeId,
    /// What happened.
    pub event: ChannelEvent,
}

/// A delivery behaviour a premise can forbid.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PremiseBreach {
    /// A message was lost.
    Loss,
    /// A message was delivered more than once.
    Duplication,
}

/// A queue violation, naming the channel and the authored and observed values.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueueViolation {
    /// Occupancy rose above the literal capacity with no overflow event.
    Capacity {
        /// The channel.
        channel: CheckedNodeId,
        /// The authored capacity.
        capacity: u64,
        /// The occupancy observed.
        occupancy: u64,
    },
    /// An overflow outcome differs from the authored policy.
    Overflow {
        /// The channel.
        channel: CheckedNodeId,
        /// The authored policy.
        authored: ChannelOverflow,
        /// The observed outcome.
        observed: OverflowOutcome,
    },
    /// A message of one key was delivered out of its send order.
    Order {
        /// The channel.
        channel: CheckedNodeId,
        /// The key.
        key: Box<[u8]>,
        /// The message that should have been delivered.
        expected: MessageId,
        /// The message delivered.
        delivered: MessageId,
    },
    /// A loss or duplication the delivery premise does not permit.
    Premise {
        /// The channel.
        channel: CheckedNodeId,
        /// The authored premise.
        premise: Delivery,
        /// What was observed.
        observed: PremiseBreach,
    },
}

/// A judged channel.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChannelAssessment {
    /// The checked `channel` node.
    pub channel: CheckedNodeId,
    /// The greatest occupancy observed.
    pub max_occupancy: u64,
    /// The authored literal capacity.
    pub capacity: u64,
    /// Whether the occupancy stayed within capacity (or each excess met an
    /// overflow event).
    pub within_capacity: bool,
    /// Every violation, in the history's order.
    pub violations: Vec<QueueViolation>,
}

/// One channel's result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ChannelOutcome {
    /// The channel was assessed.
    Assessed(ChannelAssessment),
    /// QSL would report an `Unsupported` cause for the channel (FR-057); no
    /// capacity, order or premise judgment is made.
    Unsupported {
        /// The channel.
        channel: CheckedNodeId,
        /// The cause, carried by name, and the construct.
        cause: UnsupportedCause,
        /// The channel node.
        construct: Construct,
    },
}

/// Assess `facts` against the channel declarations of `declarations`.
///
/// `construct` resolves a checked node to its [`Construct`] (node and source
/// locus) for an unsupported channel.
///
/// # Errors
///
/// [`HistoryRefusal::UndeclaredSubject`] when a fact names a node that is not
/// a declared channel.
pub fn assess_channels(
    declarations: &Declarations,
    facts: &[ChannelFact],
    construct: impl Fn(&CheckedNodeId) -> Construct,
) -> Result<Vec<ChannelOutcome>, HistoryRefusal> {
    let mut by_channel: BTreeMap<usize, Vec<&ChannelFact>> = BTreeMap::new();
    for fact in facts {
        let index = declarations
            .channels()
            .iter()
            .position(|channel| channel.node == fact.channel)
            .ok_or_else(|| HistoryRefusal::UndeclaredSubject {
                node: fact.channel.clone(),
            })?;
        by_channel.entry(index).or_default().push(fact);
    }
    let mut outcomes = Vec::new();
    for (index, channel) in declarations.channels().iter().enumerate() {
        let mut channel_facts = by_channel.remove(&index).unwrap_or_default();
        channel_facts.sort_by(|left, right| left.at.cmp(&right.at));
        outcomes.push(assess_channel(channel, &channel_facts, &construct));
    }
    Ok(outcomes)
}

/// The literal capacity QSL steps, or the `Unsupported` cause for a channel it
/// does not.
fn steppable_capacity(channel: &ChannelDeclaration) -> Result<u64, UnsupportedCause> {
    let Capacity::Literal(capacity) = channel.capacity else {
        return Err(UnsupportedCause::Construct(Unsupported::SymbolicCapacity));
    };
    let loses_ordered = matches!(
        channel.overflow,
        ChannelOverflow::Loss(LossSelector::Head | LossSelector::Tail)
    );
    if channel.ordering == Ordering::Unordered && loses_ordered {
        return Err(UnsupportedCause::Assessment(
            AssessmentUnsupported::UnorderedLossSelector,
        ));
    }
    Ok(capacity)
}

fn permits(premise: Delivery, breach: PremiseBreach) -> bool {
    match (premise, breach) {
        (Delivery::Unknown, _)
        | (Delivery::AtMostOnce, PremiseBreach::Loss)
        | (Delivery::AtLeastOnce, PremiseBreach::Duplication) => true,
        (Delivery::Between { lower, .. }, PremiseBreach::Loss) => lower == 0,
        (Delivery::Between { upper, .. }, PremiseBreach::Duplication) => upper > 1,
        (Delivery::AtMostOnce, PremiseBreach::Duplication)
        | (Delivery::AtLeastOnce, PremiseBreach::Loss)
        | (Delivery::ExactlyOncePremise, _) => false,
    }
}

fn assess_channel(
    channel: &ChannelDeclaration,
    facts: &[&ChannelFact],
    construct: &impl Fn(&CheckedNodeId) -> Construct,
) -> ChannelOutcome {
    let capacity = match steppable_capacity(channel) {
        Ok(capacity) => capacity,
        Err(cause) => {
            return ChannelOutcome::Unsupported {
                channel: channel.node.clone(),
                cause,
                construct: construct(&channel.node),
            }
        }
    };
    let overflowed: BTreeSet<&MessageId> = facts
        .iter()
        .filter_map(|fact| match &fact.event {
            ChannelEvent::Overflow { message, .. } => Some(message),
            _ => None,
        })
        .collect();
    let mut queued: Vec<(&MessageId, &[u8])> = Vec::new();
    let mut per_key: BTreeMap<&[u8], VecDeque<&MessageId>> = BTreeMap::new();
    let mut violations = Vec::new();
    let mut max_occupancy = 0u64;
    let mut within_capacity = true;
    let violation_channel = || channel.node.clone();
    for fact in facts {
        match &fact.event {
            ChannelEvent::Enqueue { message, key } => {
                queued.push((message, key));
                per_key.entry(key).or_default().push_back(message);
                let occupancy = u64::try_from(queued.len()).unwrap_or(u64::MAX);
                max_occupancy = max_occupancy.max(occupancy);
                if occupancy > capacity && !overflowed.contains(message) {
                    within_capacity = false;
                    violations.push(QueueViolation::Capacity {
                        channel: violation_channel(),
                        capacity,
                        occupancy,
                    });
                }
            }
            ChannelEvent::Deliver { message } => {
                if let Some(position) = queued.iter().position(|(queued, _)| *queued == message) {
                    let (_, key) = queued.remove(position);
                    if let (Ordering::Fifo { .. }, Some(line)) =
                        (&channel.ordering, per_key.get_mut(key))
                    {
                        if let Some(&expected) = line.front() {
                            if expected != message {
                                violations.push(QueueViolation::Order {
                                    channel: violation_channel(),
                                    key: key.into(),
                                    expected: expected.clone(),
                                    delivered: message.clone(),
                                });
                            }
                        }
                    }
                    if let Some(line) = per_key.get_mut(key) {
                        line.retain(|queued| *queued != message);
                    }
                }
            }
            ChannelEvent::Overflow { message, outcome } => {
                // The queue as the arriving message found it.
                let others: Vec<&MessageId> = queued
                    .iter()
                    .map(|(queued, _)| *queued)
                    .filter(|queued| *queued != message)
                    .collect();
                // A loss is judged against the queue, not the label: head
                // drops the oldest queued message, tail the newest, new the
                // arriving one.
                let observed_matches_queue = match outcome {
                    OverflowOutcome::Blocked | OverflowOutcome::Rejected => true,
                    OverflowOutcome::Lost { selector, dropped } => match selector {
                        LossSelector::Head => others.first().copied() == Some(dropped),
                        LossSelector::Tail => others.last().copied() == Some(dropped),
                        LossSelector::New => dropped == message,
                    },
                };
                if let OverflowOutcome::Lost { dropped, .. } = outcome {
                    if let Some(position) = queued.iter().position(|(queued, _)| *queued == dropped)
                    {
                        let (_, key) = queued.remove(position);
                        if let Some(line) = per_key.get_mut(key) {
                            line.retain(|queued| *queued != dropped);
                        }
                    }
                }
                if !(observed_matches_queue && matches_policy(channel.overflow, outcome)) {
                    violations.push(QueueViolation::Overflow {
                        channel: violation_channel(),
                        authored: channel.overflow,
                        observed: outcome.clone(),
                    });
                }
            }
            ChannelEvent::Lose { message } => {
                if let Some(position) = queued.iter().position(|(queued, _)| *queued == message) {
                    let (_, key) = queued.remove(position);
                    if let Some(line) = per_key.get_mut(key) {
                        line.retain(|queued| *queued != message);
                    }
                }
                if !permits(channel.delivery, PremiseBreach::Loss) {
                    violations.push(QueueViolation::Premise {
                        channel: violation_channel(),
                        premise: channel.delivery,
                        observed: PremiseBreach::Loss,
                    });
                }
            }
            ChannelEvent::Duplicate { .. } => {
                if !permits(channel.delivery, PremiseBreach::Duplication) {
                    violations.push(QueueViolation::Premise {
                        channel: violation_channel(),
                        premise: channel.delivery,
                        observed: PremiseBreach::Duplication,
                    });
                }
            }
        }
    }
    ChannelOutcome::Assessed(ChannelAssessment {
        channel: channel.node.clone(),
        max_occupancy,
        capacity,
        within_capacity,
        violations,
    })
}

fn matches_policy(authored: ChannelOverflow, observed: &OverflowOutcome) -> bool {
    match (authored, observed) {
        (ChannelOverflow::Block, OverflowOutcome::Blocked)
        | (ChannelOverflow::Reject, OverflowOutcome::Rejected) => true,
        (ChannelOverflow::Loss(authored), OverflowOutcome::Lost { selector, .. }) => {
            authored == *selector
        }
        (ChannelOverflow::Block | ChannelOverflow::Reject | ChannelOverflow::Loss(_), _) => false,
    }
}

// ---------------------------------------------------------------------------
// FR-055: join settlement
// ---------------------------------------------------------------------------

/// What an outstanding branch did after its join settled.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BranchOutcome {
    /// It kept running.
    Continued,
    /// It was cancelled.
    Cancelled,
}

/// What was observed of one join occurrence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JoinEvent {
    /// The branch with this label completed. Repeated observations of one
    /// label count once.
    BranchCompleted {
        /// The joined branch label.
        branch: Box<str>,
    },
    /// The join settled.
    Settled,
    /// What an outstanding branch did.
    Outstanding {
        /// The joined branch label.
        branch: Box<str>,
        /// What it did.
        outcome: BranchOutcome,
    },
}

/// One observed fact about a join occurrence: the join control in one
/// workflow instance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JoinFact {
    /// The observation.
    pub at: ObservationRef,
    /// The `parallel` control node.
    pub join: CheckedNodeId,
    /// The workflow instance holding the occurrence.
    pub workflow: WorkflowInstanceId,
    /// One ordinal for each enclosing `repeat`, outermost first, as QSL's
    /// occurrence key binds them.
    pub repeat_ordinals: Vec<Integer>,
    /// What was observed.
    pub event: JoinEvent,
}

/// A join violation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JoinViolation {
    /// The join settled before its threshold was observed.
    Settlement {
        /// The join control.
        join: CheckedNodeId,
        /// The authored policy.
        policy: JoinPolicy,
        /// The authored threshold.
        threshold: u64,
        /// The distinct branches completed when it settled.
        observed: u64,
    },
    /// An outstanding branch behaved other than the authored rule says.
    Outstanding {
        /// The join control.
        join: CheckedNodeId,
        /// The branch label.
        branch: Box<str>,
        /// The authored rule.
        authored: CheckedOutstanding,
        /// What the branch did.
        observed: BranchOutcome,
    },
}

/// One assessed join occurrence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JoinAssessment {
    /// The join control.
    pub join: CheckedNodeId,
    /// The workflow instance.
    pub workflow: WorkflowInstanceId,
    /// The enclosing `repeat` ordinals of the occurrence.
    pub repeat_ordinals: Vec<Integer>,
    /// The distinct branches observed complete before the join settled, or by
    /// the end of the history when it did not settle.
    pub completed: BTreeSet<Box<str>>,
    /// Whether the join settled.
    pub settled: bool,
    /// Whether the observed settlement is permitted by the authored policy
    /// and threshold; `None` when the join did not settle.
    pub permitted: Option<bool>,
    /// The observed outcome of each outstanding branch: `None` when the
    /// history shows none.
    pub outstanding: BTreeMap<Box<str>, Option<BranchOutcome>>,
    /// Every violation.
    pub violations: Vec<JoinViolation>,
}

/// One join occurrence's result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JoinOutcome {
    /// The occurrence was assessed.
    Assessed(JoinAssessment),
    /// QSL would report an `Unsupported` cause for the join (FR-057); the
    /// occurrence is reported neither settled nor unsettled.
    Unsupported {
        /// The workflow instance of the occurrence.
        workflow: WorkflowInstanceId,
        /// The enclosing `repeat` ordinals of the occurrence.
        repeat_ordinals: Vec<Integer>,
        /// The cause, as QSL reports it.
        cause: UnsupportedCause,
        /// The join control and its locus.
        construct: Construct,
    },
}

/// Assess `facts` against the join declarations of `declarations`.
///
/// A join declaration with no fact yields no occurrence. A join QSL would
/// report `Unsupported` for yields one [`JoinOutcome::Unsupported`] for each
/// occurrence (workflow instance and enclosing repeat ordinals) the history
/// names.
///
/// # Errors
///
/// [`HistoryRefusal::UndeclaredSubject`] for a fact naming no declared join;
/// [`HistoryRefusal::UndeclaredBranch`] for a branch label the join does not
/// list.
pub fn assess_joins(
    declarations: &Declarations,
    facts: &[JoinFact],
    construct: impl Fn(&CheckedNodeId) -> Construct,
) -> Result<Vec<JoinOutcome>, HistoryRefusal> {
    // Ordinals key by their decimal text: `Integer` is not a sound map key.
    type Key<'a> = (usize, &'a WorkflowInstanceId, Vec<String>);
    let mut occurrences: BTreeMap<Key<'_>, Vec<&JoinFact>> = BTreeMap::new();
    for fact in facts {
        let index = declarations
            .joins()
            .iter()
            .position(|join| join.control == fact.join)
            .ok_or_else(|| HistoryRefusal::UndeclaredSubject {
                node: fact.join.clone(),
            })?;
        let declaration = &declarations.joins()[index];
        let branch = match &fact.event {
            JoinEvent::BranchCompleted { branch } | JoinEvent::Outstanding { branch, .. } => {
                Some(branch)
            }
            JoinEvent::Settled => None,
        };
        if let Some(branch) = branch {
            if !declaration.joined.contains(branch) {
                return Err(HistoryRefusal::UndeclaredBranch {
                    join: fact.join.clone(),
                    branch: branch.clone(),
                });
            }
        }
        bind_occurrence(
            declarations,
            &declaration.key,
            &fact.workflow,
            &fact.repeat_ordinals,
        )?;
        occurrences
            .entry((
                index,
                &fact.workflow,
                fact.repeat_ordinals
                    .iter()
                    .map(ToString::to_string)
                    .collect(),
            ))
            .or_default()
            .push(fact);
    }
    let mut outcomes = Vec::new();
    for ((index, workflow, _), mut occurrence) in occurrences {
        let ordinals = &occurrence[0].repeat_ordinals;
        let declaration = &declarations.joins()[index];
        if matches!(declaration.policy, JoinPolicy::Predicate(_)) {
            outcomes.push(JoinOutcome::Unsupported {
                workflow: workflow.clone(),
                repeat_ordinals: ordinals.clone(),
                cause: UnsupportedCause::Assessment(AssessmentUnsupported::JoinPredicate),
                construct: construct(&declaration.control),
            });
            continue;
        }
        occurrence.sort_by(|left, right| left.at.cmp(&right.at));
        outcomes.push(JoinOutcome::Assessed(assess_join(
            declaration,
            workflow,
            ordinals,
            &occurrence,
        )));
    }
    Ok(outcomes)
}

fn assess_join(
    declaration: &crate::declarations::JoinDeclaration,
    workflow: &WorkflowInstanceId,
    ordinals: &[Integer],
    facts: &[&JoinFact],
) -> JoinAssessment {
    let mut completed: BTreeSet<Box<str>> = BTreeSet::new();
    let mut settled_with: Option<BTreeSet<Box<str>>> = None;
    let mut outcomes: BTreeMap<Box<str>, BranchOutcome> = BTreeMap::new();
    for fact in facts {
        match &fact.event {
            JoinEvent::BranchCompleted { branch } if settled_with.is_none() => {
                completed.insert(branch.clone());
            }
            JoinEvent::BranchCompleted { .. } => {}
            JoinEvent::Settled => {
                settled_with.get_or_insert_with(|| completed.clone());
            }
            JoinEvent::Outstanding { branch, outcome } => {
                outcomes.entry(branch.clone()).or_insert(*outcome);
            }
        }
    }
    let mut violations = Vec::new();
    let permitted = settled_with.as_ref().map(|at_settlement| {
        let observed = u64::try_from(at_settlement.len()).unwrap_or(u64::MAX);
        let ok = declaration
            .threshold()
            .is_some_and(|threshold| observed >= threshold);
        if let (false, Some(threshold)) = (ok, declaration.threshold()) {
            violations.push(JoinViolation::Settlement {
                join: declaration.control.clone(),
                policy: declaration.policy.clone(),
                threshold,
                observed,
            });
        }
        ok
    });
    let mut outstanding = BTreeMap::new();
    if let Some(at_settlement) = &settled_with {
        for label in &declaration.joined {
            if at_settlement.contains(label) {
                continue;
            }
            let observed = outcomes.get(label).copied();
            if let (Some(authored), Some(observed)) = (declaration.outstanding, observed) {
                let expected = match authored {
                    CheckedOutstanding::Continue => BranchOutcome::Continued,
                    CheckedOutstanding::Cancel => BranchOutcome::Cancelled,
                };
                if observed != expected {
                    violations.push(JoinViolation::Outstanding {
                        join: declaration.control.clone(),
                        branch: label.clone(),
                        authored,
                        observed,
                    });
                }
            }
            outstanding.insert(label.clone(), observed);
        }
    }
    JoinAssessment {
        join: declaration.control.clone(),
        workflow: workflow.clone(),
        repeat_ordinals: ordinals.to_vec(),
        completed,
        settled: settled_with.is_some(),
        permitted,
        outstanding,
        violations,
    }
}

// ---------------------------------------------------------------------------
// FR-056: loop iteration bound
// ---------------------------------------------------------------------------

/// What was observed of one loop occurrence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoopEvent {
    /// The loop body began an iteration.
    Iteration,
    /// The loop was left.
    Exit,
}

/// One observed fact about a loop occurrence: the `repeat` control in one
/// workflow instance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoopFact {
    /// The observation.
    pub at: ObservationRef,
    /// The `repeat` control node.
    pub control: CheckedNodeId,
    /// The workflow instance holding the occurrence.
    pub workflow: WorkflowInstanceId,
    /// One ordinal for each enclosing `repeat`, outermost first, as QSL's
    /// occurrence key binds them.
    pub repeat_ordinals: Vec<Integer>,
    /// What was observed.
    pub event: LoopEvent,
}

/// A loop-bound violation, naming the loop, the maximum and the count.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoopBoundViolation {
    /// The `repeat` control.
    pub control: CheckedNodeId,
    /// The authored maximum.
    pub maximum: u64,
    /// The iterations observed.
    pub observed: u64,
}

/// One assessed loop occurrence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoopAssessment {
    /// The `repeat` control.
    pub control: CheckedNodeId,
    /// The workflow instance.
    pub workflow: WorkflowInstanceId,
    /// The enclosing `repeat` ordinals of the occurrence.
    pub repeat_ordinals: Vec<Integer>,
    /// The iterations observed.
    pub iterations: u64,
    /// The authored maximum, or `None` when the loop authors none.
    pub maximum: Option<u64>,
    /// Whether the count stayed within the maximum; `None` with no maximum.
    pub within_maximum: Option<bool>,
    /// Whether the history shows the loop left. A history that ends inside
    /// the loop is not exited; an exit is never inferred.
    pub exited: bool,
    /// The bound violation, when the maximum was exceeded.
    pub violation: Option<LoopBoundViolation>,
}

/// Assess `facts` against the loop declarations of `declarations`.
///
/// The result reads no closure or progress axis, and reports no invariant,
/// variant, rank or termination verdict (FR-059).
///
/// # Errors
///
/// [`HistoryRefusal::UndeclaredSubject`] when a fact names no declared loop.
pub fn assess_loops(
    declarations: &Declarations,
    facts: &[LoopFact],
) -> Result<Vec<LoopAssessment>, HistoryRefusal> {
    // Ordinals key by their decimal text: `Integer` is not a sound map key.
    type Key<'a> = (usize, &'a WorkflowInstanceId, Vec<String>);
    let mut occurrences: BTreeMap<Key<'_>, Vec<&LoopFact>> = BTreeMap::new();
    for fact in facts {
        let index = declarations
            .loops()
            .iter()
            .position(|declaration| declaration.control == fact.control)
            .ok_or_else(|| HistoryRefusal::UndeclaredSubject {
                node: fact.control.clone(),
            })?;
        bind_occurrence(
            declarations,
            &declarations.loops()[index].key,
            &fact.workflow,
            &fact.repeat_ordinals,
        )?;
        occurrences
            .entry((
                index,
                &fact.workflow,
                fact.repeat_ordinals
                    .iter()
                    .map(ToString::to_string)
                    .collect(),
            ))
            .or_default()
            .push(fact);
    }
    Ok(occurrences
        .into_iter()
        .map(|((index, workflow, _), occurrence)| {
            let ordinals = &occurrence[0].repeat_ordinals;
            let declaration = &declarations.loops()[index];
            let iterations = u64::try_from(
                occurrence
                    .iter()
                    .filter(|fact| fact.event == LoopEvent::Iteration)
                    .count(),
            )
            .unwrap_or(u64::MAX);
            let violation = declaration.maximum.and_then(|maximum| {
                (iterations > maximum).then(|| LoopBoundViolation {
                    control: declaration.control.clone(),
                    maximum,
                    observed: iterations,
                })
            });
            LoopAssessment {
                control: declaration.control.clone(),
                workflow: workflow.clone(),
                repeat_ordinals: ordinals.to_vec(),
                iterations,
                maximum: declaration.maximum,
                within_maximum: declaration.maximum.map(|maximum| iterations <= maximum),
                exited: occurrence.iter().any(|fact| fact.event == LoopEvent::Exit),
                violation,
            }
        })
        .collect())
}
