// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-005: exact premise-bound dispatch through QProtocol's closed capability.

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
use quire_protocol::conformance::batch::{PremiseSnapshot, RetainedDocument};
use quire_protocol::conformance::{GlobalDisposition, Report as ConformanceReport, Usage};
use quire_protocol::refusal::{PartialOutput, ProtocolStage};

fn component(identity: &str) -> Component {
    Component::retained(
        identity.as_bytes().to_vec().into_boxed_slice(),
        format!("bytes:{identity}").into_bytes().into_boxed_slice(),
    )
}

fn premise_set() -> PremiseSet {
    PremiseSet {
        admitted_fragment: component("fragment:one"),
        backend: BackendSelection::retained(b"conformance-report".to_vec(), b"v1".to_vec()),
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

#[trace("TC-005", "FR-005-AC-2", "FR-030-AC-1", "FR-030-AC-2", "FR-030-AC-4")]
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

#[trace("TC-005", "FR-029-AC-5", "FR-029-AC-6")]
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
    assert!(dispatch.dispositions.iter().all(|disposition| disposition
        .failure
        .as_ref()
        .is_some_and(|failure| failure.code() == FailureCode::DuplicateRequest)));
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
