// SPDX-License-Identifier: AGPL-3.0-or-later
//! Settled incremental conformance and exact batch-path comparison.

use quire_observation::authority::completeness;

use crate::assessment::{AxisInputs, GlobalConformanceInput};
use crate::bounds::inventory::{ChargeError, OperationOutcome, ResourceAccountant};
use crate::closure::AssessmentExecution;

use super::batch::{
    assess, BoundObservation, Limits, Model, ObligationDisposition, Report, Request, Usage,
    OBSERVATION_SOURCE, OPERATION_SOURCE,
};
use super::{ErrorCode, Refusal, Result};

/// Producer-owned context shared by one bounded incremental assessment step.
#[derive(Clone, Copy, Debug)]
pub struct IncrementalContext<'a> {
    /// Four independently supplied progress and closure views.
    pub axes: AxisInputs<'a>,
    /// Exact producer-owned completeness view.
    pub completeness: &'a completeness::View,
    /// Exact global-conformance premise set.
    pub global_conformance: &'a GlobalConformanceInput,
    /// Execution disposition supplied by its owner.
    pub execution: AssessmentExecution,
    /// Immutable-owner-clamped conformance limits.
    pub limits: Limits,
}

/// Incremental assessor that evaluates every admitted prefix.
#[derive(Clone, Debug)]
pub struct Incremental<'a> {
    model: Model,
    observations: Vec<BoundObservation<'a>>,
    previous: Option<Report>,
}

impl<'a> Incremental<'a> {
    /// Start a new incremental path over one already validated model.
    #[must_use]
    pub const fn new(model: Model) -> Self {
        Self {
            model,
            observations: Vec::new(),
            previous: None,
        }
    }

    /// Assess the next admitted prefix immediately.
    ///
    /// A failing step is transactional: it does not mutate the retained prefix.
    pub fn push(
        &mut self,
        observation: BoundObservation<'a>,
        axes: AxisInputs<'a>,
        completeness: &'a completeness::View,
        global_conformance: &'a GlobalConformanceInput,
        execution: AssessmentExecution,
        limits: Limits,
    ) -> Result<&Report> {
        let mut candidate = self.observations.clone();
        candidate.push(observation);
        let report = assess(
            Request {
                model: &self.model,
                observations: &candidate,
                axes,
                completeness,
                global_conformance,
                execution,
            },
            limits,
        )?;
        if let Some(previous) = &self.previous {
            ensure_settled_unchanged(previous, &report)?;
        }
        self.observations = candidate;
        self.previous = Some(report);
        match self.previous.as_ref() {
            Some(report) => Ok(report),
            None => Err(Refusal::new(
                ErrorCode::SettlementReopened,
                "incremental report was not retained",
                None,
                Usage::default(),
                self.model.protocol_identity().as_bytes(),
            )),
        }
    }

    /// Assess and retain the next prefix under the shared caller budget.
    ///
    /// The whole candidate prefix and model operation population are charged
    /// transactionally before cloning the retained prefix or assessing it.
    pub fn push_bounded<'request>(
        &'request mut self,
        observation: BoundObservation<'a>,
        context: IncrementalContext<'a>,
        resources: &mut ResourceAccountant<'_>,
    ) -> OperationOutcome<&'request Report, Refusal> {
        let effective = context.limits.effective();
        let Some(candidate_len) = self.observations.len().checked_add(1) else {
            return OperationOutcome::from_charge(ChargeError::CounterOverflow {
                site: OBSERVATION_SOURCE.id(),
                consumed: self.observations.len(),
                requested: 1,
            });
        };
        let charges = [
            (
                OBSERVATION_SOURCE.id(),
                candidate_len,
                effective.max_bindings,
            ),
            (
                OPERATION_SOURCE.id(),
                self.model.obligations().len(),
                effective.max_obligations,
            ),
        ];
        if let Err(error) = resources.charge_all_with_limits(&charges) {
            return OperationOutcome::from_charge(error);
        }
        OperationOutcome::from_domain(self.push(
            observation,
            context.axes,
            context.completeness,
            context.global_conformance,
            context.execution,
            effective,
        ))
    }

    /// Re-assess the complete retained prefix under final producer-owned axes.
    pub fn finish(
        &mut self,
        axes: AxisInputs<'a>,
        completeness: &'a completeness::View,
        global_conformance: &'a GlobalConformanceInput,
        execution: AssessmentExecution,
        limits: Limits,
    ) -> Result<&Report> {
        let report = assess(
            Request {
                model: &self.model,
                observations: &self.observations,
                axes,
                completeness,
                global_conformance,
                execution,
            },
            limits,
        )?;
        if let Some(previous) = &self.previous {
            ensure_settled_unchanged(previous, &report)?;
        }
        self.previous = Some(report);
        match self.previous.as_ref() {
            Some(report) => Ok(report),
            None => Err(Refusal::new(
                ErrorCode::SettlementReopened,
                "final incremental report was not retained",
                None,
                Usage::default(),
                self.model.protocol_identity().as_bytes(),
            )),
        }
    }

    /// Re-assess the retained final prefix under the shared caller budget.
    ///
    /// The complete retained prefix and model operation population are charged
    /// transactionally before assessment constructs a replacement report.
    pub fn finish_bounded<'request>(
        &'request mut self,
        context: IncrementalContext<'a>,
        resources: &mut ResourceAccountant<'_>,
    ) -> OperationOutcome<&'request Report, Refusal> {
        let effective = context.limits.effective();
        let charges = [
            (
                OBSERVATION_SOURCE.id(),
                self.observations.len(),
                effective.max_bindings,
            ),
            (
                OPERATION_SOURCE.id(),
                self.model.obligations().len(),
                effective.max_obligations,
            ),
        ];
        if let Err(error) = resources.charge_all_with_limits(&charges) {
            return OperationOutcome::from_charge(error);
        }
        OperationOutcome::from_domain(self.finish(
            context.axes,
            context.completeness,
            context.global_conformance,
            context.execution,
            effective,
        ))
    }

    /// Exact retained admitted prefix in presentation order.
    #[must_use]
    pub fn observations(&self) -> &[BoundObservation<'a>] {
        &self.observations
    }

    /// Most recent prefix report, if at least one prefix was assessed.
    #[must_use]
    pub const fn report(&self) -> Option<&Report> {
        self.previous.as_ref()
    }
}

fn ensure_settled_unchanged(previous: &Report, current: &Report) -> Result<()> {
    for earlier in &previous.obligations {
        if earlier.disposition == ObligationDisposition::Pending {
            continue;
        }
        let later = current
            .obligations
            .iter()
            .find(|candidate| candidate.obligation == earlier.obligation);
        if later.is_none_or(|later| later.disposition != earlier.disposition) {
            let mut discriminator = Vec::new();
            push_part(&mut discriminator, earlier.obligation.as_str().as_bytes());
            push_part(&mut discriminator, earlier.disposition.tag().as_bytes());
            if let Some(later) = later {
                push_part(&mut discriminator, later.disposition.tag().as_bytes());
            }
            return Err(Refusal::new(
                ErrorCode::SettlementReopened,
                "a settled obligation changed without an explicit correction",
                Some(&earlier.obligation),
                current.usage,
                &discriminator,
            ));
        }
    }
    Ok(())
}

/// Compare batch and incremental semantic values for one exact admitted set.
///
/// Presentation order is deliberately excluded from value equality while the
/// multiset of exact retained QObs bytes and correlation bindings must match.
pub fn compare(batch: &Report, incremental: &Report) -> Result<()> {
    if semantically_equal(batch, incremental) {
        return Ok(());
    }
    let mut discriminator = Vec::new();
    push_part(&mut discriminator, batch.protocol_identity.as_bytes());
    push_part(&mut discriminator, global_tag(batch.global).as_bytes());
    push_part(
        &mut discriminator,
        global_tag(incremental.global).as_bytes(),
    );
    for result in &batch.obligations {
        push_part(&mut discriminator, result.obligation.as_str().as_bytes());
        push_part(&mut discriminator, result.disposition.tag().as_bytes());
    }
    for result in &incremental.obligations {
        push_part(&mut discriminator, result.obligation.as_str().as_bytes());
        push_part(&mut discriminator, result.disposition.tag().as_bytes());
    }
    Err(Refusal::new(
        ErrorCode::PathDivergence,
        "batch and incremental conformance values diverged",
        None,
        incremental.usage,
        &discriminator,
    ))
}

fn semantically_equal(left: &Report, right: &Report) -> bool {
    left.protocol_identity == right.protocol_identity
        && left.obligations == right.obligations
        && left.adverse == right.adverse
        && left.incomplete == right.incomplete
        && left.global_premise_defects == right.global_premise_defects
        && left.global == right.global
        && left.premises == right.premises
        && observation_multiset(left) == observation_multiset(right)
}

#[derive(Eq, Ord, PartialEq, PartialOrd)]
struct ObservationKey<'a> {
    obligation: &'a str,
    workflow: &'a str,
    effect: Option<&'a str>,
    document_identity: &'a str,
    bytes: &'a [u8],
}

fn observation_multiset(report: &Report) -> Vec<ObservationKey<'_>> {
    let mut observations: Vec<_> = report
        .observations
        .iter()
        .map(|observation| ObservationKey {
            obligation: observation.obligation.as_str(),
            workflow: observation.workflow_identity.as_ref(),
            effect: observation.effect_identity.as_deref(),
            document_identity: observation.document.identity.as_ref(),
            bytes: observation.document.bytes.as_slice(),
        })
        .collect();
    observations.sort_unstable();
    observations
}

const fn global_tag(value: super::batch::GlobalDisposition) -> &'static str {
    match value {
        super::batch::GlobalDisposition::Conforms => "conforms",
        super::batch::GlobalDisposition::Pending => "pending",
        super::batch::GlobalDisposition::Violates => "violates",
        super::batch::GlobalDisposition::Unavailable => "unavailable",
    }
}

fn push_part(target: &mut Vec<u8>, value: &[u8]) {
    let length = u64::try_from(value.len()).unwrap_or(u64::MAX);
    target.extend_from_slice(&length.to_be_bytes());
    target.extend_from_slice(value);
}
