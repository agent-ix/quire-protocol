// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-005: exact premise-bound, independent claim dispatch.

use ix_trace_rs::trace;
use quire_protocol::bounds::inventory::{
    production_inventory, Budget, BudgetDimension, OperationOutcome, ResourceAccountant,
};
use quire_protocol::claims::dispatch::{
    dispatch, dispatch_bounded, Backend, BackendExecution, BackendReply, ClaimRequest, Decision,
    FailureCode, Limits,
};
use quire_protocol::claims::premises::{
    Assumption, AssumptionState, BackendSelection, Component, CorrespondenceAuthority,
    DerivedArtifact, PremiseAxis, PremiseDefectKind, PremiseSet,
};
use quire_protocol::claims::Claim;
use quire_protocol::closure::AssessmentExecution;
use quire_protocol::refusal::PartialOutput;

fn component(identity: &str) -> Component {
    Component::retained(
        identity.as_bytes().to_vec().into_boxed_slice(),
        format!("bytes:{identity}").into_bytes().into_boxed_slice(),
    )
}

fn premise_set(backend: BackendSelection) -> PremiseSet {
    PremiseSet {
        admitted_fragment: component("fragment:one"),
        backend,
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
    strength: &str,
    backend: BackendSelection,
    correspondence: &'a dyn CorrespondenceAuthority,
) -> ClaimRequest<'a> {
    ClaimRequest {
        identity: identity.as_bytes().to_vec().into_boxed_slice(),
        claim,
        strength: strength.as_bytes().to_vec().into_boxed_slice(),
        premises: premise_set(backend),
        correspondence: Some(correspondence),
    }
}

fn two_requests(correspondence: &dyn CorrespondenceAuthority) -> Vec<ClaimRequest<'_>> {
    let selection = BackendSelection::retained(b"backend:one".to_vec(), b"v1".to_vec());
    vec![
        request(
            "request:one",
            Claim::Conformance,
            "tested",
            selection.clone(),
            correspondence,
        ),
        request(
            "request:two",
            Claim::Projection,
            "tested",
            selection,
            correspondence,
        ),
    ]
}

struct FakeBackend {
    selection: BackendSelection,
    supported: Vec<Claim>,
    replies: Vec<(Claim, BackendExecution)>,
    substitute_evidence: bool,
    calls: Vec<Claim>,
}

impl FakeBackend {
    fn new(identity: &str, version: &str, supported: Vec<Claim>) -> Self {
        Self {
            selection: BackendSelection::retained(
                identity.as_bytes().to_vec().into_boxed_slice(),
                version.as_bytes().to_vec().into_boxed_slice(),
            ),
            supported,
            replies: Vec::new(),
            substitute_evidence: false,
            calls: Vec::new(),
        }
    }

    fn execution(&self, claim: Claim) -> BackendExecution {
        self.replies
            .iter()
            .find_map(|(selected, execution)| (*selected == claim).then_some(*execution))
            .unwrap_or(BackendExecution::Completed(Decision::Established))
    }
}

impl Backend for FakeBackend {
    fn selection(&self) -> &BackendSelection {
        &self.selection
    }

    fn supports(&self, claim: Claim, _strength: &[u8]) -> bool {
        self.supported.contains(&claim)
    }

    fn assess(&mut self, claim: Claim, _strength: &[u8], premises: &PremiseSet) -> BackendReply {
        self.calls.push(claim);
        let mut evidence = premises.clone();
        if self.substitute_evidence {
            evidence.environment[0].bytes = b"substituted".to_vec().into_boxed_slice();
        }
        BackendReply {
            execution: self.execution(claim),
            evidence,
        }
    }
}

/// Supported, unsupported, and failed claims retain independent dispositions;
/// neither missing capability nor backend failure erases conformance.
#[trace(
    "TC-005",
    "FR-005-AC-1",
    "FR-005-AC-3",
    "FR-005-AC-4",
    "FR-029-AC-1",
    "FR-029-AC-3",
    "FR-029-AC-4"
)]
#[test]
fn tc_005_supported_unsupported_and_failed_claims_are_independent() {
    let correspondence = ExactCorrespondence;
    let selection = BackendSelection::retained(b"backend:one".to_vec(), b"v1".to_vec());
    let requests = vec![
        request(
            "request:conformance",
            Claim::Conformance,
            "bounded-checked",
            selection.clone(),
            &correspondence,
        ),
        request(
            "request:projection",
            Claim::Projection,
            "proven",
            selection.clone(),
            &correspondence,
        ),
        request(
            "request:monitorability",
            Claim::Monitorability,
            "tested",
            selection,
            &correspondence,
        ),
    ];
    let mut backend = FakeBackend::new(
        "backend:one",
        "v1",
        vec![Claim::Conformance, Claim::Monitorability],
    );
    backend
        .replies
        .push((Claim::Monitorability, BackendExecution::Failed));
    let mut backends: [&mut dyn Backend; 1] = [&mut backend];
    let report = dispatch(&requests, &mut backends, Limits::default()).expect("dispatch");
    assert_eq!(report.dispositions.len(), 3);
    assert_eq!(
        report.dispositions[0].execution,
        AssessmentExecution::Completed
    );
    assert_eq!(report.dispositions[0].decision, Some(Decision::Established));
    assert_eq!(
        report.dispositions[1].execution,
        AssessmentExecution::Unsupported
    );
    assert_eq!(
        report.dispositions[2].execution,
        AssessmentExecution::Failed
    );
    assert_eq!(backend.calls, [Claim::Conformance, Claim::Monitorability]);
}

/// Two requests for one kind at different exact strengths produce two answers;
/// strength is retained byte-exactly and never promoted.
#[trace("TC-005", "FR-029-AC-5", "FR-029-AC-6")]
#[test]
fn tc_005_one_disposition_is_returned_per_exact_request_and_strength() {
    let correspondence = ExactCorrespondence;
    let selection = BackendSelection::retained(b"backend:one".to_vec(), b"v1".to_vec());
    let requests = vec![
        request(
            "request:observed",
            Claim::Conformance,
            "observed",
            selection.clone(),
            &correspondence,
        ),
        request(
            "request:proven",
            Claim::Conformance,
            "proven",
            selection,
            &correspondence,
        ),
    ];
    let mut backend = FakeBackend::new("backend:one", "v1", vec![Claim::Conformance]);
    let mut backends: [&mut dyn Backend; 1] = [&mut backend];
    let report = dispatch(&requests, &mut backends, Limits::default()).expect("dispatch");
    assert_eq!(report.dispositions.len(), requests.len());
    assert_eq!(report.dispositions[0].strength.as_ref(), b"observed");
    assert_eq!(report.dispositions[1].strength.as_ref(), b"proven");
    assert_ne!(
        report.dispositions[0].request_identity,
        report.dispositions[1].request_identity
    );
}

/// A circular or absent-state assumption refuses only that exact disposition
/// before backend execution and leaves an independent sibling unchanged.
#[trace("TC-005", "FR-005-AC-2", "FR-030-AC-2", "FR-030-AC-4")]
#[test]
fn tc_005_circular_and_absent_assumption_state_refuse_before_dispatch() {
    let correspondence = ExactCorrespondence;
    let selection = BackendSelection::retained(b"backend:one".to_vec(), b"v1".to_vec());
    let good = request(
        "request:good",
        Claim::Conformance,
        "tested",
        selection.clone(),
        &correspondence,
    );
    let mut circular = request(
        "request:circular",
        Claim::Composition,
        "tested",
        selection,
        &correspondence,
    );
    circular.premises.assumptions[0].state = None;
    circular.premises.assumptions[0].dependencies =
        vec![b"assumption:leaf".to_vec().into_boxed_slice()];
    let requests = vec![good, circular];
    let mut backend = FakeBackend::new(
        "backend:one",
        "v1",
        vec![Claim::Conformance, Claim::Composition],
    );
    let mut backends: [&mut dyn Backend; 1] = [&mut backend];
    let report = dispatch(&requests, &mut backends, Limits::default()).expect("dispatch");
    assert_eq!(
        report.dispositions[0].execution,
        AssessmentExecution::Completed
    );
    assert_eq!(
        report.dispositions[1].execution,
        AssessmentExecution::Refused
    );
    let failure = report.dispositions[1].failure.as_ref().expect("failure");
    assert_eq!(failure.code(), FailureCode::InvalidPremises);
    assert!(failure.defects().iter().any(|defect| {
        defect.axis == PremiseAxis::Assumption
            && defect.kind == PremiseDefectKind::CircularDependency
    }));
    assert!(failure.defects().iter().any(|defect| {
        defect.axis == PremiseAxis::Assumption && defect.kind == PremiseDefectKind::Missing
    }));
    assert_eq!(
        failure.envelope().partial_output(),
        PartialOutput::IndependentResultsRetained
    );
    assert_eq!(backend.calls, [Claim::Conformance]);
}

/// Every retained component is exact, owner correspondence is consulted, and
/// backend evidence substitution becomes a claim-local refusal.
#[trace("TC-005", "FR-005-AC-5", "FR-030-AC-1", "FR-030-AC-5", "FR-030-AC-6")]
#[test]
fn tc_005_exact_premises_and_owner_correspondence_are_enforced() {
    let correspondence = ExactCorrespondence;
    let selection = BackendSelection::retained(b"backend:one".to_vec(), b"v1".to_vec());
    let exact = request(
        "request:exact",
        Claim::Refinement,
        "proven",
        selection.clone(),
        &correspondence,
    );
    let expected = exact.premises.clone();
    let mut invalid_correspondence = request(
        "request:invalid-correspondence",
        Claim::Projection,
        "proven",
        selection,
        &correspondence,
    );
    invalid_correspondence
        .premises
        .derived_artifact
        .as_mut()
        .expect("derived artifact")
        .artifact
        .identity = b"derived:wrong".to_vec().into_boxed_slice();
    let requests = vec![exact, invalid_correspondence];
    let mut backend = FakeBackend::new(
        "backend:one",
        "v1",
        vec![Claim::Refinement, Claim::Projection],
    );
    backend.substitute_evidence = true;
    let mut backends: [&mut dyn Backend; 1] = [&mut backend];
    let report = dispatch(&requests, &mut backends, Limits::default()).expect("dispatch");
    assert_eq!(report.dispositions[0].premises, expected);
    assert_eq!(
        report.dispositions[0]
            .failure
            .as_ref()
            .expect("evidence mismatch")
            .code(),
        FailureCode::EvidenceMismatch
    );
    assert!(report.dispositions[0]
        .failure
        .as_ref()
        .expect("evidence mismatch")
        .defects()
        .iter()
        .any(|defect| {
            defect.axis == PremiseAxis::Environment && defect.kind == PremiseDefectKind::Substituted
        }));
    let correspondence_failure = report.dispositions[1]
        .failure
        .as_ref()
        .expect("correspondence failure");
    assert_eq!(correspondence_failure.code(), FailureCode::InvalidPremises);
    assert!(correspondence_failure
        .defects()
        .iter()
        .any(|defect| { defect.kind == PremiseDefectKind::CorrespondenceMismatch }));
    assert_eq!(backend.calls, [Claim::Refinement]);
}

/// Exact backend version mismatch refuses before execution, while a selected
/// failing backend invokes no available fallback.
#[trace("TC-005", "FR-030-AC-3")]
#[test]
fn tc_005_backend_match_is_exact_and_failure_invokes_no_fallback() {
    let correspondence = ExactCorrespondence;
    let wrong_version = BackendSelection::retained(b"backend:one".to_vec(), b"v2".to_vec());
    let mismatch = request(
        "request:mismatch",
        Claim::Conformance,
        "tested",
        wrong_version,
        &correspondence,
    );
    let selected = BackendSelection::retained(b"backend:one".to_vec(), b"v1".to_vec());
    let failed = request(
        "request:failed",
        Claim::Conformance,
        "tested",
        selected,
        &correspondence,
    );
    let requests = vec![mismatch, failed];
    let mut primary = FakeBackend::new("backend:one", "v1", vec![Claim::Conformance]);
    primary
        .replies
        .push((Claim::Conformance, BackendExecution::Failed));
    let mut fallback = FakeBackend::new("backend:fallback", "v1", vec![Claim::Conformance]);
    let mut backends: [&mut dyn Backend; 2] = [&mut primary, &mut fallback];
    let report = dispatch(&requests, &mut backends, Limits::default()).expect("dispatch");
    assert_eq!(
        report.dispositions[0]
            .failure
            .as_ref()
            .expect("version mismatch")
            .code(),
        FailureCode::BackendMismatch
    );
    assert_eq!(
        report.dispositions[1].execution,
        AssessmentExecution::Failed
    );
    assert_eq!(primary.calls, [Claim::Conformance]);
    assert!(fallback.calls.is_empty());
}

/// The whole-request bound refuses before allocation or backend execution.
#[trace("TC-005", "NFR-002-AC-2")]
#[test]
fn tc_005_dispatch_limits_are_checked_before_backend_calls() {
    let correspondence = ExactCorrespondence;
    let selection = BackendSelection::retained(b"backend:one".to_vec(), b"v1".to_vec());
    let requests = vec![request(
        "request:one",
        Claim::Conformance,
        "tested",
        selection,
        &correspondence,
    )];
    let mut backend = FakeBackend::new("backend:one", "v1", vec![Claim::Conformance]);
    let mut backends: [&mut dyn Backend; 1] = [&mut backend];
    let limits = Limits {
        max_requests: 0,
        ..Limits::default()
    };
    let refusal = dispatch(&requests, &mut backends, limits).expect_err("one over request bound");
    assert_eq!(refusal.usage().requests, 1);
    assert!(backend.calls.is_empty());
}

#[trace("TC-005", "TC-009", "NFR-002-AC-1", "NFR-002-AC-4", "FR-016-AC-2")]
#[test]
fn tc_009_dispatch_charges_caller_operations_before_backend_calls() {
    let inventory = production_inventory().expect("production inventory");
    let correspondence = ExactCorrespondence;

    let requests = two_requests(&correspondence);
    let mut backend = FakeBackend::new(
        "backend:one",
        "v1",
        vec![Claim::Conformance, Claim::Projection],
    );
    let mut backends: [&mut dyn Backend; 1] = [&mut backend];
    let short = Budget::nothing_granted()
        .grant(BudgetDimension::Operation, 1)
        .expect("positive one-short grant");
    let mut resources = ResourceAccountant::new(short, &inventory);
    let OperationOutcome::ResourceIncomplete(incomplete) =
        dispatch_bounded(&requests, &mut backends, Limits::default(), &mut resources)
    else {
        panic!("one-short operation budget must be resource-incomplete");
    };
    assert_eq!(incomplete.dimension(), BudgetDimension::Operation);
    assert!(
        backend.calls.is_empty(),
        "no backend call started past the grant"
    );
    assert_eq!(
        resources.usage().consumed(BudgetDimension::Operation),
        0,
        "multi-dimension preflight is transactional"
    );

    let requests = two_requests(&correspondence);
    let mut backend = FakeBackend::new(
        "backend:one",
        "v1",
        vec![Claim::Conformance, Claim::Projection],
    );
    let mut backends: [&mut dyn Backend; 1] = [&mut backend];
    let owner_short = Limits {
        max_requests: 1,
        ..Limits::default()
    };
    let mut owner_resources = ResourceAccountant::new(Budget::nothing_granted(), &inventory);
    assert!(matches!(
        dispatch_bounded(
            &requests,
            &mut backends,
            owner_short,
            &mut owner_resources
        ),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::ResultItem
                && value.granted() == 1
                && value.requested() == 2
    ));
    assert!(backend.calls.is_empty());
    assert_eq!(
        owner_resources
            .usage()
            .consumed(BudgetDimension::ResultItem),
        0
    );

    let requests = two_requests(&correspondence);
    let mut backend = FakeBackend::new(
        "backend:one",
        "v1",
        vec![Claim::Conformance, Claim::Projection],
    );
    let mut backends: [&mut dyn Backend; 1] = [&mut backend];
    let exact = Budget::nothing_granted()
        .grant(BudgetDimension::Operation, 2)
        .expect("positive exact grant");
    let mut fresh = ResourceAccountant::new(exact, &inventory);
    let OperationOutcome::Completed(report) =
        dispatch_bounded(&requests, &mut backends, Limits::default(), &mut fresh)
    else {
        panic!("fresh exact operation budget must complete");
    };
    assert_eq!(report.dispositions.len(), 2);
    assert_eq!(backend.calls.len(), 2);
    assert_eq!(fresh.usage().consumed(BudgetDimension::Operation), 2);

    let requests = two_requests(&correspondence);
    let mut backend = FakeBackend::new(
        "backend:one",
        "v1",
        vec![Claim::Conformance, Claim::Projection],
    );
    let one_allocation = Budget::nothing_granted()
        .grant(BudgetDimension::AccountedAllocation, 1)
        .expect("one dispatch-report allocation");
    let mut resources = ResourceAccountant::new(one_allocation, &inventory);
    let first = {
        let mut backends: [&mut dyn Backend; 1] = [&mut backend];
        dispatch_bounded(&requests, &mut backends, Limits::default(), &mut resources)
    };
    assert!(matches!(first, OperationOutcome::Completed(_)));
    assert_eq!(backend.calls.len(), 2);
    let second = {
        let mut backends: [&mut dyn Backend; 1] = [&mut backend];
        dispatch_bounded(&requests, &mut backends, Limits::default(), &mut resources)
    };
    assert!(matches!(
        second,
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::AccountedAllocation
                && value.consumed() == 1
                && value.requested() == 1
    ));
    assert_eq!(
        backend.calls.len(),
        2,
        "no call starts before report allocation"
    );
}

fn empty_fragment(premises: &mut PremiseSet) {
    premises.admitted_fragment.bytes = Box::new([]);
}

fn empty_backend(premises: &mut PremiseSet) {
    premises.backend.version = Box::new([]);
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

/// Every premise axis is validated independently and one broken axis never
/// suppresses an exact sibling disposition.
#[trace("TC-005", "FR-030-AC-1", "FR-030-AC-2")]
#[test]
fn tc_005_each_premise_axis_refuses_independently() {
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
    for (axis, mutate) in mutations {
        let correspondence = ExactCorrespondence;
        let selection = BackendSelection::retained(b"backend:one".to_vec(), b"v1".to_vec());
        let good = request(
            "request:good",
            Claim::Conformance,
            "tested",
            selection.clone(),
            &correspondence,
        );
        let mut broken = request(
            "request:broken",
            Claim::Composition,
            "tested",
            selection,
            &correspondence,
        );
        mutate(&mut broken.premises);
        let requests = vec![good, broken];
        let mut backend = FakeBackend::new(
            "backend:one",
            "v1",
            vec![Claim::Conformance, Claim::Composition],
        );
        let mut backends: [&mut dyn Backend; 1] = [&mut backend];
        let report = dispatch(&requests, &mut backends, Limits::default()).expect("dispatch");
        assert_eq!(
            report.dispositions[0].execution,
            AssessmentExecution::Completed
        );
        let failure = report.dispositions[1].failure.as_ref().expect("failure");
        assert_eq!(failure.code(), FailureCode::InvalidPremises);
        assert!(failure.defects().iter().any(|defect| defect.axis == axis));
        assert_eq!(backend.calls, [Claim::Conformance]);
    }
}

/// Duplicate exact request identities are both refused, rather than answered
/// twice, and no backend is invoked.
#[trace("TC-005", "FR-029-AC-5")]
#[test]
fn tc_005_duplicate_request_identity_is_not_dispatched_twice() {
    let correspondence = ExactCorrespondence;
    let selection = BackendSelection::retained(b"backend:one".to_vec(), b"v1".to_vec());
    let requests = vec![
        request(
            "request:duplicate",
            Claim::Conformance,
            "tested",
            selection.clone(),
            &correspondence,
        ),
        request(
            "request:duplicate",
            Claim::Projection,
            "proven",
            selection,
            &correspondence,
        ),
    ];
    let mut backend = FakeBackend::new(
        "backend:one",
        "v1",
        vec![Claim::Conformance, Claim::Projection],
    );
    let mut backends: [&mut dyn Backend; 1] = [&mut backend];
    let report = dispatch(&requests, &mut backends, Limits::default()).expect("dispatch");
    assert!(report.dispositions.iter().all(|disposition| {
        disposition
            .failure
            .as_ref()
            .is_some_and(|failure| failure.code() == FailureCode::DuplicateRequest)
    }));
    assert!(backend.calls.is_empty());
}
