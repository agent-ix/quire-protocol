// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-006/TC-015: in-crate canonical result owner and Contract-IR mapping.

use agent_ix_baseline_producer::StaticProducerBundle;
use ix_trace_rs::trace;
use quire_observation::authority::{
    self, AuthoritySelection, Context, History, Limits as ObservationLimits, OpenClosed,
    SubjectSelection, TemporalBoundary,
};
use quire_observation::{
    admit, AdmissionOutcome, AdmissionRequest, AdmittedRecord, Anchor, ClockRange, Digest,
    Identity, Member, ObservationBinding, PackageSelection, QualifiedObservation, QualifiedSubject,
    ResourceLimits, ScopeKind, ScopeSelection, SubjectIdentity, SubjectKind, ValueState,
    Visibility, NATIVE_LINKED_PACKAGE_FORMAT,
};
use quire_protocol::assessment::{CanonicalRowControls, Engine};
use quire_protocol::bounds::inventory::{
    production_inventory, Budget, BudgetDimension, OperationOutcome, ResourceAccountant,
};
use quire_protocol::bounds::SystemMonotonicClock;
use quire_protocol::claims::dispatch::{ClaimRequest, Limits as DispatchLimits};
use quire_protocol::claims::premises::{BackendSelection, Component, PremiseSet};
use quire_protocol::claims::Claim as DispatchClaim;
use quire_protocol::closure::{AssessmentExecution, GlobalConformanceClosure, ScopeState};
use quire_protocol::conformance::Limits as ConformanceLimits;
use quire_protocol::model::v2::{
    Internality, LinkedAssessment, RoleClassification, RuntimeBinding, RuntimeBindingSet,
    SlotCompatibility, SlotHandle, StaticBindingDefinition, StaticSelection, StaticSlot,
    RUNTIME_BINDING_CONTRACT, STATIC_BINDING_CONTRACT,
};
use quire_protocol::producer_contracts::{
    resolve_producer_contracts, ProducerContract, ProducerResolution, ProducerResolutionRequest,
};
use quire_protocol::projection::{
    resolve_e02_contract, DecisionSupportBinding, E02Contract, E02Resolution,
    ProjectedTechniqueResult, ProjectionContext, ProjectionRequestId,
};
use quire_protocol::refusal::ProtocolStage;
use quire_protocol::repro::{CorpusIdentity, ReproductionInputs, Seed, ToolchainIdentity};
use quire_protocol::result::{
    self, contract_ir, runtime_reference, Activation, Adequacy, Assumption, AssumptionState,
    AxisInputs, Claim, ClaimKind, ClaimStrength, CorrectionInput, CorrectionRelation,
    DecisionSupport, DependenceRelation, ErrorCode, GlobalConformanceInput, GlobalPremise, Input,
    Limitation, Limits, Participation, PremiseKind, ProtocolAdequacy, ProvenanceRecord, Settlement,
    SettlementBasis, SupportRef, Truth,
};
use quire_protocol::version_lock::{ResolutionBudget, VersionLockManifest};
use quire_protocol::{v2, wire as w, ByteDigest};
use quire_spec_language::protocol_artifact::{
    checked_predicate,
    native_temporal::{self, request as temporal_request, v2 as temporal_v2},
    occurrence_key_schema, temporal_subject, v3, WorkflowInstanceIdentity,
};
use quire_verification::contracts::wire::{
    ObservationReference, OracleReference, SharedArtifactEnvelope,
};
use quire_verification::contracts::{self, jcs_sha256};
use serde_json::{json, Value};
use std::time::Duration;

#[path = "../../tests/support/v2_handoff/mod.rs"]
#[allow(dead_code, unused_imports)]
mod handoff;

#[path = "../../tests/support/observation_owner.rs"]
#[allow(
    clippy::duplicate_mod,
    dead_code,
    reason = "the result semantic corpus deliberately owns an independent QObs fixture instance from the conformance corpus"
)]
mod observation_owner;

const ORDER_KIND: &str = "ix://agent-ix/commerce/type/Order";

fn producer_resolution(identity: &[u8]) -> ProducerResolution {
    let request = ProducerResolutionRequest::new(identity)
        .expect("the test assessment resolution identity is nonempty");
    let manifest = VersionLockManifest::new(
        ProducerContract::ALL
            .iter()
            .copied()
            .map(ProducerContract::manifest_entry)
            .collect(),
    );
    let budget = ResolutionBudget::new(ProducerContract::ALL.len(), 1, Duration::from_secs(1))
        .expect("the exact producer-resolution budget is valid");
    match resolve_producer_contracts(&request, &manifest, budget, SystemMonotonicClock) {
        quire_protocol::outcome::OperationOutcome::Completed(resolution) => resolution,
        quire_protocol::outcome::OperationOutcome::Refused(refusal) => {
            panic!("exact producer manifest must resolve: {}", refusal.code())
        }
        quire_protocol::outcome::OperationOutcome::ResourceIncomplete(incomplete) => {
            panic!("exact producer manifest must fit: {incomplete:?}")
        }
    }
}

struct Fixture {
    predicate: checked_predicate::ValidatedCheckedPredicate,
    temporal: temporal_subject::ValidatedTemporalSubject,
    decision: ScopeViews,
    surrounding: ScopeViews,
}

struct ScopeViews {
    progress_open: authority::progress::View,
    progress_closed: authority::progress::View,
    closure_open: authority::closure::View,
    closure_closed: authority::closure::View,
    completeness_complete: authority::completeness::View,
    completeness_incomplete: authority::completeness::View,
    completeness_contradicted: authority::completeness::View,
}

fn id(value: impl Into<String>) -> Identity {
    Identity::new(value)
}
fn digest(value: u8) -> Digest {
    Digest::new([value; 32])
}

fn producer() -> quire_observation::AdmittedStaticBundle {
    StaticProducerBundle::admit_json(include_bytes!(
        "../../tests/fixtures/fcd-static-bundle-1.2.json"
    ))
    .expect("shared FCD static bundle is admitted")
}

fn runtime_subject(
    producer: &quire_observation::AdmittedStaticBundle,
    tag: &str,
) -> QualifiedSubject {
    runtime_subject_bytes(
        producer,
        ORDER_KIND.as_bytes().to_vec(),
        format!("order:{tag}").into_bytes(),
    )
}

fn runtime_subject_bytes(
    producer: &quire_observation::AdmittedStaticBundle,
    kind: Vec<u8>,
    identity: Vec<u8>,
) -> QualifiedSubject {
    QualifiedSubject::new(
        producer,
        SubjectKind::new(kind).expect("non-empty subject kind"),
        SubjectIdentity::new(identity).expect("non-empty subject identity"),
    )
}

fn carrier_payload(encoded: &str) -> Vec<u8> {
    let payload = encoded
        .strip_prefix(runtime_reference::PREFIX)
        .expect("selected runtime-reference prefix");
    let (pairs, remainder) = payload.as_bytes().as_chunks::<2>();
    assert!(
        remainder.is_empty(),
        "carrier payload has complete hex pairs"
    );
    pairs
        .iter()
        .map(|pair| {
            let text = core::str::from_utf8(pair).expect("hex pair is UTF-8");
            u8::from_str_radix(text, 16).expect("lowercase hex pair")
        })
        .collect()
}

fn carrier(bytes: &[u8]) -> String {
    format!(
        "{}{}",
        runtime_reference::PREFIX,
        bytes
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    )
}

fn fixture() -> Fixture {
    let loaded = handoff::Handoff::load().expect("published v2 handoff");
    let declarations = loaded.declarations();
    let inventories = loaded.inventories(&declarations);
    let expected = loaded.expected(&inventories);
    let admitted = quire_protocol::intake::v2::admit(loaded.offer(), &expected, loaded.limits())
        .into_result()
        .expect("strict v2 package");
    let temporal_declaration = admitted
        .inherited()
        .declarations
        .iter()
        .position(|declaration| matches!(declaration.body, w::Body::Temporal { .. }))
        .and_then(|index| u32::try_from(index).ok())
        .expect("temporal declaration");
    let temporal_selection = temporal_subject::DeclarationSelection::new(temporal_declaration);
    let temporal_document = temporal_subject::derive(
        &admitted,
        temporal_selection,
        temporal_subject::Limits::default(),
    )
    .into_result()
    .expect("temporal subject document");
    let temporal = temporal_subject::read(
        temporal_document.bytes(),
        &admitted,
        temporal_selection,
        temporal_subject::Limits::default(),
    )
    .into_result()
    .expect("temporal subject view");
    let predicate_root = temporal
        .predicate_leaves()
        .and_then(|leaves| leaves.first())
        .cloned()
        .expect("temporal subject has a checked Boolean leaf");
    let predicate_selection =
        checked_predicate::ClauseSelection::new(predicate_root.declaration, predicate_root);
    let predicate_document = checked_predicate::derive(
        &admitted,
        predicate_selection.clone(),
        checked_predicate::Limits::default(),
    )
    .into_result()
    .expect("checked predicate document");
    let predicate = checked_predicate::read(
        predicate_document.bytes(),
        &admitted,
        predicate_selection,
        checked_predicate::Limits::default(),
    )
    .into_result()
    .expect("checked predicate view");
    Fixture {
        predicate,
        temporal,
        decision: scope_views("decision"),
        surrounding: scope_views("surrounding"),
    }
}

fn first_checked_leaf(package: &v2::AdmittedPackage) -> (u32, w::Handle) {
    package
        .inherited()
        .declarations
        .iter()
        .enumerate()
        .find_map(|(index, declaration)| {
            let root = match &declaration.body {
                w::Body::Predicate { root, .. } | w::Body::State { root, .. } => root.clone(),
                w::Body::Temporal { .. } => declaration.temporal.iter().find_map(|node| {
                    if let w::TemporalOperation::Holds { value } = &node.operation {
                        Some(value.clone())
                    } else {
                        None
                    }
                })?,
                w::Body::Protocol {
                    controls, finish, ..
                } => controls
                    .iter()
                    .find_map(|control| {
                        if let w::ControlOperation::Check { value, .. } = &control.operation {
                            Some(value.clone())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| finish.constraint.clone()),
            };
            Some((u32::try_from(index).ok()?, root))
        })
        .expect("checked Boolean leaf")
}

fn scope_views(tag: &str) -> ScopeViews {
    let qualified = qualified(tag);
    let owner = AuthoritySelection {
        definition_identity: id(format!("definition:{tag}")),
        definition_revision: id("1"),
        definition_digest: digest(9),
    };
    let subject = SubjectSelection {
        scope_identity: id(format!("window:{tag}")),
        population_identity: qualified.scope().population_identity.clone(),
    };
    let context = Context::new(History::batch(&qualified), &owner, &subject, 1, None);
    let progress_open = progress_view(context, OpenClosed::Open, tag);
    let progress_closed = progress_view(context, OpenClosed::Closed, tag);
    let closure_open = closure_view(context, OpenClosed::Open, tag);
    let closure_closed = closure_view(context, OpenClosed::Closed, tag);
    let completeness_complete = completeness_view(
        context,
        &qualified,
        authority::completeness::FactStatus::Available,
        true,
        tag,
    );
    let completeness_incomplete = completeness_view(
        context,
        &qualified,
        authority::completeness::FactStatus::Incomplete,
        false,
        tag,
    );
    let completeness_contradicted = completeness_view(
        context,
        &qualified,
        authority::completeness::FactStatus::Contradicted,
        true,
        tag,
    );
    ScopeViews {
        progress_open,
        progress_closed,
        closure_open,
        closure_closed,
        completeness_complete,
        completeness_incomplete,
        completeness_contradicted,
    }
}

fn qualified(tag: &str) -> Box<QualifiedObservation> {
    let producer = producer();
    let subject = runtime_subject(&producer, tag);
    let mut request = AdmissionRequest {
        package: PackageSelection {
            format: NATIVE_LINKED_PACKAGE_FORMAT.to_owned(),
            identity: id(format!("package:{tag}")),
            revision: id("1"),
            digest: digest(1),
        },
        producer,
        binding: ObservationBinding {
            identity: id(format!("binding:{tag}")),
            source_identity: id(format!("source:{tag}")),
            schema_identity: id(format!("schema:{tag}")),
            signal_identity: id(format!("signal:{tag}")),
            trigger_identity: id(format!("trigger:{tag}")),
            unit: id("unit"),
            subject_kind: SubjectKind::new(ORDER_KIND.as_bytes().to_vec())
                .expect("non-empty order kind"),
            required: true,
        },
        expected_subject: subject.clone(),
        relationships: vec![],
        required_relationships: vec![],
        scope: ScopeSelection {
            population_identity: id("unsealed-population"),
            membership_rule_identity: id(format!("membership:{tag}")),
            membership_digest: digest(4),
            membership_document: vec![],
            required_member_identities: vec![id(format!("member:{tag}"))],
            observation_sources: vec![id(format!("source:{tag}"))],
            completeness_dependencies: vec![id(format!("completeness:{tag}"))],
            progress_dependencies: vec![id(format!("progress:{tag}"))],
            clock_identity: id(format!("clock:{tag}")),
            clock_revision: id("1"),
            membership_complete: true,
            closure_identity: Some(id(format!("closure:{tag}"))),
            closure_digest: Some(digest(5)),
            kind: ScopeKind::Window {
                window_identity: id(format!("window:{tag}")),
            },
            range: ClockRange::Timestamp {
                start_nanos: 0,
                end_nanos: 30,
            },
            members: vec![Member {
                object_identity: id(format!("member:{tag}")),
                record_identity: id("unsealed-record"),
                anchor: Anchor::TimestampNanos(10),
            }],
        },
        records: vec![AdmittedRecord {
            identity: id("unsealed-record"),
            binding_identity: id(format!("binding:{tag}")),
            source_identity: id(format!("source:{tag}")),
            schema_identity: id(format!("schema:{tag}")),
            subject,
            signal_identity: id(format!("signal:{tag}")),
            trigger_identity: id(format!("trigger:{tag}")),
            unit: id("unit"),
            value: ValueState::Present {
                value_type: id("text"),
                canonical_value: tag.to_owned(),
            },
            visibility: Visibility::External,
            anchor: Anchor::TimestampNanos(10),
            event_time_nanos: 10,
            ingestion_time_nanos: 11,
            causal_relationship_identity: None,
            clock_identity: id(format!("clock:{tag}")),
            clock_revision: id("1"),
            clock_uncertainty_nanos: 1,
        }],
        limits: ResourceLimits {
            max_records: 1,
            max_members: 1,
            max_relationships: 0,
            max_required_relationships: 0,
        },
    };
    let record_identity = authority::observation::record_identity(
        &request.records[0],
        ObservationLimits::owner_max(),
    )
    .expect("record identity");
    request.records[0].identity = record_identity.clone();
    request.scope.members[0].record_identity = record_identity;
    authority::population::assign_request_identities(&mut request, ObservationLimits::owner_max())
        .expect("population identities");
    match admit(request) {
        AdmissionOutcome::Available { observation } => observation,
        other => panic!("qualification failed: {other:?}"),
    }
}

fn boundary(state: OpenClosed) -> TemporalBoundary {
    TemporalBoundary::TimestampedEvent {
        lower_nanos: 0,
        upper_inclusive_nanos: 29,
        carrier_end_exclusive_nanos: 30,
        watermark_nanos: if state == OpenClosed::Closed { 30 } else { 20 },
    }
}
fn cutoff(tag: &str) -> authority::observation::CutoffSelection {
    authority::observation::CutoffSelection::new(
        id(format!("clock:{tag}")),
        id("1"),
        40,
        authority::observation::CutoffRule::IngestionTimeAtOrBefore,
        id("1"),
    )
}
fn progress_selection(state: OpenClosed, tag: &str) -> authority::progress::Selection {
    authority::progress::Selection::new(
        authority::clock::Selection::new(id(format!("clock:{tag}")), id("1")),
        vec![id(format!("source:{tag}"))],
        boundary(state),
        state,
        id(format!("trigger:{tag}")),
        cutoff(tag),
        id(format!("restoration:{tag}")),
    )
}
fn progress_view(context: Context<'_>, state: OpenClosed, tag: &str) -> authority::progress::View {
    let selection = progress_selection(state, tag);
    let document = authority::progress::derive(context, &selection, ObservationLimits::owner_max())
        .expect("progress document");
    authority::progress::read(
        document.bytes(),
        context,
        &selection,
        ObservationLimits::owner_max(),
    )
    .expect("progress view")
}
fn closure_selection(state: OpenClosed, tag: &str) -> authority::closure::Selection {
    authority::closure::Selection::new(
        id(format!("clock:{tag}")),
        id("1"),
        vec![id(format!("source:{tag}"))],
        boundary(state),
        state,
    )
}
fn closure_view(context: Context<'_>, state: OpenClosed, tag: &str) -> authority::closure::View {
    let selection = closure_selection(state, tag);
    let document = authority::closure::derive(context, &selection, ObservationLimits::owner_max())
        .expect("closure document");
    authority::closure::read(
        document.bytes(),
        context,
        &selection,
        ObservationLimits::owner_max(),
    )
    .expect("closure view")
}
fn completeness_view(
    context: Context<'_>,
    qualified: &QualifiedObservation,
    status: authority::completeness::FactStatus,
    observed: bool,
    tag: &str,
) -> authority::completeness::View {
    let selection = authority::completeness::Selection::new(
        id(format!("boundary:{tag}")),
        vec![authority::completeness::Fact::new(
            id(format!("member:{tag}")),
            observed.then(|| qualified.records()[0].identity.clone()),
            status,
        )],
    );
    let document =
        authority::completeness::derive(context, &selection, ObservationLimits::owner_max())
            .expect("completeness document");
    authority::completeness::read(
        document.bytes(),
        context,
        &selection,
        ObservationLimits::owner_max(),
    )
    .expect("completeness view")
}

fn global_premises() -> Vec<GlobalPremise> {
    vec![
        GlobalPremise {
            kind: PremiseKind::Execution,
            identity: "premise:execution".into(),
            scope_identity: "scope:global".into(),
            authority_identity: "authority:global".into(),
        },
        GlobalPremise {
            kind: PremiseKind::Branch,
            identity: "premise:branch".into(),
            scope_identity: "scope:global".into(),
            authority_identity: "authority:global".into(),
        },
        GlobalPremise {
            kind: PremiseKind::Workflow,
            identity: "premise:workflow".into(),
            scope_identity: "scope:global".into(),
            authority_identity: "authority:global".into(),
        },
    ]
}

fn reproduction(
    seed: u64,
    toolchain_identity: &[u8],
    corpus_identity: &[u8],
    display: &str,
) -> quire_protocol::repro::ReproductionRecord {
    ReproductionInputs::nothing_offered()
        .with_seed(Seed::new(seed))
        .with_toolchain(ToolchainIdentity::new(toolchain_identity, display))
        .with_corpus(CorpusIdentity::new(corpus_identity, display))
        .record()
        .expect("complete reproduction identity")
}

fn base_input<'a>(fixture: &'a Fixture) -> Input<'a> {
    let observation_identity = fixture
        .decision
        .completeness_complete
        .payload()
        .facts()
        .next()
        .and_then(|fact| fact.observation_identity)
        .expect("available fact")
        .to_owned();
    let premises = global_premises();
    Input {
        checked_predicate: &fixture.predicate,
        temporal_subject: &fixture.temporal,
        axes: AxisInputs {
            decision_scope_progress: &fixture.decision.progress_closed,
            decision_scope_closure: &fixture.decision.closure_closed,
            surrounding_execution_progress: &fixture.surrounding.progress_closed,
            surrounding_execution_closure: &fixture.surrounding.closure_closed,
        },
        completeness: &fixture.decision.completeness_complete,
        execution: AssessmentExecution::Completed,
        truth: Truth::Satisfied,
        settlement: Settlement {
            basis: SettlementBasis::ClosedScope,
            supporting_fact_identities: vec![observation_identity.clone()],
            supporting_progress_identities: vec![fixture
                .decision
                .progress_closed
                .identity()
                .as_str()
                .to_owned()],
        },
        activation: Activation::Triggered,
        participation: Participation::Known {
            required_role_instance_refs: vec!["role:buyer".into()],
            participating_role_instance_refs: vec!["role:buyer".into()],
        },
        assumptions: vec![Assumption {
            identity: "assumption:one".into(),
            state: AssumptionState::Met,
        }],
        observation_adequacy: Adequacy::Adequate,
        protocol_adequacy: ProtocolAdequacy::Demonstrated,
        claim: Claim {
            kind: ClaimKind::GlobalConformance,
            strength: Some(ClaimStrength::Proven),
            admitted_fragment_identity: Some("fragment:finite".into()),
            backend_identity: Some("backend:model-checker".into()),
        },
        decision_support: vec![DecisionSupport {
            oracle_identity: "oracle:one".into(),
            observation_identity,
            relation: DependenceRelation::Independent,
            predecessor: None,
            dependence_group: None,
        }],
        global_conformance: GlobalConformanceInput {
            state: GlobalConformanceClosure::Closed,
            required_premises: premises.clone(),
            premises,
        },
        limitations: vec![Limitation {
            code: "bound:finite".into(),
            affected_identity: "claim:one".into(),
        }],
        provenance: ProvenanceRecord {
            source_identity: "source:one".into(),
            profile_identity: "profile:one".into(),
            model_identity: "model:one".into(),
            protocol_identity: "protocol:one".into(),
            request_identity: "request:one".into(),
            binding_identity: "binding:one".into(),
            input_identity: "input:one".into(),
            backend_identity: "backend:model-checker".into(),
            producer_identity: "producer:one".into(),
            reproduction: reproduction(
                0x5eed_0000_0000_0001,
                b"rustc-1.98.0-x86_64-unknown-linux-gnu",
                b"corpus:9f2c1ad4",
                "local reproduction inputs",
            ),
        },
        correction: None,
    }
}

fn canonical_row_controls(input: Input<'_>, limits: Limits) -> CanonicalRowControls {
    CanonicalRowControls::new(input.provenance.reproduction, limits)
}

fn temporal_digest(label: &str) -> String {
    format!(
        "{:x}",
        quire_spec_language::ByteDigest::of(label.as_bytes())
    )
}

fn temporal_evidence(scope: &str, identity: &str) -> native_temporal::EvidenceRef {
    native_temporal::EvidenceRef::new(
        "quire.owner-evidence/v1",
        temporal_digest(&format!("schema:{scope}")),
        identity,
        temporal_digest(&format!("document:{identity}")),
        format!("authority:{scope}"),
        "1",
        temporal_digest(&format!("authority:{scope}")),
        scope,
        1,
    )
}

fn activated_request(
    package: &v2::AdmittedPackage,
    subject: &temporal_subject::ValidatedTemporalSubject,
    trigger: &[u8],
) -> temporal_v2::ValidatedRequest {
    let declaration = subject.declaration();
    let leaf = package.inherited().declarations[usize::try_from(declaration).unwrap()]
        .temporal
        .iter()
        .enumerate()
        .find_map(|(index, node)| {
            matches!(node.operation, w::TemporalOperation::Holds { .. })
                .then(|| u32::try_from(index).expect("fixture temporal node fits"))
        })
        .expect("temporal fixture has a holds leaf");
    let input = temporal_v2::Input {
        trigger: temporal_v2::SemanticTriggerIdentity::new(trigger.to_vec())
            .expect("QObs subject identity is nonempty"),
        correspondence: temporal_evidence("correspondence", "correspondence:canonical-engine"),
        positions: vec![temporal_request::ObservedPosition {
            observation: temporal_evidence("observation", "observation:canonical-engine"),
            position: quire_spec_language::temporal::Position {
                coordinate: 0,
                order: Some(quire_spec_language::temporal::OrderKey {
                    authority: "events".into(),
                    key: 0,
                }),
                valuations: std::collections::BTreeMap::from([(leaf, true)]),
            },
        }],
        anchor: "origin".into(),
        triggers: vec![temporal_v2::TriggerInput {
            receipt: "receipt:canonical-engine".into(),
            anchor: "origin".into(),
            payload: String::new(),
            guard: None,
            captures: Vec::new(),
        }],
        trigger_evidence: quire_spec_language::temporal::Evidence::Admitted,
        trigger_scope: quire_spec_language::temporal::Closure::Closed,
        decision_progress: temporal_request::ProgressInput {
            reference: temporal_evidence("decision-progress", "decision-progress:canonical-engine"),
            watermark: 0,
        },
        decision_closure: temporal_request::ClosureInput {
            reference: temporal_evidence("decision-closure", "decision-closure:canonical-engine"),
            state: quire_spec_language::temporal::Closure::Closed,
        },
        surrounding_progress: temporal_request::ProgressInput {
            reference: temporal_evidence(
                "surrounding-progress",
                "surrounding-progress:canonical-engine",
            ),
            watermark: 0,
        },
        surrounding_closure: temporal_request::ClosureInput {
            reference: temporal_evidence(
                "surrounding-closure",
                "surrounding-closure:canonical-engine",
            ),
            state: quire_spec_language::temporal::Closure::Open,
        },
        execution: quire_spec_language::temporal::Execution::Completed,
        completeness: temporal_request::CompletenessInput {
            reference: temporal_evidence("completeness", "completeness:canonical-engine"),
            state: quire_spec_language::temporal::Completeness::Complete,
            facts: vec![temporal_evidence(
                "completeness-fact",
                "fact:canonical-engine",
            )],
        },
        authoritative_origin: true,
        evicted: Vec::new(),
    };
    let document = temporal_v2::produce(subject, input, native_temporal::Limits::default())
        .into_result()
        .expect("produce activation temporal request");
    temporal_v2::read(
        document.bytes(),
        subject,
        native_temporal::Limits::default(),
    )
    .into_result()
    .expect("strict-read activation temporal request")
}

fn activated_result(request: &temporal_v2::ValidatedRequest) -> temporal_v2::ValidatedResult {
    let document = temporal_v2::evaluate(
        request,
        temporal_v2::Relation::Original,
        native_temporal::Limits::default(),
    )
    .into_result()
    .expect("evaluate activation temporal result");
    temporal_v2::read_result(
        document.bytes(),
        request,
        temporal_v2::Relation::Original,
        native_temporal::Limits::default(),
    )
    .into_result()
    .expect("strict-read activation temporal result")
}

fn admitted_activation_map(package: &v2::AdmittedPackage) -> v3::AdmittedPackage {
    let loaded = handoff::Handoff::load().expect("published v2 handoff");
    let declarations = loaded.declarations();
    let inventories = loaded.inventories(&declarations);
    let (declaration, control) = package
        .inherited()
        .declarations
        .iter()
        .enumerate()
        .find_map(|(declaration, item)| {
            let w::Body::Protocol { controls, .. } = &item.body else {
                return None;
            };
            controls.iter().enumerate().find_map(|(control, item)| {
                matches!(item.operation, w::ControlOperation::Event { .. }).then_some((
                    u32::try_from(declaration).expect("published declaration index fits"),
                    u32::try_from(control).expect("published control index fits"),
                ))
            })
        })
        .expect("published handoff has an event control");
    let temporal_declaration = match &package.inherited().declarations
        [usize::try_from(declaration).expect("published declaration index fits")]
    .body
    {
        w::Body::Protocol {
            temporal_requirements,
            ..
        } => *temporal_requirements
            .first()
            .expect("selected protocol requires one temporal declaration"),
        _ => unreachable!("selected declaration is a protocol"),
    };
    let activation = v3::ExpectedActivation {
        control: w::Handle {
            declaration,
            index: control,
        },
        temporal_declaration,
    };
    let mut inherited = package.inherited().clone();
    inherited.wire = v3::WIRE.into();
    inherited.media = v3::MEDIA.into();
    inherited.schema = v3::SCHEMA.into();
    let offer = v3::wire::Package {
        inherited,
        temporal_bindings: package.package().temporal_bindings.clone(),
        activation_mappings: vec![v3::wire::ActivationMapping {
            control: activation.control.clone(),
            temporal_declaration,
        }],
    };
    let candidate = v3::encode_candidate(
        &offer,
        quire_spec_language::protocol_artifact::Limits::default(),
    )
    .into_result()
    .expect("canonical v3 mapping offer encodes");
    let mut seal = package
        .artifact()
        .expect("strict v2 handoff retained its artifact seal")
        .clone();
    seal.wire.version = "3".into();
    seal.digest = candidate.digest();
    let inherited = loaded.expected_for(&inventories, &seal);
    let expected = v3::Expected {
        inherited,
        activations: std::slice::from_ref(&activation),
    };
    v3::read(
        candidate.bytes(),
        &expected,
        quire_spec_language::protocol_artifact::Limits::default(),
    )
    .into_result()
    .expect("strict v3 mapping package admits")
}

/// The second request has no admitted fragment, so Task-005 returns a
/// claim-local refusal while retaining the established conformance disposition
/// beside it.
fn dispatch_inputs_with_claim_refusal() -> Vec<ClaimRequest<'static>> {
    let component = |identity: &[u8]| {
        Component::retained(
            identity.to_vec().into_boxed_slice(),
            identity.to_vec().into_boxed_slice(),
        )
    };
    let premises = |fragment: &[u8]| PremiseSet {
        admitted_fragment: component(fragment),
        backend: BackendSelection::retained(
            b"qprotocol:conformance".to_vec().into_boxed_slice(),
            b"v1".to_vec().into_boxed_slice(),
        ),
        assumptions: Vec::new(),
        communication: vec![component(b"communication:canonical-engine")],
        visibility: vec![component(b"visibility:canonical-engine")],
        environment: vec![component(b"environment:canonical-engine")],
        sources: vec![component(b"source:canonical-engine")],
        derived_artifact: None,
    };
    let request = |identity: &[u8], claim, fragment: &[u8]| ClaimRequest {
        identity: identity.to_vec().into_boxed_slice(),
        claim,
        strength: b"tested".to_vec().into_boxed_slice(),
        premises: premises(fragment),
        correspondence: None,
    };
    vec![
        request(
            b"request:conformance",
            DispatchClaim::Conformance,
            b"fragment:canonical-engine",
        ),
        request(
            b"request:monitorability",
            DispatchClaim::Monitorability,
            b"",
        ),
    ]
}

/// IT-002 SC-01 through SC-04's canonical-result half: the public Engine
/// constructs and strict-reads owner rows against its one linked `/2` package;
/// it has no caller-provided result document seam.
///
/// This consumes the QObs-qualified activation fact and QSL strict temporal
/// request/result only through the FR-300 binding, then retains the owned
/// decision/surrounding scope axes and complete global closure in the canonical
/// result. The paired IT-002 binding in `tc_002_binding_model` exercises the
/// same bridge's batch and incremental conformance paths and adverse joins.
#[test]
#[trace(
    "TC-006",
    "IT-002",
    "Task-010",
    "FR-006-AC-1",
    "FR-006-AC-2",
    "FR-006-AC-9",
    "FR-008-AC-1",
    "FR-009-AC-1"
)]
fn tc_006_engine_constructs_and_strict_reads_canonical_rows() {
    let loaded = handoff::Handoff::load().expect("published v2 handoff");
    let declarations = loaded.declarations();
    let inventories = loaded.inventories(&declarations);
    let expected = loaded.expected(&inventories);
    let package = quire_protocol::intake::v2::admit(loaded.offer(), &expected, loaded.limits())
        .into_result()
        .expect("strict v2 package");
    let (declaration_handle, roles, subject_slot) = package
        .inherited()
        .declarations
        .iter()
        .enumerate()
        .find_map(|(declaration, value)| {
            let w::Body::Protocol { roles, .. } = &value.body else {
                return None;
            };
            let role_instance_slots = roles.iter().map(|role| role.instance).collect::<Vec<_>>();
            let subject_slot = value
                .bindings
                .iter()
                .enumerate()
                .find(|(slot, binding)| {
                    binding.kind != w::BindingKind::Relationship
                        && !role_instance_slots.contains(&u32::try_from(*slot).unwrap_or(u32::MAX))
                })
                .map(|(slot, _)| slot)?;
            Some((
                u32::try_from(declaration).ok()?,
                roles,
                u32::try_from(subject_slot).ok()?,
            ))
        })
        .expect("published handoff has a protocol subject slot");
    let producer = producer();
    let subject = runtime_subject(&producer, "canonical-engine");
    let role_instance_kind =
        SubjectKind::new(b"role-instance".to_vec()).expect("non-empty role-instance kind");
    let mut static_slots = vec![StaticSlot {
        handle: SlotHandle::new(subject_slot),
        compatibility: SlotCompatibility::Subject {
            kind: SubjectKind::new(ORDER_KIND.as_bytes().to_vec()).expect("non-empty kind"),
        },
    }];
    static_slots.extend(roles.iter().map(|role| StaticSlot {
        handle: SlotHandle::new(role.instance),
        compatibility: SlotCompatibility::Subject {
            kind: role_instance_kind.clone(),
        },
    }));
    let definition = StaticBindingDefinition::new(
        STATIC_BINDING_CONTRACT,
        StaticSelection {
            identity: "binding-definition:canonical-engine".into(),
            revision: "1".into(),
            digest: [0x61; 32],
        },
        producer.key().clone(),
        declaration_handle,
        roles
            .iter()
            .enumerate()
            .map(|(index, _)| RoleClassification {
                role_handle: u32::try_from(index).expect("published role index fits"),
                internality: Internality::Internal,
            })
            .collect(),
        static_slots,
    )
    .expect("selected static binding definition admits");
    let mut runtime_bindings = vec![RuntimeBinding::subject(
        SlotHandle::new(subject_slot),
        subject.clone(),
    )];
    runtime_bindings.extend(roles.iter().enumerate().map(|(index, role)| {
        RuntimeBinding::subject(
            SlotHandle::new(role.instance),
            runtime_subject_bytes(
                &producer,
                b"role-instance".to_vec(),
                format!("role-instance:canonical-engine:{index}").into_bytes(),
            ),
        )
    }));
    let runtime = RuntimeBindingSet::new(
        RUNTIME_BINDING_CONTRACT,
        definition.selection(),
        &definition,
        &producer,
        runtime_bindings,
    )
    .expect("selected runtime binding admits");
    let context = LinkedAssessment::new(&package, &definition, &runtime)
        .expect("one exact linked assessment");
    let definition_identity = context
        .definition_identity()
        .expect("linked assessment has exact definition provenance");
    let selected_model = definition_identity
        .closure()
        .models()
        .first()
        .expect("published protocol selects a model");
    let model_artifact = &package.inherited().dependencies
        [usize::try_from(selected_model.value().artifact).expect("model index fits")]
    .artifact;

    let fixture = fixture();
    let mut input = base_input(&fixture);
    assert_eq!(input.checked_predicate.package_digest(), package.digest());
    assert_eq!(input.temporal_subject.package_digest(), package.digest());
    let requested = [DispatchClaim::Conformance, DispatchClaim::Monitorability];
    let mapping = admitted_activation_map(&package);
    let temporal_request =
        activated_request(&package, &fixture.temporal, subject.identity().as_bytes());
    let temporal_result = activated_result(&temporal_request);
    let quire_protocol::outcome::OperationOutcome::Completed(engine) =
        producer_resolution(b"tc006:activated-engine").start_activated(
            context,
            requested.to_vec(),
            &mapping,
            &subject,
            &fixture.temporal,
            &temporal_request,
            &temporal_result,
        )
    else {
        panic!("linked assessment starts one activation-bound engine")
    };
    assert_eq!(
        engine.producer_resolution().request().as_bytes(),
        b"tc006:activated-engine",
    );
    let (foreign_declaration, foreign_root) = first_checked_leaf(&package);
    assert!(!fixture
        .temporal
        .predicate_leaves()
        .expect("temporal subject has predicate leaves")
        .contains(&foreign_root));
    let foreign_selection =
        checked_predicate::ClauseSelection::new(foreign_declaration, foreign_root);
    let foreign_document = checked_predicate::derive(
        &package,
        foreign_selection.clone(),
        checked_predicate::Limits::default(),
    )
    .into_result()
    .expect("same-package foreign checked predicate derives");
    let foreign_predicate = checked_predicate::read(
        foreign_document.bytes(),
        &package,
        foreign_selection,
        checked_predicate::Limits::default(),
    )
    .into_result()
    .expect("same-package foreign checked predicate strict-reads");
    assert!(matches!(
        producer_resolution(b"tc006:foreign-result-engine").start_result_assessment(
            context,
            requested.to_vec(),
            &mapping,
            &subject,
            &temporal_request,
            &temporal_result,
            &foreign_predicate,
            &fixture.temporal,
        ),
        quire_protocol::outcome::OperationOutcome::Refused(refusal)
            if refusal.code().as_str()
                == "assessment-result.checked-predicate-declaration-mismatch"
    ));
    let activation = engine
        .conformance()
        .bind_activation(
            engine.linked(),
            &mapping,
            &subject,
            &fixture.temporal,
            &temporal_request,
            &temporal_result,
        )
        .expect("the strict mapping binds the canonical assessment subject");
    let occurrence_schema = occurrence_key_schema(
        &package,
        declaration_handle,
        quire_spec_language::protocol_artifact::Limits::default(),
    )
    .into_result()
    .expect("selected protocol produces an occurrence-key schema");
    let occurrence_node = occurrence_schema
        .nodes()
        .get(usize::try_from(activation.control().index).expect("control index fits"))
        .expect("activation control has an occurrence schema");
    assert!(occurrence_node.repeats().is_empty());
    let occurrence = occurrence_schema
        .bind(
            WorkflowInstanceIdentity::new("workflow:canonical-engine")
                .expect("nonempty workflow identity"),
            activation.control(),
            occurrence_node.role().clone(),
            &[],
        )
        .expect("activation occurrence admits");
    let activated = engine
        .conformance()
        .activate_occurrence(&activation, occurrence)
        .expect("mapped activation produces an obligation occurrence");
    let decision = observation_owner::scope_views("canonical-engine");
    let surrounding = observation_owner::scope_views("canonical-engine-surrounding");
    let support_observation = decision
        .completeness_complete
        .payload()
        .facts()
        .next()
        .and_then(|fact| fact.observation_identity)
        .expect("the assessed completeness view retains its available fact")
        .to_owned();
    input.settlement.supporting_fact_identities = vec![support_observation.clone()];
    input.settlement.supporting_progress_identities =
        vec![decision.progress_closed.identity().as_str().to_owned()];
    input.decision_support[0].observation_identity = support_observation;
    let fact = engine
        .conformance()
        .activation_fact(activated, Some(&decision.observation));
    let global_premises = vec![
        quire_protocol::assessment::GlobalPremise {
            kind: quire_protocol::assessment::PremiseKind::Execution,
            identity: "premise:canonical-engine:execution".into(),
            scope_identity: "scope:canonical-engine".into(),
            authority_identity: "authority:canonical-engine".into(),
        },
        quire_protocol::assessment::GlobalPremise {
            kind: quire_protocol::assessment::PremiseKind::Branch,
            identity: "premise:canonical-engine:branch".into(),
            scope_identity: "scope:canonical-engine".into(),
            authority_identity: "authority:canonical-engine".into(),
        },
        quire_protocol::assessment::GlobalPremise {
            kind: quire_protocol::assessment::PremiseKind::Workflow,
            identity: "premise:canonical-engine:workflow".into(),
            scope_identity: "scope:canonical-engine".into(),
            authority_identity: "authority:canonical-engine".into(),
        },
    ];
    let global = quire_protocol::assessment::GlobalConformanceInput {
        state: GlobalConformanceClosure::Closed,
        required_premises: global_premises.clone(),
        premises: global_premises,
    };
    let axes = quire_protocol::assessment::AxisInputs {
        decision_scope_progress: &decision.progress_closed,
        decision_scope_closure: &decision.closure_closed,
        surrounding_execution_progress: &surrounding.progress_closed,
        surrounding_execution_closure: &surrounding.closure_closed,
    };
    let mut incomplete_requests = dispatch_inputs_with_claim_refusal();
    incomplete_requests.pop();
    assert!(matches!(
        engine.dispatch_activated(
            &incomplete_requests,
            std::slice::from_ref(&fact),
            axes,
            &decision.completeness_complete,
            &global,
            AssessmentExecution::Completed,
            ConformanceLimits::default(),
            DispatchLimits::default(),
            Budget::nothing_granted()
        ),
        quire_protocol::outcome::OperationOutcome::Refused(_)
    ));
    let dispatch_requests = dispatch_inputs_with_claim_refusal();
    let quire_protocol::outcome::OperationOutcome::Completed(unselected_engine) =
        Engine::start(context, requested.to_vec())
    else {
        panic!("linked assessment starts an engine without an activation selection")
    };
    assert!(matches!(
        unselected_engine.dispatch_activated(
            &dispatch_requests,
            std::slice::from_ref(&fact),
            axes,
            &decision.completeness_complete,
            &global,
            AssessmentExecution::Completed,
            ConformanceLimits::default(),
            DispatchLimits::default(),
            Budget::nothing_granted(),
        ),
        quire_protocol::outcome::OperationOutcome::Refused(refusal)
            if refusal.code().as_str() == "assessment-activation.missing-selection"
    ));
    let foreign_subject = runtime_subject(&producer, "canonical-engine-foreign");
    let foreign_request = activated_request(
        &package,
        &fixture.temporal,
        foreign_subject.identity().as_bytes(),
    );
    let foreign_result = activated_result(&foreign_request);
    let foreign_activation = engine
        .conformance()
        .bind_activation(
            engine.linked(),
            &mapping,
            &foreign_subject,
            &fixture.temporal,
            &foreign_request,
            &foreign_result,
        )
        .expect("a distinct subject forms its own valid activation");
    let foreign_occurrence = occurrence_schema
        .bind(
            WorkflowInstanceIdentity::new("workflow:canonical-engine-foreign")
                .expect("nonempty workflow identity"),
            foreign_activation.control(),
            occurrence_node.role().clone(),
            &[],
        )
        .expect("the foreign activation occurrence admits");
    let foreign_activated = engine
        .conformance()
        .activate_occurrence(&foreign_activation, foreign_occurrence)
        .expect("the distinct activation produces an occurrence");
    let foreign_fact = engine
        .conformance()
        .activation_fact(foreign_activated, None);
    let quire_protocol::outcome::OperationOutcome::Completed(fact_bound_engine) =
        producer_resolution(b"tc006:result-engine").start_result_assessment(
            context,
            requested.to_vec(),
            &mapping,
            &subject,
            &temporal_request,
            &temporal_result,
            &fixture.predicate,
            &fixture.temporal,
        )
    else {
        panic!("strict owner views start the activation-fact binding check")
    };
    assert!(matches!(
        fact_bound_engine.dispatch_activated(
            &dispatch_requests,
            std::slice::from_ref(&foreign_fact),
            axes,
            &decision.completeness_complete,
            &global,
            AssessmentExecution::Completed,
            ConformanceLimits::default(),
            DispatchLimits::default(),
            Budget::nothing_granted(),
        ),
        quire_protocol::outcome::OperationOutcome::Refused(refusal)
            if refusal.code().as_str() == "assessment-activation.fact-mismatch"
    ));
    let quire_protocol::outcome::OperationOutcome::Completed(dispatch) = engine.dispatch_activated(
        &dispatch_requests,
        std::slice::from_ref(&fact),
        axes,
        &decision.completeness_complete,
        &global,
        AssessmentExecution::Completed,
        ConformanceLimits::default(),
        DispatchLimits::default(),
        Budget::nothing_granted(),
    ) else {
        panic!("engine dispatches only its exact claim population")
    };
    assert_eq!(dispatch.dispositions()[0].claim, DispatchClaim::Conformance);
    assert_eq!(
        dispatch.dispositions()[0].execution,
        AssessmentExecution::Completed
    );
    assert_eq!(
        dispatch.dispositions()[0].decision,
        Some(quire_protocol::claims::dispatch::Decision::Established)
    );
    assert_eq!(
        dispatch.dispositions()[1].claim,
        DispatchClaim::Monitorability
    );
    assert_eq!(
        dispatch.dispositions()[1].execution,
        AssessmentExecution::Refused
    );
    assert!(dispatch.dispositions()[1].failure.is_some());
    let short_budget = Budget::nothing_granted()
        .grant(BudgetDimension::InputByte, 1)
        .expect("positive one-short input grant");
    assert!(matches!(
        engine.dispatch_activated(
            &dispatch_requests,
            std::slice::from_ref(&fact),
            axes,
            &decision.completeness_complete,
            &global,
            AssessmentExecution::Completed,
            ConformanceLimits::default(),
            DispatchLimits::default(),
            short_budget,
        ),
        quire_protocol::outcome::OperationOutcome::ResourceIncomplete(incomplete)
            if incomplete.dimension() == BudgetDimension::InputByte
    ));
    let short_result_engine = match producer_resolution(b"tc006:short-output-assessment")
        .start_result_assessment(
            context,
            requested.to_vec(),
            &mapping,
            &subject,
            &temporal_request,
            &temporal_result,
            &fixture.predicate,
            &fixture.temporal,
        ) {
        quire_protocol::outcome::OperationOutcome::Completed(engine) => engine,
        quire_protocol::outcome::OperationOutcome::Refused(refusal) => {
            panic!("strict owner views refused: {}", refusal.code().as_str())
        }
        quire_protocol::outcome::OperationOutcome::ResourceIncomplete(_) => {
            panic!("strict owner views exhausted resources during setup")
        }
    };
    let short_output = Budget::nothing_granted()
        .grant(BudgetDimension::OutputByte, 1)
        .expect("positive one-short output grant");
    let short_outcome = short_result_engine.assess_activated(
        &dispatch_requests,
        std::slice::from_ref(&fact),
        axes,
        &decision.completeness_complete,
        &global,
        AssessmentExecution::Completed,
        ConformanceLimits::default(),
        DispatchLimits::default(),
        vec![
            Some(canonical_row_controls(input.clone(), Limits::owner_max())),
            None,
        ],
        short_output,
    );
    assert!(
        matches!(
            short_outcome,
            quire_protocol::outcome::OperationOutcome::ResourceIncomplete(ref incomplete)
                if incomplete.dimension() == BudgetDimension::OutputByte
        ),
        "expected output exhaustion"
    );
    let quire_protocol::outcome::OperationOutcome::Completed(cumulative_engine) =
        producer_resolution(b"tc006:cumulative-assessment").start_result_assessment(
            context,
            requested.to_vec(),
            &mapping,
            &subject,
            &temporal_request,
            &temporal_result,
            &fixture.predicate,
            &fixture.temporal,
        )
    else {
        panic!("strict owner views start the accountant-span control")
    };
    let cumulative_limit = Budget::nothing_granted()
        .grant(BudgetDimension::AccountedAllocation, 3)
        .expect("three allocations reach into result construction");
    assert!(matches!(
        cumulative_engine.assess_activated(
            &dispatch_requests,
            std::slice::from_ref(&fact),
            axes,
            &decision.completeness_complete,
            &global,
            AssessmentExecution::Completed,
            ConformanceLimits::default(),
            DispatchLimits::default(),
            vec![
                Some(canonical_row_controls(input.clone(), Limits::owner_max())),
                None,
            ],
            cumulative_limit,
        ),
        quire_protocol::outcome::OperationOutcome::ResourceIncomplete(incomplete)
            if incomplete.dimension() == BudgetDimension::AccountedAllocation
                && incomplete.consumed() == 3
                && incomplete.requested() == 1
    ));
    let quire_protocol::outcome::OperationOutcome::Completed(owner_limited_engine) =
        producer_resolution(b"tc006:owner-limited-assessment").start_result_assessment(
            context,
            requested.to_vec(),
            &mapping,
            &subject,
            &temporal_request,
            &temporal_result,
            &fixture.predicate,
            &fixture.temporal,
        )
    else {
        panic!("strict owner views start the owner-limit control")
    };
    let mut owner_limited = Limits::owner_max();
    owner_limited.max_string_bytes = 1;
    assert!(matches!(
        owner_limited_engine.assess_activated(
            &dispatch_requests,
            std::slice::from_ref(&fact),
            axes,
            &decision.completeness_complete,
            &global,
            AssessmentExecution::Completed,
            ConformanceLimits::default(),
            DispatchLimits::default(),
            vec![Some(canonical_row_controls(input.clone(), owner_limited)), None],
            Budget::nothing_granted(),
        ),
        quire_protocol::outcome::OperationOutcome::ResourceIncomplete(incomplete)
            if incomplete.dimension() == BudgetDimension::OutputByte
                && incomplete.site().as_str() == "result/output/string/retain"
    ));
    let quire_protocol::outcome::OperationOutcome::Completed(result_engine) =
        producer_resolution(b"tc006:completed-assessment").start_result_assessment(
            context,
            requested.to_vec(),
            &mapping,
            &subject,
            &temporal_request,
            &temporal_result,
            &fixture.predicate,
            &fixture.temporal,
        )
    else {
        panic!("strict owner views start canonical result orchestration")
    };
    let outcome = result_engine.assess_activated(
        &dispatch_requests,
        std::slice::from_ref(&fact),
        axes,
        &decision.completeness_complete,
        &global,
        AssessmentExecution::Completed,
        ConformanceLimits::default(),
        DispatchLimits::default(),
        vec![
            Some(canonical_row_controls(input, Limits::owner_max())),
            None,
        ],
        Budget::nothing_granted(),
    );
    let quire_protocol::outcome::OperationOutcome::Completed(bundle) = outcome else {
        panic!("the complete assessment returns its bundle only in the completed outcome")
    };
    assert_eq!(bundle.rows().len(), 2);
    let quire_protocol::outcome::ClaimOutcome::Canonical(canonical) = &bundle.rows()[0].outcome
    else {
        panic!("the completed conformance disposition must produce its canonical row")
    };
    assert_eq!(canonical.execution(), AssessmentExecution::Completed);
    assert_eq!(canonical.truth(), Truth::Unavailable);
    assert_eq!(canonical.settlement_basis(), SettlementBasis::Unavailable);
    assert!(canonical.decision_support().is_empty());
    let Participation::Known {
        required_role_instance_refs,
        participating_role_instance_refs,
    } = canonical.participation()
    else {
        panic!("complete activated role bindings produce known participation")
    };
    assert_eq!(
        required_role_instance_refs,
        participating_role_instance_refs
    );
    assert!(!required_role_instance_refs.is_empty());
    assert!(required_role_instance_refs
        .iter()
        .all(|identity| identity.starts_with(runtime_reference::PREFIX)));
    assert_eq!(
        canonical.decision_scope_progress().document_identity(),
        decision.progress_closed.identity().as_str()
    );
    assert_eq!(
        canonical.completeness().document_identity(),
        decision.completeness_complete.identity().as_str()
    );
    assert_eq!(
        canonical.global_conformance_state(),
        GlobalConformanceClosure::Closed
    );
    assert_eq!(canonical.claim().kind, ClaimKind::GlobalConformance);
    assert_eq!(canonical.claim().strength, Some(ClaimStrength::Tested));
    assert_eq!(
        canonical.claim().admitted_fragment_identity.as_deref(),
        Some("fragment:canonical-engine")
    );
    assert_eq!(
        canonical.claim().backend_identity.as_deref(),
        Some("qprotocol:conformance")
    );
    assert_eq!(
        canonical.provenance().source_identity,
        definition_identity.source_document().identity
    );
    assert_eq!(
        canonical.provenance().profile_identity,
        definition_identity.closure().profile().value().identity
    );
    assert_eq!(
        canonical.provenance().model_identity,
        model_artifact.identity
    );
    assert_eq!(
        canonical.provenance().protocol_identity,
        format!(
            "{}:{}",
            definition_identity.source_document().identity,
            definition_identity.declaration_handle()
        )
    );
    assert_eq!(
        canonical.provenance().request_identity,
        "request:conformance"
    );
    assert_eq!(
        canonical.provenance().binding_identity,
        definition_identity.static_binding().identity.as_ref()
    );
    runtime_reference::validate_subject(&canonical.provenance().input_identity, &subject)
        .expect("Engine provenance retains the exact runtime subject carrier");
    assert_eq!(
        canonical.provenance().backend_identity,
        "qprotocol:conformance"
    );
    assert_eq!(
        canonical.provenance().producer_identity,
        context.definition().producer().bundle_identity
    );
    assert!(canonical.assumptions().is_empty());
    let quire_protocol::outcome::ClaimOutcome::Refused(refusal) = &bundle.rows()[1].outcome else {
        panic!("only the constrained claim construction must refuse");
    };
    assert_eq!(
        refusal.partial_output(),
        quire_protocol::refusal::PartialOutput::IndependentResultsRetained
    );
    assert_eq!(
        refusal.stage(),
        quire_protocol::refusal::ProtocolStage::ClaimDispatch
    );
}

fn validated<'a>(input: Input<'a>) -> result::ValidatedResult {
    let document = result::produce(input.clone(), Limits::owner_max()).expect("canonical result");
    result::read(
        document.bytes(),
        result::Expected {
            input,
            enclosing_digest: document.digest(),
        },
        Limits::owner_max(),
    )
    .expect("strict result")
}

fn assert_result_mutation(label: &str, baseline: &result::Document, input: Input<'_>) {
    match result::produce(input, Limits::owner_max()) {
        Ok(mutant) => {
            assert_ne!(
                mutant.result_id(),
                baseline.result_id(),
                "{label} did not affect the result identity"
            );
            assert_ne!(
                mutant.bytes(),
                baseline.bytes(),
                "{label} did not affect the canonical result bytes"
            );
        }
        Err(refusal) => {
            assert!(
                !refusal.code().as_str().is_empty(),
                "{label} returned an untyped refusal"
            );
            assert_eq!(
                refusal.envelope().partial_output(),
                quire_protocol::refusal::PartialOutput::NoneEmitted,
                "{label} exposed partial output beside its refusal"
            );
            assert_eq!(
                refusal.envelope().stage(),
                ProtocolStage::ResultAssembly,
                "{label} was not raised by result assembly"
            );
        }
    }
}

#[test]
#[trace("TC-008", "NFR-001-AC-1", "FR-017-AC-5", "NFR-008-AC-7")]
fn tc_008_result_bytes_ignore_allocation_layout_and_localized_rendering() {
    let first_fixture = fixture();
    let first_input = base_input(&first_fixture);
    let first = result::produce(first_input, Limits::owner_max()).expect("first result");

    // Perturb allocator history and rebuild every owner view independently.
    let allocation_noise: Vec<Vec<u8>> = (0_u16..257)
        .map(|length| vec![0xa5; usize::from(length)])
        .collect();
    assert_eq!(allocation_noise.len(), 257);
    let second_fixture = fixture();
    let mut second_input = base_input(&second_fixture);
    second_input.provenance.reproduction = reproduction(
        0x5eed_0000_0000_0001,
        b"rustc-1.98.0-x86_64-unknown-linux-gnu",
        b"corpus:9f2c1ad4",
        "Yerelleştirilmiş çalıştırma",
    );
    let second = result::produce(second_input, Limits::owner_max()).expect("second result");

    assert_eq!(first.result_id(), second.result_id());
    assert_eq!(first.digest(), second.digest());
    assert_eq!(first.bytes(), second.bytes());

    let json: Value = serde_json::from_slice(first.bytes()).expect("owner JSON");
    let reproduction = &json["provenance"]["reproduction"];
    assert_eq!(reproduction["seed"], "6840123409045651457");
    assert_eq!(
        reproduction["toolchainIdentity"],
        "72757374632d312e39382e302d7838365f36342d756e6b6e6f776e2d6c696e75782d676e75"
    );
    assert_eq!(
        reproduction["corpusIdentity"],
        "636f727075733a3966326331616434"
    );
}

#[test]
#[trace("TC-008", "NFR-001-AC-1")]
fn tc_008_process_locale_does_not_change_canonical_result() {
    const CHILD: &str = "QUIRE_TC008_LOCALE_CHILD";
    const MARKER: &str = "TC008-CANONICAL:";
    if std::env::var_os(CHILD).is_some() {
        let owner = fixture();
        let document =
            result::produce(base_input(&owner), Limits::owner_max()).expect("locale result");
        println!("{MARKER}{}:{}", document.result_id(), document.digest());
        return;
    }

    fn identity_under(locale: &str, child: &str, marker: &str) -> String {
        let output = std::process::Command::new(
            std::env::current_exe().expect("current integration-test executable"),
        )
        .args([
            "--exact",
            "result::tc_006_tests::tc_008_process_locale_does_not_change_canonical_result",
            "--nocapture",
        ])
        .env(child, "1")
        .env("LC_ALL", locale)
        .output()
        .expect("locale-isolated child process");
        assert!(
            output.status.success(),
            "locale child {locale} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout)
            .expect("Rust test output is UTF-8")
            .lines()
            .find_map(|line| line.strip_prefix(marker).map(str::to_owned))
            .expect("locale child emitted the canonical marker")
    }

    assert_eq!(
        identity_under("C", CHILD, MARKER),
        identity_under("tr_TR.UTF-8", CHILD, MARKER)
    );
}

#[test]
#[trace("TC-008", "NFR-001-AC-2", "NFR-001-AC-3", "FR-031-AC-5")]
fn tc_008_every_named_semantic_identity_is_discriminated() {
    let owner = fixture();
    let baseline = result::produce(base_input(&owner), Limits::owner_max()).expect("baseline");

    macro_rules! mutate {
        ($label:literal, |$input:ident| $mutation:block) => {{
            let mut input = base_input(&owner);
            let $input = &mut input;
            $mutation
            assert_result_mutation($label, &baseline, input);
        }};
    }

    mutate!("source", |input| {
        input.provenance.source_identity = "source:two".into();
    });
    mutate!("accepted source-baseline/profile", |input| {
        input.provenance.profile_identity = "profile:two".into();
    });
    mutate!("model", |input| {
        input.provenance.model_identity = "model:two".into();
    });
    mutate!("compiled protocol", |input| {
        input.provenance.protocol_identity = "protocol:two".into();
    });
    mutate!("request", |input| {
        input.provenance.request_identity = "request:two".into();
    });
    mutate!("binding", |input| {
        input.provenance.binding_identity = "binding:two".into();
    });
    mutate!("input", |input| {
        input.provenance.input_identity = "input:two".into();
    });
    mutate!("producer", |input| {
        input.provenance.producer_identity = "producer:two".into();
    });
    mutate!("backend", |input| {
        input.provenance.backend_identity = "backend:two".into();
        input.claim.backend_identity = Some("backend:two".into());
    });
    mutate!("implementation", |input| {
        input.provenance.reproduction = reproduction(
            0x5eed_0000_0000_0001,
            b"rustc-1.99.0-x86_64-unknown-linux-gnu",
            b"corpus:9f2c1ad4",
            "implementation mutation",
        );
    });
    mutate!("observation", |input| {
        input.decision_support[0].observation_identity = "observation:substituted".into();
    });
    mutate!("decision-scope progress", |input| {
        input.axes.decision_scope_progress = &owner.decision.progress_open;
    });
    mutate!("decision-scope closure", |input| {
        input.axes.decision_scope_closure = &owner.decision.closure_open;
    });
    mutate!("surrounding-execution progress", |input| {
        input.axes.surrounding_execution_progress = &owner.surrounding.progress_open;
    });
    mutate!("surrounding-execution closure", |input| {
        input.axes.surrounding_execution_closure = &owner.surrounding.closure_open;
    });
    mutate!("complete-global closure", |input| {
        input.global_conformance.state = GlobalConformanceClosure::Incomplete;
    });

    // Source identity bytes are exact: a presentation-changing trim would be
    // a semantic rewrite, not an equivalent presentation.
    mutate!("identity-changing source presentation", |input| {
        input.provenance.source_identity = " source:one ".into();
    });

    // The execution/branch/workflow list is an exact authored sequence. It is
    // not presentation-sorted into acceptance.
    mutate!("authored global-premise order", |input| {
        input.global_conformance.premises.swap(0, 1);
    });
}

#[test]
#[trace("TC-008", "TC-009", "NFR-002-AC-1")]
fn tc_009_reproduction_provenance_obeys_result_string_bounds() {
    let owner = fixture();
    let mut input = base_input(&owner);
    input.provenance.reproduction = reproduction(1, b"abc", b"d", "bounded identities");
    let limits = Limits {
        max_string_bytes: 5,
        ..Limits::owner_max()
    };
    let refusal = result::produce(input, limits).expect_err("six encoded bytes exceed five");
    assert_eq!(refusal.code(), ErrorCode::ResourceIncomplete);
    assert_eq!(refusal.usage().string_bytes, 6);
    assert_eq!(
        refusal.envelope().partial_output(),
        quire_protocol::refusal::PartialOutput::NoneEmitted
    );
}

#[test]
#[trace(
    "TC-006",
    "TC-015",
    "FR-006-AC-1",
    "FR-031-AC-1",
    "FR-031-AC-2",
    "FR-031-AC-3",
    "FR-042-AC-1",
    "FR-042-AC-4",
    "FR-042-AC-5"
)]
fn tc_015_every_result_combination_maps_without_boolean_coercion() {
    let fixture = fixture();
    let executions = AssessmentExecution::ALL;
    let truths = [
        Truth::Satisfied,
        Truth::Violated,
        Truth::Pending,
        Truth::Unavailable,
    ];
    let settlements = [
        SettlementBasis::ClosedScope,
        SettlementBasis::DecisiveWitness,
        SettlementBasis::DecisiveCounterexample,
        SettlementBasis::Unsettled,
        SettlementBasis::Unavailable,
    ];
    let activations = [
        Activation::Triggered,
        Activation::Inactive,
        Activation::ActivationUnknown,
    ];
    let scope_states = [OpenClosed::Open, OpenClosed::Closed];
    let mut admitted = 0usize;
    let mut refused = 0usize;

    for execution in executions {
        for truth in truths {
            for settlement in settlements {
                for activation in activations {
                    for progress in scope_states {
                        for closure in scope_states {
                            for has_support in [false, true] {
                                let expected = expected_result_combination(
                                    execution,
                                    truth,
                                    settlement,
                                    activation,
                                    progress,
                                    closure,
                                    has_support,
                                );
                                let mut input = base_input(&fixture);
                                input.execution = execution;
                                input.truth = truth;
                                input.settlement.basis = settlement;
                                input.activation = activation;
                                input.axes.decision_scope_progress = match progress {
                                    OpenClosed::Open => &fixture.decision.progress_open,
                                    OpenClosed::Closed => &fixture.decision.progress_closed,
                                };
                                input.axes.decision_scope_closure = match closure {
                                    OpenClosed::Open => &fixture.decision.closure_open,
                                    OpenClosed::Closed => &fixture.decision.closure_closed,
                                };
                                input.settlement.supporting_progress_identities = vec![input
                                    .axes
                                    .decision_scope_progress
                                    .identity()
                                    .as_str()
                                    .to_owned()];
                                if !has_support {
                                    input.decision_support.clear();
                                    input.settlement.supporting_fact_identities.clear();
                                }

                                match result::produce(input.clone(), Limits::owner_max()) {
                                    Ok(document) => {
                                        assert!(expected, "unexpectedly admitted {input:?}");
                                        let source = result::read(
                                            document.bytes(),
                                            result::Expected {
                                                input,
                                                enclosing_digest: document.digest(),
                                            },
                                            Limits::owner_max(),
                                        )
                                        .expect("every admitted row strict-reads");
                                        let mapped = contract_ir::map(
                                            &source,
                                            contract_ir::MappingSelection::current(),
                                            Limits::owner_max(),
                                        )
                                        .expect("every admitted row maps");
                                        let expected_value = match (execution, truth) {
                                            (AssessmentExecution::Completed, Truth::Satisfied) => {
                                                Some(true)
                                            }
                                            (AssessmentExecution::Completed, Truth::Violated) => {
                                                Some(false)
                                            }
                                            _ => None,
                                        };
                                        assert_eq!(mapped.value(), expected_value);
                                        match (mapped.non_value(), expected_value) {
                                            (Some(non_value), None) => {
                                                assert_eq!(non_value.execution(), execution);
                                                assert_eq!(non_value.truth(), truth);
                                                assert_eq!(non_value.settlement(), settlement);
                                                assert_eq!(
                                                    non_value.completeness(),
                                                    source.completeness_state()
                                                );
                                            }
                                            (None, Some(_)) => {}
                                            _ => panic!("value/non-value discriminator diverged"),
                                        }
                                        let reread = contract_ir::read(
                                            mapped.bytes(),
                                            &source,
                                            contract_ir::MappingSelection::current(),
                                            Limits::owner_max(),
                                        )
                                        .expect("every admitted mapping strict-reads");
                                        assert_eq!(reread.bytes(), mapped.bytes());
                                        admitted += 1;
                                    }
                                    Err(error) => {
                                        assert!(!expected, "unexpected refusal: {error:?}");
                                        assert_eq!(error.code(), ErrorCode::InvalidCombination);
                                        refused += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    assert_eq!(admitted, 126);
    assert_eq!(refused, 2_274);
}

fn expected_result_combination(
    execution: AssessmentExecution,
    truth: Truth,
    settlement: SettlementBasis,
    activation: Activation,
    progress: OpenClosed,
    closure: OpenClosed,
    has_support: bool,
) -> bool {
    use AssessmentExecution as E;
    use SettlementBasis as S;
    use Truth as T;

    match (execution, truth, settlement, activation) {
        (E::Completed, T::Satisfied | T::Violated, S::ClosedScope, Activation::Triggered) => {
            progress == OpenClosed::Closed && closure == OpenClosed::Closed && has_support
        }
        (E::Completed, T::Satisfied, S::DecisiveWitness, Activation::Triggered)
        | (E::Completed, T::Violated, S::DecisiveCounterexample, Activation::Triggered) => {
            closure == OpenClosed::Open && has_support
        }
        (E::Completed, T::Pending, S::Unsettled, Activation::Triggered) => {
            progress == OpenClosed::Open
        }
        (
            E::Completed,
            T::Unavailable,
            S::Unavailable,
            Activation::Inactive | Activation::ActivationUnknown,
        ) => true,
        (E::Completed, T::Unavailable, S::Unavailable, Activation::Triggered) => !has_support,
        (
            E::Refused | E::Unsupported | E::ResourceIncomplete | E::Failed,
            T::Unavailable,
            S::Unavailable,
            _,
        ) => true,
        _ => false,
    }
}

#[test]
#[trace(
    "TC-006",
    "TC-015",
    "FR-006-AC-1",
    "FR-032-AC-3",
    "FR-042-AC-1",
    "FR-042-AC-3",
    "FR-042-AC-5"
)]
fn tc_015_validated_and_mapped_views_expose_every_contract_axis_without_json_parsing() {
    let fixture = fixture();
    let source = validated(base_input(&fixture));

    assert_eq!(source.execution(), AssessmentExecution::Completed);
    assert_eq!(source.truth(), Truth::Satisfied);
    assert_eq!(source.activation(), Activation::Triggered);
    assert_eq!(source.observation_adequacy(), Adequacy::Adequate);
    assert_eq!(source.protocol_adequacy(), ProtocolAdequacy::Demonstrated);
    assert_eq!(source.claim().kind, ClaimKind::GlobalConformance);
    assert_eq!(source.assumptions().len(), 1);
    assert!(matches!(
        source.participation(),
        Participation::Known { .. }
    ));
    assert_eq!(
        source.global_conformance_state(),
        GlobalConformanceClosure::Closed
    );
    assert_eq!(source.global_premises().len(), 3);
    assert_eq!(source.limitations().len(), 1);
    assert_eq!(source.provenance().source_identity, "source:one");
    assert_eq!(source.decision_scope_progress().state(), ScopeState::Closed);
    assert_eq!(source.decision_scope_closure().state(), ScopeState::Closed);
    assert_eq!(
        source.surrounding_execution_progress().state(),
        ScopeState::Closed
    );
    assert_eq!(
        source.surrounding_execution_closure().state(),
        ScopeState::Closed
    );
    assert_eq!(
        source.completeness().state(),
        authority::completeness::State::Complete
    );
    assert_eq!(source.completeness().fact_identities().len(), 1);
    assert!(source.correction().is_none());

    let mapped = contract_ir::map(
        &source,
        contract_ir::MappingSelection::current(),
        Limits::owner_max(),
    )
    .expect("complete typed mapping view");
    assert_eq!(mapped.execution(), source.execution());
    assert_eq!(mapped.truth(), source.truth());
    assert_eq!(
        mapped.native_subject_identity(),
        source.native_subject_identity()
    );
    assert_eq!(
        mapped.native_correspondence_identity(),
        source.native_correspondence_identity()
    );
    assert_eq!(
        mapped.decision_scope_progress().document_identity(),
        source.decision_scope_progress().document_identity()
    );
    assert_eq!(
        mapped.decision_scope_progress().contract(),
        source.decision_scope_progress().contract()
    );
    assert_eq!(
        mapped.decision_scope_progress().schema_sha256(),
        source.decision_scope_progress().schema_sha256()
    );
    assert_eq!(
        mapped.decision_scope_progress().document_digest(),
        source.decision_scope_progress().document_digest()
    );
    assert_eq!(
        mapped.decision_scope_progress().authority_identity(),
        source.decision_scope_progress().authority_identity()
    );
    assert_eq!(
        mapped.decision_scope_progress().authority_revision(),
        source.decision_scope_progress().authority_revision()
    );
    assert_eq!(
        mapped.decision_scope_progress().authority_digest(),
        source.decision_scope_progress().authority_digest()
    );
    assert_eq!(
        mapped.decision_scope_progress().population_identity(),
        source.decision_scope_progress().population_identity()
    );
    assert_eq!(
        mapped.decision_scope_progress().scope_identity(),
        source.decision_scope_progress().scope_identity()
    );
    assert_eq!(
        mapped.decision_scope_progress().state(),
        source.decision_scope_progress().state()
    );
    assert_ne!(
        source.decision_scope_progress().document_digest(),
        source.decision_scope_progress().authority_digest(),
        "the fixture must kill a swapped digest delegation"
    );
    assert_eq!(
        mapped.decision_scope_closure().scope_identity(),
        source.decision_scope_closure().scope_identity()
    );
    assert_eq!(
        mapped.surrounding_execution_progress().state(),
        ScopeState::Closed
    );
    assert_eq!(
        mapped.surrounding_execution_closure().state(),
        ScopeState::Closed
    );
    assert_eq!(
        mapped.completeness().document_identity(),
        source.completeness().document_identity()
    );
    assert_eq!(mapped.completeness().fact_identities().len(), 1);
    assert_eq!(mapped.settlement().basis, SettlementBasis::ClosedScope);
    assert_eq!(mapped.decision_support(), source.decision_support());
    assert!(mapped.correction().is_none());
}

#[test]
#[trace(
    "TC-006",
    "TC-015",
    "FR-006-AC-3",
    "FR-032-AC-3",
    "FR-032-AC-4",
    "FR-032-AC-7",
    "FR-042-AC-3"
)]
fn tc_015_four_axes_and_fr_295_completeness_are_independent() {
    let fixture = fixture();
    let progress = [
        &fixture.decision.progress_open,
        &fixture.decision.progress_closed,
    ];
    let closure = [
        &fixture.decision.closure_open,
        &fixture.decision.closure_closed,
    ];
    let surrounding_progress = [
        &fixture.surrounding.progress_open,
        &fixture.surrounding.progress_closed,
    ];
    let surrounding_closure = [
        &fixture.surrounding.closure_open,
        &fixture.surrounding.closure_closed,
    ];
    let completeness = [
        &fixture.decision.completeness_complete,
        &fixture.decision.completeness_incomplete,
        &fixture.decision.completeness_contradicted,
    ];
    let mut combinations = 0usize;
    for decision_progress in progress {
        for decision_closure in closure {
            for execution_progress in surrounding_progress {
                for execution_closure in surrounding_closure {
                    for complete in completeness {
                        let mut input = base_input(&fixture);
                        input.axes = AxisInputs {
                            decision_scope_progress: decision_progress,
                            decision_scope_closure: decision_closure,
                            surrounding_execution_progress: execution_progress,
                            surrounding_execution_closure: execution_closure,
                        };
                        input.settlement.supporting_progress_identities =
                            vec![decision_progress.identity().as_str().to_owned()];
                        input.completeness = complete;
                        input.execution = AssessmentExecution::Unsupported;
                        input.truth = Truth::Unavailable;
                        input.settlement.basis = SettlementBasis::Unavailable;
                        input.claim = Claim {
                            kind: ClaimKind::Monitorability,
                            strength: None,
                            admitted_fragment_identity: None,
                            backend_identity: None,
                        };
                        input.global_conformance = GlobalConformanceInput {
                            state: GlobalConformanceClosure::NotRequired,
                            required_premises: vec![],
                            premises: vec![],
                        };
                        validated(input);
                        combinations += 1;
                    }
                }
            }
        }
    }
    assert_eq!(combinations, 48);
}

#[test]
#[trace(
    "TC-006",
    "TC-015",
    "FR-006-AC-4",
    "FR-006-AC-5",
    "FR-006-AC-7",
    "FR-031-AC-5",
    "FR-033-AC-1",
    "FR-042-AC-2",
    "FR-042-AC-6"
)]
fn tc_015_strict_mutations_lineage_and_limits_fail_closed() {
    let fixture = fixture();
    let input = base_input(&fixture);
    let document = result::produce(input.clone(), Limits::owner_max()).expect("control result");
    let mut value: Value = serde_json::from_slice(document.bytes()).expect("result JSON");
    value["unexpected"] = Value::Bool(true);
    let unknown = quire_protocol::digest::jcs::canonicalize(&value).expect("canonical mutation");
    let error = result::read(
        unknown.as_bytes(),
        result::Expected {
            input: input.clone(),
            enclosing_digest: document.digest(),
        },
        Limits::owner_max(),
    )
    .expect_err("unknown field");
    assert_eq!(error.code(), ErrorCode::InvalidJson);
    assert_eq!(error.envelope().stage(), ProtocolStage::ResultRead);

    let mut missing_reproduction: Value =
        serde_json::from_slice(document.bytes()).expect("result JSON");
    missing_reproduction["provenance"]
        .as_object_mut()
        .expect("provenance object")
        .remove("reproduction");
    let missing_reproduction = quire_protocol::digest::jcs::canonicalize(&missing_reproduction)
        .expect("canonical missing-field mutation");
    assert_eq!(
        result::read(
            missing_reproduction.as_bytes(),
            result::Expected {
                input: input.clone(),
                enclosing_digest: document.digest(),
            },
            Limits::owner_max(),
        )
        .expect_err("missing reproduction provenance")
        .code(),
        ErrorCode::InvalidJson
    );

    let mut changed_reproduction: Value =
        serde_json::from_slice(document.bytes()).expect("result JSON");
    changed_reproduction["provenance"]["reproduction"]["toolchainIdentity"] =
        Value::String("00".into());
    let changed_reproduction = quire_protocol::digest::jcs::canonicalize(&changed_reproduction)
        .expect("canonical reproduction mutation");
    let changed_digest = quire_protocol::digest::jcs::ResultDigest::of_result(
        &serde_json::from_str(&changed_reproduction).expect("mutated result value"),
    )
    .expect("mutated enclosing digest")
    .text();
    assert_eq!(
        result::read(
            changed_reproduction.as_bytes(),
            result::Expected {
                input: input.clone(),
                enclosing_digest: &changed_digest,
            },
            Limits::owner_max(),
        )
        .expect_err("changed reproduction identity")
        .code(),
        ErrorCode::IdentityMismatch
    );

    let mut trailing = document.bytes().to_vec();
    trailing.push(b'\n');
    assert_eq!(
        result::read(
            &trailing,
            result::Expected {
                input: input.clone(),
                enclosing_digest: document.digest()
            },
            Limits::owner_max()
        )
        .expect_err("trailing data")
        .code(),
        ErrorCode::NonCanonical
    );
    assert_eq!(
        result::read(
            document.bytes(),
            result::Expected {
                input: input.clone(),
                enclosing_digest:
                    "sha256-jcs:0000000000000000000000000000000000000000000000000000000000000000"
            },
            Limits::owner_max()
        )
        .expect_err("digest mismatch")
        .code(),
        ErrorCode::DigestMismatch
    );

    let mut missing_global = input.clone();
    missing_global.global_conformance.premises.pop();
    assert_eq!(
        result::produce(missing_global, Limits::owner_max())
            .expect_err("missing premise")
            .code(),
        ErrorCode::GlobalPremiseMismatch
    );

    let mut mismatched_settlement_fact = input.clone();
    mismatched_settlement_fact
        .settlement
        .supporting_fact_identities[0] = "observation:other".into();
    assert_eq!(
        result::produce(mismatched_settlement_fact, Limits::owner_max())
            .expect_err("settlement fact outside exact decision support")
            .code(),
        ErrorCode::InvalidCombination
    );

    let mut mismatched_progress = input.clone();
    mismatched_progress
        .settlement
        .supporting_progress_identities[0] = "progress:other".into();
    assert_eq!(
        result::produce(mismatched_progress, Limits::owner_max())
            .expect_err("settlement progress outside selected owner assertions")
            .code(),
        ErrorCode::InvalidCombination
    );

    let mut open_closed_scope = input.clone();
    open_closed_scope.axes.decision_scope_progress = &fixture.decision.progress_open;
    open_closed_scope.settlement.supporting_progress_identities = vec![fixture
        .decision
        .progress_open
        .identity()
        .as_str()
        .to_owned()];
    assert_eq!(
        result::produce(open_closed_scope, Limits::owner_max())
            .expect_err("closed-scope settlement on open progress")
            .code(),
        ErrorCode::InvalidCombination
    );

    let mut closed_decisive_witness = input.clone();
    closed_decisive_witness.settlement.basis = SettlementBasis::DecisiveWitness;
    assert_eq!(
        result::produce(closed_decisive_witness, Limits::owner_max())
            .expect_err("decisive witness on a closed decision scope")
            .code(),
        ErrorCode::InvalidCombination
    );

    let mut incomplete_closed_scope = input.clone();
    incomplete_closed_scope.completeness = &fixture.decision.completeness_incomplete;
    assert_eq!(
        result::produce(incomplete_closed_scope, Limits::owner_max())
            .expect_err("closed-scope settlement over incomplete input")
            .code(),
        ErrorCode::InvalidCombination
    );

    let mut missing_predecessor = input.clone();
    missing_predecessor.decision_support.push(DecisionSupport {
        oracle_identity: "oracle:derived".into(),
        observation_identity: "observation:derived".into(),
        relation: DependenceRelation::Derived,
        predecessor: Some(SupportRef {
            oracle_identity: "oracle:missing".into(),
            observation_identity: "observation:missing".into(),
        }),
        dependence_group: None,
    });
    assert_eq!(
        result::produce(missing_predecessor, Limits::owner_max())
            .expect_err("missing predecessor")
            .code(),
        ErrorCode::LineageInvalid
    );

    let mut limited = Limits::owner_max();
    limited.max_support_records = 0;
    assert_eq!(
        result::produce(input, limited)
            .expect_err("support limit")
            .code(),
        ErrorCode::ResourceIncomplete
    );
}

#[test]
#[trace("TC-009", "TC-015", "NFR-002-AC-1", "FR-016-AC-2", "FR-016-AC-4")]
fn tc_009_result_output_budget_precedes_canonical_allocation() {
    let fixture = fixture();
    let input = base_input(&fixture);
    let control = result::produce(input.clone(), Limits::owner_max()).expect("control result");
    let required = control.bytes().len();
    let retained_string = control.usage().string_bytes;
    let total_output = required
        .checked_add(retained_string)
        .expect("bounded output and retained string");
    assert!(required > 1);
    let inventory = production_inventory().expect("production inventory");

    let short = Budget::nothing_granted()
        .grant(BudgetDimension::OutputByte, retained_string + required - 1)
        .expect("positive one-short grant");
    let mut short_resources = ResourceAccountant::new(short, &inventory);
    assert!(matches!(
        result::produce_bounded(
            input.clone(),
            Limits::owner_max(),
            &mut short_resources
        ),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::OutputByte
                && value.site().as_str() == "result/output/allocate"
                && value.consumed() == retained_string
                && value.requested() == required
    ));
    assert_eq!(
        short_resources
            .usage()
            .consumed(BudgetDimension::OutputByte),
        retained_string
    );

    let exact = Budget::nothing_granted()
        .grant(BudgetDimension::OutputByte, total_output)
        .expect("positive exact grant");
    let mut exact_resources = ResourceAccountant::new(exact, &inventory);
    let OperationOutcome::Completed(exact_document) =
        result::produce_bounded(input.clone(), Limits::owner_max(), &mut exact_resources)
    else {
        panic!("fresh exact output budget must complete");
    };
    assert_eq!(exact_document.bytes(), control.bytes());
    assert_eq!(
        exact_resources
            .usage()
            .consumed(BudgetDimension::OutputByte),
        total_output
    );

    let mut owner_clamped_input = input.clone();
    owner_clamped_input.provenance.source_identity =
        "x".repeat(Limits::owner_max().max_string_bytes + 1);
    let mut owner_clamped_resources =
        ResourceAccountant::new(Budget::nothing_granted(), &inventory);
    assert!(matches!(
        result::produce_bounded(
            owner_clamped_input,
            Limits {
                max_string_bytes: usize::MAX,
                ..Limits::owner_max()
            },
            &mut owner_clamped_resources,
        ),
        OperationOutcome::ResourceIncomplete(value)
            if value.site().as_str() == "result/output/string/retain"
                && value.granted() == Limits::owner_max().max_string_bytes
                && value.consumed().saturating_add(value.requested()) > value.granted()
    ));
    assert_eq!(
        owner_clamped_resources
            .usage()
            .consumed(BudgetDimension::OutputByte),
        control.result_id().len(),
        "only the fixed result-identity string is admitted before oversized caller text refuses"
    );

    let structural = control.usage();
    let operation_total = structural
        .depth
        .checked_add(structural.visited_fields)
        .expect("bounded structural work");
    let depth_short = Budget::nothing_granted()
        .grant(BudgetDimension::Operation, structural.depth - 1)
        .expect("positive one-short depth grant");
    let mut depth_short_resources = ResourceAccountant::new(depth_short, &inventory);
    let depth_outcome = result::produce_bounded(
        input.clone(),
        Limits::owner_max(),
        &mut depth_short_resources,
    );
    assert!(
        matches!(
            depth_outcome,
            OperationOutcome::ResourceIncomplete(value)
                if value.site().as_str() == "result/output/container/depth"
                    && value.consumed() == 0
                    && value.requested() == structural.depth
        ),
        "unexpected depth outcome: {depth_outcome:?}"
    );
    assert_eq!(
        depth_short_resources
            .usage()
            .consumed(BudgetDimension::Operation),
        0
    );

    let field_short = Budget::nothing_granted()
        .grant(BudgetDimension::Operation, operation_total - 1)
        .expect("positive one-short structural grant");
    let mut field_short_resources = ResourceAccountant::new(field_short, &inventory);
    assert!(matches!(
        result::produce_bounded(
            input.clone(),
            Limits::owner_max(),
            &mut field_short_resources,
        ),
        OperationOutcome::ResourceIncomplete(value)
            if value.site().as_str() == "result/output/field/visit"
                && value.consumed() == structural.depth
                && value.requested() == structural.visited_fields
    ));
    assert_eq!(
        field_short_resources
            .usage()
            .consumed(BudgetDimension::Operation),
        structural.depth,
        "returned consumed evidence must equal the accountant state retained after refusal"
    );

    let operation_exact = Budget::nothing_granted()
        .grant(BudgetDimension::Operation, operation_total)
        .expect("positive exact structural grant");
    let mut operation_exact_resources = ResourceAccountant::new(operation_exact, &inventory);
    assert!(matches!(
        result::produce_bounded(
            input.clone(),
            Limits::owner_max(),
            &mut operation_exact_resources,
        ),
        OperationOutcome::Completed(_)
    ));
    assert_eq!(
        operation_exact_resources
            .usage()
            .consumed(BudgetDimension::Operation),
        operation_total
    );

    let exact_per_document = Limits {
        max_input_bytes: Limits::owner_max().max_input_bytes,
        max_output_bytes: required,
        max_depth: structural.depth,
        max_string_bytes: structural.string_bytes,
        max_claims: structural.claims,
        max_premises: structural.premises,
        max_support_records: structural.support_records,
        max_limitations: structural.limitations,
        max_lineage_nodes: structural.lineage_nodes,
        max_visited_fields: structural.visited_fields,
    };
    let mut repeated_resources = ResourceAccountant::new(Budget::nothing_granted(), &inventory);
    for _ in 0..2 {
        assert!(matches!(
            result::produce_bounded(input.clone(), exact_per_document, &mut repeated_resources),
            OperationOutcome::Completed(_)
        ));
    }
    assert_eq!(
        repeated_resources
            .usage()
            .consumed(BudgetDimension::OutputByte),
        total_output * 2,
        "caller usage remains cumulative while each result gets fresh authored maxima"
    );
    assert_eq!(
        repeated_resources
            .usage()
            .consumed(BudgetDimension::Operation),
        operation_total * 2
    );

    let mut assumption_input = input.clone();
    assumption_input.assumptions.push(Assumption {
        identity: "x".repeat(128 * 1024),
        state: AssumptionState::Met,
    });
    let assumption_control = result::produce(assumption_input.clone(), Limits::owner_max())
        .expect("large admitted assumption has an exact structural usage");
    let assumption_operation = assumption_control
        .usage()
        .depth
        .checked_add(assumption_control.usage().visited_fields)
        .expect("bounded assumption structure");
    let assumption_short = Budget::nothing_granted()
        .grant(BudgetDimension::Operation, assumption_operation - 1)
        .expect("positive one-short assumption traversal grant");
    let mut assumption_resources = ResourceAccountant::new(assumption_short, &inventory);
    assert!(matches!(
        result::produce_bounded(
            assumption_input,
            Limits::owner_max(),
            &mut assumption_resources,
        ),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::Operation
                && value.site().as_str() == "result/output/field/visit"
    ));
    assert_eq!(
        assumption_resources
            .usage()
            .consumed(BudgetDimension::OutputByte),
        0,
        "an operation refusal precedes traversal of the large assumption string"
    );

    let one_allocation = Budget::nothing_granted()
        .grant(BudgetDimension::AccountedAllocation, 1)
        .expect("one logical result allocation");
    let mut allocation_resources = ResourceAccountant::new(one_allocation, &inventory);
    assert!(matches!(
        result::produce_bounded(
            input.clone(),
            Limits::owner_max(),
            &mut allocation_resources
        ),
        OperationOutcome::Completed(_)
    ));
    assert!(matches!(
        result::produce_bounded(
            input.clone(),
            Limits::owner_max(),
            &mut allocation_resources
        ),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::AccountedAllocation
                && value.consumed() == 1
                && value.requested() == 1
    ));

    let fresh_allocation = Budget::nothing_granted()
        .grant(BudgetDimension::AccountedAllocation, 1)
        .expect("fresh logical result allocation");
    let mut fresh_resources = ResourceAccountant::new(fresh_allocation, &inventory);
    assert!(matches!(
        result::produce_bounded(input.clone(), Limits::owner_max(), &mut fresh_resources),
        OperationOutcome::Completed(_)
    ));

    let usage = control.usage();
    let result_items = usage
        .claims
        .checked_add(usage.premises)
        .and_then(|value| value.checked_add(usage.support_records))
        .and_then(|value| value.checked_add(usage.limitations))
        .expect("bounded result items");
    let item_short = Budget::nothing_granted()
        .grant(BudgetDimension::ResultItem, result_items - 1)
        .expect("positive one-short result-item grant");
    let mut item_resources = ResourceAccountant::new(item_short, &inventory);
    assert!(matches!(
        result::produce_bounded(input.clone(), Limits::owner_max(), &mut item_resources),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::ResultItem
    ));
    assert_eq!(
        item_resources.usage().consumed(BudgetDimension::ResultItem),
        0
    );

    let item_exact = Budget::nothing_granted()
        .grant(BudgetDimension::ResultItem, result_items)
        .expect("positive exact result-item grant");
    let mut item_resources = ResourceAccountant::new(item_exact, &inventory);
    assert!(matches!(
        result::produce_bounded(input.clone(), Limits::owner_max(), &mut item_resources),
        OperationOutcome::Completed(_)
    ));
    assert_eq!(
        item_resources.usage().consumed(BudgetDimension::ResultItem),
        result_items
    );

    let one_lineage = Budget::nothing_granted()
        .grant(BudgetDimension::Lineage, usage.lineage_nodes)
        .expect("positive one-result lineage grant");
    let mut lineage_resources = ResourceAccountant::new(one_lineage, &inventory);
    assert!(matches!(
        result::produce_bounded(input.clone(), Limits::owner_max(), &mut lineage_resources),
        OperationOutcome::Completed(_)
    ));
    assert!(matches!(
        result::produce_bounded(
            input.clone(),
            Limits::owner_max(),
            &mut lineage_resources
        ),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::Lineage
                && value.consumed() == usage.lineage_nodes
                && value.requested() == usage.lineage_nodes
    ));

    let owner_support_short = Limits {
        max_support_records: usage.support_records - 1,
        ..Limits::owner_max()
    };
    let mut owner_resources = ResourceAccountant::new(Budget::nothing_granted(), &inventory);
    assert!(matches!(
        result::produce_bounded(input.clone(), owner_support_short, &mut owner_resources),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::ResultItem
                && value.granted() == usage.support_records - 1
                && value.requested() == usage.support_records
    ));

    let short = Budget::nothing_granted()
        .grant(BudgetDimension::InputByte, required - 1)
        .expect("positive one-short grant");
    let mut short_resources = ResourceAccountant::new(short, &inventory);
    assert!(matches!(
        result::read_bounded(
            control.bytes(),
            result::Expected {
                input: input.clone(),
                enclosing_digest: control.digest(),
            },
            Limits::owner_max(),
            &mut short_resources,
        ),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::InputByte
                && value.requested() == required
    ));
    assert_eq!(
        short_resources.usage().consumed(BudgetDimension::InputByte),
        0
    );

    let retained_string = control.usage().string_bytes;
    let total_input = required
        .checked_add(retained_string)
        .expect("bounded test input");
    let retained_short = Budget::nothing_granted()
        .grant(BudgetDimension::InputByte, total_input - 1)
        .expect("positive retained-string one-short grant");
    let mut retained_short_resources = ResourceAccountant::new(retained_short, &inventory);
    assert!(matches!(
        result::read_bounded(
            control.bytes(),
            result::Expected {
                input: input.clone(),
                enclosing_digest: control.digest(),
            },
            Limits::owner_max(),
            &mut retained_short_resources,
        ),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::InputByte
                && value.consumed() == required
                && value.requested() == retained_string
    ));

    let exact = Budget::nothing_granted()
        .grant(BudgetDimension::InputByte, total_input)
        .expect("positive exact grant");
    let mut exact_resources = ResourceAccountant::new(exact, &inventory);
    assert!(matches!(
        result::read_bounded(
            control.bytes(),
            result::Expected {
                input: input.clone(),
                enclosing_digest: control.digest(),
            },
            Limits::owner_max(),
            &mut exact_resources,
        ),
        OperationOutcome::Completed(_)
    ));

    let one_read_allocation = Budget::nothing_granted()
        .grant(BudgetDimension::AccountedAllocation, 1)
        .expect("one logical result-read allocation");
    let mut read_resources = ResourceAccountant::new(one_read_allocation, &inventory);
    assert!(matches!(
        result::read_bounded(
            control.bytes(),
            result::Expected {
                input: input.clone(),
                enclosing_digest: control.digest(),
            },
            Limits::owner_max(),
            &mut read_resources,
        ),
        OperationOutcome::Completed(_)
    ));
    assert!(matches!(
        result::read_bounded(
            control.bytes(),
            result::Expected {
                input: input.clone(),
                enclosing_digest: control.digest(),
            },
            Limits::owner_max(),
            &mut read_resources,
        ),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::AccountedAllocation
                && value.consumed() == 1
                && value.requested() == 1
    ));
    assert_eq!(
        exact_resources.usage().consumed(BudgetDimension::InputByte),
        total_input
    );

    let visited = control.usage().visited_fields;
    let depth = control.usage().depth;
    let operations = visited.checked_add(depth).expect("bounded test document");
    assert!(operations > 1);
    let short = Budget::nothing_granted()
        .grant(BudgetDimension::Operation, operations - 1)
        .expect("positive one-short grant");
    let mut short_resources = ResourceAccountant::new(short, &inventory);
    let short_outcome = result::read_bounded(
        control.bytes(),
        result::Expected {
            input: input.clone(),
            enclosing_digest: control.digest(),
        },
        Limits::owner_max(),
        &mut short_resources,
    );
    let consumed = short_resources.usage().consumed(BudgetDimension::Operation);
    assert!(
        matches!(
            short_outcome,
            OperationOutcome::ResourceIncomplete(value)
                if value.dimension() == BudgetDimension::Operation
                    && value.consumed() == depth
                    && value.requested() == visited
        ),
        "unexpected outcome; depth={depth}; visited={visited}; consumed={consumed}"
    );

    let exact = Budget::nothing_granted()
        .grant(BudgetDimension::Operation, operations)
        .expect("positive exact grant");
    let mut exact_resources = ResourceAccountant::new(exact, &inventory);
    assert!(matches!(
        result::read_bounded(
            control.bytes(),
            result::Expected {
                input: input.clone(),
                enclosing_digest: control.digest(),
            },
            Limits::owner_max(),
            &mut exact_resources,
        ),
        OperationOutcome::Completed(_)
    ));
    assert_eq!(
        exact_resources.usage().consumed(BudgetDimension::Operation),
        operations
    );

    let owner_input_short = Limits {
        max_input_bytes: required - 1,
        ..Limits::owner_max()
    };
    let mut owner_resources = ResourceAccountant::new(Budget::nothing_granted(), &inventory);
    assert!(matches!(
        result::read_bounded(
            control.bytes(),
            result::Expected {
                input: input.clone(),
                enclosing_digest: control.digest(),
            },
            owner_input_short,
            &mut owner_resources,
        ),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::InputByte
                && value.granted() == required - 1
                && value.requested() == required
    ));

    let owner_depth_short = Limits {
        max_depth: depth - 1,
        ..Limits::owner_max()
    };
    let mut owner_resources = ResourceAccountant::new(Budget::nothing_granted(), &inventory);
    assert!(matches!(
        result::read_bounded(
            control.bytes(),
            result::Expected {
                input: input.clone(),
                enclosing_digest: control.digest(),
            },
            owner_depth_short,
            &mut owner_resources,
        ),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::Operation
                && value.granted() == depth - 1
                && value.requested() == depth
    ));

    let owner_string_short = Limits {
        max_string_bytes: retained_string - 1,
        ..Limits::owner_max()
    };
    let mut owner_resources = ResourceAccountant::new(Budget::nothing_granted(), &inventory);
    assert!(matches!(
        result::read_bounded(
            control.bytes(),
            result::Expected {
                input: input.clone(),
                enclosing_digest: control.digest(),
            },
            owner_string_short,
            &mut owner_resources,
        ),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::InputByte
                && value.granted() == retained_string - 1
                && value.requested() == retained_string
    ));

    let owner_field_short = Limits {
        max_visited_fields: visited - 1,
        ..Limits::owner_max()
    };
    let mut owner_resources = ResourceAccountant::new(Budget::nothing_granted(), &inventory);
    assert!(matches!(
        result::read_bounded(
            control.bytes(),
            result::Expected {
                input: input.clone(),
                enclosing_digest: control.digest(),
            },
            owner_field_short,
            &mut owner_resources,
        ),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::Operation
                && value.granted() == visited - 1
                && value.consumed() == 0
                && value.requested() == visited
    ));

    let owner_short = Limits {
        max_output_bytes: required - 1,
        ..Limits::owner_max()
    };
    let mut owner_resources = ResourceAccountant::new(Budget::nothing_granted(), &inventory);
    assert!(matches!(
        result::produce_bounded(input, owner_short, &mut owner_resources),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::OutputByte
                && value.granted() == required - 1
                && value.requested() == required
    ));
}

#[test]
#[trace("TC-009", "TC-015", "NFR-002-AC-1", "FR-016-AC-2", "FR-042-AC-6")]
fn tc_009_contract_ir_mapping_charges_derived_artifacts_and_output() {
    let fixture = fixture();
    let source = validated(base_input(&fixture));
    let selection = contract_ir::MappingSelection::current();
    let control = contract_ir::map(&source, selection, Limits::owner_max()).expect("control map");
    let required = control.bytes().len();
    let inventory = production_inventory().expect("production inventory");

    let short = Budget::nothing_granted()
        .grant(BudgetDimension::OutputByte, required - 1)
        .expect("positive one-short output grant");
    let mut resources = ResourceAccountant::new(short, &inventory);
    assert!(matches!(
        contract_ir::map_bounded(&source, selection, Limits::owner_max(), &mut resources),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::OutputByte
                && value.requested() == required
    ));
    assert_eq!(resources.usage().consumed(BudgetDimension::OutputByte), 0);

    let exact = Budget::nothing_granted()
        .grant(BudgetDimension::OutputByte, required)
        .expect("positive exact output grant");
    let mut resources = ResourceAccountant::new(exact, &inventory);
    let OperationOutcome::Completed(mapped) =
        contract_ir::map_bounded(&source, selection, Limits::owner_max(), &mut resources)
    else {
        panic!("exact output budget must complete mapping");
    };
    assert_eq!(mapped.bytes(), control.bytes());

    let result_items = control.usage().claims + control.usage().support_records;
    let one_short_items = Budget::nothing_granted()
        .grant(BudgetDimension::ResultItem, result_items - 1)
        .expect("positive one-short mapping result-item grant");
    let mut item_resources = ResourceAccountant::new(one_short_items, &inventory);
    assert!(matches!(
        contract_ir::map_bounded(
            &source,
            selection,
            Limits::owner_max(),
            &mut item_resources,
        ),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::ResultItem
                && value.consumed() == 1
                && value.requested() == 1
    ));
    assert_eq!(
        item_resources.usage().consumed(BudgetDimension::ResultItem),
        0,
        "the mapping result-item preflight is transactional"
    );

    let exact_items = Budget::nothing_granted()
        .grant(BudgetDimension::ResultItem, result_items)
        .expect("exact mapping result-item grant");
    let mut item_resources = ResourceAccountant::new(exact_items, &inventory);
    assert!(matches!(
        contract_ir::map_bounded(&source, selection, Limits::owner_max(), &mut item_resources,),
        OperationOutcome::Completed(_)
    ));
    assert_eq!(
        item_resources.usage().consumed(BudgetDimension::ResultItem),
        result_items
    );

    let support_owner_short = Limits {
        max_support_records: 0,
        ..Limits::owner_max()
    };
    let mut item_resources = ResourceAccountant::new(Budget::nothing_granted(), &inventory);
    assert!(matches!(
        contract_ir::map_bounded(&source, selection, support_owner_short, &mut item_resources),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::ResultItem
                && value.granted() == 0
                && value.requested() == 1
    ));

    let one_artifact = Budget::nothing_granted()
        .grant(BudgetDimension::DerivedArtifact, 1)
        .expect("positive one-artifact grant");
    let mut resources = ResourceAccountant::new(one_artifact, &inventory);
    assert!(matches!(
        contract_ir::map_bounded(&source, selection, Limits::owner_max(), &mut resources),
        OperationOutcome::Completed(_)
    ));
    assert!(matches!(
        contract_ir::map_bounded(&source, selection, Limits::owner_max(), &mut resources),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::DerivedArtifact
                && value.consumed() == 1
                && value.requested() == 1
    ));

    let mut fresh_resources = ResourceAccountant::new(
        Budget::nothing_granted()
            .grant(BudgetDimension::DerivedArtifact, 1)
            .expect("positive fresh grant"),
        &inventory,
    );
    assert!(matches!(
        contract_ir::map_bounded(
            &source,
            selection,
            Limits::owner_max(),
            &mut fresh_resources,
        ),
        OperationOutcome::Completed(_)
    ));

    let one_map_allocation = Budget::nothing_granted()
        .grant(BudgetDimension::AccountedAllocation, 1)
        .expect("one logical mapping allocation");
    let mut allocation_resources = ResourceAccountant::new(one_map_allocation, &inventory);
    assert!(matches!(
        contract_ir::map_bounded(
            &source,
            selection,
            Limits::owner_max(),
            &mut allocation_resources,
        ),
        OperationOutcome::Completed(_)
    ));
    assert!(matches!(
        contract_ir::map_bounded(
            &source,
            selection,
            Limits::owner_max(),
            &mut allocation_resources,
        ),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::AccountedAllocation
                && value.consumed() == 1
                && value.requested() == 1
    ));

    let one_read_allocation = Budget::nothing_granted()
        .grant(BudgetDimension::AccountedAllocation, 1)
        .expect("one mapping-read allocation");
    let mut read_resources = ResourceAccountant::new(one_read_allocation, &inventory);
    assert!(matches!(
        contract_ir::read_bounded(
            control.bytes(),
            &source,
            selection,
            Limits::owner_max(),
            &mut read_resources,
        ),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::AccountedAllocation
                && value.consumed() == 1
                && value.requested() == 1
    ));

    let two_read_allocations = Budget::nothing_granted()
        .grant(BudgetDimension::AccountedAllocation, 2)
        .expect("decode plus rederivation allocations");
    let mut read_resources = ResourceAccountant::new(two_read_allocations, &inventory);
    assert!(matches!(
        contract_ir::read_bounded(
            control.bytes(),
            &source,
            selection,
            Limits::owner_max(),
            &mut read_resources,
        ),
        OperationOutcome::Completed(_)
    ));

    let owner_short = Limits {
        max_output_bytes: required - 1,
        ..Limits::owner_max()
    };
    let mut resources = ResourceAccountant::new(Budget::nothing_granted(), &inventory);
    assert!(matches!(
        contract_ir::map_bounded(&source, selection, owner_short, &mut resources),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::OutputByte
                && value.granted() == required - 1
                && value.requested() == required
    ));

    let retained_string = control.usage().string_bytes;
    let total_input = required
        .checked_add(retained_string)
        .expect("bounded mapping input");
    let operations = control
        .usage()
        .depth
        .checked_add(control.usage().visited_fields)
        .expect("bounded mapping traversal");
    let input_short = Budget::nothing_granted()
        .grant(BudgetDimension::InputByte, required - 1)
        .expect("positive one-short mapping input grant");
    let mut resources = ResourceAccountant::new(input_short, &inventory);
    assert!(matches!(
        contract_ir::read_bounded(
            control.bytes(),
            &source,
            selection,
            Limits::owner_max(),
            &mut resources,
        ),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::InputByte
                && value.consumed() == 0
                && value.requested() == required
    ));

    let retained_short = Budget::nothing_granted()
        .grant(BudgetDimension::InputByte, total_input - 1)
        .expect("positive one-short mapping retention grant");
    let mut resources = ResourceAccountant::new(retained_short, &inventory);
    assert!(matches!(
        contract_ir::read_bounded(
            control.bytes(),
            &source,
            selection,
            Limits::owner_max(),
            &mut resources,
        ),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::InputByte
                && value.consumed() == required
                && value.requested() == retained_string
    ));

    let exact_read = Budget::nothing_granted()
        .grant(BudgetDimension::InputByte, total_input)
        .expect("positive mapping input grant")
        .grant(BudgetDimension::Operation, operations)
        .expect("positive mapping operation grant")
        .grant(BudgetDimension::DerivedArtifact, 1)
        .expect("positive mapping artifact grant")
        .grant(BudgetDimension::OutputByte, required)
        .expect("positive mapping output grant");
    let mut resources = ResourceAccountant::new(exact_read, &inventory);
    assert!(matches!(
        contract_ir::read_bounded(
            control.bytes(),
            &source,
            selection,
            Limits::owner_max(),
            &mut resources,
        ),
        OperationOutcome::Completed(_)
    ));

    let operation_short = Budget::nothing_granted()
        .grant(BudgetDimension::Operation, operations - 1)
        .expect("positive one-short operation grant");
    let mut resources = ResourceAccountant::new(operation_short, &inventory);
    assert!(matches!(
        contract_ir::read_bounded(
            control.bytes(),
            &source,
            selection,
            Limits::owner_max(),
            &mut resources,
        ),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::Operation
                && value.consumed() == control.usage().depth
                && value.requested() == control.usage().visited_fields
    ));
}

#[test]
#[trace(
    "TC-006",
    "TC-015",
    "FR-006-AC-6",
    "FR-033-AC-2",
    "FR-033-AC-4",
    "FR-033-AC-5"
)]
fn tc_015_correction_is_direct_and_preserves_predecessor_bytes() {
    let fixture = fixture();
    let predecessor = validated(base_input(&fixture));
    let prior_bytes = predecessor.bytes().to_vec();
    let prior_id = predecessor.result_id().to_owned();
    let mut corrected = base_input(&fixture);
    corrected.execution = AssessmentExecution::Unsupported;
    corrected.truth = Truth::Unavailable;
    corrected.settlement.basis = SettlementBasis::Unavailable;
    corrected.provenance.input_identity = "input:corrected".into();
    corrected.correction = Some(CorrectionInput {
        relation: CorrectionRelation::Invalidates,
        predecessor: &predecessor,
        contradicted_premise_identity: "premise:branch",
        corrected_input_identity: "input:corrected",
    });
    let correction = validated(corrected);
    assert_ne!(correction.result_id(), prior_id);
    assert_eq!(predecessor.result_id(), prior_id);
    assert_eq!(predecessor.bytes(), prior_bytes);
    let edge = correction
        .correction()
        .expect("typed direct correction edge");
    assert_eq!(edge.relation(), CorrectionRelation::Invalidates);
    assert_eq!(edge.predecessor_result_id(), prior_id);
    assert_eq!(edge.predecessor_digest(), predecessor.digest());
    assert_eq!(edge.contradicted_premise_identity(), "premise:branch");
    assert_eq!(edge.corrected_input_identity(), "input:corrected");

    let mapped = contract_ir::map(
        &correction,
        contract_ir::MappingSelection::current(),
        Limits::owner_max(),
    )
    .expect("corrected mapping");
    let mapped_edge = mapped.correction().expect("mapped direct correction edge");
    assert_eq!(mapped_edge.predecessor_result_id(), prior_id);
    assert_eq!(mapped_edge.predecessor_digest(), predecessor.digest());
}

#[test]
#[trace("TC-015", "FR-042-AC-1", "FR-042-AC-5")]
fn tc_015_schema_bytes_and_digests_are_immutable_and_closed() {
    assert_eq!(
        format!("{:x}", ByteDigest::of(result::SCHEMA_BYTES)),
        result::SCHEMA_SHA256
    );
    assert_eq!(
        format!("{:x}", ByteDigest::of(contract_ir::SCHEMA_BYTES)),
        contract_ir::SCHEMA_SHA256
    );
    for schema in [result::SCHEMA_BYTES, contract_ir::SCHEMA_BYTES] {
        let value: Value = serde_json::from_slice(schema).expect("schema JSON");
        assert_eq!(value["additionalProperties"], Value::Bool(false));
    }
}

/// The string-only canonical-result carrier retains QObs runtime-reference
/// bytes exactly and rejects every non-reversible presentation.
#[test]
#[trace("TC-006", "FR-031-AC-7")]
fn tc_006_runtime_reference_carrier_is_reversible_and_strict() {
    let producer = producer();
    let subject = runtime_subject_bytes(
        &producer,
        vec![0xff, 0x00, 0x80],
        vec![0x00, 0xfe, 0x80, 0xff],
    );
    let encoded = runtime_reference::encode_subject(&subject).expect("encode runtime subject");
    runtime_reference::validate_subject(&encoded, &subject).expect("exact round trip");

    assert_eq!(
        runtime_reference::validate_subject("wrong:00", &subject),
        Err(runtime_reference::Error::Prefix)
    );
    assert_eq!(
        runtime_reference::validate_subject("quire-runtime-v1:0", &subject),
        Err(runtime_reference::Error::Hex)
    );
    assert_eq!(
        runtime_reference::validate_subject("quire-runtime-v1:GG", &subject),
        Err(runtime_reference::Error::Hex)
    );
    let uppercase = format!(
        "{}A{}",
        runtime_reference::PREFIX,
        &encoded[runtime_reference::PREFIX.len() + 1..]
    );
    assert_eq!(
        runtime_reference::validate_subject(&uppercase, &subject),
        Err(runtime_reference::Error::Hex)
    );

    let value: Value = serde_json::from_slice(&carrier_payload(&encoded)).expect("carrier JSON");
    let pretty = serde_json::to_vec_pretty(&value).expect("pretty JSON");
    assert_eq!(
        runtime_reference::validate_subject(&carrier(&pretty), &subject),
        Err(runtime_reference::Error::NonCanonical)
    );
    let mut missing = value;
    missing
        .as_object_mut()
        .expect("carrier object")
        .remove("authority");
    let missing = serde_json::to_vec(&missing).expect("missing-field JSON");
    assert_eq!(
        runtime_reference::validate_subject(&carrier(&missing), &subject),
        Err(runtime_reference::Error::Shape)
    );

    let mut malformed_kind =
        serde_json::from_slice::<Value>(&carrier_payload(&encoded)).expect("carrier JSON");
    malformed_kind
        .as_object_mut()
        .expect("carrier object")
        .insert("kindHex".into(), Value::String("ff0g80".into()));
    let malformed_kind = serde_json::to_vec(&malformed_kind).expect("malformed-field JSON");
    assert_eq!(
        runtime_reference::validate_subject(&carrier(&malformed_kind), &subject),
        Err(runtime_reference::Error::Hex)
    );

    let distinct = runtime_subject(&producer, "runtime:other");
    assert_eq!(
        runtime_reference::validate_subject(&encoded, &distinct),
        Err(runtime_reference::Error::Mismatch)
    );
}

fn e02_artifact(kind: &str, identity: &str) -> SharedArtifactEnvelope {
    serde_json::from_value(json!({
        "wireVersion": "ix.shared-reference/2-draft",
        "kind": "artifact",
        "value": {
            "refVersion": "ix.artifact-ref/3-draft",
            "kind": kind,
            "authority": "agent-ix/quire-protocol",
            "identity": identity,
            "revision": {"namespace": "quire-protocol", "value": "r1"},
            "digest": format!("sha256:{:064x}", ByteDigest::of(identity.as_bytes())),
            "wire": {"identity": "quire.protocol/e02-test-reference/1", "version": "1"}
        }
    }))
    .expect("valid qverify shared artifact reference")
}

fn e02_oracle(identity: &str) -> OracleReference {
    serde_json::from_value(
        serde_json::to_value(e02_artifact("oracle", identity)).expect("serialize oracle reference"),
    )
    .expect("valid qverify oracle reference")
}

fn e02_observation(identity: &str) -> ObservationReference {
    serde_json::from_value(
        serde_json::to_value(e02_artifact("observation", identity))
            .expect("serialize observation reference"),
    )
    .expect("valid qverify observation reference")
}

fn e02_resolution(request: &ProjectionRequestId) -> E02Resolution {
    let manifest = VersionLockManifest::new(vec![E02Contract::Draft2.manifest_entry()]);
    let budget = ResolutionBudget::new(1, 1, Duration::from_secs(5))
        .expect("positive exact resolution budget");
    projection_result(resolve_e02_contract(
        request,
        &manifest,
        budget,
        SystemMonotonicClock,
    ))
    .expect("exact qverify E02 interface resolves")
}

fn projection_result<T>(
    outcome: quire_protocol::OperationOutcome<T>,
) -> Result<T, quire_protocol::refusal::ProtocolRefusal> {
    match outcome {
        quire_protocol::OperationOutcome::Completed(value) => Ok(value),
        quire_protocol::OperationOutcome::Refused(refusal) => Err(refusal),
        quire_protocol::OperationOutcome::ResourceIncomplete(incomplete) => {
            panic!("unexpected projection resource stop: {incomplete:?}")
        }
    }
}

fn project_e02_without_budget(
    source_bytes: &[u8],
    source_expected: result::Expected<'_>,
    source_limits: Limits,
    request: &ProjectionRequestId,
    resolution: &E02Resolution,
    context: &ProjectionContext,
) -> quire_protocol::OperationOutcome<ProjectedTechniqueResult> {
    let inventory = production_inventory().expect("production inventory");
    let mut resources = ResourceAccountant::new(Budget::nothing_granted(), &inventory);
    match quire_protocol::projection::project_e02(
        source_bytes,
        source_expected,
        source_limits,
        request,
        resolution,
        context,
        &mut resources,
    ) {
        OperationOutcome::Completed(projected) => {
            quire_protocol::OperationOutcome::Completed(projected)
        }
        OperationOutcome::DomainError(refusal) => {
            quire_protocol::OperationOutcome::Refused(refusal)
        }
        OperationOutcome::ResourceIncomplete(incomplete) => {
            quire_protocol::OperationOutcome::ResourceIncomplete(incomplete)
        }
        OperationOutcome::AccountingFault(fault) => {
            panic!("projection accounting fault in production inventory: {fault:?}")
        }
    }
}

fn e02_context(document: &result::Document, input: &Input<'_>) -> ProjectionContext {
    ProjectionContext {
        source_contract_version: result::CONTRACT.into(),
        target_contract_version: contracts::contract_version().into(),
        required_features: Vec::new(),
        source_uri: format!(
            "ix://agent-ix/quire-protocol/result/{}",
            document.result_id()
        ),
        source_digest: document.digest().into(),
        expected_result_id: None,
        plan_id: "Plan-001-finite-protocol-rust-slice".into(),
        task_id: "Task-007-project-e02".into(),
        technique_id: "finite-protocol-conformance".into(),
        run_ref: e02_artifact("run-artifact", "run:tc-007"),
        property_ref: e02_artifact("property", "property:protocol-conformance"),
        subject_refs: vec![e02_artifact("source", "subject:protocol")],
        profile: "quire.protocol/e02-projection/1".into(),
        binding_refs: vec![
            e02_artifact("binding", "binding:z-semantic-first"),
            e02_artifact("binding", "binding:a-semantic-second"),
        ],
        dependency_closure: "sha256:protocol-dependency-closure-r1".into(),
        execution_scope: json!({
            "repository": "agent-ix/quire-protocol",
            "revision": "tc-007",
            "paths": ["src/projection.rs"]
        })
        .as_object()
        .expect("execution scope object")
        .clone(),
        produced_at: "2026-09-15T12:00:00Z".into(),
        producer_version: result::CONTRACT.into(),
        adapter_identity: "agent-ix/quire-protocol/e02-projection".into(),
        adapter_version: "1-draft.1".into(),
        decision_support_bindings: input
            .decision_support
            .iter()
            .map(|support| DecisionSupportBinding {
                source_oracle_identity: support.oracle_identity.clone(),
                source_observation_identity: support.observation_identity.clone(),
                oracle_ref: e02_oracle(&support.oracle_identity),
                observation_ref: e02_observation(&support.observation_identity),
            })
            .collect(),
    }
}

fn project_input<'a>(
    input: Input<'a>,
    request: &ProjectionRequestId,
    resolution: &E02Resolution,
    mutate: impl FnOnce(&mut ProjectionContext),
) -> quire_protocol::OperationOutcome<ProjectedTechniqueResult> {
    let document = result::produce(input.clone(), Limits::owner_max()).expect("canonical source");
    let mut context = e02_context(&document, &input);
    mutate(&mut context);
    project_e02_without_budget(
        document.bytes(),
        result::Expected {
            input,
            enclosing_digest: document.digest(),
        },
        Limits::owner_max(),
        request,
        resolution,
        &context,
    )
}

fn project_input_bounded<'a>(
    input: Input<'a>,
    request: &ProjectionRequestId,
    resolution: &E02Resolution,
    budget: Budget,
    mutate: impl FnOnce(&mut ProjectionContext),
) -> (
    OperationOutcome<ProjectedTechniqueResult, quire_protocol::refusal::ProtocolRefusal>,
    quire_protocol::bounds::inventory::Usage,
) {
    let document = result::produce(input.clone(), Limits::owner_max()).expect("canonical source");
    let mut context = e02_context(&document, &input);
    mutate(&mut context);
    let inventory = production_inventory().expect("production inventory");
    let mut resources = ResourceAccountant::new(budget, &inventory);
    let outcome = quire_protocol::projection::project_e02(
        document.bytes(),
        result::Expected {
            input,
            enclosing_digest: document.digest(),
        },
        Limits::owner_max(),
        request,
        resolution,
        &context,
        &mut resources,
    );
    (outcome, resources.usage())
}

fn projected_value(
    input: Input<'_>,
    request: &ProjectionRequestId,
    resolution: &E02Resolution,
) -> Value {
    let projected = projection_result(project_input(input, request, resolution, |_| {}))
        .expect("source row projects");
    serde_json::from_slice(projected.bytes()).expect("projected JSON")
}

fn unavailable_input(fixture: &Fixture, activation: Activation) -> Input<'_> {
    let mut input = base_input(fixture);
    input.truth = Truth::Unavailable;
    input.settlement.basis = SettlementBasis::Unavailable;
    input.settlement.supporting_fact_identities.clear();
    input.activation = activation;
    input.decision_support.clear();
    input
}

fn projected_id_after_context_mutation(
    fixture: &Fixture,
    request: &ProjectionRequestId,
    resolution: &E02Resolution,
    mutate: fn(&mut ProjectionContext),
) -> String {
    projection_result(project_input(
        base_input(fixture),
        request,
        resolution,
        mutate,
    ))
    .expect("identity-bearing target mutation projects")
    .result_id()
    .to_owned()
}

#[test]
#[trace("TC-007", "FR-007-AC-2", "FR-007-AC-5")]
#[trace("FR-034-AC-2", "FR-034-AC-3", "FR-034-AC-4")]
#[trace("FR-034-AC-5", "FR-034-AC-6")]
fn tc_007_projection_has_exact_identity_binding_and_retention() {
    let owner = fixture();
    let input = base_input(&owner);
    let document = result::produce(input.clone(), Limits::owner_max()).expect("canonical source");
    let context = e02_context(&document, &input);
    let request = ProjectionRequestId::new(b"tc-007:exact-projection").expect("request id");
    let resolution = e02_resolution(&request);
    let projected = projection_result(project_e02_without_budget(
        document.bytes(),
        result::Expected {
            input,
            enclosing_digest: document.digest(),
        },
        Limits::owner_max(),
        &request,
        &resolution,
        &context,
    ))
    .expect("projection completes");

    assert_eq!(projected.source_bytes(), document.bytes());
    assert_ne!(projected.result_id(), document.result_id());
    assert!(projected.result_id().starts_with("sha256-jcs:"));
    assert!(projected.result_id()["sha256-jcs:".len()..]
        .bytes()
        .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f')));
    let value: Value = serde_json::from_slice(projected.bytes()).expect("projected JSON");
    let identity_preimage = json!({
        "identityVersion": quire_protocol::projection::IDENTITY_VERSION,
        "sourceResultRef": {
            "identity": document.result_id(),
            "digest": document.digest(),
        },
        "targetContractVersion": context.target_contract_version,
        "targetBinding": {
            "propertyRef": context.property_ref,
            "planId": context.plan_id,
            "taskId": context.task_id,
            "runRef": context.run_ref,
            "profile": context.profile,
            "bindingRefs": context.binding_refs,
            "dependencyClosure": context.dependency_closure,
            "executionScope": context.execution_scope,
        },
        "adapter": {
            "identity": context.adapter_identity,
            "version": context.adapter_version,
        },
    });
    assert_eq!(
        projected.result_id(),
        jcs_sha256(&identity_preimage).expect("qverify-owned RFC 8785 identity")
    );
    assert_eq!(value["contractVersion"], contracts::contract_version());
    assert_eq!(value["planId"], context.plan_id);
    assert_eq!(value["taskId"], context.task_id);
    assert_eq!(
        value["runRef"],
        serde_json::to_value(&context.run_ref).unwrap()
    );
    assert_eq!(
        value["propertyRef"],
        serde_json::to_value(&context.property_ref).unwrap()
    );
    assert_eq!(
        value["subjectRefs"],
        serde_json::to_value(&context.subject_refs).unwrap()
    );
    assert_eq!(
        value["bindingRefs"],
        serde_json::to_value(&context.binding_refs).unwrap()
    );
    assert_eq!(value["profile"], context.profile);
    assert_eq!(value["dependencyClosure"], context.dependency_closure);
    assert_eq!(
        value["executionScope"],
        Value::Object(context.execution_scope.clone())
    );
    assert_eq!(
        value["bindingRefs"][0]["value"]["identity"],
        "binding:z-semantic-first"
    );
    assert_eq!(
        value["bindingRefs"][1]["value"]["identity"],
        "binding:a-semantic-second"
    );
    assert_eq!(value["nativeArtifacts"].as_array().unwrap().len(), 1);
    assert_eq!(value["nativeArtifacts"][0]["uri"], context.source_uri);
    assert_eq!(value["nativeArtifacts"][0]["digest"], document.digest());
    assert_eq!(
        value["nativeArtifacts"][0]["mediaType"],
        quire_protocol::projection::SOURCE_MEDIA_TYPE
    );
    assert_eq!(value["sourceLimitations"][0]["code"], "bound:finite");
    assert_eq!(
        value["provenance"]["producerVersion"],
        context.producer_version
    );
    assert_eq!(value["provenance"]["adapter"], context.adapter_identity);
    assert_eq!(
        value["provenance"]["adapterVersion"],
        context.adapter_version
    );
    assert!(value.get("sufficiencyDecision").is_none());
    assert!(matches!(
        contracts::validate_contract(value).expect("qverify strict reread"),
        contracts::PublicContract::TechniqueResult(_)
    ));
}

#[test]
#[trace("TC-007", "FR-007-AC-1", "FR-007-AC-3")]
#[trace("FR-035-AC-1", "FR-035-AC-2", "FR-035-AC-3")]
#[trace("FR-035-AC-4")]
fn tc_007_every_source_state_row_and_simultaneous_finding_maps() {
    let owner = fixture();
    let request = ProjectionRequestId::new(b"tc-007:mapping-table").expect("request id");
    let resolution = e02_resolution(&request);
    let cases = [
        (AssessmentExecution::Completed, "completed"),
        (AssessmentExecution::Refused, "invalid-input"),
        (AssessmentExecution::Unsupported, "unsupported"),
        (AssessmentExecution::ResourceIncomplete, "exhausted"),
        (AssessmentExecution::Failed, "failed"),
    ];
    for (execution, expected) in cases {
        let mut input = base_input(&owner);
        if execution != AssessmentExecution::Completed {
            input.execution = execution;
            input.truth = Truth::Unavailable;
            input.settlement.basis = SettlementBasis::Unavailable;
        }
        let projected = projection_result(project_input(input, &request, &resolution, |_| {}))
            .expect("execution row maps");
        let value: Value = serde_json::from_slice(projected.bytes()).expect("projected JSON");
        assert_eq!(value["execution"]["status"], expected);
        assert_eq!(
            value["logical"]["outcome"],
            if execution == AssessmentExecution::Completed {
                "satisfied"
            } else {
                "not-evaluated"
            }
        );
    }

    let mut input = base_input(&owner);
    input.truth = Truth::Violated;
    assert_eq!(
        projected_value(input, &request, &resolution)["logical"]["outcome"],
        "violated"
    );

    for (activation, expected_activation, expected_logical) in [
        (Activation::Inactive, "not-activated", "not-evaluated"),
        (Activation::ActivationUnknown, "unknown", "unknown"),
        (Activation::Triggered, "activated", "unknown"),
    ] {
        let value = projected_value(unavailable_input(&owner, activation), &request, &resolution);
        assert_eq!(value["activation"]["status"], expected_activation);
        assert_eq!(value["logical"]["outcome"], expected_logical);
    }

    for (participation, expected) in [
        (
            Participation::Known {
                required_role_instance_refs: vec!["role:a".into()],
                participating_role_instance_refs: vec!["role:a".into()],
            },
            "complete",
        ),
        (
            Participation::Known {
                required_role_instance_refs: vec!["role:a".into()],
                participating_role_instance_refs: Vec::new(),
            },
            "not-started",
        ),
        (
            Participation::Known {
                required_role_instance_refs: Vec::new(),
                participating_role_instance_refs: Vec::new(),
            },
            "not-required",
        ),
        (
            Participation::Unknown {
                participating_role_instance_refs: vec!["role:a".into()],
                missing_dependency_refs: vec!["dependency:one".into()],
                reasons: vec!["reason:unavailable".into()],
            },
            "unknown",
        ),
    ] {
        let mut input = base_input(&owner);
        input.participation = participation;
        assert_eq!(
            projected_value(input, &request, &resolution)["participation"]["status"],
            expected
        );
    }

    for (adequacy, expected) in [
        (Adequacy::Adequate, "adequate"),
        (Adequacy::Inadequate, "inadequate"),
        (Adequacy::Unknown, "unknown"),
        (Adequacy::NotAssessed, "not-assessed"),
    ] {
        let mut input = base_input(&owner);
        input.observation_adequacy = adequacy;
        assert_eq!(
            projected_value(input, &request, &resolution)["adequacy"]["status"],
            expected
        );
    }

    let mut input = base_input(&owner);
    input.truth = Truth::Pending;
    input.settlement.basis = SettlementBasis::Unsettled;
    input.axes.decision_scope_progress = &owner.decision.progress_open;
    input.settlement.supporting_progress_identities =
        vec![owner.decision.progress_open.identity().as_str().to_owned()];
    input.participation = Participation::Known {
        required_role_instance_refs: vec!["role:a".into(), "role:b".into()],
        participating_role_instance_refs: vec!["role:b".into()],
    };
    input.assumptions = vec![
        Assumption {
            identity: "assumption:met".into(),
            state: AssumptionState::Met,
        },
        Assumption {
            identity: "assumption:not-met".into(),
            state: AssumptionState::NotMet,
        },
        Assumption {
            identity: "assumption:unknown".into(),
            state: AssumptionState::Unknown,
        },
        Assumption {
            identity: "assumption:not-assessed".into(),
            state: AssumptionState::NotAssessed,
        },
    ];
    input.observation_adequacy = Adequacy::Inadequate;
    input.protocol_adequacy = ProtocolAdequacy::Demonstrated;
    let projected = projection_result(project_input(input, &request, &resolution, |_| {}))
        .expect("simultaneous findings map");
    let value: Value = serde_json::from_slice(projected.bytes()).expect("projected JSON");
    assert_eq!(value["logical"]["outcome"], "inconclusive");
    assert_eq!(value["participation"]["status"], "partial");
    assert_eq!(value["participation"]["participants"], json!(["role:b"]));
    assert_eq!(
        value["participation"]["missingParticipants"],
        json!(["role:a"])
    );
    assert_eq!(value["assumptions"].as_array().unwrap().len(), 4);
    assert_eq!(value["assumptions"][0]["status"], "met");
    assert_eq!(value["assumptions"][1]["status"], "not-met");
    assert_eq!(value["assumptions"][2]["status"], "unknown");
    assert_eq!(value["assumptions"][3]["status"], "not-assessed");
    assert_eq!(value["adequacy"]["status"], "inadequate");
    assert!(value["adequacy"].get("protocolAdequacy").is_none());
    assert!(value.get("protocolAdequacy").is_none());
    assert!(value.get("sufficiencyDecision").is_none());
}

#[test]
#[trace("TC-007", "FR-007-AC-4", "FR-007-AC-6")]
#[trace("FR-034-AC-1", "FR-034-AC-7")]
#[trace("FR-035-AC-5", "FR-035-AC-6", "FR-035-AC-7")]
fn tc_007_identity_mutations_lineage_and_refusals_are_exact() {
    let owner = fixture();
    let request = ProjectionRequestId::new(b"tc-007:refusals").expect("request id");
    let resolution = e02_resolution(&request);
    let baseline = projection_result(project_input(
        base_input(&owner),
        &request,
        &resolution,
        |_| {},
    ))
    .expect("baseline projection");
    let baseline_id = baseline.result_id().to_owned();

    let mut source_mutation = base_input(&owner);
    source_mutation.provenance.input_identity = "input:mutated".into();
    let source_changed = projection_result(project_input(
        source_mutation,
        &request,
        &resolution,
        |_| {},
    ))
    .expect("source identity mutation projects");
    assert_ne!(baseline_id, source_changed.result_id());

    let target_mutations: [fn(&mut ProjectionContext); 10] = [
        |context| context.property_ref = e02_artifact("property", "property:mutated"),
        |context| context.plan_id = "plan:mutated".into(),
        |context| context.task_id = "task:mutated".into(),
        |context| context.run_ref = e02_artifact("run-artifact", "run:mutated"),
        |context| context.profile = "profile:mutated".into(),
        |context| context.binding_refs[0] = e02_artifact("binding", "binding:mutated"),
        |context| context.dependency_closure = "sha256:mutated-closure".into(),
        |context| {
            context
                .execution_scope
                .insert("revision".into(), Value::String("mutated".into()));
        },
        |context| context.adapter_identity = "adapter:mutated".into(),
        |context| context.adapter_version = "adapter-version:mutated".into(),
    ];
    for mutate in target_mutations {
        assert_ne!(
            baseline_id,
            projected_id_after_context_mutation(&owner, &request, &resolution, mutate)
        );
    }

    let changed = projection_result(project_input(
        base_input(&owner),
        &request,
        &resolution,
        |context| {
            context.binding_refs.swap(0, 1);
        },
    ))
    .expect("semantic-order mutation remains a valid distinct projection");
    assert_ne!(baseline.result_id(), changed.result_id());

    let refusal = projection_result(project_input(
        base_input(&owner),
        &request,
        &resolution,
        |context| {
            context.source_digest = "sha256-jcs:deadbeef".into();
        },
    ))
    .expect_err("wrong source digest refuses");
    assert_eq!(refusal.code().as_str(), "invalid-source-result");
    let refusal = projection_result(project_input(
        base_input(&owner),
        &request,
        &resolution,
        |context| {
            context.source_contract_version = "quire.protocol/result/future".into();
        },
    ))
    .expect_err("unknown source version refuses");
    assert_eq!(refusal.code().as_str(), "unsupported-source-version");
    let refusal = projection_result(project_input(
        base_input(&owner),
        &request,
        &resolution,
        |context| {
            context.target_contract_version = "quire-verification/e02-future".into();
        },
    ))
    .expect_err("unknown target version refuses");
    assert_eq!(refusal.code().as_str(), "unsupported-target-version");
    let refusal = projection_result(project_input(
        base_input(&owner),
        &request,
        &resolution,
        |context| {
            context.required_features.push("future-finding-axis".into());
        },
    ))
    .expect_err("unknown target feature refuses");
    assert_eq!(refusal.code().as_str(), "unsupported-target-version");
    let refusal = projection_result(project_input(
        base_input(&owner),
        &request,
        &resolution,
        |context| {
            context.plan_id.clear();
        },
    ))
    .expect_err("missing target context refuses");
    assert_eq!(refusal.code().as_str(), "missing-context");
    let refusal = projection_result(project_input(
        base_input(&owner),
        &request,
        &resolution,
        |context| {
            context.expected_result_id = Some("sha256-jcs:wrong".into());
        },
    ))
    .expect_err("wrong asserted projection identity refuses");
    assert_eq!(refusal.code().as_str(), "identity-mismatch");

    let refusal = projection_result(project_input(
        base_input(&owner),
        &request,
        &resolution,
        |context| {
            context.decision_support_bindings.clear();
        },
    ))
    .expect_err("missing support binding refuses");
    assert_eq!(refusal.code().as_str(), "missing-context");
    let refusal = projection_result(project_input(
        base_input(&owner),
        &request,
        &resolution,
        |context| {
            let binding = &mut context.decision_support_bindings[0];
            binding.source_oracle_identity = "oracle:wrong".into();
            binding.oracle_ref = e02_oracle("oracle:wrong");
        },
    ))
    .expect_err("foreign source support identity refuses");
    assert_eq!(refusal.code().as_str(), "identity-mismatch");
    let refusal = projection_result(project_input(
        base_input(&owner),
        &request,
        &resolution,
        |context| {
            context.decision_support_bindings[0].oracle_ref = e02_oracle("oracle:wrong");
        },
    ))
    .expect_err("mismatched target oracle identity refuses");
    assert_eq!(refusal.code().as_str(), "identity-mismatch");
    let refusal = projection_result(project_input(
        base_input(&owner),
        &request,
        &resolution,
        |context| {
            context.decision_support_bindings[0].observation_ref =
                e02_observation("observation:wrong");
        },
    ))
    .expect_err("mismatched target observation identity refuses");
    assert_eq!(refusal.code().as_str(), "identity-mismatch");

    let input = base_input(&owner);
    let document = result::produce(input.clone(), Limits::owner_max()).expect("canonical source");
    let context = e02_context(&document, &input);
    let refusal = projection_result(project_e02_without_budget(
        b"not-json",
        result::Expected {
            input,
            enclosing_digest: document.digest(),
        },
        Limits::owner_max(),
        &request,
        &resolution,
        &context,
    ))
    .expect_err("malformed source bytes refuse");
    assert_eq!(refusal.code().as_str(), "invalid-source-result");

    let mut input = base_input(&owner);
    let observation = input.decision_support[0].observation_identity.clone();
    input.decision_support = vec![
        DecisionSupport {
            oracle_identity: "oracle:root".into(),
            observation_identity: observation.clone(),
            relation: DependenceRelation::Independent,
            predecessor: None,
            dependence_group: None,
        },
        DecisionSupport {
            oracle_identity: "oracle:shared".into(),
            observation_identity: observation.clone(),
            relation: DependenceRelation::Shared,
            predecessor: None,
            dependence_group: Some("dependence:one".into()),
        },
        DecisionSupport {
            oracle_identity: "oracle:derived".into(),
            observation_identity: observation.clone(),
            relation: DependenceRelation::Derived,
            predecessor: Some(SupportRef {
                oracle_identity: "oracle:root".into(),
                observation_identity: observation.clone(),
            }),
            dependence_group: None,
        },
        DecisionSupport {
            oracle_identity: "oracle:unknown".into(),
            observation_identity: observation,
            relation: DependenceRelation::Unknown,
            predecessor: None,
            dependence_group: None,
        },
    ];
    let projected = projection_result(project_input(input, &request, &resolution, |_| {}))
        .expect("all dependence relations project");
    let value: Value = serde_json::from_slice(projected.bytes()).expect("projected JSON");
    assert_eq!(value["oracleContributions"][0]["relation"], "independent");
    assert_eq!(value["oracleContributions"][1]["relation"], "shared");
    assert_eq!(
        value["oracleContributions"][1]["dependenceGroupId"],
        "dependence:one"
    );
    assert_eq!(value["oracleContributions"][2]["relation"], "derived");
    assert_eq!(
        value["oracleContributions"][2]["derivedFrom"]["contributionId"],
        "protocol-support-0"
    );
    assert_eq!(
        value["oracleContributions"][2]["derivedFrom"]["resultId"],
        projected.result_id()
    );
    assert_eq!(value["oracleContributions"][3]["relation"], "unknown");
}

#[test]
#[trace("TC-008", "NFR-001-AC-1", "NFR-001-AC-2", "NFR-001-AC-3")]
fn tc_008_e02_projection_reproduces_equivalent_maps_and_discriminates_identity() {
    let owner = fixture();
    let request = ProjectionRequestId::new(b"tc-008:e02-reproduction").expect("request id");
    let resolution = e02_resolution(&request);

    let first = projection_result(project_input(
        base_input(&owner),
        &request,
        &resolution,
        |context| {
            context.execution_scope = [
                ("repository".into(), json!("agent-ix/quire-protocol")),
                ("revision".into(), json!("tc-008")),
                ("paths".into(), json!(["src/projection.rs"])),
            ]
            .into_iter()
            .collect();
        },
    ))
    .expect("first map presentation projects");
    let second = projection_result(project_input(
        base_input(&owner),
        &request,
        &resolution,
        |context| {
            context.execution_scope = [
                ("paths".into(), json!(["src/projection.rs"])),
                ("revision".into(), json!("tc-008")),
                ("repository".into(), json!("agent-ix/quire-protocol")),
            ]
            .into_iter()
            .collect();
        },
    ))
    .expect("reverse insertion presentation projects");

    assert_eq!(first.bytes(), second.bytes());
    assert_eq!(
        serde_json::to_value(first.result()).expect("first typed result serializes"),
        serde_json::to_value(second.result()).expect("second typed result serializes")
    );
    assert_eq!(first.source_bytes(), second.source_bytes());

    let mutated = projection_result(project_input(
        base_input(&owner),
        &request,
        &resolution,
        |context| context.adapter_identity.push_str("/mutated"),
    ))
    .expect("one semantic identity mutation projects distinctly");
    assert_ne!(first.result_id(), mutated.result_id());
    assert_ne!(first.bytes(), mutated.bytes());
}

#[test]
#[trace("TC-009", "NFR-002-AC-1", "NFR-002-AC-3")]
fn tc_009_e02_projection_enforces_exact_one_short_and_fresh_exact_budgets() {
    let owner = fixture();
    let request = ProjectionRequestId::new(b"tc-009:e02-resources").expect("request id");
    let resolution = e02_resolution(&request);
    let (baseline, usage) = project_input_bounded(
        base_input(&owner),
        &request,
        &resolution,
        Budget::nothing_granted(),
        |_| {},
    );
    assert!(matches!(baseline, OperationOutcome::Completed(_)));

    for dimension in [
        BudgetDimension::ResultItem,
        BudgetDimension::OutputByte,
        BudgetDimension::Operation,
        BudgetDimension::AccountedAllocation,
    ] {
        let exact = usage.consumed(dimension);
        assert!(
            exact > 1,
            "{dimension:?} fixture needs a positive one-short grant"
        );

        let short = Budget::nothing_granted()
            .grant(dimension, exact - 1)
            .expect("positive one-short grant");
        let (short_outcome, _) =
            project_input_bounded(base_input(&owner), &request, &resolution, short, |_| {});
        assert!(matches!(
            short_outcome,
            OperationOutcome::ResourceIncomplete(incomplete)
                if incomplete.dimension() == dimension
        ));

        for _ in 0..2 {
            let exact_budget = Budget::nothing_granted()
                .grant(dimension, exact)
                .expect("positive exact grant");
            let (exact_outcome, exact_usage) = project_input_bounded(
                base_input(&owner),
                &request,
                &resolution,
                exact_budget,
                |_| {},
            );
            assert!(matches!(exact_outcome, OperationOutcome::Completed(_)));
            assert_eq!(exact_usage.consumed(dimension), exact);
        }
    }

    let exact_derived = Budget::nothing_granted()
        .grant(BudgetDimension::DerivedArtifact, 1)
        .expect("one derived artifact");
    for _ in 0..2 {
        let (outcome, exact_usage) = project_input_bounded(
            base_input(&owner),
            &request,
            &resolution,
            exact_derived.clone(),
            |_| {},
        );
        assert!(matches!(outcome, OperationOutcome::Completed(_)));
        assert_eq!(exact_usage.consumed(BudgetDimension::DerivedArtifact), 1);
    }
}
