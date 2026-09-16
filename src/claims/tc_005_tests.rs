// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-005: exact premise-bound dispatch through QProtocol's closed capability.

use std::collections::HashSet;

use ix_trace_rs::trace;
use quire_protocol::assessment::GlobalConformanceInput;
use quire_protocol::bounds::inventory::{
    production_inventory, Budget, BudgetDimension, OperationOutcome, ResourceAccountant,
};
use quire_protocol::claims::dispatch::{
    dispatch_bounded, dispatch_from_conformance, ClaimRequest, Decision, FailureCode, Limits,
};
use quire_protocol::claims::premises::{
    Assumption, AssumptionState, BackendSelection, Component, CorrespondenceAuthority,
    DerivedArtifact, PremiseAxis, PremiseDefectKind, PremiseSet,
};
use quire_protocol::claims::Claim;
use quire_protocol::closure::{AssessmentExecution, GlobalConformanceClosure};
use quire_protocol::conformance::{
    GlobalDisposition, PremiseSnapshot, Report as ConformanceReport, RetainedDocument, Usage,
};
use quire_protocol::refusal::{
    PartialOutput, PrerequisiteState, ProtocolCauseKind, ProtocolStage, RefusalCause,
};

fn component(identity: &str) -> Component {
    Component::retained(
        identity.as_bytes().to_vec().into_boxed_slice(),
        format!("bytes:{identity}").into_bytes().into_boxed_slice(),
    )
}

fn premise_set() -> PremiseSet {
    PremiseSet {
        admitted_fragment: component("fragment:one"),
        backend: BackendSelection::retained(b"qprotocol:conformance".to_vec(), b"v1".to_vec()),
        assumptions: vec![
            Assumption {
                component: component("assumption:root"),
                state: Some(AssumptionState::Met),
                dependencies: vec![],
            },
            Assumption {
                component: component("assumption:leaf"),
                state: Some(AssumptionState::Unknown),
                dependencies: vec![b"assumption:root".to_vec().into_boxed_slice()],
            },
        ],
        communication: vec![component("communication:one")],
        visibility: vec![component("visibility:one")],
        environment: vec![component("environment:one")],
        sources: vec![component("source:one")],
        derived_artifact: Some(DerivedArtifact {
            artifact: component("derived:valid"),
            source_tuple: vec![component("source-tuple:one")],
        }),
    }
}

struct ExactCorrespondence;
impl CorrespondenceAuthority for ExactCorrespondence {
    fn validates(&self, artifact: &DerivedArtifact) -> bool {
        artifact.artifact.identity.as_ref() == b"derived:valid"
            && artifact.source_tuple == [component("source-tuple:one")]
    }
}

fn request<'a>(
    identity: &str,
    claim: Claim,
    correspondence: &'a dyn CorrespondenceAuthority,
) -> ClaimRequest<'a> {
    ClaimRequest {
        identity: identity.as_bytes().to_vec().into_boxed_slice(),
        claim,
        strength: b"tested".to_vec().into_boxed_slice(),
        premises: premise_set(),
        correspondence: Some(correspondence),
    }
}

fn report(global: GlobalDisposition, execution: AssessmentExecution) -> ConformanceReport {
    let document = || RetainedDocument {
        identity: "owner:one".into(),
        bytes: b"owner:one".to_vec(),
    };
    ConformanceReport {
        protocol_identity: "protocol:one".into(),
        obligations: vec![],
        adverse: vec![],
        incomplete: vec![],
        global_premise_defects: vec![],
        global,
        observations: vec![],
        premises: PremiseSnapshot {
            decision_scope_progress: document(),
            decision_scope_closure: document(),
            surrounding_execution_progress: document(),
            surrounding_execution_closure: document(),
            completeness: document(),
            execution,
            global_conformance: GlobalConformanceInput {
                state: GlobalConformanceClosure::Closed,
                required_premises: vec![],
                premises: vec![],
            },
        },
        usage: Usage::default(),
    }
}

#[trace("TC-005", "FR-005-AC-1", "FR-005-AC-3", "FR-029-AC-1", "FR-029-AC-3")]
#[test]
fn tc_005_closed_dispatch_is_total_and_uses_only_the_conformance_report() {
    let correspondence = ExactCorrespondence;
    let requests = vec![
        request("request:one", Claim::Conformance, &correspondence),
        request("request:two", Claim::Projection, &correspondence),
    ];
    let dispatch = dispatch_from_conformance(
        &requests,
        &report(GlobalDisposition::Conforms, AssessmentExecution::Completed),
        Limits::default(),
    )
    .expect("closed dispatch");
    assert_eq!(dispatch.dispositions.len(), 2);
    assert_eq!(
        dispatch.dispositions[0].decision,
        Some(Decision::Established)
    );
    assert_eq!(
        dispatch.dispositions[1].execution,
        AssessmentExecution::Unsupported
    );
    assert_eq!(dispatch.dispositions[0].premises, requests[0].premises);
}

#[trace("TC-005", "FR-005-AC-1", "FR-029-AC-3")]
#[test]
fn tc_005_conformance_state_maps_without_callback_or_fallback() {
    let correspondence = ExactCorrespondence;
    let requests = vec![request("request:one", Claim::Conformance, &correspondence)];
    for global in [
        GlobalDisposition::Pending,
        GlobalDisposition::Violates,
        GlobalDisposition::Unavailable,
    ] {
        let dispatch = dispatch_from_conformance(
            &requests,
            &report(global, AssessmentExecution::Completed),
            Limits::default(),
        )
        .expect("closed dispatch");
        assert_eq!(
            dispatch.dispositions[0].decision,
            Some(Decision::NotEstablished)
        );
    }
    let dispatch = dispatch_from_conformance(
        &requests,
        &report(
            GlobalDisposition::Conforms,
            AssessmentExecution::ResourceIncomplete,
        ),
        Limits::default(),
    )
    .expect("closed dispatch");
    assert_eq!(
        dispatch.dispositions[0].execution,
        AssessmentExecution::ResourceIncomplete
    );
    assert_eq!(dispatch.dispositions[0].decision, None);
}

#[trace(
    "TC-005",
    "TC-011",
    "FR-005-AC-2",
    "FR-008-AC-5",
    "FR-008-AC-8",
    "FR-030-AC-1",
    "FR-030-AC-2",
    "FR-030-AC-4",
    "FR-039-AC-1",
    "FR-039-AC-3"
)]
#[test]
fn tc_005_invalid_premises_refuse_only_that_exact_claim() {
    let correspondence = ExactCorrespondence;
    let good = request("request:good", Claim::Conformance, &correspondence);
    let mut broken = request("request:broken", Claim::Composition, &correspondence);
    broken.premises.assumptions[0].state = None;
    broken.premises.assumptions[0].dependencies =
        vec![b"assumption:leaf".to_vec().into_boxed_slice()];
    let dispatch = dispatch_from_conformance(
        &[good, broken],
        &report(GlobalDisposition::Conforms, AssessmentExecution::Completed),
        Limits::default(),
    )
    .expect("closed dispatch");
    assert_eq!(
        dispatch.dispositions[0].decision,
        Some(Decision::Established)
    );
    let failure = dispatch.dispositions[1]
        .failure
        .as_ref()
        .expect("claim refusal");
    assert_eq!(failure.code(), FailureCode::InvalidPremises);
    assert!(failure
        .defects()
        .iter()
        .any(|defect| defect.axis == PremiseAxis::Assumption
            && defect.kind == PremiseDefectKind::CircularDependency));
    assert_eq!(
        failure.envelope().partial_output(),
        PartialOutput::IndependentResultsRetained
    );
    assert_eq!(failure.envelope().stage(), ProtocolStage::ClaimDispatch);
    assert_eq!(
        failure.envelope().prerequisites()[0].state(),
        PrerequisiteState::Established
    );
    assert_eq!(
        failure.envelope().causes().len(),
        failure.defects().len(),
        "zero-detail handling must not erase simultaneous typed defects"
    );
    let kinds = failure
        .envelope()
        .causes()
        .declared_order()
        .iter()
        .map(|cause| match cause {
            RefusalCause::Protocol(cause) => cause.kind(),
            RefusalCause::Source(_) | RefusalCause::UnsupportedSourceContract(_) => {
                panic!("local claim defects must remain in the local cause vocabulary")
            }
        })
        .collect::<Vec<_>>();
    assert!(kinds.contains(&ProtocolCauseKind::MissingDefinition));
    assert!(kinds.contains(&ProtocolCauseKind::ConflictingDefinition));
}

#[trace("TC-005", "FR-030-AC-1", "FR-030-AC-5", "FR-030-AC-6")]
#[test]
fn tc_005_exact_premise_validation_does_not_admit_substitution() {
    let correspondence = ExactCorrespondence;
    let good = request("request:good", Claim::Conformance, &correspondence);
    let mut wrong = request("request:wrong", Claim::Projection, &correspondence);
    wrong
        .premises
        .derived_artifact
        .as_mut()
        .expect("derived artifact")
        .artifact
        .identity = b"derived:wrong".to_vec().into_boxed_slice();
    let dispatch = dispatch_from_conformance(
        &[good, wrong],
        &report(GlobalDisposition::Conforms, AssessmentExecution::Completed),
        Limits::default(),
    )
    .expect("closed dispatch");
    let failure = dispatch.dispositions[1]
        .failure
        .as_ref()
        .expect("claim refusal");
    assert_eq!(failure.code(), FailureCode::InvalidPremises);
    assert!(failure
        .defects()
        .iter()
        .any(|defect| defect.kind == PremiseDefectKind::CorrespondenceMismatch));
}

fn empty_fragment(premises: &mut PremiseSet) {
    premises.admitted_fragment.bytes = Box::new([]);
}

fn empty_backend(premises: &mut PremiseSet) {
    premises.backend.version = Box::new([]);
}

fn substitute_backend(premises: &mut PremiseSet) {
    premises.backend.identity = b"caller:alternate".to_vec().into_boxed_slice();
}

fn duplicate_assumption(premises: &mut PremiseSet) {
    premises.assumptions.push(premises.assumptions[0].clone());
}

fn remove_communication(premises: &mut PremiseSet) {
    premises.communication.clear();
}

fn remove_visibility(premises: &mut PremiseSet) {
    premises.visibility.clear();
}

fn remove_environment(premises: &mut PremiseSet) {
    premises.environment.clear();
}

fn remove_source(premises: &mut PremiseSet) {
    premises.sources.clear();
}

fn remove_derived_tuple(premises: &mut PremiseSet) {
    premises
        .derived_artifact
        .as_mut()
        .expect("derived artifact")
        .source_tuple
        .clear();
}

type PremiseMutation = (PremiseAxis, fn(&mut PremiseSet));

/// Every premise axis remains independently checked even though execution is
/// closed over the conformance report rather than delegated to a backend.
#[trace("TC-005", "FR-030-AC-1", "FR-030-AC-2")]
#[test]
fn tc_005_every_premise_axis_refuses_without_erasing_a_sibling() {
    let mutations: [PremiseMutation; 8] = [
        (PremiseAxis::AdmittedFragment, empty_fragment),
        (PremiseAxis::Backend, empty_backend),
        (PremiseAxis::Assumption, duplicate_assumption),
        (PremiseAxis::Communication, remove_communication),
        (PremiseAxis::Visibility, remove_visibility),
        (PremiseAxis::Environment, remove_environment),
        (PremiseAxis::Source, remove_source),
        (PremiseAxis::DerivedArtifact, remove_derived_tuple),
    ];
    let mut refusal_axes = HashSet::new();
    for (axis, mutate) in mutations {
        let correspondence = ExactCorrespondence;
        let good = request("request:good", Claim::Conformance, &correspondence);
        let mut broken = request("request:broken", Claim::Composition, &correspondence);
        mutate(&mut broken.premises);
        let dispatch = dispatch_from_conformance(
            &[good, broken],
            &report(GlobalDisposition::Conforms, AssessmentExecution::Completed),
            Limits::default(),
        )
        .expect("closed dispatch");
        assert_eq!(
            dispatch.dispositions[0].decision,
            Some(Decision::Established)
        );
        let failure = dispatch.dispositions[1]
            .failure
            .as_ref()
            .expect("claim refusal");
        assert_eq!(failure.code(), FailureCode::InvalidPremises);
        assert!(failure.defects().iter().any(|defect| defect.axis == axis));
        assert!(
            refusal_axes.insert(failure.envelope().axis().as_bytes().to_vec()),
            "each independently failing premise axis needs a distinct refusal axis"
        );
    }
    assert_eq!(refusal_axes.len(), mutations.len());
}

/// The closed supported backend is not replaceable by an alternate caller
/// identity, and a failed conformance execution stays failed without promoting
/// the independent unsupported projection claim.
#[trace("TC-005", "FR-005-AC-4", "FR-029-AC-4", "FR-030-AC-3")]
#[test]
fn tc_005_selected_backend_has_no_alternate_or_fallback() {
    let correspondence = ExactCorrespondence;
    let mut substituted = request("request:conformance", Claim::Conformance, &correspondence);
    substitute_backend(&mut substituted.premises);
    let refused = dispatch_from_conformance(
        &[substituted],
        &report(GlobalDisposition::Conforms, AssessmentExecution::Completed),
        Limits::default(),
    )
    .expect("a backend mismatch is claim-local");
    assert_eq!(
        refused.dispositions[0].execution,
        AssessmentExecution::Refused
    );
    assert!(refused.dispositions[0]
        .failure
        .as_ref()
        .expect("mismatch refusal")
        .defects()
        .iter()
        .any(|defect| defect.axis == PremiseAxis::Backend
            && defect.kind == PremiseDefectKind::CorrespondenceMismatch));

    let requests = [
        request("request:conformance", Claim::Conformance, &correspondence),
        request("request:projection", Claim::Projection, &correspondence),
    ];
    let failed = dispatch_from_conformance(
        &requests,
        &report(GlobalDisposition::Conforms, AssessmentExecution::Failed),
        Limits::default(),
    )
    .expect("a failed selected execution preserves every claim row");
    assert_eq!(
        failed.dispositions[0].execution,
        AssessmentExecution::Failed
    );
    assert_eq!(failed.dispositions[0].decision, None);
    assert_eq!(
        failed.dispositions[1].execution,
        AssessmentExecution::Unsupported,
        "no unsupported claim is repaired by the failed selected capability"
    );
}

#[trace(
    "TC-005",
    "TC-011",
    "FR-008-AC-5",
    "FR-029-AC-5",
    "FR-029-AC-6",
    "FR-039-AC-1"
)]
#[test]
fn tc_005_duplicate_identity_refuses_each_claim_without_a_second_execution() {
    let correspondence = ExactCorrespondence;
    let requests = vec![
        request("request:duplicate", Claim::Conformance, &correspondence),
        request("request:duplicate", Claim::Projection, &correspondence),
    ];
    let dispatch = dispatch_from_conformance(
        &requests,
        &report(GlobalDisposition::Conforms, AssessmentExecution::Completed),
        Limits::default(),
    )
    .expect("closed dispatch");
    assert!(dispatch.dispositions.iter().all(|disposition| {
        disposition.failure.as_ref().is_some_and(|failure| {
            failure.code() == FailureCode::DuplicateRequest
                && matches!(
                    failure.envelope().causes().declared_order(),
                    [RefusalCause::Protocol(cause)]
                        if cause.kind() == ProtocolCauseKind::ConflictingDefinition
                )
        })
    }));
}

#[trace("TC-005", "TC-009", "NFR-002-AC-1", "NFR-002-AC-4", "FR-016-AC-2")]
#[test]
fn tc_009_closed_dispatch_charges_before_result_allocation() {
    let inventory = production_inventory().expect("production inventory");
    let correspondence = ExactCorrespondence;
    let requests = vec![
        request("request:one", Claim::Conformance, &correspondence),
        request("request:two", Claim::Projection, &correspondence),
    ];
    let conformance = report(GlobalDisposition::Conforms, AssessmentExecution::Completed);
    let short = Budget::nothing_granted()
        .grant(BudgetDimension::Operation, 1)
        .expect("positive one-short grant");
    let mut resources = ResourceAccountant::new(short, &inventory);
    assert!(
        matches!(dispatch_bounded(&requests, &conformance, Limits::default(), &mut resources), OperationOutcome::ResourceIncomplete(value) if value.dimension() == BudgetDimension::Operation)
    );
    assert_eq!(resources.usage().consumed(BudgetDimension::Operation), 0);
    let exact = Budget::nothing_granted()
        .grant(BudgetDimension::Operation, 2)
        .expect("exact grant");
    let mut resources = ResourceAccountant::new(exact, &inventory);
    assert!(matches!(
        dispatch_bounded(&requests, &conformance, Limits::default(), &mut resources),
        OperationOutcome::Completed(_)
    ));
    assert_eq!(resources.usage().consumed(BudgetDimension::Operation), 2);
}
