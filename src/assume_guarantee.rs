// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-200, FR-201, FR-202, FR-203, FR-204: dispatch of `composition` and
//! `refinement` claims to the backend QSL's registry selects.
//!
//! QSL decides every composition and refinement answer: the product
//! construction, the premise discharge and the fixed-point slot are QSL's
//! (QSL FR-530, `quire-specification` FR-176, FR-177, FR-489). This module
//! binds the claim to its component subjects and its four premise lists, asks
//! the one backend FR-290 names through the same registry and provider
//! boundary as FR-150 ([`ProviderBoundary`]), and carries what comes back
//! unchanged. It builds no composition graph, discharges no premise and
//! evaluates no refinement relation.
//!
//! The only decisions made here are inclusion tests on identities: a result
//! may not name a subject the request did not name, may not retain premise
//! lists that differ from the request's, and a refinement result may not
//! retain an assumption identity the request did not state.

use qsl_foundation::BackendId;
use qsl_replay::ItemLabel;
use qsl_replay::{SettlementBasis, Truth};
use qsl_semantics::check::Capability;
use quire_contract_model::CheckedNodeId;

use crate::closure::AssessmentExecution;
use crate::declarations::Declarations;
use crate::dispatch::{
    select, Claim, DispositionRefusal, Fragment, ProtocolOutcome, ProviderBoundary,
};
use crate::disposition::{settle_claim, ClaimSettlement};

/// The two claim kinds this module dispatches.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AssumeGuaranteeKind {
    /// `composition`.
    Composition,
    /// `refinement`.
    Refinement,
}

impl AssumeGuaranteeKind {
    /// QSL's capability kind for the claim kind.
    #[must_use]
    pub const fn capability(self) -> Capability {
        match self {
            Self::Composition => Capability::Composition,
            Self::Refinement => Capability::Refinement,
        }
    }
}

/// One of the four premise lists of `quire-specification` FR-489.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PremiseList {
    /// `environment`.
    Environment,
    /// `relies`.
    Relies,
    /// `liveness_dependencies`.
    LivenessDependencies,
    /// `fairness`.
    Fairness,
}

/// The four premise lists, kept separate (FR-201). `T` is one entry as the
/// requester states it; entries are compared and forwarded as stated.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PremiseLists<T> {
    /// The environment premises.
    pub environment: Vec<T>,
    /// The rely premises.
    pub relies: Vec<T>,
    /// The liveness-dependency premises.
    pub liveness_dependencies: Vec<T>,
    /// The fairness premises.
    pub fairness: Vec<T>,
}

impl<T> Default for PremiseLists<T> {
    fn default() -> Self {
        Self {
            environment: Vec::new(),
            relies: Vec::new(),
            liveness_dependencies: Vec::new(),
            fairness: Vec::new(),
        }
    }
}

impl<T: PartialEq> PremiseLists<T> {
    /// The first list of `self` that differs from the same list of `other`.
    #[must_use]
    pub fn first_difference(&self, other: &Self) -> Option<PremiseList> {
        [
            (
                PremiseList::Environment,
                &self.environment,
                &other.environment,
            ),
            (PremiseList::Relies, &self.relies, &other.relies),
            (
                PremiseList::LivenessDependencies,
                &self.liveness_dependencies,
                &other.liveness_dependencies,
            ),
            (PremiseList::Fairness, &self.fairness, &other.fairness),
        ]
        .into_iter()
        .find_map(|(list, left, right)| (left != right).then_some(list))
    }
}

/// One requested `composition` or `refinement` claim.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssumeGuaranteeRequest<T> {
    /// The claim kind.
    pub kind: AssumeGuaranteeKind,
    /// The admitted fragment the claim ranges over.
    pub fragment: Fragment,
    /// The component subjects: the component protocols of a composition, the
    /// implementation and specification protocols of a refinement.
    pub subjects: Vec<CheckedNodeId>,
    /// The backend the request names, when it names one.
    pub backend: Option<BackendId>,
    /// The identities of the assumptions the claim is requested under.
    pub assumptions: Vec<CheckedNodeId>,
    /// The four premise lists; empty for a refinement.
    pub premises: PremiseLists<T>,
    /// The declared dependency cycles, each as the checked node ids of its
    /// guarantees (FR-203). The cycle is a subject of the claim, never an
    /// assumption.
    pub cycles: Vec<Vec<CheckedNodeId>>,
    /// The source identities the disposition retains (FR-030).
    pub sources: Vec<CheckedNodeId>,
}

/// What a backend is sent and what a disposition retains: the request exactly
/// as made, and the backend FR-290 selected for it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssumeGuaranteeBinding<T> {
    /// The request, unchanged.
    pub request: AssumeGuaranteeRequest<T>,
    /// The backend the claim was selected against, once FR-290 selects one.
    pub selected: Option<BackendId>,
}

/// What the dispatcher reads of a backend's composition or refinement result.
/// The result itself is QSL's and is carried unchanged.
pub trait AssumeGuaranteeResult<T> {
    /// The result label, as QSL's backend states it.
    fn label(&self) -> ItemLabel;
    /// The assessment-execution value the result states, when it states one.
    fn stated_execution(&self) -> Option<AssessmentExecution>;
    /// The component subjects the result names.
    fn subjects(&self) -> &[CheckedNodeId];
    /// The premise lists the result retains, when it retains them.
    fn retained_premises(&self) -> Option<&PremiseLists<T>>;
    /// The assumption identities the result retains; `None` when the result
    /// omits the field.
    fn retained_assumptions(&self) -> Option<&[CheckedNodeId]>;
}

/// The values a result's label reads into (FR-202). A value is `None` where
/// no owner table derives it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LabelReading {
    /// The label, as the backend returned it.
    pub label: ItemLabel,
    /// The FR-241 assessment-execution value.
    pub execution: Option<AssessmentExecution>,
    /// The FR-242 truth.
    pub truth: Option<Truth>,
    /// The FR-243 settlement basis.
    pub basis: Option<SettlementBasis>,
}

/// Read a composition result's label through the `quire-specification`
/// FR-360 table (execution and truth) and the FR-489 Settlement table (basis).
///
/// `failed` states its own execution, `failed` or `resource-incomplete`, and
/// the backend's statement is read only when it is one of those two. A label
/// the tables do not contain, `incomplete` included (QSL FR-331 states an
/// incomplete result as `failed`), reads into nothing. No value is read from
/// a certificate, counterexample, witness or discharge record, and `refused`
/// is never derived.
#[must_use]
pub fn read_composition_label(
    label: ItemLabel,
    stated: Option<AssessmentExecution>,
) -> LabelReading {
    let (execution, truth, basis) = match label {
        ItemLabel::Proved => (
            Some(AssessmentExecution::Completed),
            Some(Truth::Satisfied),
            Some(SettlementBasis::DecisiveWitness),
        ),
        ItemLabel::Inconclusive => (
            Some(AssessmentExecution::Completed),
            Some(Truth::Pending),
            Some(SettlementBasis::Unsettled),
        ),
        ItemLabel::Unsupported => (
            Some(AssessmentExecution::Unsupported),
            Some(Truth::Unavailable),
            Some(SettlementBasis::Unavailable),
        ),
        ItemLabel::Failed => (
            stated.filter(|execution| {
                matches!(
                    execution,
                    AssessmentExecution::Failed | AssessmentExecution::ResourceIncomplete
                )
            }),
            Some(Truth::Unavailable),
            Some(SettlementBasis::Unavailable),
        ),
        _ => (None, None, None),
    };
    LabelReading {
        label,
        execution,
        truth,
        basis,
    }
}

/// A refinement label has no owner table into the axes (FR-202): it is
/// carried and reads into nothing.
#[must_use]
pub const fn read_refinement_label(label: ItemLabel) -> LabelReading {
    LabelReading {
        label,
        execution: None,
        truth: None,
        basis: None,
    }
}

/// Bind `request` to the checked package's declarations (FR-030, FR-200): the
/// fragment, subjects, assumption identities and source identities must each
/// be declared by the package, and no guarantee of a declared cycle may be
/// offered as an assumption (FR-203).
///
/// # Errors
///
/// [`DispositionRefusal`] naming the absent, undeclared or offending node.
pub fn bind<T: Clone>(
    declarations: &Declarations,
    request: &AssumeGuaranteeRequest<T>,
) -> Result<AssumeGuaranteeBinding<T>, DispositionRefusal> {
    if request.fragment.channels.is_empty() && request.fragment.roles.is_empty() {
        return Err(DispositionRefusal::NoFragment);
    }
    for node in &request.fragment.channels {
        if declarations.channel(node).is_none() {
            return Err(DispositionRefusal::UndeclaredChannel(node.clone()));
        }
    }
    for node in &request.fragment.roles {
        if declarations.role(node).is_none() {
            return Err(DispositionRefusal::UndeclaredRole(node.clone()));
        }
    }
    if request.subjects.is_empty() {
        return Err(DispositionRefusal::NoSubjects);
    }
    if let Some(node) = request
        .subjects
        .iter()
        .find(|node| !declarations.declares(node))
    {
        return Err(DispositionRefusal::UndeclaredSubject(node.clone()));
    }
    if let Some(node) = request
        .assumptions
        .iter()
        .find(|node| !declarations.declares(node))
    {
        return Err(DispositionRefusal::UndeclaredAssumption(node.clone()));
    }
    if let Some(node) = cycle_member(&request.cycles, &request.assumptions) {
        return Err(DispositionRefusal::CycleGuaranteeAsAssumption(node));
    }
    if let Some(node) = request
        .sources
        .iter()
        .find(|node| !declarations.declares(node))
    {
        return Err(DispositionRefusal::UndeclaredSource(node.clone()));
    }
    Ok(AssumeGuaranteeBinding {
        request: request.clone(),
        selected: None,
    })
}

/// The first of `offered` that is a guarantee of one of `cycles`.
fn cycle_member(cycles: &[Vec<CheckedNodeId>], offered: &[CheckedNodeId]) -> Option<CheckedNodeId> {
    offered
        .iter()
        .find(|node| cycles.iter().any(|cycle| cycle.contains(node)))
        .cloned()
}

/// Settle one claim: bind it, take FR-290's candidate set from QSL's registry,
/// send the claim to the one candidate, and carry what returns.
pub fn dispatch<A, C, I, T, R>(
    boundary: &mut ProviderBoundary<'_, Claim<A, C, I, T>, R>,
    declarations: &Declarations,
    request: &AssumeGuaranteeRequest<T>,
) -> ProtocolOutcome<R, AssumeGuaranteeBinding<T>>
where
    A: Clone,
    C: Clone,
    I: Clone,
    T: Clone + PartialEq,
    R: AssumeGuaranteeResult<T>,
{
    let binding = match bind(declarations, request) {
        Ok(binding) => binding,
        Err(refusal) => return ProtocolOutcome::Refused(refusal),
    };
    let (selected, mut binding) = match select(
        boundary.registry(),
        request.kind.capability(),
        request.backend.as_ref(),
        binding,
    ) {
        Ok(selected) => selected,
        Err(outcome) => return *outcome,
    };
    binding.selected = Some(selected.clone());
    let Some(provider) = boundary.provider_mut(&selected) else {
        return ProtocolOutcome::Refused(DispositionRefusal::NoProvider(selected));
    };
    let claim = Claim::AssumeGuarantee(binding.clone());
    let settlement = settle_claim(provider.as_mut(), &claim, binding);
    if let ClaimSettlement::Result { result, premises } = &settlement {
        if let Some(refusal) = carry_refusal(&premises.request, result) {
            return ProtocolOutcome::Refused(refusal);
        }
    }
    ProtocolOutcome::Settled(settlement)
}

impl<T, R: AssumeGuaranteeResult<T>> ProtocolOutcome<R, AssumeGuaranteeBinding<T>> {
    /// The reading of a carried result's label (FR-202). A composition label
    /// reads through the owner tables and a refinement label into nothing;
    /// the dispatcher has already refused a result whose stated execution
    /// contradicts its label, so the reading is the disposition's own.
    #[must_use]
    pub fn label_reading(&self) -> Option<LabelReading> {
        let Self::Settled(ClaimSettlement::Result { result, premises }) = self else {
            return None;
        };
        Some(match premises.request.kind {
            AssumeGuaranteeKind::Composition => {
                read_composition_label(result.label(), result.stated_execution())
            }
            AssumeGuaranteeKind::Refinement => read_refinement_label(result.label()),
        })
    }
}

/// The labels on which a verdict rests: a decisive label needs a source
/// identity to be attributed to (FR-030).
const fn is_decisive(label: ItemLabel) -> bool {
    matches!(label, ItemLabel::Proved | ItemLabel::Refuted)
}

/// The identities of `held` that `allowed` does not contain.
fn extra(held: &[CheckedNodeId], allowed: &[CheckedNodeId]) -> Vec<CheckedNodeId> {
    held.iter()
        .filter(|identity| !allowed.contains(identity))
        .cloned()
        .collect()
}

/// FR-201, FR-202, FR-203, FR-204, FR-030: a result is carried only when it
/// was established for the request it is attributed to. Identities are
/// compared by inclusion and the lists by equality; nothing is entailed,
/// ordered or discharged.
fn carry_refusal<T: PartialEq, R: AssumeGuaranteeResult<T>>(
    request: &AssumeGuaranteeRequest<T>,
    result: &R,
) -> Option<DispositionRefusal> {
    if is_decisive(result.label()) && request.sources.is_empty() {
        return Some(DispositionRefusal::NoSourceIdentity);
    }
    if request.kind == AssumeGuaranteeKind::Composition {
        let reading = read_composition_label(result.label(), result.stated_execution());
        if reading.truth.is_some()
            && result.stated_execution().is_some()
            && reading.execution != result.stated_execution()
        {
            return Some(DispositionRefusal::ContradictoryExecution(result.label()));
        }
    }
    let foreign = extra(result.subjects(), &request.subjects);
    if !foreign.is_empty() {
        return Some(DispositionRefusal::IdentityMismatch(
            IdentityMismatch::Subjects(foreign),
        ));
    }
    match result.retained_premises() {
        Some(retained) => {
            if let Some(list) = retained.first_difference(&request.premises) {
                return Some(DispositionRefusal::RetainedPremiseList(list));
            }
        }
        None if request.kind == AssumeGuaranteeKind::Composition => {
            return Some(DispositionRefusal::IdentityMismatch(
                IdentityMismatch::AbsentPremiseLists,
            ));
        }
        None => {}
    }
    let retained = result.retained_assumptions();
    if let Some(retained) = retained {
        if let Some(node) = cycle_member(&request.cycles, retained) {
            return Some(DispositionRefusal::CycleGuaranteeAsAssumption(node));
        }
        let stronger = extra(retained, &request.assumptions);
        if !stronger.is_empty() {
            return Some(DispositionRefusal::IdentityMismatch(
                IdentityMismatch::Assumptions(stronger),
            ));
        }
    } else {
        return Some(DispositionRefusal::IdentityMismatch(
            IdentityMismatch::AbsentAssumptions,
        ));
    }
    None
}

/// What an `identity-mismatch` refusal names (FR-008's cause of that name).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IdentityMismatch {
    /// The result names component subjects the request did not; each is
    /// listed.
    Subjects(Vec<CheckedNodeId>),
    /// The result retains assumption identities the request did not state: it
    /// was established under stronger assumptions (FR-204); each is listed.
    Assumptions(Vec<CheckedNodeId>),
    /// The result omits its retained assumption identities, which compare
    /// only to absence (FR-030-AC-4).
    AbsentAssumptions,
    /// The result omits its retained premise lists.
    AbsentPremiseLists,
}
