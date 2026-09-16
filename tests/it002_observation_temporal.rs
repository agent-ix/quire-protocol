// SPDX-License-Identifier: AGPL-3.0-or-later
//! IT-002: accepted QObs and QSL temporal facts through Protocol assessment.

use std::collections::BTreeMap;
use std::time::Duration;

use ix_trace_rs::trace;
use quire_observation::{QualifiedSubject, SubjectIdentity, SubjectKind};
use quire_protocol::assessment::{
    with_activated_after_resolving, with_result_assessment_after_resolving,
    ActivatedAssessmentInputs, AxisInputs, CanonicalRowControls, GlobalConformanceInput,
    GlobalPremise, PremiseKind, ResultAssessmentInputs,
};
use quire_protocol::bounds::inventory::{Budget, BudgetDimension};
use quire_protocol::bounds::SystemMonotonicClock;
use quire_protocol::claims::dispatch::{ClaimRequest, Limits as DispatchLimits};
use quire_protocol::claims::premises::{BackendSelection, Component, PremiseSet};
use quire_protocol::claims::Claim;
use quire_protocol::closure::{AssessmentExecution, GlobalConformanceClosure};
use quire_protocol::conformance::{
    ActivatedIncremental, GlobalDisposition, Limits as ConformanceLimits, ObligationDisposition,
    Report as ConformanceReport,
};
use quire_protocol::model::v2::{
    Internality, LinkedAssessment, RoleClassification, RuntimeBinding, RuntimeBindingSet,
    SlotCompatibility, SlotHandle, StaticBindingDefinition, StaticSelection, StaticSlot,
    RUNTIME_BINDING_CONTRACT, STATIC_BINDING_CONTRACT,
};
use quire_protocol::outcome::OperationOutcome;
use quire_protocol::producer_contracts::{
    ProducerContract, ProducerReadiness, ProducerResolutionRequest,
};
use quire_protocol::repro::{CorpusIdentity, ReproductionInputs, Seed, ToolchainIdentity};
use quire_protocol::result::Limits as ResultLimits;
use quire_protocol::version_lock::{ResolutionBudget, VersionLockManifest};
use quire_protocol::{v2, wire as w};
use quire_spec_language::protocol_artifact::{
    checked_predicate,
    native_temporal::{self, request as temporal_request, v2 as temporal_v2},
    occurrence_key_schema, temporal_subject, v3, WorkflowInstanceIdentity,
};

#[allow(dead_code)]
#[path = "support/observation_owner.rs"]
mod observation_owner;
#[path = "support/producer_selection.rs"]
mod producer_selection;
#[allow(dead_code, unused_imports)]
#[path = "support/v2_handoff/mod.rs"]
mod v2_handoff;

fn admitted_package() -> v2::AdmittedPackage {
    let loaded = v2_handoff::Handoff::load().expect("published v2 handoff");
    let declarations = loaded.declarations();
    let inventories = loaded.inventories(&declarations);
    let expected = loaded.expected(&inventories);
    quire_protocol::intake::v2::admit(loaded.offer(), &expected, loaded.limits())
        .into_result()
        .expect("strict v2 package")
}

fn checked_views(
    package: &v2::AdmittedPackage,
) -> (
    checked_predicate::ValidatedCheckedPredicate,
    temporal_subject::ValidatedTemporalSubject,
) {
    let temporal_declaration = package
        .inherited()
        .declarations
        .iter()
        .position(|declaration| matches!(declaration.body, w::Body::Temporal { .. }))
        .and_then(|index| u32::try_from(index).ok())
        .expect("temporal declaration");
    let temporal_selection = temporal_subject::DeclarationSelection::new(temporal_declaration);
    let temporal_document = temporal_subject::derive(
        package,
        temporal_selection,
        temporal_subject::Limits::default(),
    )
    .into_result()
    .expect("temporal subject document");
    let temporal = temporal_subject::read(
        temporal_document.bytes(),
        package,
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
        package,
        predicate_selection.clone(),
        checked_predicate::Limits::default(),
    )
    .into_result()
    .expect("checked predicate document");
    let predicate = checked_predicate::read(
        predicate_document.bytes(),
        package,
        predicate_selection,
        checked_predicate::Limits::default(),
    )
    .into_result()
    .expect("checked predicate view");
    (predicate, temporal)
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

fn assert_owner_evidence(
    actual: &native_temporal::EvidenceRef,
    expected: &native_temporal::EvidenceRef,
) {
    assert_eq!(actual.contract(), expected.contract());
    assert_eq!(actual.schema_digest(), expected.schema_digest());
    assert_eq!(actual.identity(), expected.identity());
    assert_eq!(actual.digest(), expected.digest());
    assert_eq!(actual.authority_identity(), expected.authority_identity());
    assert_eq!(actual.authority_revision(), expected.authority_revision());
    assert_eq!(actual.authority_digest(), expected.authority_digest());
    assert_eq!(actual.scope(), expected.scope());
    assert_eq!(actual.population(), expected.population());
}

fn activated_request(
    package: &v2::AdmittedPackage,
    subject: &temporal_subject::ValidatedTemporalSubject,
    trigger: &[u8],
    decision: &observation_owner::ScopeViews,
    surrounding: &observation_owner::ScopeViews,
) -> temporal_v2::ValidatedRequest {
    let declaration = subject.declaration();
    let leaf = package.inherited().declarations[usize::try_from(declaration).expect("index fits")]
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
        correspondence: temporal_evidence("correspondence", "correspondence:it002"),
        positions: vec![temporal_request::ObservedPosition {
            observation: observation_owner::observation_evidence(
                &decision.observation,
                "observation",
            ),
            position: quire_spec_language::temporal::Position {
                coordinate: 0,
                order: Some(quire_spec_language::temporal::OrderKey {
                    authority: "events".into(),
                    key: 0,
                }),
                valuations: BTreeMap::from([(leaf, true)]),
            },
        }],
        anchor: "origin".into(),
        triggers: vec![temporal_v2::TriggerInput {
            receipt: "receipt:it002".into(),
            anchor: "origin".into(),
            payload: String::new(),
            guard: None,
            captures: Vec::new(),
        }],
        trigger_evidence: quire_spec_language::temporal::Evidence::Admitted,
        trigger_scope: quire_spec_language::temporal::Closure::Closed,
        decision_progress: temporal_request::ProgressInput {
            reference: observation_owner::progress_evidence(
                &decision.progress_closed,
                "decision-progress",
            ),
            watermark: observation_owner::progress_watermark(&decision.progress_closed),
        },
        decision_closure: temporal_request::ClosureInput {
            reference: observation_owner::closure_evidence(
                &decision.closure_closed,
                "decision-closure",
            ),
            state: quire_spec_language::temporal::Closure::Closed,
        },
        surrounding_progress: temporal_request::ProgressInput {
            reference: observation_owner::progress_evidence(
                &surrounding.progress_closed,
                "surrounding-progress",
            ),
            watermark: observation_owner::progress_watermark(&surrounding.progress_closed),
        },
        surrounding_closure: temporal_request::ClosureInput {
            reference: observation_owner::closure_evidence(
                &surrounding.closure_closed,
                "surrounding-closure",
            ),
            state: quire_spec_language::temporal::Closure::Closed,
        },
        execution: quire_spec_language::temporal::Execution::Completed,
        completeness: temporal_request::CompletenessInput {
            reference: observation_owner::completeness_evidence(&decision.completeness_complete),
            state: quire_spec_language::temporal::Completeness::Complete,
            facts: vec![temporal_evidence("completeness-fact", "fact:it002")],
        },
        authoritative_origin: true,
        evicted: Vec::new(),
    };
    let document = temporal_v2::produce(subject, input, native_temporal::Limits::default())
        .into_result()
        .expect("produce activation temporal request");
    let request = temporal_v2::read(
        document.bytes(),
        subject,
        native_temporal::Limits::default(),
    )
    .into_result()
    .expect("strict-read activation temporal request");
    assert_owner_evidence(
        request.decision_progress().reference(),
        &observation_owner::progress_evidence(&decision.progress_closed, "decision-progress"),
    );
    assert_eq!(
        request.decision_progress().watermark(),
        observation_owner::progress_watermark(&decision.progress_closed)
    );
    assert_owner_evidence(
        request.decision_closure().reference(),
        &observation_owner::closure_evidence(&decision.closure_closed, "decision-closure"),
    );
    assert_owner_evidence(
        request.surrounding_progress().reference(),
        &observation_owner::progress_evidence(&surrounding.progress_closed, "surrounding-progress"),
    );
    assert_owner_evidence(
        request.surrounding_closure().reference(),
        &observation_owner::closure_evidence(&surrounding.closure_closed, "surrounding-closure"),
    );
    assert_owner_evidence(
        request.completeness().reference(),
        &observation_owner::completeness_evidence(&decision.completeness_complete),
    );
    request
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
    let loaded = v2_handoff::Handoff::load().expect("published v2 handoff");
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
    v3::read(
        candidate.bytes(),
        &v3::Expected {
            inherited,
            activations: std::slice::from_ref(&activation),
        },
        quire_spec_language::protocol_artifact::Limits::default(),
    )
    .into_result()
    .expect("strict v3 mapping package admits")
}

fn global() -> GlobalConformanceInput {
    let premises = vec![
        GlobalPremise {
            kind: PremiseKind::Execution,
            identity: "premise:it002:execution".into(),
            scope_identity: "scope:it002".into(),
            authority_identity: "authority:it002".into(),
        },
        GlobalPremise {
            kind: PremiseKind::Branch,
            identity: "premise:it002:branch".into(),
            scope_identity: "scope:it002".into(),
            authority_identity: "authority:it002".into(),
        },
        GlobalPremise {
            kind: PremiseKind::Workflow,
            identity: "premise:it002:workflow".into(),
            scope_identity: "scope:it002".into(),
            authority_identity: "authority:it002".into(),
        },
    ];
    GlobalConformanceInput {
        state: GlobalConformanceClosure::Closed,
        required_premises: premises.clone(),
        premises,
    }
}

fn producer_manifest() -> VersionLockManifest {
    VersionLockManifest::new(
        ProducerContract::ALL
            .iter()
            .copied()
            .map(ProducerContract::manifest_entry)
            .collect(),
    )
}

fn dispatch_request() -> ClaimRequest<'static> {
    let component = |identity: &[u8]| {
        Component::retained(
            identity.to_vec().into_boxed_slice(),
            identity.to_vec().into_boxed_slice(),
        )
    };
    ClaimRequest {
        identity: b"request:it002:conformance".to_vec().into_boxed_slice(),
        claim: Claim::Conformance,
        strength: b"tested".to_vec().into_boxed_slice(),
        premises: PremiseSet {
            admitted_fragment: component(b"fragment:it002"),
            backend: BackendSelection::retained(
                b"qprotocol:conformance".to_vec().into_boxed_slice(),
                b"v1".to_vec().into_boxed_slice(),
            ),
            assumptions: Vec::new(),
            communication: vec![component(b"communication:it002")],
            visibility: vec![component(b"visibility:it002")],
            environment: vec![component(b"environment:it002")],
            sources: vec![component(b"source:it002")],
            derived_artifact: None,
        },
        correspondence: None,
    }
}

fn reproduction() -> quire_protocol::repro::ReproductionRecord {
    ReproductionInputs::nothing_offered()
        .with_seed(Seed::new(0x5eed_0000_0000_0002))
        .with_toolchain(ToolchainIdentity::new(
            b"rustc-1.98.1-x86_64-unknown-linux-gnu",
            "IT-002 toolchain",
        ))
        .with_corpus(CorpusIdentity::new(b"corpus:it002", "IT-002 corpus"))
        .record()
        .expect("complete IT-002 reproduction identity")
}

fn semantic_dispositions(report: &ConformanceReport) -> BTreeMap<&str, ObligationDisposition> {
    report
        .obligations
        .iter()
        .map(|result| (result.obligation.as_str(), result.disposition))
        .collect()
}

fn assert_semantically_equal(left: &ConformanceReport, right: &ConformanceReport) {
    assert_eq!(semantic_dispositions(left), semantic_dispositions(right));
    assert_eq!(left.global, right.global);
    assert_eq!(left.global_premise_defects, right.global_premise_defects);
    assert_eq!(left.premises, right.premises);
}

#[test]
#[trace(
    "Task-021",
    "IT-002",
    "FR-004-AC-4",
    "FR-026-AC-1",
    "FR-026-AC-2",
    "FR-026-AC-3"
)]
fn it_002_sc_05_batch_and_incremental_agree_with_early_settlement() {
    let package = admitted_package();
    let (_, temporal) = checked_views(&package);
    let (declaration_handle, roles, subject_slot) = package
        .inherited()
        .declarations
        .iter()
        .enumerate()
        .find_map(|(declaration, value)| {
            let w::Body::Protocol { roles, .. } = &value.body else {
                return None;
            };
            let subject_slot = value
                .bindings
                .iter()
                .position(|binding| binding.kind != w::BindingKind::Relationship)?;
            Some((
                u32::try_from(declaration).ok()?,
                roles,
                u32::try_from(subject_slot).ok()?,
            ))
        })
        .expect("published handoff has a protocol subject slot");
    let producer = observation_owner::admitted_producer();
    let subject = observation_owner::qualified_subject(&producer, "it002-incremental");
    let definition = StaticBindingDefinition::new(
        STATIC_BINDING_CONTRACT,
        StaticSelection {
            identity: "binding-definition:it002-incremental".into(),
            revision: "1".into(),
            digest: [0x63; 32],
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
        vec![StaticSlot {
            handle: SlotHandle::new(subject_slot),
            compatibility: SlotCompatibility::Subject {
                kind: subject.kind().clone(),
            },
        }],
    )
    .expect("incremental static definition admits");
    let runtime = RuntimeBindingSet::new(
        RUNTIME_BINDING_CONTRACT,
        definition.selection(),
        &definition,
        &producer,
        vec![RuntimeBinding::subject(
            SlotHandle::new(subject_slot),
            subject.clone(),
        )],
    )
    .expect("incremental runtime binding admits");
    let context = LinkedAssessment::new(&package, &definition, &runtime)
        .expect("incremental linked assessment admits");
    let mapping = admitted_activation_map(&package);
    let decision = observation_owner::scope_views("it002-incremental");
    let surrounding = observation_owner::scope_views("it002-incremental-surrounding");
    let temporal_request = activated_request(
        &package,
        &temporal,
        subject.identity().as_bytes(),
        &decision,
        &surrounding,
    );
    let temporal_result = activated_result(&temporal_request);
    let runtime_selection = producer_selection::runtime(&runtime, 500_000)
        .expect("fixture runtime selection is bounded");
    let subject_selection = producer_selection::subject(&subject, 500_000)
        .expect("fixture subject selection is bounded");
    let selection_bytes = runtime_selection
        .framed_bytes()
        .checked_add(subject_selection.framed_bytes())
        .expect("fixture selection bytes fit usize");
    let producer_request = ProducerResolutionRequest::for_activated_assessment(
        package.digest(),
        definition.selection().digest,
        &[Claim::Conformance],
        runtime_selection,
        mapping.digest(),
        subject_selection,
        temporal.document().digest(),
        temporal_request.document().digest(),
        temporal_result.document().digest(),
    )
    .expect("activated producer selection is representable");
    let producer_budget = ResolutionBudget::new(
        ProducerContract::ALL.len(),
        selection_bytes,
        Duration::from_secs(1),
    )
    .expect("exact producer-resolution budget");
    let outcome = with_activated_after_resolving(
        &producer_request,
        &producer_manifest(),
        producer_budget,
        SystemMonotonicClock,
        |permit| {
            ActivatedAssessmentInputs::new(
                permit,
                context,
                vec![Claim::Conformance],
                &mapping,
                &subject,
                &temporal,
                &temporal_request,
                &temporal_result,
            )
        },
        |engine| {
            let activation = engine
                .conformance()
                .bind_activation(
                    engine.linked(),
                    &mapping,
                    &subject,
                    &temporal,
                    &temporal_request,
                    &temporal_result,
                )
                .expect("incremental activation binds");
            let occurrence_schema = occurrence_key_schema(
                &package,
                declaration_handle,
                quire_spec_language::protocol_artifact::Limits::default(),
            )
            .into_result()
            .expect("occurrence schema");
            let occurrence_node = occurrence_schema
                .nodes()
                .get(usize::try_from(activation.control().index).expect("control index fits"))
                .expect("event occurrence schema");
            let first_occurrence = occurrence_schema
                .bind(
                    WorkflowInstanceIdentity::new("workflow:it002:first")
                        .expect("workflow identity"),
                    activation.control(),
                    occurrence_node.role().clone(),
                    &[],
                )
                .expect("first occurrence admits");
            let second_occurrence = occurrence_schema
                .bind(
                    WorkflowInstanceIdentity::new("workflow:it002:second")
                        .expect("workflow identity"),
                    activation.control(),
                    occurrence_node.role().clone(),
                    &[],
                )
                .expect("second occurrence admits");
            let first = engine
                .conformance()
                .activate_occurrence(&activation, first_occurrence)
                .expect("first activated occurrence");
            let second = engine
                .conformance()
                .activate_occurrence(&activation, second_occurrence)
                .expect("second activated occurrence");
            let first_fact = engine
                .conformance()
                .activation_fact(first.clone(), Some(&decision.observation));
            let second_fact = engine
                .conformance()
                .activation_fact(second.clone(), Some(&decision.observation));
            let open_global = GlobalConformanceInput {
                state: GlobalConformanceClosure::Open,
                ..global()
            };
            let closed_global = global();
            let open_axes = AxisInputs {
                decision_scope_progress: &decision.progress_open,
                decision_scope_closure: &decision.closure_open,
                surrounding_execution_progress: &surrounding.progress_open,
                surrounding_execution_closure: &surrounding.closure_open,
            };
            let closed_axes = AxisInputs {
                decision_scope_progress: &decision.progress_closed,
                decision_scope_closure: &decision.closure_closed,
                surrounding_execution_progress: &surrounding.progress_closed,
                surrounding_execution_closure: &surrounding.closure_closed,
            };

            let activations = [first, second];
            let mut early = ActivatedIncremental::new(engine.conformance(), &activations)
                .expect("fixed activation population admits");
            let prefix = early
                .push(
                    &first_fact,
                    open_axes,
                    &decision.completeness_incomplete,
                    &open_global,
                    AssessmentExecution::Completed,
                    ConformanceLimits::default(),
                )
                .expect("first prefix assesses");
            assert_eq!(
                prefix
                    .obligations
                    .iter()
                    .filter(|result| result.disposition == ObligationDisposition::Conforms)
                    .count(),
                1
            );
            assert_eq!(
                prefix
                    .obligations
                    .iter()
                    .filter(|result| result.disposition == ObligationDisposition::Pending)
                    .count(),
                1
            );
            let final_incremental = early
                .push(
                    &second_fact,
                    closed_axes,
                    &decision.completeness_complete,
                    &closed_global,
                    AssessmentExecution::Completed,
                    ConformanceLimits::default(),
                )
                .expect("final prefix assesses")
                .clone();
            let authored_facts = [first_fact.clone(), second_fact.clone()];
            let authored_batch = engine
                .conformance()
                .assess_activated(
                    &authored_facts,
                    closed_axes,
                    &decision.completeness_complete,
                    &closed_global,
                    AssessmentExecution::Completed,
                    ConformanceLimits::default(),
                )
                .expect("batch assessment");
            assert_semantically_equal(&authored_batch, &final_incremental);
            assert_eq!(authored_batch.global, GlobalDisposition::Conforms);

            let mut permuted = ActivatedIncremental::new(engine.conformance(), &activations)
                .expect("same fixed activation population admits");
            permuted
                .push(
                    &second_fact,
                    open_axes,
                    &decision.completeness_incomplete,
                    &open_global,
                    AssessmentExecution::Completed,
                    ConformanceLimits::default(),
                )
                .expect("permuted first prefix assesses");
            let permuted_final = permuted
                .push(
                    &first_fact,
                    closed_axes,
                    &decision.completeness_complete,
                    &closed_global,
                    AssessmentExecution::Completed,
                    ConformanceLimits::default(),
                )
                .expect("permuted final prefix assesses")
                .clone();
            let permuted_facts = [second_fact.clone(), first_fact.clone()];
            let permuted_batch = engine
                .conformance()
                .assess_activated(
                    &permuted_facts,
                    closed_axes,
                    &decision.completeness_complete,
                    &closed_global,
                    AssessmentExecution::Completed,
                    ConformanceLimits::default(),
                )
                .expect("permuted batch assessment");
            assert_semantically_equal(&permuted_batch, &permuted_final);
            assert_eq!(
                semantic_dispositions(&authored_batch),
                semantic_dispositions(&permuted_batch)
            );

            let mut changed_global = closed_global.clone();
            changed_global
                .premises
                .retain(|premise| premise.kind != PremiseKind::Workflow);
            let changed_batch = engine
                .conformance()
                .assess_activated(
                    &authored_facts,
                    closed_axes,
                    &decision.completeness_complete,
                    &changed_global,
                    AssessmentExecution::Completed,
                    ConformanceLimits::default(),
                )
                .expect("changed premise produces a complete adverse report");
            let mut changed_incremental =
                ActivatedIncremental::new(engine.conformance(), &activations)
                    .expect("changed path uses the same activation population");
            changed_incremental
                .push(
                    &first_fact,
                    open_axes,
                    &decision.completeness_incomplete,
                    &open_global,
                    AssessmentExecution::Completed,
                    ConformanceLimits::default(),
                )
                .expect("changed first prefix assesses");
            let changed_final = changed_incremental
                .push(
                    &second_fact,
                    closed_axes,
                    &decision.completeness_complete,
                    &changed_global,
                    AssessmentExecution::Completed,
                    ConformanceLimits::default(),
                )
                .expect("changed final prefix assesses")
                .clone();
            assert_semantically_equal(&changed_batch, &changed_final);
            assert_ne!(changed_batch.global, authored_batch.global);
            assert!(!changed_batch.global_premise_defects.is_empty());
        },
    );
    assert!(matches!(
        outcome,
        ProducerReadiness::Executable(OperationOutcome::Completed(()))
    ));
}

#[test]
#[trace("Task-021", "IT-002", "NFR-002-AC-1")]
fn it_002_sc_06_each_composed_budget_is_exact_one_short_and_fresh() {
    let package = admitted_package();
    let (predicate, temporal) = checked_views(&package);
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
    let producer = observation_owner::admitted_producer();
    let subject = observation_owner::qualified_subject(&producer, "it002-budget");
    let role_instance_kind =
        SubjectKind::new(b"role-instance".to_vec()).expect("nonempty role-instance kind");
    let mut static_slots = vec![StaticSlot {
        handle: SlotHandle::new(subject_slot),
        compatibility: SlotCompatibility::Subject {
            kind: subject.kind().clone(),
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
            identity: "binding-definition:it002-budget".into(),
            revision: "1".into(),
            digest: [0x64; 32],
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
    .expect("budget static definition admits");
    let mut runtime_bindings = vec![RuntimeBinding::subject(
        SlotHandle::new(subject_slot),
        subject.clone(),
    )];
    runtime_bindings.extend(roles.iter().enumerate().map(|(index, role)| {
        RuntimeBinding::subject(
            SlotHandle::new(role.instance),
            QualifiedSubject::new(
                &producer,
                role_instance_kind.clone(),
                SubjectIdentity::new(format!("role-instance:it002-budget:{index}").into_bytes())
                    .expect("nonempty role-instance identity"),
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
    .expect("budget runtime binding admits");
    let mapping = admitted_activation_map(&package);
    let decision = observation_owner::scope_views("it002-budget");
    let surrounding = observation_owner::scope_views("it002-budget-surrounding");
    let temporal_request = activated_request(
        &package,
        &temporal,
        subject.identity().as_bytes(),
        &decision,
        &surrounding,
    );
    let temporal_result = activated_result(&temporal_request);
    let runtime_selection = producer_selection::runtime(&runtime, 500_000)
        .expect("fixture runtime selection is bounded");
    let subject_selection = producer_selection::subject(&subject, 500_000)
        .expect("fixture subject selection is bounded");
    let selection_bytes = runtime_selection
        .framed_bytes()
        .checked_add(subject_selection.framed_bytes())
        .expect("fixture selection bytes fit usize");
    let producer_request = ProducerResolutionRequest::for_result_assessment(
        package.digest(),
        definition.selection().digest,
        &[Claim::Conformance],
        runtime_selection,
        mapping.digest(),
        subject_selection,
        temporal.document().digest(),
        temporal_request.document().digest(),
        temporal_result.document().digest(),
        predicate.document().digest(),
    )
    .expect("result producer selection is representable");
    let producer_budget = || {
        ResolutionBudget::new(
            ProducerContract::ALL.len(),
            selection_bytes,
            Duration::from_secs(1),
        )
        .expect("fresh exact producer-resolution budget")
    };
    let occurrence_schema = occurrence_key_schema(
        &package,
        declaration_handle,
        quire_spec_language::protocol_artifact::Limits::default(),
    )
    .into_result()
    .expect("occurrence schema");
    let closed_axes = AxisInputs {
        decision_scope_progress: &decision.progress_closed,
        decision_scope_closure: &decision.closure_closed,
        surrounding_execution_progress: &surrounding.progress_closed,
        surrounding_execution_closure: &surrounding.closure_closed,
    };
    let foreign_decision = observation_owner::scope_views("it002-budget-foreign");
    let foreign_surrounding = observation_owner::scope_views("it002-budget-foreign-surrounding");
    let foreign_axes = AxisInputs {
        decision_scope_progress: &foreign_decision.progress_closed,
        decision_scope_closure: &foreign_decision.closure_closed,
        surrounding_execution_progress: &foreign_surrounding.progress_closed,
        surrounding_execution_closure: &foreign_surrounding.closure_closed,
    };
    let global = global();

    let run =
        |budget: Budget,
         selected_axes: AxisInputs<'_>,
         selected_completeness: &quire_observation::authority::completeness::View| {
            let readiness = with_result_assessment_after_resolving(
                &producer_request,
                &producer_manifest(),
                producer_budget(),
                SystemMonotonicClock,
                |permit| {
                    ResultAssessmentInputs::new(
                        permit,
                        LinkedAssessment::new(&package, &definition, &runtime)
                            .expect("fresh budget linked assessment admits"),
                        vec![Claim::Conformance],
                        &mapping,
                        &subject,
                        &temporal_request,
                        &temporal_result,
                        &predicate,
                        &temporal,
                    )
                },
                |engine| {
                    let activation = engine
                        .conformance()
                        .bind_activation(
                            engine.linked(),
                            &mapping,
                            &subject,
                            &temporal,
                            &temporal_request,
                            &temporal_result,
                        )
                        .expect("budget activation binds");
                    let occurrence_node = occurrence_schema
                        .nodes()
                        .get(
                            usize::try_from(activation.control().index)
                                .expect("control index fits"),
                        )
                        .expect("event occurrence schema");
                    let facts = [
                        "workflow:it002-budget:first",
                        "workflow:it002-budget:second",
                    ]
                    .map(|workflow| {
                        let occurrence = occurrence_schema
                            .bind(
                                WorkflowInstanceIdentity::new(workflow).expect("workflow identity"),
                                activation.control(),
                                occurrence_node.role().clone(),
                                &[],
                            )
                            .expect("budget occurrence admits");
                        let activated = engine
                            .conformance()
                            .activate_occurrence(&activation, occurrence)
                            .expect("budget occurrence activates");
                        engine
                            .conformance()
                            .activation_fact(activated, Some(&decision.observation))
                    });
                    let request = dispatch_request();
                    engine
                        .assess_activated(
                            std::slice::from_ref(&request),
                            &facts,
                            selected_axes,
                            selected_completeness,
                            &global,
                            AssessmentExecution::Completed,
                            ConformanceLimits::default(),
                            DispatchLimits::default(),
                            vec![Some(CanonicalRowControls::new(
                                reproduction(),
                                ResultLimits::owner_max(),
                            ))],
                            budget,
                        )
                        .map(|_| ())
                },
            );
            match readiness {
                ProducerReadiness::Executable(OperationOutcome::Completed(outcome)) => outcome,
                ProducerReadiness::Executable(OperationOutcome::Refused(refusal))
                | ProducerReadiness::Unexecutable(refusal) => OperationOutcome::Refused(refusal),
                ProducerReadiness::Executable(OperationOutcome::ResourceIncomplete(incomplete))
                | ProducerReadiness::ResourceIncomplete(incomplete) => {
                    OperationOutcome::ResourceIncomplete(incomplete)
                }
            }
        };

    let exercised = [
        (BudgetDimension::InputByte, 20_818),
        (BudgetDimension::Node, 6),
        (BudgetDimension::Observation, 2),
        (BudgetDimension::ResultItem, 5),
        (BudgetDimension::OutputByte, 9_205),
        (BudgetDimension::Operation, 243),
        (BudgetDimension::AccountedAllocation, 4),
    ];
    assert!(matches!(
        run(
            Budget::nothing_granted(),
            foreign_axes,
            &foreign_decision.completeness_complete,
        ),
        OperationOutcome::Refused(refusal)
            if refusal.code().as_str()
                == "assessment-activation.temporal-evidence-mismatch"
    ));
    for (dimension, exact) in exercised {
        let grant = |value| {
            Budget::nothing_granted()
                .grant(dimension, value)
                .expect("positive scenario grant")
        };
        assert!(matches!(
            run(grant(exact), closed_axes, &decision.completeness_complete),
            OperationOutcome::Completed(())
        ));
        assert!(matches!(
            run(
                grant(exact - 1),
                closed_axes,
                &decision.completeness_complete,
            ),
            OperationOutcome::ResourceIncomplete(incomplete)
                if incomplete.dimension() == dimension
                    && incomplete.granted() == exact - 1
        ));
        assert!(matches!(
            run(grant(exact), closed_axes, &decision.completeness_complete),
            OperationOutcome::Completed(())
        ));
    }
}
