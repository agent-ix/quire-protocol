// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-003/FR-019..FR-024: graph-owned compensation assessment state.
//!
//! Every identity in this module is either a linked QSL coordinate or an
//! authority-qualified QObs subject. QSL state-evaluator reports remain in
//! their owner type so incomplete, refused, exhausted, and unsupported
//! outcomes cannot be approximated as Boolean protocol facts.

use quire_observation::authority::{observation as observation_record, position};
use quire_observation::QualifiedSubject;
use quire_spec_language::state::{EvaluationReport, StateView};

use super::algebra::{AttemptOutcome, EffectDisposition, OrderingViolation};
use super::registration::Registration;
use crate::model::v2::SlotHandle;

/// One successful mutation of the request-scoped compensation state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Transition {
    /// The candidate added a new exact fact.
    Applied,
    /// Every offered fact was an exact replay of retained content.
    Replay,
}

/// Evidence that was absent rather than false or refusing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MissingEvidence {
    /// No established forward effect supplied the selected registration.
    ForwardEffect,
    /// No prior attempt exists for an authored retry relation.
    EarlierAttempt,
    /// A selected forbidding commit has no admitted observation.
    Commit,
    /// No independently established compensation effect is available.
    CompensationEffect,
    /// The selected observation has no entry in the owner position ledger.
    Position,
}

/// Why a graph-derived compensation candidate is rejected.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Refusal {
    /// The declaration-local compensation coordinate is unknown.
    UnknownCompensation(u32),
    /// The offered runtime binding names another selected slot.
    UnexpectedSlot {
        /// Slot selected by the admitted compensation definition.
        expected: SlotHandle,
        /// Slot carried by the offered runtime binding.
        actual: SlotHandle,
    },
    /// The selected slot is absent from the admitted static binding definition.
    UnselectedSlot(SlotHandle),
    /// The selected QSL requirement has another owner-published kind.
    UnexpectedBindingKind(SlotHandle),
    /// The runtime value is not a QObs subject.
    WrongRuntimeValue(SlotHandle),
    /// The subject belongs to another admitted Producer authority.
    ForeignAuthority(SlotHandle),
    /// The subject has another producer-local kind.
    IncompatibleSubjectKind(SlotHandle),
    /// A replay reused one exact subject identity with different facts.
    ConflictingReplay,
    /// A new distinct attempt would exceed the exact authored finite bound.
    RetryBoundExceeded {
        /// Exact signed-64 bound retained from QSL.
        bound: i64,
        /// Candidate count, including the initial attempt.
        attempted: usize,
    },
    /// The admitted retry expression completed false for this candidate.
    ///
    /// The complete owner report is retained in the refusal so its limits,
    /// usage, accounting version, and Boolean verdict are not approximated.
    RetryNotAuthorized(Box<EvaluationReport>),
    /// The admitted QSL retry bound is not a positive signed-64 integer.
    InvalidRetryBound,
    /// The admitted compensation selects `commit never`.
    CommitNotSelected,
    /// The selected commit control is not an admitted QSL commit node.
    InvalidCommitSelection,
    /// An observation does not carry the runtime subject selected for its slot.
    ObservationSubjectMismatch,
    /// Observation and position-ledger clock authorities differ.
    PositionAuthorityMismatch,
}

/// Exact disposition of a graph-owned compensation operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Outcome {
    /// A new fact was applied or every offered fact was an exact replay.
    Completed(Transition),
    /// A false guard created no compensation activation or registration.
    GuardFalse(EvaluationReport),
    /// Required owner evidence is absent and no success was fabricated.
    Unavailable(MissingEvidence),
    /// Owner evaluation completed, but a required graph fact was absent.
    ///
    /// The complete report is preserved even though no graph mutation occurs.
    UnavailableAfterEvaluation {
        /// The graph fact required to complete the transition.
        missing: MissingEvidence,
        /// QSL's exact completed owner report.
        report: EvaluationReport,
    },
    /// QSL returned its exact incomplete, refused, or exhausted result.
    Source(EvaluationReport),
    /// QSL completed with a non-Boolean value at a Boolean compensation site.
    Unsupported(EvaluationReport),
    /// Protocol rejected a foreign, contradictory, or invalid candidate.
    Refused(Refusal),
}

/// One completed-true QSL activation retained for a compensation definition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Activation {
    compensation_handle: u32,
    view: StateView,
    report: EvaluationReport,
}

impl Activation {
    /// Returns QSL's declaration-local compensation coordinate.
    #[must_use]
    pub const fn compensation_handle(&self) -> u32 {
        self.compensation_handle
    }

    /// Returns the exact state view evaluated for this activation.
    #[must_use]
    pub const fn view(&self) -> &StateView {
        &self.view
    }

    /// Returns QSL's complete owner-accounted guard report.
    #[must_use]
    pub const fn report(&self) -> &EvaluationReport {
        &self.report
    }
}

/// One exact admitted compensation attempt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Attempt {
    registration: Registration,
    subject: QualifiedSubject,
    established_effect: Option<QualifiedSubject>,
    outcome: Option<AttemptOutcome>,
    retry_view: Option<StateView>,
    retry: Option<EvaluationReport>,
}

impl Attempt {
    /// Returns the exact graph-derived registration.
    #[must_use]
    pub const fn registration(&self) -> &Registration {
        &self.registration
    }

    /// Returns QObs's complete authority-qualified attempt identity.
    #[must_use]
    pub const fn subject(&self) -> &QualifiedSubject {
        &self.subject
    }

    /// Returns the independently established related effect, when present.
    #[must_use]
    pub const fn established_effect(&self) -> Option<&QualifiedSubject> {
        self.established_effect.as_ref()
    }

    /// Returns the exact FR-236 outcome, or absence when unestablished.
    #[must_use]
    pub const fn outcome(&self) -> Option<AttemptOutcome> {
        self.outcome
    }

    /// Returns the exact QSL state view for an authored retry relation.
    #[must_use]
    pub const fn retry_view(&self) -> Option<&StateView> {
        self.retry_view.as_ref()
    }

    /// Returns QSL's exact retry-expression report for a retry.
    #[must_use]
    pub const fn retry(&self) -> Option<&EvaluationReport> {
        self.retry.as_ref()
    }
}

/// One independently established compensation effect.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Effect {
    registration: Registration,
    subject: QualifiedSubject,
    disposition: Option<EffectDisposition>,
}

impl Effect {
    /// Returns the exact graph-derived registration.
    #[must_use]
    pub const fn registration(&self) -> &Registration {
        &self.registration
    }

    /// Returns QObs's complete authority-qualified effect identity.
    #[must_use]
    pub const fn subject(&self) -> &QualifiedSubject {
        &self.subject
    }

    /// Returns the effect-only duplicate disposition, when established.
    #[must_use]
    pub const fn disposition(&self) -> Option<EffectDisposition> {
        self.disposition
    }
}

/// One exact selected commit observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Commit {
    registration: Registration,
    subject: QualifiedSubject,
}

impl Commit {
    /// Returns the exact graph-derived registration.
    #[must_use]
    pub const fn registration(&self) -> &Registration {
        &self.registration
    }

    /// Returns QObs's complete authority-qualified commit identity.
    #[must_use]
    pub const fn subject(&self) -> &QualifiedSubject {
        &self.subject
    }
}

/// One retained result of the exact authored recovery expression.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Recovery {
    registration: Registration,
    view: StateView,
    report: EvaluationReport,
}

impl Recovery {
    /// Returns the exact graph-derived registration.
    #[must_use]
    pub const fn registration(&self) -> &Registration {
        &self.registration
    }

    /// Returns the exact QSL state view containing the evaluated operands.
    #[must_use]
    pub const fn view(&self) -> &StateView {
        &self.view
    }

    /// Returns QSL's complete owner-accounted evaluation report.
    #[must_use]
    pub const fn report(&self) -> &EvaluationReport {
        &self.report
    }
}

/// Commit-order result derived from owner observation positions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Ordering {
    registration: Registration,
    forward: observation_record::View,
    effect: observation_record::View,
    commit: Option<observation_record::View>,
    positions: position::View,
    violations: Vec<OrderingViolation>,
}

impl Ordering {
    /// Returns the exact graph-derived registration.
    #[must_use]
    pub const fn registration(&self) -> &Registration {
        &self.registration
    }

    /// Returns the exact forward-effect observation assessed.
    #[must_use]
    pub const fn forward(&self) -> &observation_record::View {
        &self.forward
    }

    /// Returns the exact compensation-effect observation assessed.
    #[must_use]
    pub const fn effect(&self) -> &observation_record::View {
        &self.effect
    }

    /// Returns the exact selected commit observation, when one was authored.
    #[must_use]
    pub const fn commit(&self) -> Option<&observation_record::View> {
        self.commit.as_ref()
    }

    /// Returns the exact owner position ledger used for causal comparison.
    #[must_use]
    pub const fn positions(&self) -> &position::View {
        &self.positions
    }

    /// Returns all distinct violations in stable requirement order.
    #[must_use]
    pub fn violations(&self) -> &[OrderingViolation] {
        &self.violations
    }
}

/// Owned externally observable state for one non-retargetable Engine.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Snapshot {
    activations: Vec<Activation>,
    attempts: Vec<Attempt>,
    effects: Vec<Effect>,
    commits: Vec<Commit>,
    recoveries: Vec<Recovery>,
    orderings: Vec<Ordering>,
}

impl Snapshot {
    /// Returns completed-true activation reports in accepted operation order.
    #[must_use]
    pub fn activations(&self) -> &[Activation] {
        &self.activations
    }

    /// Returns distinct attempts in accepted operation order.
    #[must_use]
    pub fn attempts(&self) -> &[Attempt] {
        &self.attempts
    }

    /// Returns distinct established effects in accepted operation order.
    #[must_use]
    pub fn effects(&self) -> &[Effect] {
        &self.effects
    }

    /// Returns exact selected commit observations.
    #[must_use]
    pub fn commits(&self) -> &[Commit] {
        &self.commits
    }

    /// Returns exact QSL recovery-expression reports.
    #[must_use]
    pub fn recoveries(&self) -> &[Recovery] {
        &self.recoveries
    }

    /// Returns owner-position-derived ordering assessments.
    #[must_use]
    pub fn orderings(&self) -> &[Ordering] {
        &self.orderings
    }

    pub(crate) fn attempts_for(&self, registration: &Registration) -> usize {
        self.attempts
            .iter()
            .filter(|attempt| attempt.registration == *registration)
            .count()
    }

    pub(crate) fn retain_activation(
        &mut self,
        compensation_handle: u32,
        view: StateView,
        report: EvaluationReport,
    ) -> Result<Transition, Refusal> {
        if let Some(existing) = self
            .activations
            .iter()
            .find(|activation| activation.compensation_handle == compensation_handle)
        {
            return if existing.view == view && existing.report == report {
                Ok(Transition::Replay)
            } else {
                Err(Refusal::ConflictingReplay)
            };
        }
        self.activations.push(Activation {
            compensation_handle,
            view,
            report,
        });
        Ok(Transition::Applied)
    }

    pub(crate) fn attempt(&self, subject: &QualifiedSubject) -> Option<&Attempt> {
        self.attempts
            .iter()
            .find(|attempt| attempt.subject == *subject)
    }

    pub(crate) fn effect(&self, subject: &QualifiedSubject) -> Option<&Effect> {
        self.effects
            .iter()
            .find(|effect| effect.subject == *subject)
    }

    pub(crate) fn commit(&self, registration: &Registration) -> Option<&Commit> {
        self.commits
            .iter()
            .find(|commit| commit.registration == *registration)
    }

    pub(crate) fn push_attempt(
        &mut self,
        registration: Registration,
        subject: QualifiedSubject,
        established_effect: Option<QualifiedSubject>,
        outcome: Option<AttemptOutcome>,
        retry_view: Option<StateView>,
        retry: Option<EvaluationReport>,
    ) {
        self.attempts.push(Attempt {
            registration,
            subject,
            established_effect,
            outcome,
            retry_view,
            retry,
        });
    }

    pub(crate) fn push_effect(
        &mut self,
        registration: Registration,
        subject: QualifiedSubject,
        disposition: Option<EffectDisposition>,
    ) {
        self.effects.push(Effect {
            registration,
            subject,
            disposition,
        });
    }

    pub(crate) fn push_commit(&mut self, registration: Registration, subject: QualifiedSubject) {
        self.commits.push(Commit {
            registration,
            subject,
        });
    }

    pub(crate) fn retain_recovery(
        &mut self,
        registration: Registration,
        view: StateView,
        report: EvaluationReport,
    ) -> Result<Transition, Refusal> {
        if let Some(existing) = self
            .recoveries
            .iter()
            .find(|recovery| recovery.registration == registration)
        {
            return if existing.view == view && existing.report == report {
                Ok(Transition::Replay)
            } else {
                Err(Refusal::ConflictingReplay)
            };
        }
        self.recoveries.push(Recovery {
            registration,
            view,
            report,
        });
        Ok(Transition::Applied)
    }

    pub(crate) fn retain_ordering(
        &mut self,
        registration: Registration,
        forward: observation_record::View,
        effect: observation_record::View,
        commit: Option<observation_record::View>,
        positions: position::View,
        violations: Vec<OrderingViolation>,
    ) -> Result<Transition, Refusal> {
        if let Some(existing) = self
            .orderings
            .iter()
            .find(|ordering| ordering.registration == registration)
        {
            return if existing.forward == forward
                && existing.effect == effect
                && existing.commit == commit
                && existing.positions == positions
                && existing.violations == violations
            {
                Ok(Transition::Replay)
            } else {
                Err(Refusal::ConflictingReplay)
            };
        }
        self.orderings.push(Ordering {
            registration,
            forward,
            effect,
            commit,
            positions,
            violations,
        });
        Ok(Transition::Applied)
    }
}
