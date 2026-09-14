// SPDX-License-Identifier: AGPL-3.0-or-later
//! Bounded registration, authored causal ordering, and finite retry algebra.

#![deny(clippy::wildcard_enum_match_arm)]

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::num::NonZeroU32;

use crate::ids::{
    AttemptId, CommitId, DefinitionId, DeliveryId, EffectId, ReceiveId, RegistrationId, SendId,
};
use crate::ByteDigest;

use super::purity::PureAssessment;

const REGISTRATION_IDENTITY_DOMAIN: &[u8] = b"quire.protocol.compensation-registration/v1";

/// Immutable owner maxima. A caller can lower but never raise them.
pub const OWNER_MAX: Limits = Limits {
    max_definitions: 10_000,
    max_registrations: 10_000,
    max_order_nodes: 40_000,
    max_order_edges: 100_000,
    max_attempts: 10_000,
    max_identity_bytes: 64 * 1024,
};

/// Independent ceilings for compensation algebra work.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Limits {
    /// Compensation definitions inspected for one established effect.
    pub max_definitions: usize,
    /// Registrations retained by one registry.
    pub max_registrations: usize,
    /// Milestones retained in one authored causal order.
    pub max_order_nodes: usize,
    /// Edges retained in one authored causal order.
    pub max_order_edges: usize,
    /// Attempts retained for one registration.
    pub max_attempts: usize,
    /// Bytes in any one offered protocol identity.
    pub max_identity_bytes: usize,
}

impl Limits {
    /// Immutable owner maxima.
    #[must_use]
    pub const fn owner_max() -> Self {
        OWNER_MAX
    }

    /// Clamp caller ceilings to immutable owner maxima.
    #[must_use]
    pub const fn effective(self) -> Self {
        Self {
            max_definitions: min(self.max_definitions, OWNER_MAX.max_definitions),
            max_registrations: min(self.max_registrations, OWNER_MAX.max_registrations),
            max_order_nodes: min(self.max_order_nodes, OWNER_MAX.max_order_nodes),
            max_order_edges: min(self.max_order_edges, OWNER_MAX.max_order_edges),
            max_attempts: min(self.max_attempts, OWNER_MAX.max_attempts),
            max_identity_bytes: min(self.max_identity_bytes, OWNER_MAX.max_identity_bytes),
        }
    }
}

impl Default for Limits {
    fn default() -> Self {
        OWNER_MAX
    }
}

const fn min(left: usize, right: usize) -> usize {
    if left < right {
        left
    } else {
        right
    }
}

/// Resource dimension refused before the next retained allocation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Resource {
    Definitions,
    Registrations,
    OrderNodes,
    OrderEdges,
    Attempts,
    IdentityBytes,
}

/// One exact bounded-work refusal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceRefusal {
    resource: Resource,
    observed: usize,
    limit: usize,
}

impl ResourceRefusal {
    fn new(resource: Resource, observed: usize, limit: usize) -> Self {
        Self {
            resource,
            observed,
            limit,
        }
    }

    /// Refused resource dimension.
    #[must_use]
    pub const fn resource(self) -> Resource {
        self.resource
    }

    /// Candidate usage that would have been retained.
    #[must_use]
    pub const fn observed(self) -> usize {
        self.observed
    }

    /// Effective inclusive ceiling.
    #[must_use]
    pub const fn limit(self) -> usize {
        self.limit
    }
}

impl core::fmt::Display for ResourceRefusal {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "compensation {:?} usage {} exceeds effective limit {}",
            self.resource, self.observed, self.limit
        )
    }
}

impl std::error::Error for ResourceRefusal {}

/// One authored compensation definition paired to a forward effect.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Definition {
    /// Exact source compensation definition.
    pub identity: DefinitionId,
    /// Exact forward effect that establishes this registration.
    pub paired_forward_effect: EffectId,
    /// Authored finite bound counting the initial attempt, or explicit absence.
    pub retry_bound: Option<NonZeroU32>,
}

/// One derived registration retaining its complete key and retry policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Registration {
    /// Derived registration identity.
    pub identity: RegistrationId,
    /// Exact source definition.
    pub definition: DefinitionId,
    /// Exact established forward effect.
    pub forward_effect: EffectId,
    /// Exact authored finite retry bound, or absence.
    pub retry_bound: Option<NonZeroU32>,
}

impl Registration {
    fn derive(definition: &Definition) -> Self {
        let mut preimage = Vec::new();
        push_part(&mut preimage, REGISTRATION_IDENTITY_DOMAIN);
        push_part(&mut preimage, definition.identity.identity_bytes());
        push_part(
            &mut preimage,
            definition.paired_forward_effect.identity_bytes(),
        );
        let identity = format!("sha256:{:x}", ByteDigest::of(&preimage));
        Self {
            identity: RegistrationId::new(identity.as_bytes(), "derived compensation registration"),
            definition: definition.identity.clone(),
            forward_effect: definition.paired_forward_effect.clone(),
            retry_bound: definition.retry_bound,
        }
    }
}

/// One admitted forward-side fact. Only `EstablishedEffect` can register.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ForwardFact {
    /// Successful send, not a business effect.
    Send(SendId),
    /// Successful receive, not a business effect.
    Receive(ReceiveId),
    /// Delivery, including a duplicate delivery, not a business effect.
    Delivery(DeliveryId),
    /// Operation attempt, not a business effect.
    Attempt(AttemptId),
    /// Independently established business effect.
    EstablishedEffect(EffectId),
}

impl ForwardFact {
    fn identity_bytes(&self) -> &[u8] {
        match self {
            Self::Send(identity) => identity.identity_bytes(),
            Self::Receive(identity) => identity.identity_bytes(),
            Self::Delivery(identity) => identity.identity_bytes(),
            Self::Attempt(identity) => identity.identity_bytes(),
            Self::EstablishedEffect(identity) => identity.identity_bytes(),
        }
    }
}

/// Registration transition caused by one observed fact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegistrationEvent {
    /// Exact derived registration.
    pub registration: Registration,
    /// `true` only for the first observation of this exact registration key.
    pub created: bool,
}

/// Replay-idempotent registration store keyed by definition and effect.
#[derive(Clone, Debug)]
pub struct Registry {
    limits: Limits,
    registrations: BTreeMap<(DefinitionId, EffectId), Registration>,
}

impl Registry {
    /// Create an empty registry under effective caller ceilings.
    #[must_use]
    pub fn new(limits: Limits) -> Self {
        Self {
            limits: limits.effective(),
            registrations: BTreeMap::new(),
        }
    }

    /// Observe one fact and derive only definitions paired to an established effect.
    ///
    /// All counts and identity sizes are checked before a registration is cloned
    /// or inserted. A refusal leaves the registry unchanged.
    pub fn observe(
        &mut self,
        _pure: PureAssessment,
        definitions: &[Definition],
        fact: &ForwardFact,
    ) -> Result<Vec<RegistrationEvent>, ResourceRefusal> {
        precheck(
            Resource::Definitions,
            definitions.len(),
            self.limits.max_definitions,
        )?;
        for definition in definitions {
            precheck_identity(&definition.identity, self.limits.max_identity_bytes)?;
            precheck_identity(
                &definition.paired_forward_effect,
                self.limits.max_identity_bytes,
            )?;
        }
        precheck(
            Resource::IdentityBytes,
            fact.identity_bytes().len(),
            self.limits.max_identity_bytes,
        )?;
        let ForwardFact::EstablishedEffect(effect) = fact else {
            return Ok(Vec::new());
        };

        let matching = definitions
            .iter()
            .filter(|definition| definition.paired_forward_effect == *effect)
            .count();
        let new_count = definitions
            .iter()
            .filter(|definition| definition.paired_forward_effect == *effect)
            .filter(|definition| {
                !self
                    .registrations
                    .contains_key(&(definition.identity.clone(), effect.clone()))
            })
            .count();
        let Some(candidate_total) = self.registrations.len().checked_add(new_count) else {
            return Err(ResourceRefusal::new(
                Resource::Registrations,
                usize::MAX,
                self.limits.max_registrations,
            ));
        };
        precheck(
            Resource::Registrations,
            candidate_total,
            self.limits.max_registrations,
        )?;

        let mut events = Vec::with_capacity(matching);
        for definition in definitions
            .iter()
            .filter(|definition| definition.paired_forward_effect == *effect)
        {
            let key = (definition.identity.clone(), effect.clone());
            let (registration, created) = match self.registrations.get(&key) {
                Some(registration) => (registration.clone(), false),
                None => {
                    let registration = Registration::derive(definition);
                    self.registrations.insert(key, registration.clone());
                    (registration, true)
                }
            };
            events.push(RegistrationEvent {
                registration,
                created,
            });
        }
        Ok(events)
    }

    /// Registrations in exact key order, independent of observation arrival.
    pub fn registrations(&self) -> impl ExactSizeIterator<Item = &Registration> {
        self.registrations.values()
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new(Limits::owner_max())
    }
}

/// One typed milestone in the authored causal order.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum Milestone {
    /// Established forward business effect.
    ForwardEffect(EffectId),
    /// Compensation registration created for that effect.
    Registration(RegistrationId),
    /// Independently established compensation effect.
    CompensationEffect(EffectId),
    /// Authored commit observation.
    Commit(CommitId),
}

impl Milestone {
    fn identity_bytes(&self) -> &[u8] {
        match self {
            Self::ForwardEffect(identity) | Self::CompensationEffect(identity) => {
                identity.identity_bytes()
            }
            Self::Registration(identity) => identity.identity_bytes(),
            Self::Commit(identity) => identity.identity_bytes(),
        }
    }
}

/// One explicit authored causal edge. Vector/source/key order has no authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalEdge {
    /// Authored predecessor milestone.
    pub before: Milestone,
    /// Authored successor milestone.
    pub after: Milestone,
}

/// Invalid authored-order reason.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum InvalidOrder {
    EmptyIdentity,
    DuplicateNode,
    UnknownEndpoint,
    DuplicateEdge,
    SelfEdge,
    Cycle,
}

/// Construction failure for an authored causal order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OrderRefusal {
    Resource(ResourceRefusal),
    Invalid(InvalidOrder),
}

impl core::fmt::Display for OrderRefusal {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Resource(refusal) => refusal.fmt(formatter),
            Self::Invalid(reason) => write!(formatter, "invalid authored causal order: {reason:?}"),
        }
    }
}

impl std::error::Error for OrderRefusal {}

impl From<ResourceRefusal> for OrderRefusal {
    fn from(value: ResourceRefusal) -> Self {
        Self::Resource(value)
    }
}

/// Validated finite authored causal order.
#[derive(Clone, Debug)]
pub struct AuthoredOrder {
    nodes: BTreeSet<Milestone>,
    successors: BTreeMap<Milestone, Vec<Milestone>>,
}

impl AuthoredOrder {
    /// Validate a complete explicit DAG before retaining it.
    pub fn new(
        nodes: &[Milestone],
        edges: &[CausalEdge],
        limits: Limits,
    ) -> Result<Self, OrderRefusal> {
        let limits = limits.effective();
        precheck(Resource::OrderNodes, nodes.len(), limits.max_order_nodes)?;
        precheck(Resource::OrderEdges, edges.len(), limits.max_order_edges)?;
        for node in nodes {
            precheck(
                Resource::IdentityBytes,
                node.identity_bytes().len(),
                limits.max_identity_bytes,
            )?;
            if node.identity_bytes().is_empty() {
                return Err(OrderRefusal::Invalid(InvalidOrder::EmptyIdentity));
            }
        }
        let node_set: BTreeSet<_> = nodes.iter().cloned().collect();
        if node_set.len() != nodes.len() {
            return Err(OrderRefusal::Invalid(InvalidOrder::DuplicateNode));
        }

        let mut edge_set = BTreeSet::new();
        let mut successors: BTreeMap<Milestone, Vec<Milestone>> = node_set
            .iter()
            .cloned()
            .map(|node| (node, Vec::new()))
            .collect();
        let mut indegree: BTreeMap<Milestone, usize> =
            node_set.iter().cloned().map(|node| (node, 0)).collect();
        for edge in edges {
            if edge.before == edge.after {
                return Err(OrderRefusal::Invalid(InvalidOrder::SelfEdge));
            }
            if !node_set.contains(&edge.before) || !node_set.contains(&edge.after) {
                return Err(OrderRefusal::Invalid(InvalidOrder::UnknownEndpoint));
            }
            if !edge_set.insert((edge.before.clone(), edge.after.clone())) {
                return Err(OrderRefusal::Invalid(InvalidOrder::DuplicateEdge));
            }
            if let Some(next) = successors.get_mut(&edge.before) {
                next.push(edge.after.clone());
            }
            if let Some(degree) = indegree.get_mut(&edge.after) {
                let Some(next) = degree.checked_add(1) else {
                    return Err(ResourceRefusal::new(
                        Resource::OrderEdges,
                        usize::MAX,
                        limits.max_order_edges,
                    )
                    .into());
                };
                *degree = next;
            }
        }

        let mut ready: VecDeque<_> = indegree
            .iter()
            .filter_map(|(node, degree)| (*degree == 0).then_some(node.clone()))
            .collect();
        let mut visited = 0usize;
        while let Some(node) = ready.pop_front() {
            let Some(next_visited) = visited.checked_add(1) else {
                return Err(ResourceRefusal::new(
                    Resource::OrderNodes,
                    usize::MAX,
                    limits.max_order_nodes,
                )
                .into());
            };
            visited = next_visited;
            if let Some(next) = successors.get(&node) {
                for successor in next {
                    if let Some(degree) = indegree.get_mut(successor) {
                        let Some(next) = degree.checked_sub(1) else {
                            return Err(OrderRefusal::Invalid(InvalidOrder::Cycle));
                        };
                        *degree = next;
                        if *degree == 0 {
                            ready.push_back(successor.clone());
                        }
                    }
                }
            }
        }
        if visited != node_set.len() {
            return Err(OrderRefusal::Invalid(InvalidOrder::Cycle));
        }
        Ok(Self {
            nodes: node_set,
            successors,
        })
    }

    /// Whether the exact authored graph has a non-empty path from `before` to `after`.
    #[must_use]
    pub fn precedes(&self, before: &Milestone, after: &Milestone) -> bool {
        if before == after || !self.nodes.contains(before) || !self.nodes.contains(after) {
            return false;
        }
        let mut visited = BTreeSet::new();
        let mut queue = VecDeque::from([before.clone()]);
        while let Some(node) = queue.pop_front() {
            if !visited.insert(node.clone()) {
                continue;
            }
            if let Some(next) = self.successors.get(&node) {
                if next.iter().any(|candidate| candidate == after) {
                    return true;
                }
                queue.extend(next.iter().cloned());
            }
        }
        false
    }
}

/// The three upstream-authored ordering violations, kept mutually distinct.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum OrderingViolation {
    RegistrationAfterCommit,
    CompensationBeforeForwardEffect,
    CompensationAfterCommit,
}

/// Required ordering input that was absent or not present in the authored DAG.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum MissingOrderingInput {
    Registration,
    ForwardEffect,
    CompensationEffect,
    Commit,
}

/// Complete ordering assessment. Missing inputs never become a permitting order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrderingAssessment {
    violations: Vec<OrderingViolation>,
    missing: Vec<MissingOrderingInput>,
}

impl OrderingAssessment {
    /// Distinct violations in declared order.
    #[must_use]
    pub fn violations(&self) -> &[OrderingViolation] {
        &self.violations
    }

    /// Missing authorities in declared order.
    #[must_use]
    pub fn missing(&self) -> &[MissingOrderingInput] {
        &self.missing
    }

    /// True only when every required input was available.
    #[must_use]
    pub fn complete(&self) -> bool {
        self.missing.is_empty()
    }
}

/// Assess the three causal-order laws without consulting timestamps or source order.
#[must_use]
pub fn assess_ordering(
    _pure: PureAssessment,
    registration: Option<&Registration>,
    compensation_effect: Option<&EffectId>,
    commit: Option<&CommitId>,
    order: &AuthoredOrder,
) -> OrderingAssessment {
    let mut missing = Vec::new();
    let registration_node =
        registration.map(|value| Milestone::Registration(value.identity.clone()));
    let forward_node =
        registration.map(|value| Milestone::ForwardEffect(value.forward_effect.clone()));
    let compensation_node = compensation_effect
        .cloned()
        .map(Milestone::CompensationEffect);
    let commit_node = commit.cloned().map(Milestone::Commit);

    retain_missing(
        &mut missing,
        MissingOrderingInput::Registration,
        registration_node.as_ref(),
        order,
    );
    retain_missing(
        &mut missing,
        MissingOrderingInput::ForwardEffect,
        forward_node.as_ref(),
        order,
    );
    retain_missing(
        &mut missing,
        MissingOrderingInput::CompensationEffect,
        compensation_node.as_ref(),
        order,
    );
    retain_missing(
        &mut missing,
        MissingOrderingInput::Commit,
        commit_node.as_ref(),
        order,
    );

    let mut violations = Vec::new();
    if pair_precedes(order, commit_node.as_ref(), registration_node.as_ref()) {
        violations.push(OrderingViolation::RegistrationAfterCommit);
    }
    if pair_precedes(order, compensation_node.as_ref(), forward_node.as_ref()) {
        violations.push(OrderingViolation::CompensationBeforeForwardEffect);
    }
    if pair_precedes(order, commit_node.as_ref(), compensation_node.as_ref()) {
        violations.push(OrderingViolation::CompensationAfterCommit);
    }
    OrderingAssessment {
        violations,
        missing,
    }
}

fn retain_missing(
    missing: &mut Vec<MissingOrderingInput>,
    kind: MissingOrderingInput,
    node: Option<&Milestone>,
    order: &AuthoredOrder,
) {
    if node.is_none_or(|node| !order.nodes.contains(node)) {
        missing.push(kind);
    }
}

fn pair_precedes(
    order: &AuthoredOrder,
    before: Option<&Milestone>,
    after: Option<&Milestone>,
) -> bool {
    before
        .zip(after)
        .is_some_and(|(before, after)| order.precedes(before, after))
}

/// Adopted FR-236 exact attempt outcome vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum AttemptOutcome {
    Failed,
    TimedOut,
    Duplicate,
    Pending,
    Successful,
}

impl AttemptOutcome {
    /// Exact owner label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Failed => "failed",
            Self::TimedOut => "timed-out",
            Self::Duplicate => "duplicate",
            Self::Pending => "pending",
            Self::Successful => "successful",
        }
    }
}

/// One compensation attempt retained separately from any effect.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttemptFact {
    /// Exact attempt identity.
    pub attempt: AttemptId,
    /// Exact owning registration.
    pub registration: RegistrationId,
    /// Independently established compensation effect, when present.
    pub established_effect: Option<EffectId>,
    /// Exact outcome, or absence when no outcome was established.
    pub outcome: Option<AttemptOutcome>,
}

/// Retry admission outcome. Every non-admitted arm leaves the ledger unchanged.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AttemptAdmission {
    /// Attempt lies at or below the authored bound and was retained at this position.
    Admitted { attempt: AttemptId, position: usize },
    /// No authored bound exists; absence is not interpreted as any bound.
    RetryBoundAbsent { attempt: AttemptId },
    /// Attempt names another registration.
    RegistrationMismatch {
        attempt: AttemptId,
        expected: RegistrationId,
        offered: RegistrationId,
    },
    /// Attempt would exceed the authored finite bound.
    RetryBoundExceeded {
        attempt: AttemptId,
        bound: NonZeroU32,
    },
    /// The independent owner ceiling would be exceeded before retention.
    ResourceLimitExceeded {
        attempt: AttemptId,
        refusal: ResourceRefusal,
    },
}

/// Ordered attempts for one exact registration.
#[derive(Clone, Debug)]
pub struct AttemptLedger {
    registration: Registration,
    max_attempts: usize,
    max_identity_bytes: usize,
    attempts: Vec<AttemptFact>,
}

impl AttemptLedger {
    /// Begin retaining attempts for one registration under effective caller ceilings.
    pub fn new(registration: Registration, limits: Limits) -> Result<Self, ResourceRefusal> {
        let limits = limits.effective();
        precheck(
            Resource::IdentityBytes,
            registration.identity.identity_bytes().len(),
            limits.max_identity_bytes,
        )?;
        precheck_identity(&registration.definition, limits.max_identity_bytes)?;
        precheck_identity(&registration.forward_effect, limits.max_identity_bytes)?;
        Ok(Self {
            registration,
            max_attempts: limits.max_attempts,
            max_identity_bytes: limits.max_identity_bytes,
            attempts: Vec::new(),
        })
    }

    /// Admit one exact attempt under the authored finite bound.
    pub fn admit(&mut self, _pure: PureAssessment, attempt: AttemptFact) -> AttemptAdmission {
        let oversized = [
            attempt.attempt.identity_bytes().len(),
            attempt.registration.identity_bytes().len(),
            attempt
                .established_effect
                .as_ref()
                .map_or(0, |identity| identity.identity_bytes().len()),
        ]
        .into_iter()
        .max()
        .unwrap_or(0);
        if oversized > self.max_identity_bytes {
            return AttemptAdmission::ResourceLimitExceeded {
                attempt: attempt.attempt,
                refusal: ResourceRefusal::new(
                    Resource::IdentityBytes,
                    oversized,
                    self.max_identity_bytes,
                ),
            };
        }
        if attempt.registration != self.registration.identity {
            return AttemptAdmission::RegistrationMismatch {
                attempt: attempt.attempt,
                expected: self.registration.identity.clone(),
                offered: attempt.registration,
            };
        }
        let attempt_identity = attempt.attempt.clone();
        let Some(bound) = self.registration.retry_bound else {
            return AttemptAdmission::RetryBoundAbsent {
                attempt: attempt_identity,
            };
        };
        let Some(next) = self.attempts.len().checked_add(1) else {
            return AttemptAdmission::ResourceLimitExceeded {
                attempt: attempt_identity,
                refusal: ResourceRefusal::new(Resource::Attempts, usize::MAX, self.max_attempts),
            };
        };
        if next > self.max_attempts {
            return AttemptAdmission::ResourceLimitExceeded {
                attempt: attempt_identity,
                refusal: ResourceRefusal::new(Resource::Attempts, next, self.max_attempts),
            };
        }
        if u128::from(bound.get()) < next as u128 {
            return AttemptAdmission::RetryBoundExceeded {
                attempt: attempt_identity,
                bound,
            };
        }
        self.attempts.push(attempt);
        AttemptAdmission::Admitted {
            attempt: attempt_identity,
            position: next,
        }
    }

    /// Previously admitted attempts in presentation order.
    #[must_use]
    pub fn attempts(&self) -> &[AttemptFact] {
        &self.attempts
    }

    /// Exact owning registration.
    #[must_use]
    pub const fn registration(&self) -> &Registration {
        &self.registration
    }
}

fn precheck_identity<T>(identity: &T, limit: usize) -> Result<(), ResourceRefusal>
where
    T: ExactIdentity,
{
    precheck(Resource::IdentityBytes, identity.exact_bytes().len(), limit)
}

trait ExactIdentity {
    fn exact_bytes(&self) -> &[u8];
}

impl ExactIdentity for DefinitionId {
    fn exact_bytes(&self) -> &[u8] {
        self.identity_bytes()
    }
}

impl ExactIdentity for EffectId {
    fn exact_bytes(&self) -> &[u8] {
        self.identity_bytes()
    }
}

impl ExactIdentity for RegistrationId {
    fn exact_bytes(&self) -> &[u8] {
        self.identity_bytes()
    }
}

fn precheck(resource: Resource, observed: usize, limit: usize) -> Result<(), ResourceRefusal> {
    if observed > limit {
        Err(ResourceRefusal::new(resource, observed, limit))
    } else {
        Ok(())
    }
}

fn push_part(target: &mut Vec<u8>, value: &[u8]) {
    let length = u64::try_from(value.len()).unwrap_or(u64::MAX);
    target.extend_from_slice(&length.to_be_bytes());
    target.extend_from_slice(value);
}
