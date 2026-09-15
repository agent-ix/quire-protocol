// SPDX-License-Identifier: AGPL-3.0-or-later
//! Pure compensation assessment over exact linked and owner-issued facts.
//!
//! The subsystem is deliberately split by authority rather than by workflow
//! step: [`activate`] owns obligation activation, [`algebra`] owns established
//! effects/registration/authored order/retry, [`recovery`] owns relation-bound
//! recovery, and [`purity`] constructs the capability-free witness required by
//! every operation. This facade only composes already-typed facts; it neither
//! runs a business operation nor infers one fact from another.

#![deny(clippy::wildcard_enum_match_arm)]

/// Trigger, capture, and anchor activation.
pub mod activate;
/// Replay-idempotent registration, causal commit ordering, and finite retry.
pub mod algebra;
/// Constructor-checked capability boundary for assessment purity.
pub mod purity;
/// Full and partial recovery through exact authored relation owners.
pub mod recovery;
/// Derived registration keys over QSL and QObs owner values.
pub mod registration;

use crate::bounds::inventory::{BudgetDimension, OperationOutcome, ResourceAccountant, SourceSite};
use crate::bounds::{SiteId, SiteKind, UnitLabel};

use activate::{Activation, Record as ActivationRecord};
use algebra::{
    AttemptAdmission, AttemptFact, CompensationEffectFact, OrderingAssessment, OrderingViolation,
    Registration,
};
use purity::PureAssessment;
use recovery::Decision as RecoveryDecision;

pub(crate) const ACTIVATION_RESULT_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("compensation/result/activation/retain"),
    "compensation::compose",
    BudgetDimension::ResultItem,
    SiteKind::Retention,
);
pub(crate) const REGISTRATION_RESULT_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("compensation/result/registration/retain"),
    "compensation::compose",
    BudgetDimension::ResultItem,
    SiteKind::Retention,
);
pub(crate) const ORDERING_RESULT_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("compensation/result/ordering/retain"),
    "compensation::compose",
    BudgetDimension::ResultItem,
    SiteKind::Retention,
);
pub(crate) const ATTEMPT_RESULT_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("compensation/result/attempt/retain"),
    "compensation::compose",
    BudgetDimension::ResultItem,
    SiteKind::Retention,
);
pub(crate) const EFFECT_RESULT_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("compensation/result/effect/retain"),
    "compensation::compose",
    BudgetDimension::ResultItem,
    SiteKind::Retention,
);
pub(crate) const RETRY_RESULT_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("compensation/result/retry/retain"),
    "compensation::compose",
    BudgetDimension::ResultItem,
    SiteKind::Retention,
);
pub(crate) const RECOVERY_RESULT_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("compensation/result/recovery/retain"),
    "compensation::compose",
    BudgetDimension::ResultItem,
    SiteKind::Retention,
);
pub(crate) const ALLOCATION_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("compensation/snapshot/allocate"),
    "compensation::compose",
    BudgetDimension::AccountedAllocation,
    SiteKind::Allocation,
);

/// Immutable composition maxima.
pub const OWNER_MAX: Limits = Limits {
    max_attempts: 10_000,
    max_effects: 10_000,
    max_retry_outcomes: 10_000,
    max_recovery_decisions: 10_000,
};

/// Caller-lowerable ceilings for one composed compensation snapshot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Limits {
    /// Retained admitted attempts.
    pub max_attempts: usize,
    /// Retained established compensation effects.
    pub max_effects: usize,
    /// Retained retry outcomes, including refusals.
    pub max_retry_outcomes: usize,
    /// Independently retained recovery-relation decisions.
    pub max_recovery_decisions: usize,
}

impl Limits {
    /// Immutable owner maxima.
    #[must_use]
    pub const fn owner_max() -> Self {
        OWNER_MAX
    }

    /// Clamp caller ceilings to owner maxima.
    #[must_use]
    pub const fn effective(self) -> Self {
        Self {
            max_attempts: min(self.max_attempts, OWNER_MAX.max_attempts),
            max_effects: min(self.max_effects, OWNER_MAX.max_effects),
            max_retry_outcomes: min(self.max_retry_outcomes, OWNER_MAX.max_retry_outcomes),
            max_recovery_decisions: min(
                self.max_recovery_decisions,
                OWNER_MAX.max_recovery_decisions,
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

/// One already-assessed atomic fact set offered to the aggregate boundary.
#[derive(Clone, Copy, Debug)]
pub struct Parts<'a> {
    /// Reported activation, or explicit absence before any activation report.
    pub activation: Option<&'a ActivationRecord>,
    /// Registration created from the paired established effect.
    pub registration: Option<&'a Registration>,
    /// Authored causal-order assessment, required for an active registration.
    pub ordering: Option<&'a OrderingAssessment>,
    /// Exact attempts retained independently from effects.
    pub attempts: &'a [AttemptFact],
    /// Exact established effects and their effect-only dispositions.
    pub effects: &'a [CompensationEffectFact],
    /// Every retry admission/refusal retained independently.
    pub retry_outcomes: &'a [AttemptAdmission],
    /// Independent full, partial, or other owner relation decisions.
    pub recoveries: &'a [RecoveryDecision],
}

/// Why an aggregate compensation assessment cannot report availability.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum UnavailableCause {
    ActivationAbsent,
    ActivationUnknown,
    RegistrationAbsent,
    OrderingAbsent,
    OrderingIncomplete,
    OrderingViolation(OrderingViolation),
    RetryBoundAbsent,
    RetryRegistrationMismatch,
    RetryBoundExceeded,
    RetryResourceExceeded,
    RecoveryAbsent,
    RecoveryPending,
    RecoveryIncomplete,
    InactiveFactsPresent,
}

/// Availability of the complete composed assessment, not recovery truth.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Availability {
    /// Every atomic requirement needed for this activation state returned a fact.
    Available,
    /// At least one atomic requirement was absent, incomplete, or refused.
    Unavailable(Vec<UnavailableCause>),
}

/// Immutable aggregate retaining every atomic fact side by side.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Snapshot {
    /// Exact activation fact, if one was reported.
    pub activation: Option<ActivationRecord>,
    /// Exact registration, if one was established.
    pub registration: Option<Registration>,
    /// Independent authored-order assessment.
    pub ordering: Option<OrderingAssessment>,
    /// Exact attempts, never collapsed into their effects.
    pub attempts: Vec<AttemptFact>,
    /// Exact established effects, never collapsed into their attempts.
    pub effects: Vec<CompensationEffectFact>,
    /// Exact retry results, including typed refusals.
    pub retry_outcomes: Vec<AttemptAdmission>,
    /// Independent recovery-relation decisions.
    pub recoveries: Vec<RecoveryDecision>,
    /// Aggregate availability derived only from the retained atomic outcomes.
    pub availability: Availability,
}

/// Composition population refused before any clone.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceRefusal {
    resource: Resource,
    observed: usize,
    limit: usize,
}

impl ResourceRefusal {
    /// Refused population.
    #[must_use]
    pub const fn resource(self) -> Resource {
        self.resource
    }

    /// Candidate population size.
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
            "compensation {:?} population {} exceeds effective limit {}",
            self.resource, self.observed, self.limit
        )
    }
}

impl std::error::Error for ResourceRefusal {}

/// Bounded aggregate population.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Resource {
    Attempts,
    Effects,
    RetryOutcomes,
    RecoveryDecisions,
}

/// Compose under the shared QProtocol caller budget.
///
/// Every optional or repeated result item is charged transactionally before
/// availability assessment or cloning begins. The module's immutable owner
/// maxima remain independently enforced by [`compose`].
pub fn compose_bounded(
    pure: PureAssessment,
    parts: Parts<'_>,
    limits: Limits,
    resources: &mut ResourceAccountant<'_>,
) -> OperationOutcome<Snapshot, ResourceRefusal> {
    resources.run_unit(
        UnitLabel::new("compensation::compose_bounded"),
        |resources| compose_bounded_unit(pure, parts, limits, resources),
    )
}

fn compose_bounded_unit(
    pure: PureAssessment,
    parts: Parts<'_>,
    limits: Limits,
    resources: &mut ResourceAccountant<'_>,
) -> OperationOutcome<Snapshot, ResourceRefusal> {
    let effective = limits.effective();
    let charges = [
        (ALLOCATION_SOURCE.id(), 1, None),
        (
            ACTIVATION_RESULT_SOURCE.id(),
            usize::from(parts.activation.is_some()),
            Some(1),
        ),
        (
            REGISTRATION_RESULT_SOURCE.id(),
            usize::from(parts.registration.is_some()),
            Some(1),
        ),
        (
            ORDERING_RESULT_SOURCE.id(),
            usize::from(parts.ordering.is_some()),
            Some(1),
        ),
        (
            ATTEMPT_RESULT_SOURCE.id(),
            parts.attempts.len(),
            Some(effective.max_attempts),
        ),
        (
            EFFECT_RESULT_SOURCE.id(),
            parts.effects.len(),
            Some(effective.max_effects),
        ),
        (
            RETRY_RESULT_SOURCE.id(),
            parts.retry_outcomes.len(),
            Some(effective.max_retry_outcomes),
        ),
        (
            RECOVERY_RESULT_SOURCE.id(),
            parts.recoveries.len(),
            Some(effective.max_recovery_decisions),
        ),
    ];
    if let Err(error) = resources.charge_all_mixed(&charges) {
        return OperationOutcome::from_charge(error);
    }
    OperationOutcome::from_domain(compose(pure, parts, effective))
}

/// Compose a complete snapshot without reinterpreting any atomic fact.
pub fn compose(
    _pure: PureAssessment,
    parts: Parts<'_>,
    limits: Limits,
) -> Result<Snapshot, ResourceRefusal> {
    let limits = limits.effective();
    precheck(
        Resource::Attempts,
        parts.attempts.len(),
        limits.max_attempts,
    )?;
    precheck(Resource::Effects, parts.effects.len(), limits.max_effects)?;
    precheck(
        Resource::RetryOutcomes,
        parts.retry_outcomes.len(),
        limits.max_retry_outcomes,
    )?;
    precheck(
        Resource::RecoveryDecisions,
        parts.recoveries.len(),
        limits.max_recovery_decisions,
    )?;
    let availability = Availability::from_parts(parts);
    Ok(Snapshot {
        activation: parts.activation.cloned(),
        registration: parts.registration.cloned(),
        ordering: parts.ordering.cloned(),
        attempts: parts.attempts.to_vec(),
        effects: parts.effects.to_vec(),
        retry_outcomes: parts.retry_outcomes.to_vec(),
        recoveries: parts.recoveries.to_vec(),
        availability,
    })
}

impl Availability {
    fn from_parts(parts: Parts<'_>) -> Self {
        let Some(activation) = parts.activation else {
            return Self::Unavailable(vec![UnavailableCause::ActivationAbsent]);
        };
        match activation.activation {
            Activation::Inactive => {
                if parts.registration.is_none()
                    && parts.ordering.is_none()
                    && parts.attempts.is_empty()
                    && parts.retry_outcomes.is_empty()
                    && parts.recoveries.is_empty()
                {
                    Self::Available
                } else {
                    Self::Unavailable(vec![UnavailableCause::InactiveFactsPresent])
                }
            }
            Activation::ActivationUnknown => {
                Self::Unavailable(vec![UnavailableCause::ActivationUnknown])
            }
            Activation::Triggered => {
                let mut causes = Vec::new();
                if parts.registration.is_none() {
                    causes.push(UnavailableCause::RegistrationAbsent);
                }
                match parts.ordering {
                    None => causes.push(UnavailableCause::OrderingAbsent),
                    Some(ordering) => {
                        if !ordering.complete() {
                            causes.push(UnavailableCause::OrderingIncomplete);
                        }
                        causes.extend(
                            ordering
                                .violations()
                                .iter()
                                .copied()
                                .map(UnavailableCause::OrderingViolation),
                        );
                    }
                }
                causes.extend(
                    parts
                        .retry_outcomes
                        .iter()
                        .filter_map(|outcome| match outcome {
                            AttemptAdmission::Admitted { .. } => None,
                            AttemptAdmission::RetryBoundAbsent { .. } => {
                                Some(UnavailableCause::RetryBoundAbsent)
                            }
                            AttemptAdmission::RegistrationMismatch { .. } => {
                                Some(UnavailableCause::RetryRegistrationMismatch)
                            }
                            AttemptAdmission::RetryBoundExceeded { .. } => {
                                Some(UnavailableCause::RetryBoundExceeded)
                            }
                            AttemptAdmission::ResourceLimitExceeded { .. } => {
                                Some(UnavailableCause::RetryResourceExceeded)
                            }
                        }),
                );
                if parts.recoveries.is_empty() {
                    causes.push(UnavailableCause::RecoveryAbsent);
                }
                causes.extend(
                    parts
                        .recoveries
                        .iter()
                        .filter_map(|decision| match decision {
                            RecoveryDecision::Satisfied(_) | RecoveryDecision::NotSatisfied(_) => {
                                None
                            }
                            RecoveryDecision::Pending(_) => Some(UnavailableCause::RecoveryPending),
                            RecoveryDecision::Incomplete(_) => {
                                Some(UnavailableCause::RecoveryIncomplete)
                            }
                        }),
                );
                if causes.is_empty() {
                    Self::Available
                } else {
                    Self::Unavailable(causes)
                }
            }
        }
    }
}

fn precheck(resource: Resource, observed: usize, limit: usize) -> Result<(), ResourceRefusal> {
    if observed > limit {
        Err(ResourceRefusal {
            resource,
            observed,
            limit,
        })
    } else {
        Ok(())
    }
}
