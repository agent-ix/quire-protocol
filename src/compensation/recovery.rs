// SPDX-License-Identifier: AGPL-3.0-or-later
//! Recovery decisions retained from exact authored relations.
//!
//! Production recovery evaluation is owned by
//! [`crate::assessment::Engine::evaluate_compensation_recovery`] and walks the
//! selected admitted QSL expression. The former relation double is retained
//! only as a crate-private test seam and is not importable by consumers:
//!
//! ```compile_fail
//! use quire_protocol::compensation::recovery::RecoveryRelation;
//! ```

#![deny(clippy::wildcard_enum_match_arm)]

use std::collections::BTreeSet;

use quire_observation::authority::{completeness, population};

use crate::bounds::inventory::{BudgetDimension, SourceSite};
use crate::bounds::{SiteId, SiteKind};
use crate::ids::{CommitId, EffectId, RecoveryId, RegistrationId};

use super::algebra::Registration;
use super::purity::PureAssessment;

pub(crate) const OBSERVATION_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("compensation/observation/consume"),
    "compensation::recovery::assess",
    BudgetDimension::Observation,
    SiteKind::Iteration,
);

/// Immutable recovery-assessment owner maxima.
pub const OWNER_MAX: Limits = Limits {
    max_identity_bytes: 64 * 1024,
    max_state_bytes: 1024 * 1024,
    max_relation_bytes: 1024 * 1024,
    max_authority_bytes: 8 * 1024 * 1024,
    max_population_members: 10_000,
};

/// Caller-lowerable recovery-assessment ceilings.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Limits {
    /// Bytes in one identity.
    pub max_identity_bytes: usize,
    /// Canonical bytes in one retained state.
    pub max_state_bytes: usize,
    /// Exact authored bytes in one selected relation.
    pub max_relation_bytes: usize,
    /// Bytes retained from one strict QObs authority document.
    pub max_authority_bytes: usize,
    /// Members in one aggregating relation.
    pub max_population_members: usize,
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
            max_identity_bytes: min(self.max_identity_bytes, OWNER_MAX.max_identity_bytes),
            max_state_bytes: min(self.max_state_bytes, OWNER_MAX.max_state_bytes),
            max_relation_bytes: min(self.max_relation_bytes, OWNER_MAX.max_relation_bytes),
            max_authority_bytes: min(self.max_authority_bytes, OWNER_MAX.max_authority_bytes),
            max_population_members: min(
                self.max_population_members,
                OWNER_MAX.max_population_members,
            ),
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

/// Retained state identity and canonical value bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct State {
    identity: Box<[u8]>,
    canonical_value: Box<[u8]>,
}

impl State {
    /// Validate and retain one exact state under effective ceilings.
    pub fn new(
        identity: impl Into<Box<[u8]>>,
        canonical_value: impl Into<Box<[u8]>>,
        limits: Limits,
    ) -> Result<Self, Refusal> {
        let identity = identity.into();
        let canonical_value = canonical_value.into();
        let limits = limits.effective();
        precheck_nonempty(
            &identity,
            Resource::IdentityBytes,
            limits.max_identity_bytes,
        )?;
        precheck_nonempty(
            &canonical_value,
            Resource::StateBytes,
            limits.max_state_bytes,
        )?;
        Ok(Self {
            identity,
            canonical_value,
        })
    }

    /// Exact retained state identity bytes.
    #[must_use]
    pub fn identity_bytes(&self) -> &[u8] {
        &self.identity
    }

    /// Exact retained canonical state bytes.
    #[must_use]
    pub fn canonical_value(&self) -> &[u8] {
        &self.canonical_value
    }
}

/// Exact population and membership selection authored into a relation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PopulationRequirement {
    population_identity: Box<str>,
    membership_rule_identity: Box<str>,
    membership_digest: Box<str>,
    required_members: Vec<EffectId>,
}

impl PopulationRequirement {
    /// Validate and retain an exact finite population selection.
    pub fn new(
        population_identity: impl Into<Box<str>>,
        membership_rule_identity: impl Into<Box<str>>,
        membership_digest: impl Into<Box<str>>,
        required_members: Vec<EffectId>,
        limits: Limits,
    ) -> Result<Self, Refusal> {
        let population_identity = population_identity.into();
        let membership_rule_identity = membership_rule_identity.into();
        let membership_digest = membership_digest.into();
        let limits = limits.effective();
        precheck_nonempty(
            population_identity.as_bytes(),
            Resource::IdentityBytes,
            limits.max_identity_bytes,
        )?;
        precheck_nonempty(
            membership_rule_identity.as_bytes(),
            Resource::IdentityBytes,
            limits.max_identity_bytes,
        )?;
        precheck_nonempty(
            membership_digest.as_bytes(),
            Resource::IdentityBytes,
            limits.max_identity_bytes,
        )?;
        precheck(
            Resource::PopulationMembers,
            required_members.len(),
            limits.max_population_members,
        )?;
        if required_members.is_empty()
            || required_members
                .iter()
                .any(|member| member.identity_bytes().is_empty())
        {
            return Err(Refusal::Invalid(InvalidInput::EmptyValue));
        }
        if required_members
            .iter()
            .any(|member| member.identity_bytes().len() > limits.max_identity_bytes)
        {
            let observed = required_members
                .iter()
                .map(|member| member.identity_bytes().len())
                .max()
                .unwrap_or(0);
            return Err(Refusal::Resource(ResourceRefusal {
                resource: Resource::IdentityBytes,
                observed,
                limit: limits.max_identity_bytes,
            }));
        }
        let distinct: BTreeSet<_> = required_members.iter().cloned().collect();
        if distinct.len() != required_members.len() {
            return Err(Refusal::Invalid(InvalidInput::DuplicatePopulationMember));
        }
        Ok(Self {
            population_identity,
            membership_rule_identity,
            membership_digest,
            required_members,
        })
    }

    /// Exact selected population identity.
    #[must_use]
    pub fn population_identity(&self) -> &str {
        &self.population_identity
    }

    /// Exact selected membership-rule identity.
    #[must_use]
    pub fn membership_rule_identity(&self) -> &str {
        &self.membership_rule_identity
    }

    /// Exact selected membership-document digest text.
    #[must_use]
    pub fn membership_digest(&self) -> &str {
        &self.membership_digest
    }

    /// Exact required effect population in authored order.
    #[must_use]
    pub fn required_members(&self) -> &[EffectId] {
        &self.required_members
    }
}

/// Exact authored relation selection. Relation types remain an open owner set.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationSelection {
    recovery: RecoveryId,
    registration: RegistrationId,
    target_identity: Box<[u8]>,
    relation_type: Box<[u8]>,
    authored_bytes: Box<[u8]>,
    population: Option<PopulationRequirement>,
}

impl RelationSelection {
    /// Validate and retain one selected relation without interpreting its bytes.
    pub fn new(
        recovery: RecoveryId,
        registration: RegistrationId,
        target_identity: impl Into<Box<[u8]>>,
        relation_type: impl Into<Box<[u8]>>,
        authored_bytes: impl Into<Box<[u8]>>,
        population: Option<PopulationRequirement>,
        limits: Limits,
    ) -> Result<Self, Refusal> {
        let target_identity = target_identity.into();
        let relation_type = relation_type.into();
        let authored_bytes = authored_bytes.into();
        let limits = limits.effective();
        precheck_nonempty(
            recovery.identity_bytes(),
            Resource::IdentityBytes,
            limits.max_identity_bytes,
        )?;
        precheck_nonempty(
            registration.identity_bytes(),
            Resource::IdentityBytes,
            limits.max_identity_bytes,
        )?;
        precheck_nonempty(
            &target_identity,
            Resource::IdentityBytes,
            limits.max_identity_bytes,
        )?;
        precheck_nonempty(
            &relation_type,
            Resource::IdentityBytes,
            limits.max_identity_bytes,
        )?;
        precheck_nonempty(
            &authored_bytes,
            Resource::RelationBytes,
            limits.max_relation_bytes,
        )?;
        Ok(Self {
            recovery,
            registration,
            target_identity,
            relation_type,
            authored_bytes,
            population,
        })
    }

    /// Exact recovery obligation to which the relation is authored.
    #[must_use]
    pub const fn recovery(&self) -> &RecoveryId {
        &self.recovery
    }

    /// Exact compensation registration to which the relation is authored.
    #[must_use]
    pub const fn registration(&self) -> &RegistrationId {
        &self.registration
    }

    /// Exact captured target selected by the authored relation.
    #[must_use]
    pub fn target_identity(&self) -> &[u8] {
        &self.target_identity
    }

    /// Exact owner relation-type bytes.
    #[must_use]
    pub fn relation_type(&self) -> &[u8] {
        &self.relation_type
    }

    /// Exact authored relation bytes.
    #[must_use]
    pub fn authored_bytes(&self) -> &[u8] {
        &self.authored_bytes
    }

    /// Aggregating population selection, when this relation requires one.
    #[must_use]
    pub const fn population(&self) -> Option<&PopulationRequirement> {
        self.population.as_ref()
    }
}

/// Crate-private test seam for one authored typed recovery relation.
///
/// The evaluator receives no clock, transport, storage, telemetry, or operation
/// capability. Its identity and exact authored bytes are checked before it is
/// invoked. No production Engine method accepts this crate-private trait.
pub(crate) trait RecoveryRelation {
    /// Exact open-set owner relation type.
    fn relation_type(&self) -> &[u8];
    /// Exact authored relation bytes.
    fn authored_bytes(&self) -> &[u8];
    /// Evaluate only the supplied retained state and exact member effects.
    fn evaluate(&self, target: &State, achieved: &State, members: &[EffectId]) -> bool;
}

/// Strict QObs population and completeness authority for an aggregate relation.
#[derive(Clone, Copy, Debug)]
pub(crate) struct PopulationEvidence<'a> {
    /// Strict-read QObs declared population.
    pub population: &'a population::View,
    /// Strict-read QObs completeness assertion.
    pub completeness: &'a completeness::View,
    /// Independently established compensation effects offered to the relation.
    pub effects: &'a [EffectId],
}

/// Exact inputs for one recovery decision.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Input<'a> {
    /// Exact recovery-obligation identity.
    pub recovery: &'a RecoveryId,
    /// Exact compensation registration.
    pub registration: &'a Registration,
    /// Retained captured target state, when available.
    pub target: Option<&'a State>,
    /// Retained achieved state, when available.
    pub achieved: Option<&'a State>,
    /// Required commit observation, when available.
    pub commit: Option<&'a CommitId>,
    /// Required established compensation effect, when available.
    pub compensation_effect: Option<&'a EffectId>,
    /// Aggregate authority, when selected and available.
    pub population: Option<PopulationEvidence<'a>>,
}

/// Missing input that prevents a recovery-success decision.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum MissingInput {
    TargetState,
    AchievedState,
    Commit,
    CompensationEffect,
    Population,
    PopulationMember,
    MembershipAuthority,
    CompletenessAuthority,
}

/// Identity/authority substitution axis.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Substitution {
    Recovery,
    Registration,
    TargetState,
    RelationType,
    RelationBytes,
    Population,
    MembershipRule,
    MembershipDigest,
    PopulationMembers,
}

/// Invalid recovery input.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum InvalidInput {
    EmptyValue,
    DuplicatePopulationMember,
}

/// Recovery resource dimension.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Resource {
    IdentityBytes,
    StateBytes,
    RelationBytes,
    AuthorityBytes,
    PopulationMembers,
}

/// Bounded-work refusal emitted before relation evaluation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceRefusal {
    resource: Resource,
    observed: usize,
    limit: usize,
}

impl ResourceRefusal {
    /// Refused dimension.
    #[must_use]
    pub const fn resource(self) -> Resource {
        self.resource
    }

    /// Candidate usage.
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

/// Typed recovery refusal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Refusal {
    Resource(ResourceRefusal),
    Invalid(InvalidInput),
    Substitution(Substitution),
}

impl core::fmt::Display for Refusal {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Resource(value) => write!(
                formatter,
                "recovery {:?} usage {} exceeds effective limit {}",
                value.resource, value.observed, value.limit
            ),
            Self::Invalid(value) => write!(formatter, "invalid recovery input: {value:?}"),
            Self::Substitution(value) => write!(formatter, "recovery substitution: {value:?}"),
        }
    }
}

impl std::error::Error for Refusal {}

/// One relation-bound recovery fact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fact {
    /// Exact recovery obligation.
    pub recovery: RecoveryId,
    /// Exact owning registration.
    pub registration: RegistrationId,
    /// Exact observed commit.
    pub commit: CommitId,
    /// Exact independently established compensation effect.
    pub compensation_effect: EffectId,
    /// Exact target state retained by the relation.
    pub target: State,
    /// Exact achieved state retained by the relation.
    pub achieved: State,
    /// Open-set owner relation type.
    pub relation_type: Box<[u8]>,
    /// Exact authored relation bytes.
    pub relation_bytes: Box<[u8]>,
    /// Strict owner population bytes, only for an aggregate relation.
    pub population_bytes: Option<Vec<u8>>,
    /// Strict owner completeness bytes, only for an aggregate relation.
    pub completeness_bytes: Option<Vec<u8>>,
}

/// Recovery outcome. Full and partial relations each produce their own value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Decision {
    Satisfied(Fact),
    NotSatisfied(Fact),
    Pending(Vec<MissingInput>),
    Incomplete(Vec<MissingInput>),
}

/// Evaluate exactly one selected authored relation.
#[expect(
    dead_code,
    reason = "crate-private relation-double seam cannot enter the production Engine"
)]
pub(crate) fn assess(
    _pure: PureAssessment,
    selection: &RelationSelection,
    relation: &dyn RecoveryRelation,
    input: Input<'_>,
    limits: Limits,
) -> Result<Decision, Refusal> {
    let limits = limits.effective();
    precheck_nonempty(
        selection.recovery().identity_bytes(),
        Resource::IdentityBytes,
        limits.max_identity_bytes,
    )?;
    precheck_nonempty(
        selection.registration().identity_bytes(),
        Resource::IdentityBytes,
        limits.max_identity_bytes,
    )?;
    precheck_nonempty(
        selection.target_identity(),
        Resource::IdentityBytes,
        limits.max_identity_bytes,
    )?;
    precheck_nonempty(
        relation.relation_type(),
        Resource::IdentityBytes,
        limits.max_identity_bytes,
    )?;
    precheck_nonempty(
        relation.authored_bytes(),
        Resource::RelationBytes,
        limits.max_relation_bytes,
    )?;
    precheck_nonempty(
        selection.relation_type(),
        Resource::IdentityBytes,
        limits.max_identity_bytes,
    )?;
    precheck_nonempty(
        selection.authored_bytes(),
        Resource::RelationBytes,
        limits.max_relation_bytes,
    )?;
    if input.recovery != selection.recovery() {
        return Err(Refusal::Substitution(Substitution::Recovery));
    }
    if input.registration.identity != *selection.registration() {
        return Err(Refusal::Substitution(Substitution::Registration));
    }
    if relation.relation_type() != selection.relation_type() {
        return Err(Refusal::Substitution(Substitution::RelationType));
    }
    if relation.authored_bytes() != selection.authored_bytes() {
        return Err(Refusal::Substitution(Substitution::RelationBytes));
    }
    precheck_nonempty(
        input.recovery.identity_bytes(),
        Resource::IdentityBytes,
        limits.max_identity_bytes,
    )?;
    precheck_nonempty(
        input.registration.identity.identity_bytes(),
        Resource::IdentityBytes,
        limits.max_identity_bytes,
    )?;

    let mut missing = Vec::new();
    if input.target.is_none() {
        missing.push(MissingInput::TargetState);
    } else if input
        .target
        .is_some_and(|target| target.identity_bytes() != selection.target_identity())
    {
        return Err(Refusal::Substitution(Substitution::TargetState));
    }
    if input.achieved.is_none() {
        missing.push(MissingInput::AchievedState);
    }
    if input.commit.is_none() {
        missing.push(MissingInput::Commit);
    }
    if input.compensation_effect.is_none() {
        missing.push(MissingInput::CompensationEffect);
    }
    let members = validate_population(
        selection.population(),
        input.population,
        &mut missing,
        limits,
    )?;
    if !missing.is_empty() {
        return Ok(if missing == [MissingInput::Commit] {
            Decision::Pending(missing)
        } else {
            Decision::Incomplete(missing)
        });
    }

    let (Some(target), Some(achieved), Some(commit), Some(compensation_effect)) = (
        input.target,
        input.achieved,
        input.commit,
        input.compensation_effect,
    ) else {
        return Ok(Decision::Incomplete(missing));
    };
    validate_state(target, limits)?;
    validate_state(achieved, limits)?;
    precheck_nonempty(
        commit.identity_bytes(),
        Resource::IdentityBytes,
        limits.max_identity_bytes,
    )?;
    precheck_nonempty(
        compensation_effect.identity_bytes(),
        Resource::IdentityBytes,
        limits.max_identity_bytes,
    )?;

    let fact = Fact {
        recovery: input.recovery.clone(),
        registration: input.registration.identity.clone(),
        commit: commit.clone(),
        compensation_effect: compensation_effect.clone(),
        target: target.clone(),
        achieved: achieved.clone(),
        relation_type: selection.relation_type.clone(),
        relation_bytes: selection.authored_bytes.clone(),
        population_bytes: input
            .population
            .map(|evidence| evidence.population.bytes().to_vec()),
        completeness_bytes: input
            .population
            .map(|evidence| evidence.completeness.bytes().to_vec()),
    };
    if relation.evaluate(target, achieved, members) {
        Ok(Decision::Satisfied(fact))
    } else {
        Ok(Decision::NotSatisfied(fact))
    }
}

fn validate_population<'a>(
    requirement: Option<&PopulationRequirement>,
    evidence: Option<PopulationEvidence<'a>>,
    missing: &mut Vec<MissingInput>,
    limits: Limits,
) -> Result<&'a [EffectId], Refusal> {
    let Some(requirement) = requirement else {
        if evidence.is_some() {
            return Err(Refusal::Substitution(Substitution::Population));
        }
        return Ok(&[]);
    };
    let Some(evidence) = evidence else {
        missing.extend([
            MissingInput::Population,
            MissingInput::PopulationMember,
            MissingInput::MembershipAuthority,
            MissingInput::CompletenessAuthority,
        ]);
        return Ok(&[]);
    };
    precheck(
        Resource::PopulationMembers,
        evidence.effects.len(),
        limits.max_population_members,
    )?;
    precheck(
        Resource::PopulationMembers,
        requirement.required_members().len(),
        limits.max_population_members,
    )?;
    for effect in requirement.required_members() {
        precheck_nonempty(
            effect.identity_bytes(),
            Resource::IdentityBytes,
            limits.max_identity_bytes,
        )?;
    }
    for effect in evidence.effects {
        precheck_nonempty(
            effect.identity_bytes(),
            Resource::IdentityBytes,
            limits.max_identity_bytes,
        )?;
    }
    precheck(
        Resource::AuthorityBytes,
        evidence.population.bytes().len(),
        limits.max_authority_bytes,
    )?;
    precheck(
        Resource::AuthorityBytes,
        evidence.completeness.bytes().len(),
        limits.max_authority_bytes,
    )?;
    let population = evidence.population.payload();
    for identity in [
        population.population_identity(),
        population.membership_rule_identity(),
        population.membership_digest(),
        evidence.completeness.payload().population_identity(),
    ] {
        precheck_nonempty(
            identity.as_bytes(),
            Resource::IdentityBytes,
            limits.max_identity_bytes,
        )?;
    }
    if population.population_identity() != requirement.population_identity() {
        return Err(Refusal::Substitution(Substitution::Population));
    }
    if population.membership_rule_identity() != requirement.membership_rule_identity() {
        return Err(Refusal::Substitution(Substitution::MembershipRule));
    }
    if population.membership_digest() != requirement.membership_digest() {
        return Err(Refusal::Substitution(Substitution::MembershipDigest));
    }
    if population.required_members().len() != requirement.required_members().len()
        || population
            .required_members()
            .iter()
            .zip(requirement.required_members())
            .any(|(owner, selected)| owner.as_bytes() != selected.identity_bytes())
    {
        return Err(Refusal::Substitution(Substitution::PopulationMembers));
    }
    if evidence.effects.len() != requirement.required_members().len()
        || requirement
            .required_members()
            .iter()
            .any(|required| !evidence.effects.iter().any(|effect| effect == required))
    {
        missing.push(MissingInput::PopulationMember);
    }
    let completeness = evidence.completeness.payload();
    if completeness.population_identity() != population.population_identity()
        || completeness.fact_count() != population.required_members().len()
        || completeness
            .facts()
            .zip(population.required_members())
            .any(|(fact, member)| {
                fact.member_identity != member
                    || fact.status != completeness::FactStatus::Available
                    || fact.observation_identity.is_none()
            })
        || completeness.state() != completeness::State::Complete
    {
        missing.push(MissingInput::CompletenessAuthority);
    }
    Ok(evidence.effects)
}

fn validate_state(state: &State, limits: Limits) -> Result<(), Refusal> {
    precheck_nonempty(
        state.identity_bytes(),
        Resource::IdentityBytes,
        limits.max_identity_bytes,
    )?;
    precheck_nonempty(
        state.canonical_value(),
        Resource::StateBytes,
        limits.max_state_bytes,
    )
}

fn precheck_nonempty(bytes: &[u8], resource: Resource, limit: usize) -> Result<(), Refusal> {
    if bytes.is_empty() {
        return Err(Refusal::Invalid(InvalidInput::EmptyValue));
    }
    precheck(resource, bytes.len(), limit)
}

fn precheck(resource: Resource, observed: usize, limit: usize) -> Result<(), Refusal> {
    if observed > limit {
        Err(Refusal::Resource(ResourceRefusal {
            resource,
            observed,
            limit,
        }))
    } else {
        Ok(())
    }
}
