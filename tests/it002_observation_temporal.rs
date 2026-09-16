// SPDX-License-Identifier: AGPL-3.0-or-later
//! IT-002: accepted QObs and QSL temporal facts through Protocol assessment.

use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};
use std::time::{Duration, Instant};

use ix_trace_rs::trace;
use quire_observation::SubjectKind;
use quire_protocol::assessment::{
    AxisInputs, BundleRefusal, CanonicalRowFacts, Engine, GlobalConformanceInput, GlobalPremise,
    PremiseKind,
};
use quire_protocol::bounds::inventory::{Budget, BudgetDimension};
use quire_protocol::bounds::Clock;
use quire_protocol::claims::dispatch::{ClaimRequest, Limits as DispatchLimits};
use quire_protocol::claims::premises::{BackendSelection, Component, PremiseSet};
use quire_protocol::claims::Claim;
use quire_protocol::closure::{AssessmentExecution, GlobalConformanceClosure};
use quire_protocol::conformance::{
    ActivatedAssessmentError, ActivatedIncremental, ErrorCode as ConformanceErrorCode,
    GlobalDisposition, GlobalPremiseDefectKind, KernelRefusal, Limits as ConformanceLimits,
    ObligationDisposition, Report as ConformanceReport,
};
use quire_protocol::model::v2::{
    Internality, LinkedAssessment, RoleClassification, RuntimeBinding, RuntimeBindingSet,
    SlotCompatibility, SlotHandle, StaticBindingDefinition, StaticSelection, StaticSlot,
    RUNTIME_BINDING_CONTRACT, STATIC_BINDING_CONTRACT,
};
use quire_protocol::outcome::OperationOutcome;
use quire_protocol::producer_contracts::{
    resolve_producer_contracts, ProducerContract, ProducerResolutionRequest,
    UnsupportedObservationCapability, QOBS_REVISION,
};
use quire_protocol::repro::{CorpusIdentity, ReproductionInputs, Seed, ToolchainIdentity};
use quire_protocol::result::{
    runtime_reference, Activation, Adequacy, DecisionSupport, DependenceRelation, Limitation,
    Limits as ResultLimits, Participation, ProtocolAdequacy, Settlement, SettlementBasis, Truth,
};
use quire_protocol::version_lock::{
    EntryStatus, ManifestEntry, ResolutionBudget, ResolutionRefusalCode, VersionLockManifest,
};
use quire_protocol::{v2, wire as w, ByteDigest};
use quire_spec_language::protocol_artifact::{
    checked_predicate,
    native_temporal::{self, request as temporal_request, v2 as temporal_v2},
    occurrence_key_schema, temporal_subject, v3, WorkflowInstanceIdentity,
};

#[allow(dead_code)]
#[path = "support/observation_owner.rs"]
mod observation_owner;
#[allow(dead_code, unused_imports)]
#[path = "support/v2_handoff/mod.rs"]
mod v2_handoff;

#[derive(Debug)]
struct ScriptedClock {
    origin: Instant,
    nanos: Vec<u64>,
    next: Cell<usize>,
}

impl ScriptedClock {
    fn new(nanos: &[u64]) -> Self {
        Self {
            origin: Instant::now(),
            nanos: nanos.to_vec(),
            next: Cell::new(0),
        }
    }
}

impl Clock for ScriptedClock {
    fn monotonic(&self) -> Instant {
        let index = self.next.get();
        self.next.set(index.saturating_add(1));
        self.origin + Duration::from_nanos(*self.nanos.get(index).unwrap_or(&0))
    }
}

fn generous_budget() -> ResolutionBudget {
    ResolutionBudget::new(ProducerContract::ALL.len() + 2, 1, Duration::from_secs(1))
        .expect("positive exact-contract resolution budget")
}

fn exact_budget() -> ResolutionBudget {
    ResolutionBudget::new(ProducerContract::ALL.len(), 1, Duration::from_nanos(1))
        .expect("positive exact-contract resolution budget")
}

fn generous_clock() -> ScriptedClock {
    ScriptedClock::new(&[0; 64])
}

fn manifest() -> VersionLockManifest {
    VersionLockManifest::new(
        ProducerContract::ALL
            .iter()
            .copied()
            .map(ProducerContract::manifest_entry)
            .collect(),
    )
}

fn refusal_code(
    outcome: OperationOutcome<quire_protocol::producer_contracts::ProducerResolution>,
) -> String {
    let OperationOutcome::Refused(refusal) = outcome else {
        panic!("mutated producer-contract resolution must refuse")
    };
    refusal.code().as_str().to_owned()
}

#[test]
#[trace(
    "IT-002",
    "FR-009-AC-1",
    "FR-009-AC-2",
    "FR-009-AC-3",
    "FR-009-AC-4",
    "FR-009-AC-5",
    "FR-009-AC-6"
)]
fn it_002_sc_08_resolves_only_the_exact_scenario_contract_inventory() {
    assert_eq!(ProducerContract::ALL.len(), 19);
    assert_eq!(QOBS_REVISION, "581d98f1ac9f1467cc5355785d67b242f8d5d150");
    let request = ProducerResolutionRequest::new(b"it002:scenario-resolution")
        .expect("nonempty scenario resolution identity");
    let outcome =
        resolve_producer_contracts(&request, &manifest(), exact_budget(), generous_clock());
    let OperationOutcome::Completed(resolved) = outcome else {
        panic!("the exact accepted producer contracts must resolve")
    };
    assert_eq!(resolved.request(), &request);
    assert_eq!(
        resolved.manifest_entries().len(),
        ProducerContract::ALL.len()
    );
    assert!(resolved.manifest_entries().iter().all(|entry| {
        entry.status() == EntryStatus::Pinned
            && entry.immutability_assertion().is_some_and(|assertion| {
                assertion.as_bytes() == b"owner asserts immutable and published"
            })
    }));
    assert_eq!(
        resolved
            .usage()
            .consumed(BudgetDimension::ResolutionEntryCount),
        ProducerContract::ALL.len()
    );
    assert_eq!(
        resolved
            .contracts()
            .iter()
            .map(|contract| contract.contract())
            .collect::<Vec<_>>(),
        ProducerContract::ALL
    );
    for contract in resolved.contracts() {
        assert_eq!(
            contract.revision(),
            contract.contract().expected_revision().as_bytes()
        );
    }
    assert_eq!(
        resolved
            .contracts()
            .iter()
            .find(|resolved| { resolved.contract() == ProducerContract::NativeTemporalRequestV1 })
            .expect("v1 native-temporal request resolved")
            .revision(),
        b"f1700a9264d6d3bcdd07e0f77b70f3dae9ed4c07"
    );
    assert_eq!(
        resolved
            .contracts()
            .iter()
            .find(|resolved| { resolved.contract() == ProducerContract::NativeTemporalRequestV2 })
            .expect("v2 native-temporal request resolved")
            .revision(),
        b"0c2f13e261c0ffeaf89f0638931b573ec038e99b"
    );

    let qobs_schemas = ProducerContract::ALL
        .iter()
        .copied()
        .filter_map(|contract| contract.owner_schema().map(|schema| (contract, schema)))
        .collect::<Vec<_>>();
    assert_eq!(qobs_schemas.len(), 15);
    for (contract, schema) in &qobs_schemas {
        assert_eq!(contract.expected_revision(), QOBS_REVISION);
        assert_eq!(
            format!("{:x}", ByteDigest::of(schema.bytes())),
            schema.sha256()
        );
    }
    assert_eq!(
        qobs_schemas
            .iter()
            .filter(|(contract, _)| {
                matches!(
                    contract,
                    ProducerContract::ObservationPartialFact
                        | ProducerContract::ObservationActivationAuthority
                        | ProducerContract::ObservationAuthorityBundleV1
                        | ProducerContract::ObservationAuthorityBundleV2
                        | ProducerContract::ObservationAuthorityLineage
                )
            })
            .map(|(contract, schema)| (contract.contract_version(), schema.sha256()))
            .collect::<Vec<_>>(),
        vec![
            (
                "quire.observation.partial-fact/v1",
                "9dc068376a7db5a7509810910c36e2efff6b9c3701d83c474701ec572f13f23c",
            ),
            (
                "quire.observation.activation-scope-authority/v1",
                "6d8b3b41a7fdc9b7d4dba085809b396bc41e41c89f9c512ff1580108d43663e8",
            ),
            (
                "quire.observation-authority/v1",
                "6b6f4e3b08a4b16476b55ae6cdfdff4440b8c053f056ffd7b9bdcde47e200296",
            ),
            (
                "quire.observation-authority/v2",
                "f92335c5e5e1e3434f1c2484557ca1f512c22b496eb97319c94fcac3491f116a",
            ),
            (
                "quire.observation-authority-lineage/v1",
                "d58c70516e53bde40ab2d276df143fffb05740348b98d09a71e89857a82a22ef",
            ),
        ]
    );
    assert_eq!(
        UnsupportedObservationCapability::ALL,
        [
            UnsupportedObservationCapability::Repair,
            UnsupportedObservationCapability::AggregateQuery,
        ]
    );

    let absent = VersionLockManifest::new(
        ProducerContract::ALL
            .iter()
            .copied()
            .filter(|contract| *contract != ProducerContract::ObservationRecord)
            .map(ProducerContract::manifest_entry)
            .collect(),
    );
    assert_eq!(
        refusal_code(resolve_producer_contracts(
            &request,
            &absent,
            generous_budget(),
            generous_clock(),
        )),
        ResolutionRefusalCode::EntryAbsent.as_str()
    );

    let mut unpinned_entries = manifest().entries().to_vec();
    unpinned_entries[0] = ManifestEntry::pinned_interface(
        ProducerContract::ALL[0].key(),
        None,
        Some(b"owner asserts immutable and published"),
        None,
    );
    assert_eq!(
        refusal_code(resolve_producer_contracts(
            &request,
            &VersionLockManifest::new(unpinned_entries),
            generous_budget(),
            generous_clock(),
        )),
        ResolutionRefusalCode::RevisionAbsent.as_str()
    );

    let mut duplicate_entries = manifest().entries().to_vec();
    duplicate_entries.push(ProducerContract::ObservationRecord.manifest_entry());
    assert_eq!(
        refusal_code(resolve_producer_contracts(
            &request,
            &VersionLockManifest::new(duplicate_entries),
            generous_budget(),
            generous_clock(),
        )),
        ResolutionRefusalCode::DuplicateEntryKey.as_str()
    );

    let mut unresolved_entries = manifest().entries().to_vec();
    unresolved_entries[0] = ManifestEntry::unresolved_interface(
        ProducerContract::ALL[0].key(),
        b"owner acceptance withdrawn",
    );
    assert_eq!(
        refusal_code(resolve_producer_contracts(
            &request,
            &VersionLockManifest::new(unresolved_entries),
            generous_budget(),
            generous_clock(),
        )),
        ResolutionRefusalCode::EntryUnresolved.as_str()
    );

    let mut repointed_entries = manifest().entries().to_vec();
    let selected = ProducerContract::NativeTemporalRequestV2;
    let selected_index = ProducerContract::ALL
        .iter()
        .position(|contract| *contract == selected)
        .expect("selected contract belongs to the exact inventory");
    repointed_entries[selected_index] = ManifestEntry::pinned_interface(
        selected.key(),
        Some(b"foreign-revision"),
        Some(b"owner asserts immutable and published"),
        None,
    );
    assert_eq!(
        refusal_code(resolve_producer_contracts(
            &request,
            &VersionLockManifest::new(repointed_entries),
            generous_budget(),
            generous_clock(),
        )),
        ResolutionRefusalCode::InterfaceSelectionMismatch.as_str()
    );

    let one_short =
        ResolutionBudget::new(ProducerContract::ALL.len() - 1, 1, Duration::from_nanos(1))
            .expect("positive one-short entry budget");
    assert!(matches!(
        resolve_producer_contracts(&request, &manifest(), one_short, generous_clock()),
        OperationOutcome::ResourceIncomplete(incomplete)
            if incomplete.dimension() == BudgetDimension::ResolutionEntryCount
    ));
    assert!(matches!(
        resolve_producer_contracts(&request, &manifest(), exact_budget(), generous_clock()),
        OperationOutcome::Completed(_)
    ));
}

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
    let (predicate_declaration, predicate_root) = package
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
        .expect("checked Boolean leaf");
    let predicate_selection =
        checked_predicate::ClauseSelection::new(predicate_declaration, predicate_root);
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

fn activated_request(
    package: &v2::AdmittedPackage,
    subject: &temporal_subject::ValidatedTemporalSubject,
    trigger: &[u8],
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
            observation: temporal_evidence("observation", "observation:it002"),
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
            reference: temporal_evidence("decision-progress", "decision-progress:it002"),
            watermark: 0,
        },
        decision_closure: temporal_request::ClosureInput {
            reference: temporal_evidence("decision-closure", "decision-closure:it002"),
            state: quire_spec_language::temporal::Closure::Closed,
        },
        surrounding_progress: temporal_request::ProgressInput {
            reference: temporal_evidence("surrounding-progress", "surrounding-progress:it002"),
            watermark: 0,
        },
        surrounding_closure: temporal_request::ClosureInput {
            reference: temporal_evidence("surrounding-closure", "surrounding-closure:it002"),
            state: quire_spec_language::temporal::Closure::Closed,
        },
        execution: quire_spec_language::temporal::Execution::Completed,
        completeness: temporal_request::CompletenessInput {
            reference: temporal_evidence("completeness", "completeness:it002"),
            state: quire_spec_language::temporal::Completeness::Complete,
            facts: vec![temporal_evidence("completeness-fact", "fact:it002")],
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

#[test]
#[trace(
    "IT-002",
    "FR-003-AC-1",
    "FR-004-AC-1",
    "FR-004-AC-2",
    "FR-006-AC-1",
    "FR-006-AC-2",
    "FR-028-AC-1",
    "NFR-002-AC-1"
)]
fn it_002_sc_01_through_04_flow_owner_facts_to_one_canonical_result() {
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
    let subject = observation_owner::qualified_subject(&producer, "it002");
    let definition = StaticBindingDefinition::new(
        STATIC_BINDING_CONTRACT,
        StaticSelection {
            identity: "binding-definition:it002".into(),
            revision: "1".into(),
            digest: [0x62; 32],
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
                kind: SubjectKind::new(b"ix://agent-ix/commerce/type/Order".to_vec())
                    .expect("nonempty subject kind"),
            },
        }],
    )
    .expect("selected static binding definition admits");
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

    let temporal_request = activated_request(&package, &temporal, subject.identity().as_bytes());
    let temporal_result = activated_result(&temporal_request);
    let requested = [Claim::Conformance];
    let engine = Engine::new_activated(context, requested.to_vec(), &subject, &temporal_request)
        .expect("strict QSL/QObs activation starts one Engine");
    let mapping = admitted_activation_map(&package);
    let activation = engine
        .conformance()
        .bind_activation(
            engine.linked(),
            &mapping,
            &subject,
            &temporal_request,
            &temporal_result,
        )
        .expect("strict mapping binds the exact QObs trigger");
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
    let occurrence = occurrence_schema
        .bind(
            WorkflowInstanceIdentity::new("workflow:it002").expect("nonempty workflow identity"),
            activation.control(),
            occurrence_node.role().clone(),
            &[],
        )
        .expect("activation occurrence admits");
    let activated = engine
        .conformance()
        .activate_occurrence(&activation, occurrence)
        .expect("mapped activation produces an obligation occurrence");
    let decision = observation_owner::scope_views("it002");
    let surrounding = observation_owner::scope_views("it002-surrounding");
    let fact = engine
        .conformance()
        .activation_fact(activated, Some(&decision.observation));
    let global = global();
    let closed_axes = AxisInputs {
        decision_scope_progress: &decision.progress_closed,
        decision_scope_closure: &decision.closure_closed,
        surrounding_execution_progress: &surrounding.progress_closed,
        surrounding_execution_closure: &surrounding.closure_closed,
    };
    let open_decision_progress = AxisInputs {
        decision_scope_progress: &decision.progress_open,
        ..closed_axes
    };
    let open_report = engine
        .conformance()
        .assess_activated(
            std::slice::from_ref(&fact),
            open_decision_progress,
            &decision.completeness_complete,
            &global,
            AssessmentExecution::Completed,
            ConformanceLimits::default(),
        )
        .expect("one independently open axis is a typed report");
    assert_eq!(
        open_report
            .premises
            .decision_scope_progress
            .identity
            .as_ref(),
        decision.progress_open.identity().as_str()
    );
    assert_eq!(
        open_report
            .premises
            .decision_scope_closure
            .identity
            .as_ref(),
        decision.closure_closed.identity().as_str()
    );

    let closed_report = engine
        .conformance()
        .assess_activated(
            std::slice::from_ref(&fact),
            closed_axes,
            &decision.completeness_complete,
            &global,
            AssessmentExecution::Completed,
            ConformanceLimits::default(),
        )
        .expect("all closed axes conform");
    assert_eq!(closed_report.global, GlobalDisposition::Conforms);
    let axis_cases = [
        (
            AxisInputs {
                decision_scope_progress: &decision.progress_open,
                ..closed_axes
            },
            [
                decision.progress_open.identity().as_str(),
                decision.closure_closed.identity().as_str(),
                surrounding.progress_closed.identity().as_str(),
                surrounding.closure_closed.identity().as_str(),
            ],
        ),
        (
            AxisInputs {
                decision_scope_closure: &decision.closure_open,
                ..closed_axes
            },
            [
                decision.progress_closed.identity().as_str(),
                decision.closure_open.identity().as_str(),
                surrounding.progress_closed.identity().as_str(),
                surrounding.closure_closed.identity().as_str(),
            ],
        ),
        (
            AxisInputs {
                surrounding_execution_progress: &surrounding.progress_open,
                ..closed_axes
            },
            [
                decision.progress_closed.identity().as_str(),
                decision.closure_closed.identity().as_str(),
                surrounding.progress_open.identity().as_str(),
                surrounding.closure_closed.identity().as_str(),
            ],
        ),
        (
            AxisInputs {
                surrounding_execution_closure: &surrounding.closure_open,
                ..closed_axes
            },
            [
                decision.progress_closed.identity().as_str(),
                decision.closure_closed.identity().as_str(),
                surrounding.progress_closed.identity().as_str(),
                surrounding.closure_open.identity().as_str(),
            ],
        ),
    ];
    for (selected_axes, expected) in axis_cases {
        let report = engine
            .conformance()
            .assess_activated(
                std::slice::from_ref(&fact),
                selected_axes,
                &decision.completeness_complete,
                &global,
                AssessmentExecution::Completed,
                ConformanceLimits::default(),
            )
            .expect("one independently changed scope axis is retained");
        assert_eq!(
            [
                report.premises.decision_scope_progress.identity.as_ref(),
                report.premises.decision_scope_closure.identity.as_ref(),
                report
                    .premises
                    .surrounding_execution_progress
                    .identity
                    .as_ref(),
                report
                    .premises
                    .surrounding_execution_closure
                    .identity
                    .as_ref(),
            ],
            expected
        );
        assert_eq!(
            report.obligations[0].disposition,
            closed_report.obligations[0].disposition
        );
        assert_eq!(report.global, GlobalDisposition::Pending);
    }

    for completeness in [
        &decision.completeness_complete,
        &decision.completeness_incomplete,
        &decision.completeness_contradicted,
    ] {
        let report = engine
            .conformance()
            .assess_activated(
                std::slice::from_ref(&fact),
                closed_axes,
                completeness,
                &global,
                AssessmentExecution::Completed,
                ConformanceLimits::default(),
            )
            .expect("each owner completeness state remains representable");
        assert_eq!(
            report.premises.completeness.identity.as_ref(),
            completeness.identity().as_str()
        );
    }

    for state in [
        GlobalConformanceClosure::Closed,
        GlobalConformanceClosure::Open,
        GlobalConformanceClosure::Incomplete,
        GlobalConformanceClosure::Contradicted,
    ] {
        let selected_global = GlobalConformanceInput {
            state,
            ..global.clone()
        };
        let report = engine
            .conformance()
            .assess_activated(
                std::slice::from_ref(&fact),
                closed_axes,
                &decision.completeness_complete,
                &selected_global,
                AssessmentExecution::Completed,
                ConformanceLimits::default(),
            )
            .expect("each global closure state remains representable");
        assert_eq!(report.premises.global_conformance.state, state);
        assert_eq!(
            report.obligations[0].disposition,
            closed_report.obligations[0].disposition
        );
    }
    let not_required = GlobalConformanceInput {
        state: GlobalConformanceClosure::NotRequired,
        ..global.clone()
    };
    assert!(matches!(
        engine.conformance().assess_activated(
            std::slice::from_ref(&fact),
            closed_axes,
            &decision.completeness_complete,
            &not_required,
            AssessmentExecution::Completed,
            ConformanceLimits::default(),
        ),
        Err(ActivatedAssessmentError::Conformance(refusal))
            if refusal.code() == ConformanceErrorCode::InvalidPremises
    ));

    for kind in [
        PremiseKind::Execution,
        PremiseKind::Branch,
        PremiseKind::Workflow,
    ] {
        let mut omitted = global.clone();
        omitted.premises.retain(|premise| premise.kind != kind);
        let report = engine
            .conformance()
            .assess_activated(
                std::slice::from_ref(&fact),
                closed_axes,
                &decision.completeness_complete,
                &omitted,
                AssessmentExecution::Completed,
                ConformanceLimits::default(),
            )
            .expect("an omitted global premise is a complete adverse report");
        assert_eq!(
            report
                .global_premise_defects
                .iter()
                .map(|defect| defect.kind)
                .collect::<BTreeSet<_>>(),
            BTreeSet::from([GlobalPremiseDefectKind::Missing])
        );
        assert_eq!(
            report.obligations[0].disposition,
            closed_report.obligations[0].disposition
        );

        for component in 0..3 {
            let mut mutated = global.clone();
            let selected = mutated
                .premises
                .iter_mut()
                .find(|premise| premise.kind == kind)
                .expect("selected global premise exists");
            match component {
                0 => selected.identity.push_str(":mutated"),
                1 => selected.scope_identity.push_str(":mutated"),
                2 => selected.authority_identity.push_str(":mutated"),
                _ => unreachable!("three exact premise components"),
            }
            let report = engine
                .conformance()
                .assess_activated(
                    std::slice::from_ref(&fact),
                    closed_axes,
                    &decision.completeness_complete,
                    &mutated,
                    AssessmentExecution::Completed,
                    ConformanceLimits::default(),
                )
                .expect("an exact premise mutation is a complete adverse report");
            let expected = if component == 0 {
                BTreeSet::from([
                    GlobalPremiseDefectKind::Missing,
                    GlobalPremiseDefectKind::Unexpected,
                ])
            } else {
                BTreeSet::from([GlobalPremiseDefectKind::CrossWired])
            };
            assert_eq!(
                report
                    .global_premise_defects
                    .iter()
                    .map(|defect| defect.kind)
                    .collect::<BTreeSet<_>>(),
                expected
            );
            assert_eq!(
                report.obligations[0].disposition,
                closed_report.obligations[0].disposition
            );
        }
    }

    let mut duplicated = global.clone();
    duplicated.premises.push(duplicated.premises[0].clone());
    let duplicate_report = engine
        .conformance()
        .assess_activated(
            std::slice::from_ref(&fact),
            closed_axes,
            &decision.completeness_complete,
            &duplicated,
            AssessmentExecution::Completed,
            ConformanceLimits::default(),
        )
        .expect("a duplicate global premise is retained as an adverse report");
    assert_eq!(
        duplicate_report
            .global_premise_defects
            .iter()
            .map(|defect| defect.kind)
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([GlobalPremiseDefectKind::Duplicate])
    );
    assert_eq!(
        duplicate_report.obligations[0].disposition,
        closed_report.obligations[0].disposition
    );

    let mut missing_branch = global.clone();
    missing_branch
        .premises
        .retain(|premise| premise.kind != PremiseKind::Branch);
    let changed = engine
        .conformance()
        .assess_activated(
            std::slice::from_ref(&fact),
            closed_axes,
            &decision.completeness_complete,
            &missing_branch,
            AssessmentExecution::Completed,
            ConformanceLimits::default(),
        )
        .expect("a missing global premise is retained as an adverse report");
    assert!(!changed.global_premise_defects.is_empty());
    assert_eq!(
        changed.obligations[0].disposition,
        open_report.obligations[0].disposition
    );

    let request_identity = engine.producer_resolution_request();
    let OperationOutcome::Completed(resolution) = resolve_producer_contracts(
        &request_identity,
        &manifest(),
        exact_budget(),
        generous_clock(),
    ) else {
        panic!("the exact producer set resolves for this Engine")
    };
    let foreign_subject = observation_owner::qualified_subject(&producer, "it002-foreign");
    let foreign_temporal_request =
        activated_request(&package, &temporal, foreign_subject.identity().as_bytes());
    assert!(matches!(
        Engine::new_activated(
            context,
            requested.to_vec(),
            &subject,
            &foreign_temporal_request,
        ),
        Err(BundleRefusal::ActivationSubjectMismatch)
    ));
    let foreign_temporal_result = activated_result(&foreign_temporal_request);
    assert!(matches!(
        engine.conformance().bind_activation(
            engine.linked(),
            &mapping,
            &subject,
            &temporal_request,
            &foreign_temporal_result,
        ),
        Err(KernelRefusal::ActivationResultMismatch)
    ));
    let foreign_engine = Engine::new_activated(
        context,
        requested.to_vec(),
        &foreign_subject,
        &foreign_temporal_request,
    )
    .expect("foreign activation still forms its own immutable Engine request");
    assert_ne!(
        foreign_engine.producer_resolution_request(),
        request_identity,
        "producer authorization cannot be retargeted across activation subjects"
    );

    let support_observation = decision
        .completeness_complete
        .payload()
        .facts()
        .next()
        .and_then(|available| available.observation_identity)
        .expect("complete QObs view retains its supporting observation")
        .to_owned();
    let row = CanonicalRowFacts {
        checked_predicate: &predicate,
        temporal_subject: &temporal,
        settlement: Settlement {
            basis: SettlementBasis::ClosedScope,
            supporting_fact_identities: vec![support_observation.clone()],
            supporting_progress_identities: vec![decision
                .progress_closed
                .identity()
                .as_str()
                .to_owned()],
        },
        activation: Activation::Triggered,
        participation: Participation::Known {
            required_role_instance_refs: vec!["role:it002".into()],
            participating_role_instance_refs: vec!["role:it002".into()],
        },
        observation_adequacy: Adequacy::Adequate,
        protocol_adequacy: ProtocolAdequacy::Demonstrated,
        decision_support: vec![DecisionSupport {
            oracle_identity: "oracle:it002".into(),
            observation_identity: support_observation,
            relation: DependenceRelation::Independent,
            predecessor: None,
            dependence_group: None,
        }],
        limitations: vec![Limitation {
            code: "bound:finite".into(),
            affected_identity: "claim:it002".into(),
        }],
        model_artifact,
        runtime_input: &subject,
        reproduction: reproduction(),
        correction: None,
        limits: ResultLimits::owner_max(),
    };
    let foreign_outcome = foreign_engine.assess_activated(
        &resolution,
        &[],
        &[],
        closed_axes,
        &decision.completeness_complete,
        &global,
        AssessmentExecution::Completed,
        ConformanceLimits::default(),
        DispatchLimits::default(),
        Vec::new(),
        Budget::nothing_granted(),
    );
    assert!(matches!(
        foreign_outcome,
        OperationOutcome::Refused(refusal)
            if refusal.code().as_str()
                == ResolutionRefusalCode::CapabilityRequestMismatch.as_str()
    ));
    let budget_engine = engine.clone();
    let budget_row = row.clone();
    let dispatch = dispatch_request();
    let outcome = engine.assess_activated(
        &resolution,
        std::slice::from_ref(&dispatch),
        std::slice::from_ref(&fact),
        closed_axes,
        &decision.completeness_complete,
        &global,
        AssessmentExecution::Completed,
        ConformanceLimits::default(),
        DispatchLimits::default(),
        vec![Some(row)],
        Budget::nothing_granted(),
    );
    let OperationOutcome::Completed(bundle) = outcome else {
        panic!("complete owner facts produce one canonical bundle: {outcome:?}")
    };
    let quire_protocol::outcome::ClaimOutcome::Canonical(canonical) = &bundle.rows()[0].outcome
    else {
        panic!("the conformance disposition must produce its canonical row")
    };
    assert_eq!(canonical.truth(), Truth::Satisfied);
    assert_eq!(
        canonical.decision_scope_progress().document_identity(),
        decision.progress_closed.identity().as_str()
    );
    assert_eq!(
        canonical.decision_scope_closure().document_identity(),
        decision.closure_closed.identity().as_str()
    );
    assert_eq!(
        canonical
            .surrounding_execution_progress()
            .document_identity(),
        surrounding.progress_closed.identity().as_str()
    );
    assert_eq!(
        canonical
            .surrounding_execution_closure()
            .document_identity(),
        surrounding.closure_closed.identity().as_str()
    );
    assert_eq!(
        canonical.completeness().document_identity(),
        decision.completeness_complete.identity().as_str()
    );
    assert_eq!(
        canonical.global_conformance_state(),
        GlobalConformanceClosure::Closed
    );
    assert_eq!(canonical.global_premises(), global.premises);
    assert_eq!(
        canonical.provenance().producer_identity,
        producer.key().bundle_identity
    );
    runtime_reference::validate_subject(&canonical.provenance().input_identity, &subject)
        .expect("canonical provenance retains the exact FR-287 subject");
    let retained_resolution = bundle
        .producer_resolution()
        .expect("public composed assessment retains its producer resolution");
    assert_eq!(retained_resolution.request(), resolution.request());
    assert_eq!(
        retained_resolution.manifest_entries(),
        resolution.manifest_entries()
    );
    assert_eq!(retained_resolution.contracts(), resolution.contracts());
    assert_eq!(retained_resolution.usage(), resolution.usage());
    assert_eq!(bundle.budget(), Some(&Budget::nothing_granted()));

    let usage = bundle.usage();
    let mut exercised_dimensions = 0usize;
    let mut one_short_dimensions = 0usize;
    for dimension in BudgetDimension::ALL {
        let consumed = usage.consumed(dimension);
        if consumed == 0 {
            continue;
        }
        exercised_dimensions += 1;
        if consumed > 1 {
            one_short_dimensions += 1;
            let one_short = Budget::nothing_granted()
                .grant(dimension, consumed - 1)
                .expect("positive one-short scenario grant");
            let request = dispatch_request();
            assert!(matches!(
                budget_engine.clone().assess_activated(
                    &resolution,
                    std::slice::from_ref(&request),
                    std::slice::from_ref(&fact),
                    closed_axes,
                    &decision.completeness_complete,
                    &global,
                    AssessmentExecution::Completed,
                    ConformanceLimits::default(),
                    DispatchLimits::default(),
                    vec![Some(budget_row.clone())],
                    one_short,
                ),
                OperationOutcome::ResourceIncomplete(incomplete)
                    if incomplete.dimension() == dimension
            ));
        }

        let exact = || {
            Budget::nothing_granted()
                .grant(dimension, consumed)
                .expect("positive exact scenario grant")
        };
        for _ in 0..2 {
            let request = dispatch_request();
            assert!(matches!(
                budget_engine.clone().assess_activated(
                    &resolution,
                    std::slice::from_ref(&request),
                    std::slice::from_ref(&fact),
                    closed_axes,
                    &decision.completeness_complete,
                    &global,
                    AssessmentExecution::Completed,
                    ConformanceLimits::default(),
                    DispatchLimits::default(),
                    vec![Some(budget_row.clone())],
                    exact(),
                ),
                OperationOutcome::Completed(_)
            ));
        }
    }
    assert!(
        exercised_dimensions >= 4,
        "the composed scenario must exercise multiple independent budget dimensions"
    );
    assert!(
        one_short_dimensions >= 3,
        "the composed scenario must exercise representable one-short grants across stages"
    );
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
#[trace("IT-002", "FR-004-AC-4", "FR-026-AC-1", "FR-026-AC-2", "FR-026-AC-3")]
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
    let temporal_request = activated_request(&package, &temporal, subject.identity().as_bytes());
    let temporal_result = activated_result(&temporal_request);
    let engine = Engine::new_activated(
        context,
        vec![Claim::Conformance],
        &subject,
        &temporal_request,
    )
    .expect("incremental Engine starts");
    let mapping = admitted_activation_map(&package);
    let activation = engine
        .conformance()
        .bind_activation(
            engine.linked(),
            &mapping,
            &subject,
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
            WorkflowInstanceIdentity::new("workflow:it002:first").expect("workflow identity"),
            activation.control(),
            occurrence_node.role().clone(),
            &[],
        )
        .expect("first occurrence admits");
    let second_occurrence = occurrence_schema
        .bind(
            WorkflowInstanceIdentity::new("workflow:it002:second").expect("workflow identity"),
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
    let decision = observation_owner::scope_views("it002-incremental");
    let surrounding = observation_owner::scope_views("it002-incremental-surrounding");
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
    let mut changed_incremental = ActivatedIncremental::new(engine.conformance(), &activations)
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
}
