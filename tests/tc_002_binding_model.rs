// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-002: versioned static/runtime binding capability admission.

use std::cell::{Cell, OnceCell};
use std::collections::{BTreeMap, BTreeSet};
use std::time::Duration;

use agent_ix_baseline_producer::StaticProducerBundle;
use ix_trace_rs::trace;
use quire_observation::{
    QualifiedSubject, Relationship, RelationshipIdentity, SubjectIdentity, SubjectKind,
};
use quire_protocol::bounds::inventory::Budget;
use quire_protocol::bounds::SystemMonotonicClock;
use quire_protocol::model::v2::{
    link as link_v2, ChannelHandle, DefinitionRegistry, FifoPartitionRefusal, Internality,
    LinkedAssessment, RegistryRefusal, RoleClassification, RuntimeBinding, RuntimeBindingSet,
    RuntimeRefusal, SlotCompatibility, SlotHandle, StaticBindingDefinition, StaticRefusal,
    StaticSelection, StaticSlot, RUNTIME_BINDING_CONTRACT, STATIC_BINDING_CONTRACT,
};
use quire_protocol::wire::{BindingKind, Body, CausalEdge, ControlOperation, EdgeKind, Port};
use quire_protocol::{
    assessment::{
        with_activated_after_resolving, with_assessment_after_resolving, ActivatedAssessmentInputs,
        AssessmentInputs, AxisInputs, GlobalConformanceInput, GlobalPremise, PremiseKind,
    },
    claims::{
        dispatch::{ClaimRequest, Decision, Limits as DispatchLimits},
        premises::{Assumption, AssumptionState, BackendSelection, Component, PremiseSet},
        Claim,
    },
    closure::{AssessmentExecution, GlobalConformanceClosure},
    conformance::{
        ActivatedAssessmentError, ActivatedIncremental, AdverseKind, ErrorCode, GlobalDisposition,
        Limits as ConformanceLimits, ObligationDisposition,
    },
    outcome::OperationOutcome,
    producer_contracts::{
        select_runtime_bindings, select_subject, ProducerContract, ProducerReadiness,
        ProducerResolutionRequest, ProducerValueSelection,
    },
    version_lock::{ManifestEntry, ResolutionBudget, VersionLockManifest},
};
use quire_spec_language::protocol_artifact::{
    native_temporal::{self, request, v2 as temporal_v2},
    occurrence_key_schema, temporal_subject, v2 as artifact_v2, v3, wire as qsl_wire,
    WorkflowInstanceIdentity,
};
use quire_spec_language::{temporal, ByteDigest};

// The shared handoff reader carries all public mutation helpers; this focused
// binding lane exercises only `Handoff`, so the remaining helper surface is
// intentionally unused here.
#[allow(dead_code, unused_imports)]
#[path = "support/v2_handoff/mod.rs"]
mod handoff;

#[path = "support/observation_owner.rs"]
#[allow(dead_code)]
mod observation_owner;

const ORDER_KIND: &[u8] = b"ix://agent-ix/commerce/type/Order";
const SHIPMENT_KIND: &[u8] = b"ix://agent-ix/commerce/type/Shipment";
const ORDER_SHIPMENT: &str = "ix://agent-ix/commerce/relationship/Order-shipment";

fn producer_manifest() -> VersionLockManifest {
    VersionLockManifest::new(
        ProducerContract::ALL
            .iter()
            .copied()
            .map(ProducerContract::manifest_entry)
            .collect(),
    )
}

fn producer_budget() -> ResolutionBudget {
    ResolutionBudget::new(
        ProducerContract::ALL.len(),
        1_000_000,
        Duration::from_secs(1),
    )
    .expect("the exact producer-resolution budget is valid")
}

fn global_conformance(state: GlobalConformanceClosure) -> GlobalConformanceInput {
    let premises = vec![
        GlobalPremise {
            kind: PremiseKind::Execution,
            identity: "premise:activation:execution".into(),
            scope_identity: "scope:activation:execution".into(),
            authority_identity: "authority:activation:execution".into(),
        },
        GlobalPremise {
            kind: PremiseKind::Branch,
            identity: "premise:activation:branch".into(),
            scope_identity: "scope:activation:branch".into(),
            authority_identity: "authority:activation:branch".into(),
        },
        GlobalPremise {
            kind: PremiseKind::Workflow,
            identity: "premise:activation:workflow".into(),
            scope_identity: "scope:activation:workflow".into(),
            authority_identity: "authority:activation:workflow".into(),
        },
    ];
    GlobalConformanceInput {
        state,
        required_premises: premises.clone(),
        premises,
    }
}

fn closed_global_conformance() -> GlobalConformanceInput {
    global_conformance(GlobalConformanceClosure::Closed)
}

fn closed_axes<'a>(
    decision: &'a observation_owner::ScopeViews,
    surrounding: &'a observation_owner::ScopeViews,
) -> AxisInputs<'a> {
    AxisInputs {
        decision_scope_progress: &decision.progress_closed,
        decision_scope_closure: &decision.closure_closed,
        surrounding_execution_progress: &surrounding.progress_closed,
        surrounding_execution_closure: &surrounding.closure_closed,
    }
}

fn admitted_published_v2() -> quire_spec_language::protocol_artifact::v2::AdmittedPackage {
    let loaded = handoff::Handoff::load().expect("published v2 handoff");
    let declarations = loaded.declarations();
    let inventories = loaded.inventories(&declarations);
    let expected = loaded.expected(&inventories);
    quire_protocol::intake::v2::admit(loaded.offer(), &expected, loaded.limits())
        .into_result()
        .expect("published v2 offer admits")
}

fn foreign_clock(
    clock: &artifact_v2::wire::ClockConfiguration,
) -> artifact_v2::wire::ClockConfiguration {
    match clock {
        artifact_v2::wire::ClockConfiguration::EventPosition { sequence_authority } => {
            artifact_v2::wire::ClockConfiguration::EventPosition {
                sequence_authority: format!("{sequence_authority}:foreign"),
            }
        }
        artifact_v2::wire::ClockConfiguration::FixedSample {
            epoch,
            period,
            unit,
        } => artifact_v2::wire::ClockConfiguration::FixedSample {
            epoch: epoch.clone(),
            period: period.clone(),
            unit: format!("{unit}:foreign"),
        },
        artifact_v2::wire::ClockConfiguration::TimestampedEvent { timestamp_unit } => {
            artifact_v2::wire::ClockConfiguration::TimestampedEvent {
                timestamp_unit: format!("{timestamp_unit}:foreign"),
            }
        }
    }
}

fn admitted_v2_with_foreign_temporal_table() -> artifact_v2::AdmittedPackage {
    let package = admitted_published_v2();
    let mut offer = package.package().clone();
    let binding = offer
        .temporal_bindings
        .first_mut()
        .expect("published v2 package retains a temporal binding");
    let changed_clock = foreign_clock(&binding.clock);
    binding.clock = changed_clock.clone();
    let candidate = artifact_v2::encode_candidate(
        &offer,
        quire_spec_language::protocol_artifact::Limits::default(),
    )
    .into_result()
    .expect("foreign temporal v2 offer encodes");
    let loaded = handoff::Handoff::load().expect("published v2 handoff");
    let declarations = loaded.declarations();
    let inventories = loaded.inventories(&declarations);
    let mut seal = package
        .artifact()
        .expect("strict v2 handoff retained its artifact seal")
        .clone();
    seal.digest = candidate.digest();
    let base = loaded.expected_for(&inventories, &seal);
    let mut temporal = base.temporal.to_vec();
    temporal
        .first_mut()
        .expect("published expectation retains a temporal selection")
        .clock = &changed_clock;
    let expected = artifact_v2::Expected {
        inherited: base.inherited,
        temporal: &temporal,
    };
    quire_protocol::intake::v2::admit(candidate.bytes(), &expected, loaded.limits())
        .into_result()
        .expect("independently selected foreign temporal v2 package admits")
}

/// Strictly admit the published `/2` graph with one independently authored
/// `/3` event-to-temporal mapping.  The test owns the offer and independent
/// expectation separately so the Protocol kernel never needs a local mapping
/// mirror.
fn admitted_published_v3(
    package: &quire_spec_language::protocol_artifact::v2::AdmittedPackage,
) -> v3::AdmittedPackage {
    let loaded = handoff::Handoff::load().expect("published v2 handoff");
    let declarations = loaded.declarations();
    let inventories = loaded.inventories(&declarations);
    let (declaration, control) = package
        .inherited()
        .declarations
        .iter()
        .enumerate()
        .find_map(|(declaration, item)| {
            let Body::Protocol { controls, .. } = &item.body else {
                return None;
            };
            controls.iter().enumerate().find_map(|(control, item)| {
                matches!(item.operation, ControlOperation::Event { .. }).then_some((
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
        Body::Protocol {
            temporal_requirements,
            ..
        } => *temporal_requirements
            .first()
            .expect("selected protocol requires one temporal declaration"),
        _ => unreachable!("selected declaration is a protocol"),
    };
    let activation = v3::ExpectedActivation {
        control: qsl_wire::Handle {
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

/// Strictly admit another `/3` package whose inherited v1 graph and activation
/// mapping equal the linked package while its independently selected complete
/// `/2` temporal table differs. Protocol must not treat this as an extension of
/// the linked `/2` owner value.
fn admitted_v3_with_foreign_temporal_table(
    package: &artifact_v2::AdmittedPackage,
) -> v3::AdmittedPackage {
    let canonical = admitted_published_v3(package);
    let mut offer = canonical.package().clone();
    let binding = offer
        .temporal_bindings
        .first_mut()
        .expect("published v3 package retains a temporal binding");
    let changed_clock = foreign_clock(&binding.clock);
    binding.clock = changed_clock.clone();

    let candidate = v3::encode_candidate(
        &offer,
        quire_spec_language::protocol_artifact::Limits::default(),
    )
    .into_result()
    .expect("foreign temporal v3 mapping offer encodes");
    let loaded = handoff::Handoff::load().expect("published v2 handoff");
    let declarations = loaded.declarations();
    let inventories = loaded.inventories(&declarations);
    let mut seal = package
        .artifact()
        .expect("strict v2 handoff retained its artifact seal")
        .clone();
    seal.wire.version = "3".into();
    seal.digest = candidate.digest();
    let base = loaded.expected_for(&inventories, &seal);
    let mut temporal = base.temporal.to_vec();
    let first = temporal
        .first_mut()
        .expect("published expectation retains a temporal selection");
    first.clock = &changed_clock;
    let inherited = artifact_v2::Expected {
        inherited: base.inherited,
        temporal: &temporal,
    };
    let selected = offer
        .activation_mappings
        .first()
        .expect("published v3 package retains an activation mapping");
    let activation = v3::ExpectedActivation {
        control: selected.control.clone(),
        temporal_declaration: selected.temporal_declaration,
    };
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
    .expect("independently selected foreign temporal v3 package admits")
}

fn protocol_coordinates(
    package: &quire_spec_language::protocol_artifact::v2::AdmittedPackage,
) -> (u32, u32) {
    package
        .inherited()
        .declarations
        .iter()
        .enumerate()
        .find_map(|(index, declaration)| {
            let Body::Protocol { .. } = &declaration.body else {
                return None;
            };
            let subject_slot = declaration
                .bindings
                .iter()
                .position(|binding| binding.kind != BindingKind::Relationship)?;
            Some((
                u32::try_from(index).expect("published declaration index fits"),
                u32::try_from(subject_slot).expect("published slot index fits"),
            ))
        })
        .expect("published handoff has a protocol with a subject slot")
}

fn effect_coordinates(
    package: &quire_spec_language::protocol_artifact::v2::AdmittedPackage,
) -> (u32, u32) {
    package
        .inherited()
        .declarations
        .iter()
        .enumerate()
        .find_map(|(index, declaration)| {
            let Body::Protocol { .. } = &declaration.body else {
                return None;
            };
            let effect_slot = declaration
                .bindings
                .iter()
                .position(|binding| binding.kind == BindingKind::Effect)?;
            Some((
                u32::try_from(index).expect("published declaration index fits"),
                u32::try_from(effect_slot).expect("published slot index fits"),
            ))
        })
        .expect("published handoff has a protocol business-effect slot")
}

fn context_refusal(
    result: Result<LinkedAssessment<'_>, quire_protocol::model::v2::ContextRefusal>,
) -> quire_protocol::model::v2::ContextRefusal {
    result.expect_err("the one-axis context mutation must refuse")
}

/// An owner-declared causal endpoint reduced only to its local table coordinate
/// and its closed owner port tag. This test helper never constructs QSL edges.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct CausalEndpoint {
    control: u32,
    port: u8,
}

fn causal_endpoint(edge: &CausalEdge, from: bool) -> CausalEndpoint {
    let endpoint = if from { &edge.from } else { &edge.to };
    CausalEndpoint {
        control: endpoint.node.index,
        port: match endpoint.port {
            Port::Enter => 0,
            Port::Exit => 1,
        },
    }
}

/// Enumerates every linear extension of the owner-declared acyclic half of a
/// selected parallel subgraph. The source reader owns edge derivation and the
/// repeat back-edge; this helper only proves the retained relation permits
/// every order that preserves those edges.
fn parallel_orders(edges: &[&CausalEdge]) -> Vec<Vec<CausalEndpoint>> {
    let relation: Vec<_> = edges
        .iter()
        .filter(|edge| edge.kind != EdgeKind::RepeatProgress)
        .map(|edge| (causal_endpoint(edge, true), causal_endpoint(edge, false)))
        .collect();
    let endpoints: Vec<_> = relation
        .iter()
        .flat_map(|(from, to)| [*from, *to])
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let mut placed = Vec::new();
    let mut orders = Vec::new();
    extend_parallel_order(&endpoints, &relation, &mut placed, &mut orders);
    orders
}

fn extend_parallel_order(
    endpoints: &[CausalEndpoint],
    relation: &[(CausalEndpoint, CausalEndpoint)],
    placed: &mut Vec<CausalEndpoint>,
    orders: &mut Vec<Vec<CausalEndpoint>>,
) {
    if placed.len() == endpoints.len() {
        orders.push(placed.clone());
        return;
    }
    for endpoint in endpoints {
        if placed.contains(endpoint)
            || relation
                .iter()
                .any(|(from, to)| to == endpoint && !placed.contains(from))
        {
            continue;
        }
        placed.push(*endpoint);
        extend_parallel_order(endpoints, relation, placed, orders);
        let removed = placed.pop();
        assert_eq!(
            removed,
            Some(*endpoint),
            "the deterministic traversal unwinds"
        );
    }
}

fn producer() -> quire_observation::AdmittedStaticBundle {
    StaticProducerBundle::admit_json(include_bytes!("fixtures/fcd-static-bundle-1.2.json"))
        .expect("shared FCD static bundle is admitted")
}

fn foreign_producer() -> quire_observation::AdmittedStaticBundle {
    let mut offer: StaticProducerBundle =
        serde_json::from_slice(include_bytes!("fixtures/fcd-static-bundle-1.2.json"))
            .expect("shared FCD static offer decodes through its owner type");
    offer.bundle_identity = Some("ix://agent-ix/commerce/bundle/foreign-static".to_owned());
    offer.digest = Some(
        offer
            .canonical_digest_selection()
            .expect("the owner recomputes a valid foreign bundle digest"),
    );
    offer
        .admit()
        .expect("the independent producer-owned static offer admits")
}

fn selection() -> StaticSelection {
    StaticSelection {
        identity: "binding-definition:orders".into(),
        revision: "1".into(),
        digest: [1; 32],
    }
}

fn definition() -> StaticBindingDefinition {
    StaticBindingDefinition::new(
        STATIC_BINDING_CONTRACT,
        selection(),
        producer().key().clone(),
        3,
        vec![RoleClassification {
            role_handle: 7,
            internality: Internality::Internal,
        }],
        vec![StaticSlot {
            handle: SlotHandle::new(11),
            compatibility: SlotCompatibility::Subject {
                kind: SubjectKind::new(ORDER_KIND.to_vec()).expect("non-empty kind"),
            },
        }],
    )
    .expect("complete selected static definition")
}

fn subject(producer: &quire_observation::AdmittedStaticBundle) -> QualifiedSubject {
    QualifiedSubject::new(
        producer,
        SubjectKind::new(ORDER_KIND.to_vec()).expect("non-empty kind"),
        SubjectIdentity::new(b"order:42".to_vec()).expect("non-empty identity"),
    )
}

fn digest(label: &str) -> String {
    format!("{:x}", ByteDigest::of(label.as_bytes()))
}
fn evidence(scope: &str, identity: &str) -> native_temporal::EvidenceRef {
    native_temporal::EvidenceRef::new(
        "quire.owner-evidence/v1",
        digest(&format!("schema:{scope}")),
        identity,
        digest(&format!("document:{identity}")),
        format!("authority:{scope}"),
        "1",
        digest(&format!("authority:{scope}")),
        scope,
        1,
    )
}
fn activated_request(
    package: &quire_spec_language::protocol_artifact::v2::AdmittedPackage,
    trigger: &[u8],
) -> temporal_v2::ValidatedRequest {
    activated_request_at(package, trigger, "origin")
}

fn activated_request_at(
    package: &quire_spec_language::protocol_artifact::v2::AdmittedPackage,
    trigger: &[u8],
    anchor: &str,
) -> temporal_v2::ValidatedRequest {
    let subject = activated_temporal_subject(package);
    let leaf = package.inherited().declarations
        [usize::try_from(subject.declaration()).expect("temporal declaration fits")]
    .temporal
    .iter()
    .enumerate()
    .find_map(|(index, node)| {
        matches!(node.operation, qsl_wire::TemporalOperation::Holds { .. })
            .then(|| u32::try_from(index).expect("fixture temporal node fits"))
    })
    .expect("temporal fixture has a holds leaf");
    let input = temporal_v2::Input {
        trigger: temporal_v2::SemanticTriggerIdentity::new(trigger.to_vec())
            .expect("QObs subject identity is nonempty"),
        correspondence: evidence("correspondence", "correspondence:engine"),
        positions: vec![request::ObservedPosition {
            observation: evidence("observation", "observation:0"),
            position: temporal::Position {
                coordinate: 0,
                order: Some(temporal::OrderKey {
                    authority: "events".into(),
                    key: 0,
                }),
                valuations: BTreeMap::from([(leaf, true)]),
            },
        }],
        anchor: anchor.into(),
        triggers: vec![temporal_v2::TriggerInput {
            receipt: "receipt:1".into(),
            anchor: anchor.into(),
            payload: String::new(),
            guard: None,
            captures: Vec::new(),
        }],
        trigger_evidence: temporal::Evidence::Admitted,
        trigger_scope: temporal::Closure::Closed,
        decision_progress: request::ProgressInput {
            reference: evidence("decision-progress", "decision-progress:1"),
            watermark: 0,
        },
        decision_closure: request::ClosureInput {
            reference: evidence("decision-closure", "decision-closure:1"),
            state: temporal::Closure::Closed,
        },
        surrounding_progress: request::ProgressInput {
            reference: evidence("surrounding-progress", "surrounding-progress:1"),
            watermark: 0,
        },
        surrounding_closure: request::ClosureInput {
            reference: evidence("surrounding-closure", "surrounding-closure:1"),
            state: temporal::Closure::Open,
        },
        execution: temporal::Execution::Completed,
        completeness: request::CompletenessInput {
            reference: evidence("completeness", "completeness:1"),
            state: temporal::Completeness::Complete,
            facts: vec![evidence("completeness-fact", "fact:observation:0")],
        },
        authoritative_origin: true,
        evicted: Vec::new(),
    };
    let document = temporal_v2::produce(&subject, input, native_temporal::Limits::default())
        .into_result()
        .expect("produce opaque temporal request");
    temporal_v2::read(
        document.bytes(),
        &subject,
        native_temporal::Limits::default(),
    )
    .into_result()
    .expect("strict-read opaque temporal request")
}

fn activated_temporal_subject(
    package: &quire_spec_language::protocol_artifact::v2::AdmittedPackage,
) -> quire_spec_language::protocol_artifact::temporal_subject::ValidatedTemporalSubject {
    let declaration = package
        .inherited()
        .declarations
        .iter()
        .enumerate()
        .find_map(|(index, declaration)| {
            matches!(declaration.body, Body::Temporal { .. })
                .then(|| u32::try_from(index).expect("fixture temporal declaration fits"))
        })
        .expect("published fixture has one temporal declaration");
    let selection = temporal_subject::DeclarationSelection::new(declaration);
    let subject = temporal_subject::derive(package, selection, temporal_subject::Limits::default())
        .into_result()
        .expect("derive temporal subject");
    let subject = temporal_subject::read(
        subject.bytes(),
        package,
        selection,
        temporal_subject::Limits::default(),
    )
    .into_result()
    .expect("strict-read temporal subject");
    subject
}
fn order_shipment(producer: &quire_observation::AdmittedStaticBundle) -> Relationship {
    order_shipment_with_identity(
        producer,
        b"order:42->shipment:99",
        b"order:42",
        b"shipment:99",
    )
}

fn order_shipment_with_identity(
    producer: &quire_observation::AdmittedStaticBundle,
    relationship_identity: &[u8],
    order_identity: &[u8],
    shipment_identity: &[u8],
) -> Relationship {
    Relationship::new(
        producer,
        ORDER_SHIPMENT,
        RelationshipIdentity::new(relationship_identity.to_vec())
            .expect("non-empty relationship identity"),
        QualifiedSubject::new(
            producer,
            SubjectKind::new(ORDER_KIND.to_vec()).expect("non-empty kind"),
            SubjectIdentity::new(order_identity.to_vec()).expect("non-empty identity"),
        ),
        QualifiedSubject::new(
            producer,
            SubjectKind::new(SHIPMENT_KIND.to_vec()).expect("non-empty kind"),
            SubjectIdentity::new(shipment_identity.to_vec()).expect("non-empty identity"),
        ),
    )
    .expect("fixture declares the order-to-shipment relationship")
}

/// Versioned static and runtime contracts admit one exact selected slot.
#[test]
#[trace("TC-002", "FR-002-AC-1")]
fn tc_002_static_and_runtime_bindings_keep_static_slots_and_runtime_subjects_distinct() {
    let definition = definition();
    let producer = producer();
    let subject = subject(&producer);
    let bindings = RuntimeBindingSet::new(
        RUNTIME_BINDING_CONTRACT,
        definition.selection(),
        &definition,
        &producer,
        vec![RuntimeBinding::subject(
            SlotHandle::new(11),
            subject.clone(),
        )],
    )
    .expect("one selected runtime subject admits");

    assert_eq!(bindings.bindings().len(), 1);
    assert_eq!(bindings.bindings()[0].subject_value(), Some(&subject));
    assert_eq!(definition.declaration_handle(), 3);
    assert_eq!(definition.roles()[0].internality, Internality::Internal);
}

/// Contract, selection, slot and compatibility mutations refuse before a set
/// can represent a full-assessment runtime context.
#[test]
#[trace("TC-002", "FR-002-AC-3")]
fn tc_002_binding_contract_and_slot_mutations_refuse_without_a_runtime_set() {
    assert_eq!(
        StaticBindingDefinition::new(
            "quire.protocol.binding-definition/0",
            selection(),
            producer().key().clone(),
            3,
            vec![],
            vec![]
        ),
        Err(StaticRefusal::UnsupportedContract)
    );

    let definition = definition();
    let producer = producer();
    let subject = subject(&producer);
    let binding = RuntimeBinding::subject(SlotHandle::new(11), subject);
    assert_eq!(
        RuntimeBindingSet::new(
            "quire.protocol.runtime-binding/0",
            definition.selection(),
            &definition,
            &producer,
            vec![binding.clone()],
        ),
        Err(RuntimeRefusal::UnsupportedContract)
    );
    assert_eq!(
        RuntimeBindingSet::new(
            RUNTIME_BINDING_CONTRACT,
            &StaticSelection {
                identity: "other".into(),
                revision: "1".into(),
                digest: [1; 32],
            },
            &definition,
            &producer,
            vec![binding.clone()],
        ),
        Err(RuntimeRefusal::ForeignStaticSelection)
    );
    assert_eq!(
        RuntimeBindingSet::new(
            RUNTIME_BINDING_CONTRACT,
            &StaticSelection {
                identity: definition.selection().identity.clone(),
                revision: "2".into(),
                digest: definition.selection().digest,
            },
            &definition,
            &producer,
            vec![binding.clone()],
        ),
        Err(RuntimeRefusal::ForeignStaticSelection)
    );
    assert_eq!(
        RuntimeBindingSet::new(
            RUNTIME_BINDING_CONTRACT,
            &StaticSelection {
                identity: definition.selection().identity.clone(),
                revision: definition.selection().revision.clone(),
                digest: [2; 32],
            },
            &definition,
            &producer,
            vec![binding.clone()],
        ),
        Err(RuntimeRefusal::ForeignStaticSelection)
    );
    assert_eq!(
        RuntimeBindingSet::new(
            RUNTIME_BINDING_CONTRACT,
            definition.selection(),
            &definition,
            &producer,
            vec![],
        ),
        Err(RuntimeRefusal::MissingSlot(SlotHandle::new(11)))
    );
    assert_eq!(
        RuntimeBindingSet::new(
            RUNTIME_BINDING_CONTRACT,
            definition.selection(),
            &definition,
            &producer,
            vec![RuntimeBinding::subject(
                SlotHandle::new(99),
                binding.subject_value().expect("subject binding").clone(),
            )],
        ),
        Err(RuntimeRefusal::UnknownSlot(SlotHandle::new(99)))
    );
    assert_eq!(
        RuntimeBindingSet::new(
            RUNTIME_BINDING_CONTRACT,
            definition.selection(),
            &definition,
            &producer,
            vec![binding.clone(), binding],
        ),
        Err(RuntimeRefusal::DuplicateSlot(SlotHandle::new(11)))
    );
}

/// Every static capability validation axis has a distinct one-axis mutation;
/// none is silently repaired into a different binding definition.
#[test]
#[trace("TC-002", "FR-002-AC-3")]
fn tc_002_static_definition_mutations_refuse_before_runtime_values_exist() {
    let selected_producer = producer().key().clone();
    assert_eq!(
        StaticBindingDefinition::new(
            STATIC_BINDING_CONTRACT,
            StaticSelection {
                identity: String::new().into(),
                revision: "1".into(),
                digest: [1; 32],
            },
            selected_producer.clone(),
            3,
            vec![],
            vec![StaticSlot {
                handle: SlotHandle::new(11),
                compatibility: SlotCompatibility::Subject {
                    kind: SubjectKind::new(ORDER_KIND.to_vec()).expect("non-empty kind"),
                },
            }],
        ),
        Err(StaticRefusal::EmptySelection)
    );
    assert_eq!(
        StaticBindingDefinition::new(
            STATIC_BINDING_CONTRACT,
            StaticSelection {
                identity: "binding-definition:orders".into(),
                revision: String::new().into(),
                digest: [1; 32],
            },
            selected_producer.clone(),
            3,
            vec![],
            vec![StaticSlot {
                handle: SlotHandle::new(11),
                compatibility: SlotCompatibility::Subject {
                    kind: SubjectKind::new(ORDER_KIND.to_vec()).expect("non-empty kind"),
                },
            }],
        ),
        Err(StaticRefusal::EmptySelection)
    );
    assert_eq!(
        StaticBindingDefinition::new(
            STATIC_BINDING_CONTRACT,
            selection(),
            selected_producer.clone(),
            3,
            vec![
                RoleClassification {
                    role_handle: 7,
                    internality: Internality::Internal,
                },
                RoleClassification {
                    role_handle: 7,
                    internality: Internality::External,
                },
            ],
            vec![StaticSlot {
                handle: SlotHandle::new(11),
                compatibility: SlotCompatibility::Subject {
                    kind: SubjectKind::new(ORDER_KIND.to_vec()).expect("non-empty kind"),
                },
            }],
        ),
        Err(StaticRefusal::DuplicateRole)
    );
    let slot = StaticSlot {
        handle: SlotHandle::new(11),
        compatibility: SlotCompatibility::Subject {
            kind: SubjectKind::new(ORDER_KIND.to_vec()).expect("non-empty kind"),
        },
    };
    assert_eq!(
        StaticBindingDefinition::new(
            STATIC_BINDING_CONTRACT,
            selection(),
            selected_producer.clone(),
            3,
            vec![],
            vec![slot.clone(), slot],
        ),
        Err(StaticRefusal::DuplicateSlot)
    );
    assert_eq!(
        StaticBindingDefinition::new(
            STATIC_BINDING_CONTRACT,
            selection(),
            selected_producer.clone(),
            3,
            vec![],
            vec![],
        ),
        Err(StaticRefusal::EmptySlots)
    );
    assert_eq!(
        StaticBindingDefinition::new(
            STATIC_BINDING_CONTRACT,
            selection(),
            selected_producer,
            3,
            vec![],
            vec![StaticSlot {
                handle: SlotHandle::new(11),
                compatibility: SlotCompatibility::Relationship {
                    declaration_identity: String::new().into(),
                    source_kind: SubjectKind::new(ORDER_KIND.to_vec()).expect("non-empty kind"),
                    target_kind: SubjectKind::new(SHIPMENT_KIND.to_vec()).expect("non-empty kind"),
                },
            }],
        ),
        Err(StaticRefusal::EmptyRelationshipDeclaration)
    );
}

/// The runtime producer authority is selected by the static definition. A
/// separately admitted static bundle cannot supply a concrete runtime value to
/// that definition even when its subject kind and identity spelling match.
#[test]
#[trace("TC-002", "FR-002-AC-1", "FR-002-AC-3")]
fn tc_002_runtime_bindings_refuse_a_foreign_selected_producer() {
    let definition = definition();
    let foreign = foreign_producer();
    assert_ne!(
        definition.producer(),
        foreign.key(),
        "the test varies only the producer authority selected by the static definition"
    );
    assert_eq!(
        RuntimeBindingSet::new(
            RUNTIME_BINDING_CONTRACT,
            definition.selection(),
            &definition,
            &foreign,
            vec![RuntimeBinding::subject(
                SlotHandle::new(11),
                subject(&foreign),
            )],
        ),
        Err(RuntimeRefusal::ForeignStaticProducer)
    );
}

/// A runtime subject must itself carry the authority selected by the static
/// definition; supplying the right bundle argument cannot bless another
/// bundle's concrete QObs subject.
#[test]
#[trace("TC-002", "FR-002-AC-3")]
fn tc_002_runtime_binding_refuses_a_foreign_value_authority() {
    let definition = definition();
    let selected = producer();
    let foreign = foreign_producer();
    assert_eq!(
        RuntimeBindingSet::new(
            RUNTIME_BINDING_CONTRACT,
            definition.selection(),
            &definition,
            &selected,
            vec![RuntimeBinding::subject(
                SlotHandle::new(11),
                subject(&foreign)
            )],
        ),
        Err(RuntimeRefusal::ForeignAuthority(SlotHandle::new(11)))
    );
}

/// Producer-local kinds are compared only against the independently selected
/// compatibility fact; a stringly static slot cannot stand in for a subject.
#[test]
#[trace("TC-002", "FR-002-AC-1", "FR-002-AC-3")]
fn tc_002_runtime_subject_kind_must_match_the_explicit_slot_compatibility() {
    let definition = definition();
    let producer = producer();
    let incompatible = QualifiedSubject::new(
        &producer,
        SubjectKind::new(b"ix://agent-ix/commerce/type/Shipment".to_vec()).expect("non-empty kind"),
        SubjectIdentity::new(b"shipment:42".to_vec()).expect("non-empty identity"),
    );

    assert_eq!(
        RuntimeBindingSet::new(
            RUNTIME_BINDING_CONTRACT,
            definition.selection(),
            &definition,
            &producer,
            vec![RuntimeBinding::subject(SlotHandle::new(11), incompatible)],
        ),
        Err(RuntimeRefusal::IncompatibleSubjectKind(SlotHandle::new(11)))
    );
}

/// Relationships retain their independently selected declaration and complete
/// endpoint identities; neither a QSL relationship label nor a subject can
/// substitute for this QObs-owned value family.
#[test]
#[trace("TC-002", "FR-002-AC-1", "FR-002-AC-3")]
fn tc_002_runtime_relationships_are_not_subjects_or_stringly_slots() {
    let producer = producer();
    let definition = StaticBindingDefinition::new(
        STATIC_BINDING_CONTRACT,
        selection(),
        producer.key().clone(),
        3,
        vec![],
        vec![StaticSlot {
            handle: SlotHandle::new(17),
            compatibility: SlotCompatibility::Relationship {
                declaration_identity: ORDER_SHIPMENT.into(),
                source_kind: SubjectKind::new(ORDER_KIND.to_vec()).expect("non-empty kind"),
                target_kind: SubjectKind::new(SHIPMENT_KIND.to_vec()).expect("non-empty kind"),
            },
        }],
    )
    .expect("relationship slot is a complete static selection");
    let relationship = order_shipment(&producer);

    let bindings = RuntimeBindingSet::new(
        RUNTIME_BINDING_CONTRACT,
        definition.selection(),
        &definition,
        &producer,
        vec![RuntimeBinding::relationship(
            SlotHandle::new(17),
            relationship.clone(),
        )],
    )
    .expect("the exact selected relationship admits");
    assert_eq!(
        bindings.bindings()[0].relationship_value(),
        Some(&relationship)
    );

    let wrong_endpoint_definition = StaticBindingDefinition::new(
        STATIC_BINDING_CONTRACT,
        selection(),
        producer.key().clone(),
        3,
        vec![],
        vec![StaticSlot {
            handle: SlotHandle::new(17),
            compatibility: SlotCompatibility::Relationship {
                declaration_identity: ORDER_SHIPMENT.into(),
                source_kind: SubjectKind::new(ORDER_KIND.to_vec()).expect("non-empty kind"),
                target_kind: SubjectKind::new(ORDER_KIND.to_vec()).expect("non-empty kind"),
            },
        }],
    )
    .expect("the altered endpoint fact is structurally selectable");
    assert_eq!(
        RuntimeBindingSet::new(
            RUNTIME_BINDING_CONTRACT,
            wrong_endpoint_definition.selection(),
            &wrong_endpoint_definition,
            &producer,
            vec![RuntimeBinding::relationship(
                SlotHandle::new(17),
                relationship.clone(),
            )],
        ),
        Err(RuntimeRefusal::IncompatibleRelationship(SlotHandle::new(
            17
        )))
    );

    assert_eq!(
        RuntimeBindingSet::new(
            RUNTIME_BINDING_CONTRACT,
            definition.selection(),
            &definition,
            &producer,
            vec![RuntimeBinding::subject(
                SlotHandle::new(17),
                subject(&producer)
            )],
        ),
        Err(RuntimeRefusal::WrongValueKind(SlotHandle::new(17)))
    );
}

/// FR-262 replays preserve one complete relationship, while a different
/// relationship key cannot silently become a second value for the same slot.
#[test]
#[trace("TC-002", "FR-002-AC-1", "FR-002-AC-3", "FR-002-AC-4")]
fn tc_002_relationship_replay_is_idempotent_and_distinct_keys_are_not_collapsed() {
    let producer = producer();
    let relationship = order_shipment(&producer);
    let relationship_definition = |slots| {
        StaticBindingDefinition::new(
            STATIC_BINDING_CONTRACT,
            selection(),
            producer.key().clone(),
            3,
            vec![],
            slots,
        )
        .expect("complete relationship slots")
    };
    let compatibility = || SlotCompatibility::Relationship {
        declaration_identity: ORDER_SHIPMENT.into(),
        source_kind: SubjectKind::new(ORDER_KIND.to_vec()).expect("non-empty kind"),
        target_kind: SubjectKind::new(SHIPMENT_KIND.to_vec()).expect("non-empty kind"),
    };
    let one_slot = relationship_definition(vec![StaticSlot {
        handle: SlotHandle::new(17),
        compatibility: compatibility(),
    }]);

    let replay = RuntimeBindingSet::new(
        RUNTIME_BINDING_CONTRACT,
        one_slot.selection(),
        &one_slot,
        &producer,
        vec![
            RuntimeBinding::relationship(SlotHandle::new(17), relationship.clone()),
            RuntimeBinding::relationship(SlotHandle::new(17), relationship.clone()),
        ],
    )
    .expect("one exact relationship replay is idempotent");
    assert_eq!(replay.bindings().len(), 1);

    let conflicting_facts = order_shipment_with_identity(
        &producer,
        b"order:42->shipment:99",
        b"order:43",
        b"shipment:100",
    );
    assert_eq!(
        RuntimeBindingSet::new(
            RUNTIME_BINDING_CONTRACT,
            one_slot.selection(),
            &one_slot,
            &producer,
            vec![
                RuntimeBinding::relationship(SlotHandle::new(17), relationship.clone()),
                RuntimeBinding::relationship(SlotHandle::new(17), conflicting_facts),
            ],
        ),
        Err(RuntimeRefusal::RelationshipContradiction(SlotHandle::new(
            17
        )))
    );

    let distinct = order_shipment_with_identity(
        &producer,
        b"order:44->shipment:101",
        b"order:44",
        b"shipment:101",
    );
    assert_eq!(
        RuntimeBindingSet::new(
            RUNTIME_BINDING_CONTRACT,
            one_slot.selection(),
            &one_slot,
            &producer,
            vec![
                RuntimeBinding::relationship(SlotHandle::new(17), relationship.clone()),
                RuntimeBinding::relationship(SlotHandle::new(17), distinct.clone()),
            ],
        ),
        Err(RuntimeRefusal::AmbiguousRelationship(SlotHandle::new(17)))
    );

    let two_slots = relationship_definition(vec![
        StaticSlot {
            handle: SlotHandle::new(17),
            compatibility: compatibility(),
        },
        StaticSlot {
            handle: SlotHandle::new(18),
            compatibility: compatibility(),
        },
    ]);
    let distinct_slots = RuntimeBindingSet::new(
        RUNTIME_BINDING_CONTRACT,
        two_slots.selection(),
        &two_slots,
        &producer,
        vec![
            RuntimeBinding::relationship(SlotHandle::new(17), relationship),
            RuntimeBinding::relationship(SlotHandle::new(18), distinct),
        ],
    )
    .expect("two exact keys at two static slots remain distinct");
    assert_eq!(distinct_slots.bindings().len(), 2);
}

/// The full-assessment boundary accepts only an admitted `/2` protocol and
/// retains its static and runtime capabilities without retargeting either one.
#[test]
#[trace("TC-002", "FR-002-AC-3")]
fn tc_002_linked_assessment_refuses_every_context_coordinate_mutation() {
    let package = admitted_published_v2();
    let (declaration_handle, subject_slot) = protocol_coordinates(&package);
    let Body::Protocol { roles, .. } = &package.inherited().declarations
        [usize::try_from(declaration_handle).expect("published index fits")]
    .body
    else {
        unreachable!("selected declaration was found as a protocol")
    };
    let classifications = || {
        roles
            .iter()
            .enumerate()
            .map(|(index, _)| RoleClassification {
                role_handle: u32::try_from(index).expect("published role index fits"),
                internality: Internality::Internal,
            })
            .collect::<Vec<_>>()
    };
    let selected_producer = producer();
    let subject_slot_definition = |selection, declaration_handle, roles, slot| {
        StaticBindingDefinition::new(
            STATIC_BINDING_CONTRACT,
            selection,
            selected_producer.key().clone(),
            declaration_handle,
            roles,
            vec![StaticSlot {
                handle: SlotHandle::new(slot),
                compatibility: SlotCompatibility::Subject {
                    kind: SubjectKind::new(ORDER_KIND.to_vec()).expect("non-empty kind"),
                },
            }],
        )
        .expect("independently valid static binding capability")
    };
    let runtime_for = |definition: &StaticBindingDefinition, slot| {
        RuntimeBindingSet::new(
            RUNTIME_BINDING_CONTRACT,
            definition.selection(),
            definition,
            &selected_producer,
            vec![RuntimeBinding::subject(slot, subject(&selected_producer))],
        )
        .expect("runtime binding only validates its static capability")
    };

    let unknown_declaration =
        subject_slot_definition(selection(), u32::MAX, classifications(), subject_slot);
    let runtime = runtime_for(&unknown_declaration, SlotHandle::new(subject_slot));
    assert_eq!(
        context_refusal(LinkedAssessment::new(
            &package,
            &unknown_declaration,
            &runtime
        )),
        quire_protocol::model::v2::ContextRefusal::UnknownDeclaration
    );

    let non_protocol_handle = package
        .inherited()
        .declarations
        .iter()
        .enumerate()
        .find_map(|(index, declaration)| {
            (!matches!(declaration.body, Body::Protocol { .. }))
                .then(|| u32::try_from(index).expect("published declaration index fits"))
        })
        .expect("published package has a non-protocol declaration");
    let non_protocol =
        subject_slot_definition(selection(), non_protocol_handle, vec![], subject_slot);
    let runtime = runtime_for(&non_protocol, SlotHandle::new(subject_slot));
    assert_eq!(
        context_refusal(LinkedAssessment::new(&package, &non_protocol, &runtime)),
        quire_protocol::model::v2::ContextRefusal::NotProtocol
    );

    let base = subject_slot_definition(
        selection(),
        declaration_handle,
        classifications(),
        subject_slot,
    );
    let runtime = runtime_for(&base, SlotHandle::new(subject_slot));
    let retargeted = subject_slot_definition(
        StaticSelection {
            identity: "different-static-selection".into(),
            revision: "1".into(),
            digest: [1; 32],
        },
        declaration_handle,
        classifications(),
        subject_slot,
    );
    assert_eq!(
        context_refusal(LinkedAssessment::new(&package, &retargeted, &runtime)),
        quire_protocol::model::v2::ContextRefusal::RuntimeTargetMismatch
    );

    let unknown_role = subject_slot_definition(
        selection(),
        declaration_handle,
        vec![RoleClassification {
            role_handle: u32::MAX,
            internality: Internality::Internal,
        }],
        subject_slot,
    );
    let runtime = runtime_for(&unknown_role, SlotHandle::new(subject_slot));
    assert_eq!(
        context_refusal(LinkedAssessment::new(&package, &unknown_role, &runtime)),
        quire_protocol::model::v2::ContextRefusal::UnknownRole
    );

    let missing_role =
        subject_slot_definition(selection(), declaration_handle, vec![], subject_slot);
    let runtime = runtime_for(&missing_role, SlotHandle::new(subject_slot));
    assert_eq!(
        context_refusal(LinkedAssessment::new(&package, &missing_role, &runtime)),
        quire_protocol::model::v2::ContextRefusal::MissingRoleClassification
    );

    let unknown_slot =
        subject_slot_definition(selection(), declaration_handle, classifications(), u32::MAX);
    let runtime = runtime_for(&unknown_slot, SlotHandle::new(u32::MAX));
    assert_eq!(
        context_refusal(LinkedAssessment::new(&package, &unknown_slot, &runtime)),
        quire_protocol::model::v2::ContextRefusal::UnknownSlot
    );

    let wrong_family = StaticBindingDefinition::new(
        STATIC_BINDING_CONTRACT,
        selection(),
        selected_producer.key().clone(),
        declaration_handle,
        classifications(),
        vec![StaticSlot {
            handle: SlotHandle::new(subject_slot),
            compatibility: SlotCompatibility::Relationship {
                declaration_identity: ORDER_SHIPMENT.into(),
                source_kind: SubjectKind::new(ORDER_KIND.to_vec()).expect("non-empty kind"),
                target_kind: SubjectKind::new(SHIPMENT_KIND.to_vec()).expect("non-empty kind"),
            },
        }],
    )
    .expect("the opposite static family is independently valid");
    let relationship_runtime = RuntimeBindingSet::new(
        RUNTIME_BINDING_CONTRACT,
        wrong_family.selection(),
        &wrong_family,
        &selected_producer,
        vec![RuntimeBinding::relationship(
            SlotHandle::new(subject_slot),
            order_shipment(&selected_producer),
        )],
    )
    .expect("relationship satisfies the opposite static family");
    assert_eq!(
        context_refusal(LinkedAssessment::new(
            &package,
            &wrong_family,
            &relationship_runtime,
        )),
        quire_protocol::model::v2::ContextRefusal::WrongSlotFamily
    );
}

/// IT-002 SC-01 through SC-04's conformance half: the full-assessment boundary
/// accepts only an admitted `/2` protocol and retains its static and runtime
/// capabilities without retargeting either one.
///
/// The test crosses QObs's qualified subject and authority facts with QSL's
/// strict v2 request/result and `/3` mapping. It then proves the exact FR-300
/// binding, batch/incremental agreement, independently retained scope/global
/// closure axes, and adverse identity, occurrence, completeness, and execution
/// paths. Canonical-result assembly over that same bridge is exercised by the
/// paired IT-002 binding in `tc_006_engine_constructs_and_strict_reads_canonical_rows`.
#[test]
#[trace(
    "TC-002",
    "TC-004",
    "TC-005",
    "IT-002",
    "Task-010",
    "FR-002-AC-1",
    "FR-002-AC-3",
    "FR-004-AC-1",
    "FR-005-AC-1",
    "FR-029-AC-1",
    "FR-009-AC-2",
    "FR-009-AC-8"
)]
fn tc_002_linked_assessment_borrows_one_admitted_v2_protocol_context() {
    let package = admitted_published_v2();
    let (declaration_handle, subject_slot) = protocol_coordinates(&package);
    let Body::Protocol { roles, .. } = &package.inherited().declarations
        [usize::try_from(declaration_handle).expect("published index fits")]
    .body
    else {
        unreachable!("selected declaration was found as a protocol")
    };
    let definition = StaticBindingDefinition::new(
        STATIC_BINDING_CONTRACT,
        selection(),
        producer().key().clone(),
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
                kind: SubjectKind::new(ORDER_KIND.to_vec()).expect("non-empty kind"),
            },
        }],
    )
    .expect("selected static capability admits");
    let producer = producer();
    let activation_subject = subject(&producer);
    let runtime = RuntimeBindingSet::new(
        RUNTIME_BINDING_CONTRACT,
        definition.selection(),
        &definition,
        &producer,
        vec![RuntimeBinding::subject(
            SlotHandle::new(subject_slot),
            activation_subject.clone(),
        )],
    )
    .expect("selected runtime capability admits");
    let context = LinkedAssessment::new(&package, &definition, &runtime)
        .expect("same admitted package and selected capabilities link");
    assert_eq!(context.package().digest(), package.digest());
    assert_eq!(context.definition().selection(), definition.selection());
    assert_eq!(context.runtime().target(), definition.selection());
    let identity = context
        .definition_identity()
        .expect("the admitted static selection has an FR-288 identity");

    let temporal_subject = activated_temporal_subject(&package);
    let request = activated_request(&package, activation_subject.identity().as_bytes());
    let mappings = admitted_published_v3(&package);
    let result_document = temporal_v2::evaluate(
        &request,
        temporal_v2::Relation::Original,
        native_temporal::Limits::default(),
    )
    .into_result()
    .expect("strict temporal result evaluates");
    let result_cell = OnceCell::new();
    let runtime_selection = select_runtime_bindings(context.runtime(), 500_000)
        .expect("fixture runtime selection is bounded");
    let subject_selection =
        select_subject(&activation_subject, 500_000).expect("fixture subject selection is bounded");
    assert!(
        select_runtime_bindings(context.runtime(), runtime_selection.framed_bytes() - 1,).is_err()
    );
    assert!(select_subject(&activation_subject, subject_selection.framed_bytes() - 1,).is_err());
    let producer_request = ProducerResolutionRequest::for_activated_assessment(
        package.digest(),
        definition.selection().digest,
        &[Claim::Conformance],
        runtime_selection,
        mappings.digest(),
        subject_selection,
        temporal_subject.document().digest(),
        request.document().digest(),
        result_document.digest(),
    )
    .expect("the closed claim selection is representable");
    let producer_reads = Cell::new(0usize);
    let mut unresolved_entries = producer_manifest().entries().to_vec();
    unresolved_entries[0] = ManifestEntry::unresolved_interface(
        ProducerContract::ALL[0].key(),
        b"owner acceptance withdrawn",
    );
    let unresolved = with_activated_after_resolving(
        &producer_request,
        &VersionLockManifest::new(unresolved_entries),
        producer_budget(),
        SystemMonotonicClock,
        |permit| {
            producer_reads.set(producer_reads.get() + 1);
            let result = result_cell.get_or_init(|| {
                temporal_v2::read_result(
                    result_document.bytes(),
                    &request,
                    temporal_v2::Relation::Original,
                    native_temporal::Limits::default(),
                )
                .into_result()
                .expect("strict temporal result reads only after resolution")
            });
            ActivatedAssessmentInputs::new(
                permit,
                context,
                vec![Claim::Conformance],
                &mappings,
                &activation_subject,
                &temporal_subject,
                &request,
                result,
            )
        },
        |_| (),
    );
    assert!(matches!(unresolved, ProducerReadiness::Unexecutable(_)));
    assert_eq!(producer_reads.get(), 0, "owner inputs are not read");
    assert!(
        result_cell.get().is_none(),
        "the strict QSL result reader is not invoked before resolution"
    );

    let mismatched_request = ProducerResolutionRequest::for_activated_assessment(
        package.digest(),
        definition.selection().digest,
        &[Claim::Conformance],
        runtime_selection,
        mappings.digest(),
        subject_selection,
        temporal_subject.document().digest(),
        ByteDigest::of(b"another temporal request"),
        result_document.digest(),
    )
    .expect("the closed claim selection is representable");
    let mismatched_resolution = with_activated_after_resolving(
        &mismatched_request,
        &producer_manifest(),
        producer_budget(),
        SystemMonotonicClock,
        |permit| {
            producer_reads.set(producer_reads.get() + 1);
            let result = result_cell.get_or_init(|| {
                temporal_v2::read_result(
                    result_document.bytes(),
                    &request,
                    temporal_v2::Relation::Original,
                    native_temporal::Limits::default(),
                )
                .into_result()
                .expect("strict temporal result reads only after resolution")
            });
            ActivatedAssessmentInputs::new(
                permit,
                context,
                vec![Claim::Conformance],
                &mappings,
                &activation_subject,
                &temporal_subject,
                &request,
                result,
            )
        },
        |_| (),
    );
    assert!(matches!(
        mismatched_resolution,
        ProducerReadiness::Executable(OperationOutcome::Refused(refusal))
            if refusal.code().as_str() == "version-lock.capability.request-mismatch"
    ));
    assert_eq!(
        producer_reads.get(),
        1,
        "one resolved owner load was attempted"
    );

    let completed = with_activated_after_resolving(
        &producer_request,
        &producer_manifest(),
        producer_budget(),
        SystemMonotonicClock,
        |permit| {
            producer_reads.set(producer_reads.get() + 1);
            let result = result_cell.get_or_init(|| {
                temporal_v2::read_result(
                    result_document.bytes(),
                    &request,
                    temporal_v2::Relation::Original,
                    native_temporal::Limits::default(),
                )
                .into_result()
                .expect("strict temporal result reads only after resolution")
            });
            ActivatedAssessmentInputs::new(
                permit,
                context,
                vec![Claim::Conformance],
                &mappings,
                &activation_subject,
                &temporal_subject,
                &request,
                result,
            )
        },
        |activated| {
            let result = result_cell
                .get()
                .expect("one post-resolution strict read retains the validated result");
            assert_eq!(producer_reads.get(), 2, "resolved inputs load exactly once");
            assert_eq!(
                activated.producer_resolution().request().as_bytes(),
                producer_request.as_bytes(),
            );
            let retargeted_requests = [
                (
                    "claim selection",
                    ProducerResolutionRequest::for_activated_assessment(
                        package.digest(),
                        definition.selection().digest,
                        &[Claim::Monitorability],
                        runtime_selection,
                        mappings.digest(),
                        subject_selection,
                        temporal_subject.document().digest(),
                        request.document().digest(),
                        result.document().digest(),
                    )
                    .expect("the closed claim selection is representable"),
                ),
                (
                    "activation mapping",
                    ProducerResolutionRequest::for_activated_assessment(
                        package.digest(),
                        definition.selection().digest,
                        &[Claim::Conformance],
                        runtime_selection,
                        ByteDigest::of(b"another activation mapping"),
                        subject_selection,
                        temporal_subject.document().digest(),
                        request.document().digest(),
                        result.document().digest(),
                    )
                    .expect("the closed claim selection is representable"),
                ),
                (
                    "QObs subject",
                    ProducerResolutionRequest::for_activated_assessment(
                        package.digest(),
                        definition.selection().digest,
                        &[Claim::Conformance],
                        runtime_selection,
                        mappings.digest(),
                        ProducerValueSelection::new([0x51; 32], subject_selection.framed_bytes()),
                        temporal_subject.document().digest(),
                        request.document().digest(),
                        result.document().digest(),
                    )
                    .expect("the closed claim selection is representable"),
                ),
                (
                    "temporal subject",
                    ProducerResolutionRequest::for_activated_assessment(
                        package.digest(),
                        definition.selection().digest,
                        &[Claim::Conformance],
                        runtime_selection,
                        mappings.digest(),
                        subject_selection,
                        ByteDigest::of(b"another temporal subject"),
                        request.document().digest(),
                        result.document().digest(),
                    )
                    .expect("the closed claim selection is representable"),
                ),
                (
                    "temporal result",
                    ProducerResolutionRequest::for_activated_assessment(
                        package.digest(),
                        definition.selection().digest,
                        &[Claim::Conformance],
                        runtime_selection,
                        mappings.digest(),
                        subject_selection,
                        temporal_subject.document().digest(),
                        request.document().digest(),
                        ByteDigest::of(b"another temporal result"),
                    )
                    .expect("the closed claim selection is representable"),
                ),
            ];
            for (axis, retargeted) in retargeted_requests {
                let outcome = with_activated_after_resolving(
                    &retargeted,
                    &producer_manifest(),
                    producer_budget(),
                    SystemMonotonicClock,
                    |permit| {
                        producer_reads.set(producer_reads.get() + 1);
                        ActivatedAssessmentInputs::new(
                            permit,
                            context,
                            vec![Claim::Conformance],
                            &mappings,
                            &activation_subject,
                            &temporal_subject,
                            &request,
                            result,
                        )
                    },
                    |_| (),
                );
                assert!(
                    matches!(
                        outcome,
                        ProducerReadiness::Executable(OperationOutcome::Refused(refusal))
                            if refusal.code().as_str() == "version-lock.capability.request-mismatch"
                    ),
                    "a resolved capability cannot be retargeted on its {axis} axis",
                );
            }
            assert_eq!(
                activated
                    .activation()
                    .expect("event-triggered engine retains its activation binding")
                    .subject(),
                &activation_subject
            );
            assert_eq!(
                activated
                    .activation()
                    .expect("event-triggered engine retains its activation binding")
                    .semantic_trigger(),
                activation_subject.identity().as_bytes()
            );
            let foreign_temporal_mappings = admitted_v3_with_foreign_temporal_table(&package);
            assert_eq!(
        activated
            .conformance()
            .bind_activation(
                activated.linked(),
                &foreign_temporal_mappings,
                &activation_subject,
                &temporal_subject,
                &request,
                result,
            )
            .expect_err(
                "a separately admitted v3 temporal table cannot extend this exact v2 package",
            ),
        quire_protocol::conformance::KernelRefusal::ForeignActivationPackage
    );
            let foreign_temporal_package = admitted_v2_with_foreign_temporal_table();
            let foreign_temporal_subject = activated_temporal_subject(&foreign_temporal_package);
            let foreign_temporal_request = activated_request(
                &foreign_temporal_package,
                activation_subject.identity().as_bytes(),
            );
            let foreign_temporal_result_document = temporal_v2::evaluate(
                &foreign_temporal_request,
                temporal_v2::Relation::Original,
                native_temporal::Limits::default(),
            )
            .into_result()
            .expect("foreign strict temporal result evaluates");
            let foreign_temporal_result = temporal_v2::read_result(
                foreign_temporal_result_document.bytes(),
                &foreign_temporal_request,
                temporal_v2::Relation::Original,
                native_temporal::Limits::default(),
            )
            .into_result()
            .expect("foreign strict temporal result reads");
            assert_ne!(foreign_temporal_package.digest(), package.digest());
            assert_eq!(
                foreign_temporal_request.declaration(),
                request.declaration()
            );
            assert_eq!(
                foreign_temporal_request.semantic_trigger(),
                request.semantic_trigger()
            );
            assert_eq!(
        activated
            .conformance()
            .bind_activation(
                activated.linked(),
                &mappings,
                &activation_subject,
                &foreign_temporal_subject,
                &foreign_temporal_request,
                &foreign_temporal_result,
            )
            .expect_err(
                "the linked mappings cannot authorize a request from another strict package",
            ),
        quire_protocol::conformance::KernelRefusal::InvalidActivationTemporal
    );
            let mismatched_subject = QualifiedSubject::new(
                &producer,
                SubjectKind::new(ORDER_KIND.to_vec()).expect("non-empty kind"),
                SubjectIdentity::new(b"order:43".to_vec()).expect("non-empty identity"),
            );
            let retargeted_runtime = RuntimeBindingSet::new(
                RUNTIME_BINDING_CONTRACT,
                definition.selection(),
                &definition,
                &producer,
                runtime
                    .bindings()
                    .iter()
                    .map(|binding| {
                        if binding.slot() == SlotHandle::new(subject_slot) {
                            RuntimeBinding::subject(binding.slot(), mismatched_subject.clone())
                        } else {
                            binding.clone()
                        }
                    })
                    .collect(),
            )
            .expect("one same-shape QObs runtime substitution remains independently valid");
            let retargeted_context =
                LinkedAssessment::new(&package, &definition, &retargeted_runtime)
                    .expect("the substituted runtime still links to the same static selection");
            assert!(matches!(
                with_activated_after_resolving(
                    &producer_request,
                    &producer_manifest(),
                    producer_budget(),
                    SystemMonotonicClock,
                    |permit| ActivatedAssessmentInputs::new(
                        permit,
                        retargeted_context,
                        vec![Claim::Conformance],
                        &mappings,
                        &activation_subject,
                        &temporal_subject,
                        &request,
                        result,
                    ),
                    |_| (),
                ),
                ProducerReadiness::Executable(OperationOutcome::Refused(refusal))
                    if refusal.code().as_str() == "version-lock.capability.request-mismatch"
            ));
            let ProducerReadiness::Executable(OperationOutcome::Refused(refusal)) =
                with_activated_after_resolving(
                    &producer_request,
                    &producer_manifest(),
                    producer_budget(),
                    SystemMonotonicClock,
                    |permit| {
                        ActivatedAssessmentInputs::new(
                            permit,
                            context,
                            vec![Claim::Conformance],
                            &mappings,
                            &mismatched_subject,
                            &temporal_subject,
                            &request,
                            result,
                        )
                    },
                    |_| (),
                )
            else {
                panic!("different QObs subject cannot reuse a strict temporal request")
            };
            assert_eq!(
                refusal.code().as_str(),
                "version-lock.capability.request-mismatch"
            );

            let engine = activated;
            assert_eq!(engine.linked().identity(), &identity);
            assert_eq!(engine.conformance().definition(), &identity);
            match engine.conformance().bind_activation(
                engine.linked(),
                &mappings,
                &mismatched_subject,
                &temporal_subject,
                &request,
                result,
            ) {
                Err(refusal) => assert_eq!(
                    refusal,
                    quire_protocol::conformance::KernelRefusal::ActivationSubjectMismatch
                ),
                Ok(_) => panic!("a strict request cannot be substituted onto another QObs subject"),
            }
            let activation = engine
                .conformance()
                .bind_activation(
                    engine.linked(),
                    &mappings,
                    &activation_subject,
                    &temporal_subject,
                    &request,
                    result,
                )
                .expect("only the strict /3 mapping joins the selected event activation");
            assert_eq!(activation.definition(), &identity);
            assert_eq!(activation.subject(), &activation_subject);
            assert_eq!(
                activation.semantic_trigger(),
                activation_subject.identity().as_bytes()
            );
            assert_eq!(activation.evaluation_anchor(), "origin");
            assert_eq!(activation.control().declaration, declaration_handle);
            assert!(matches!(
                activation.activation(),
                native_temporal::result::ActivationState::Active
            ));
            let mut activations = quire_protocol::conformance::ActivationRegistry::default();
            activations
                .admit(activation.clone())
                .expect("first strict activation retains");
            activations
                .admit(activation.clone())
                .expect("an exact strict activation replay is idempotent");
            assert_eq!(activations.bindings().len(), 1);
            let conflicting_request = activated_request_at(
                &package,
                activation_subject.identity().as_bytes(),
                "other-origin",
            );
            let conflicting_result_document = temporal_v2::evaluate(
                &conflicting_request,
                temporal_v2::Relation::Original,
                native_temporal::Limits::default(),
            )
            .into_result()
            .expect("changed strict temporal request evaluates");
            let conflicting_result = temporal_v2::read_result(
                conflicting_result_document.bytes(),
                &conflicting_request,
                temporal_v2::Relation::Original,
                native_temporal::Limits::default(),
            )
            .into_result()
            .expect("changed strict temporal result reads");
            assert!(
                temporal_v2::evaluate(
                    &request,
                    temporal_v2::Relation::Superseding(result),
                    native_temporal::Limits::default(),
                )
                .into_result()
                .is_err(),
                "QSL refuses a second relation for the same request identity"
            );
            let correction_document = temporal_v2::evaluate(
                &conflicting_request,
                temporal_v2::Relation::Superseding(result),
                native_temporal::Limits::default(),
            )
            .into_result()
            .expect("same-trigger changed request admits a superseding result");
            let correction_result = temporal_v2::read_result(
                correction_document.bytes(),
                &conflicting_request,
                temporal_v2::Relation::Superseding(result),
                native_temporal::Limits::default(),
            )
            .into_result()
            .expect("strict superseding result reads against its changed request");
            assert_eq!(
                engine
                    .conformance()
                    .bind_activation(
                        engine.linked(),
                        &mappings,
                        &activation_subject,
                        &temporal_subject,
                        &request,
                        &conflicting_result,
                    )
                    .expect_err(
                        "a temporal result cannot be substituted for another strict request"
                    ),
                quire_protocol::conformance::KernelRefusal::ActivationResultMismatch
            );
            assert_eq!(
                activations
                    .admit(
                        engine
                            .conformance()
                            .bind_activation(
                                engine.linked(),
                                &mappings,
                                &activation_subject,
                                &temporal_subject,
                                &conflicting_request,
                                &conflicting_result,
                            )
                            .expect("changed checked anchor still forms an owner-valid binding"),
                    )
                    .expect_err("a changed checked anchor cannot overwrite the same FR-300 key"),
                quire_protocol::conformance::KernelRefusal::ConflictingActivation
            );
            assert_eq!(activations.bindings().len(), 1);
            let occurrence_schema = occurrence_key_schema(
                &package,
                declaration_handle,
                quire_spec_language::protocol_artifact::Limits::default(),
            )
            .into_result()
            .expect("selected protocol produces a strict occurrence-key schema");
            let node = occurrence_schema
                .nodes()
                .get(usize::try_from(activation.control().index).expect("control index fits"))
                .expect("selected activation control has an occurrence schema");
            assert!(
                node.repeats().is_empty(),
                "published selected activation control has no repeat dimensions"
            );
            let occurrence = occurrence_schema
                .bind(
                    WorkflowInstanceIdentity::new("workflow:one")
                        .expect("nonempty workflow identity"),
                    activation.control(),
                    node.role().clone(),
                    &[],
                )
                .expect("QSL validates the selected event occurrence");
            activation
                .require_occurrence(&occurrence)
                .expect("the activation cannot be retargeted away from its mapped control");
            let derived_occurrence = engine
                .conformance()
                .activate_occurrence(&activation, occurrence.clone())
                .expect("a mapped activation and QSL occurrence form one derived obligation");
            let decision = observation_owner::scope_views("42");
            let surrounding = observation_owner::scope_views("activation-derived-surrounding");
            let global = closed_global_conformance();
            let fact = engine
                .conformance()
                .activation_fact(derived_occurrence.clone(), Some(&decision.observation));
            let corrected_activation = engine
                .conformance()
                .bind_activation(
                    engine.linked(),
                    &mappings,
                    &activation_subject,
                    &temporal_subject,
                    &conflicting_request,
                    &correction_result,
                )
                .expect("the owner admits a same-trigger superseding activation");
            let corrected_occurrence = occurrence_schema
                .bind(
                    WorkflowInstanceIdentity::new("workflow:corrected")
                        .expect("nonempty corrected workflow identity"),
                    corrected_activation.control(),
                    node.role().clone(),
                    &[],
                )
                .expect("the corrected activation occurrence admits");
            let corrected_occurrence = engine
                .conformance()
                .activate_occurrence(&corrected_activation, corrected_occurrence)
                .expect("the corrected activation produces an occurrence");
            let corrected_fact = engine
                .conformance()
                .activation_fact(corrected_occurrence, Some(&decision.observation));
            let batch = engine
                .conformance()
                .assess_activated(
                    std::slice::from_ref(&fact),
                    closed_axes(&decision, &surrounding),
                    &decision.completeness_complete,
                    &global,
                    AssessmentExecution::Completed,
                    ConformanceLimits::default(),
                )
                .expect("the activation-derived batch frontend has no caller-built model");
            let mut incremental = ActivatedIncremental::new(
                engine.conformance(),
                std::slice::from_ref(&derived_occurrence),
            )
            .expect("the activation-derived incremental frontend uses the same derived model");
            let pushed = incremental
                .push(
                    &fact,
                    closed_axes(&decision, &surrounding),
                    &decision.completeness_complete,
                    &global,
                    AssessmentExecution::Completed,
                    ConformanceLimits::default(),
                )
                .expect("the activation-derived incremental frontend admits the same QObs fact")
                .clone();
            let incremental = incremental
                .finish(
                    closed_axes(&decision, &surrounding),
                    &decision.completeness_complete,
                    &global,
                    AssessmentExecution::Completed,
                    ConformanceLimits::default(),
                )
                .expect("the activation-derived path finishes with the admitted QObs fact")
                .clone();
            assert_eq!(
                batch, incremental,
                "activation-derived batch and incremental reports agree exactly"
            );
            assert_eq!(
                pushed, incremental,
                "a closed admitted prefix stays settled"
            );
            assert_eq!(batch.global, GlobalDisposition::Conforms);
            assert_eq!(
                batch.obligations[0].disposition,
                ObligationDisposition::Conforms,
                "the QObs view is correlated only through its admitted activation occurrence"
            );
            let component =
                |identity: &[u8]| Component::retained(identity.to_vec(), identity.to_vec());
            let dispatch_request = ClaimRequest {
                identity: Box::from(*b"claim:conformance"),
                claim: Claim::Conformance,
                strength: Box::from(*b"strength:finite"),
                premises: PremiseSet {
                    admitted_fragment: component(b"fragment:admitted"),
                    backend: BackendSelection::retained(
                        b"qprotocol:conformance".to_vec(),
                        b"v1".to_vec(),
                    ),
                    assumptions: vec![Assumption {
                        component: component(b"assumption:one"),
                        state: Some(AssumptionState::Met),
                        dependencies: vec![],
                    }],
                    communication: vec![component(b"communication:one")],
                    visibility: vec![component(b"visibility:one")],
                    environment: vec![component(b"environment:one")],
                    sources: vec![component(b"source:one")],
                    derived_artifact: None,
                },
                correspondence: None,
            };
            let OperationOutcome::Completed(dispatched) = engine.dispatch_activated(
                std::slice::from_ref(&dispatch_request),
                std::slice::from_ref(&fact),
                closed_axes(&decision, &surrounding),
                &decision.completeness_complete,
                &global,
                AssessmentExecution::Completed,
                ConformanceLimits::default(),
                DispatchLimits::default(),
                Budget::nothing_granted(),
            ) else {
                panic!("dispatch consumes only the engine's activation-derived report")
            };
            assert_eq!(dispatched.dispositions().len(), 1);
            assert_eq!(
                dispatched.dispositions()[0].decision,
                Some(Decision::Established),
                "the claim result comes from the actual activation-derived conformance report"
            );
            assert!(matches!(
                engine.dispatch_activated(
                    std::slice::from_ref(&dispatch_request),
                    std::slice::from_ref(&corrected_fact),
                    closed_axes(&decision, &surrounding),
                    &decision.completeness_complete,
                    &global,
                    AssessmentExecution::Completed,
                    ConformanceLimits::default(),
                    DispatchLimits::default(),
                    Budget::nothing_granted(),
                ),
                OperationOutcome::Refused(refusal)
                    if refusal.code().as_str() == "assessment-activation.fact-mismatch"
            ));
            let unexpected_fact = engine
                .conformance()
                .activation_fact(derived_occurrence.clone(), Some(&surrounding.observation));
            let unexpected = engine
                .conformance()
                .assess_activated(
                    std::slice::from_ref(&unexpected_fact),
                    closed_axes(&decision, &surrounding),
                    &decision.completeness_complete,
                    &global,
                    AssessmentExecution::Completed,
                    ConformanceLimits::default(),
                )
                .expect("a mismatched admitted QObs view is reported, not repaired");
            assert_eq!(unexpected.global, GlobalDisposition::Violates);
            assert_eq!(
                unexpected.obligations[0].disposition,
                ObligationDisposition::Violates
            );
            assert_eq!(unexpected.adverse.len(), 1);
            assert_eq!(unexpected.adverse[0].kind, AdverseKind::Unexpected);
            for (state, expected) in [
                (GlobalConformanceClosure::Open, GlobalDisposition::Pending),
                (
                    GlobalConformanceClosure::Incomplete,
                    GlobalDisposition::Pending,
                ),
                (
                    GlobalConformanceClosure::Contradicted,
                    GlobalDisposition::Violates,
                ),
            ] {
                let report = engine
                    .conformance()
                    .assess_activated(
                        std::slice::from_ref(&fact),
                        closed_axes(&decision, &surrounding),
                        &decision.completeness_complete,
                        &global_conformance(state),
                        AssessmentExecution::Completed,
                        ConformanceLimits::default(),
                    )
                    .expect("each required global-closure state remains independently observable");
                assert_eq!(report.global, expected);
            }
            assert!(matches!(
                engine.conformance().assess_activated(
                    std::slice::from_ref(&fact),
                    closed_axes(&decision, &surrounding),
                    &decision.completeness_complete,
                    &global_conformance(GlobalConformanceClosure::NotRequired),
                    AssessmentExecution::Completed,
                    ConformanceLimits::default(),
                ),
                Err(ActivatedAssessmentError::Conformance(refusal))
                    if refusal.code() == ErrorCode::InvalidPremises
            ));
            for axes in [
                AxisInputs {
                    decision_scope_progress: &decision.progress_open,
                    decision_scope_closure: &decision.closure_closed,
                    surrounding_execution_progress: &surrounding.progress_closed,
                    surrounding_execution_closure: &surrounding.closure_closed,
                },
                AxisInputs {
                    decision_scope_progress: &decision.progress_closed,
                    decision_scope_closure: &decision.closure_open,
                    surrounding_execution_progress: &surrounding.progress_closed,
                    surrounding_execution_closure: &surrounding.closure_closed,
                },
                AxisInputs {
                    decision_scope_progress: &decision.progress_closed,
                    decision_scope_closure: &decision.closure_closed,
                    surrounding_execution_progress: &surrounding.progress_open,
                    surrounding_execution_closure: &surrounding.closure_closed,
                },
                AxisInputs {
                    decision_scope_progress: &decision.progress_closed,
                    decision_scope_closure: &decision.closure_closed,
                    surrounding_execution_progress: &surrounding.progress_closed,
                    surrounding_execution_closure: &surrounding.closure_open,
                },
            ] {
                let report = engine
                    .conformance()
                    .assess_activated(
                        std::slice::from_ref(&fact),
                        axes,
                        &decision.completeness_complete,
                        &global,
                        AssessmentExecution::Completed,
                        ConformanceLimits::default(),
                    )
                    .expect("each open scope axis remains independently observable");
                assert_eq!(report.global, GlobalDisposition::Pending);
            }
            let incomplete = engine
                .conformance()
                .assess_activated(
                    std::slice::from_ref(&fact),
                    closed_axes(&decision, &surrounding),
                    &decision.completeness_incomplete,
                    &global,
                    AssessmentExecution::Completed,
                    ConformanceLimits::default(),
                )
                .expect("incomplete QObs authority remains independently observable");
            assert_eq!(incomplete.global, GlobalDisposition::Unavailable);
            for execution in [
                AssessmentExecution::Refused,
                AssessmentExecution::Unsupported,
                AssessmentExecution::ResourceIncomplete,
                AssessmentExecution::Failed,
            ] {
                let report = engine
                    .conformance()
                    .assess_activated(
                        std::slice::from_ref(&fact),
                        closed_axes(&decision, &surrounding),
                        &decision.completeness_complete,
                        &global,
                        execution,
                        ConformanceLimits::default(),
                    )
                    .expect("each non-completed execution remains independently observable");
                assert_eq!(report.global, GlobalDisposition::Unavailable);
                assert_eq!(
                    report.obligations[0].disposition,
                    ObligationDisposition::Unavailable
                );
            }
            let other = occurrence_schema
                .nodes()
                .iter()
                .find(|candidate| {
                    candidate.node() != activation.control() && candidate.repeats().is_empty()
                })
                .expect("published protocol has another non-repeating control occurrence");
            let other_occurrence = occurrence_schema
                .bind(
                    WorkflowInstanceIdentity::new("workflow:one")
                        .expect("nonempty workflow identity"),
                    other.node(),
                    other.role().clone(),
                    &[],
                )
                .expect("QSL validates a distinct control occurrence");
            assert_eq!(
                activation.require_occurrence(&other_occurrence),
                Err(quire_protocol::conformance::KernelRefusal::ActivationOccurrenceMismatch)
            );

            let different_runtime = RuntimeBindingSet::new(
                RUNTIME_BINDING_CONTRACT,
                definition.selection(),
                &definition,
                &producer,
                vec![RuntimeBinding::subject(
                    SlotHandle::new(subject_slot),
                    QualifiedSubject::new(
                        &producer,
                        SubjectKind::new(ORDER_KIND.to_vec()).expect("non-empty kind"),
                        SubjectIdentity::new(b"order:43".to_vec()).expect("non-empty identity"),
                    ),
                )],
            )
            .expect("another selected runtime subject admits");
            let different_runtime_context =
                LinkedAssessment::new(&package, &definition, &different_runtime)
                    .expect("a changed runtime subject cannot retarget the static context");
            assert_eq!(
                different_runtime_context
                    .definition_identity()
                    .expect("runtime values are outside the FR-288 identity"),
                identity,
                "concrete runtime subjects do not independently change FR-288"
            );

            let mut registry = DefinitionRegistry::default();
            registry
                .register(&context)
                .expect("first static definition registers");
            registry
                .register(&context)
                .expect("exact static definition replay is idempotent");
            registry
                .register(&different_runtime_context)
                .expect("a changed concrete runtime subject is the same static definition");
            assert_eq!(registry.len(), 1);
            let linked = link_v2(context, &mut registry)
                .expect("validated context links through the static definition registry");
            assert_eq!(linked.identity().declaration_handle(), declaration_handle);
            assert!(matches!(linked.declaration().body, Body::Protocol { .. }));
            let Body::Protocol {
                temporal_requirements,
                ..
            } = &linked.declaration().body
            else {
                unreachable!("the linked declaration was checked as a protocol");
            };
            assert_eq!(
                linked.identity().temporal_bindings().len(),
                temporal_requirements.len(),
                "every protocol-selected temporal declaration remains in FR-288"
            );
            let closure = linked.identity().closure();
            assert!(
        !closure.definitions().is_empty()
            && !closure.dependencies().is_empty()
            && !closure.models().is_empty()
            && !closure.types().is_empty(),
        "FR-288 retains the declaration's typed definition, dependency, model, and type closure"
    );
            assert_eq!(
                closure.profile().index(),
                linked.declaration().profile,
                "the selected profile remains source-qualified rather than name-derived"
            );
            assert_eq!(
                linked.compensations().len(),
                2,
                "the linked graph retains every admitted compensation definition"
            );
            assert_eq!(
                linked.compensations()[0].handle(),
                0,
                "compensation table coordinates remain distinct from runtime registrations"
            );
            assert_eq!(
                linked.compensations()[0].identity().definition(),
                linked.identity(),
                "a compensation key retains its full FR-288 definition selection"
            );
            assert_ne!(
                linked.compensations()[0].identity(),
                linked.compensations()[1].identity(),
                "declaration-local compensation handles remain distinct within one definition"
            );
            let Body::Protocol { channels, .. } = &linked.declaration().body else {
                unreachable!("the linked declaration was checked as a protocol");
            };
            assert_eq!(
                linked.channels()[0].definition(),
                &channels[0],
                "the linked channel retains every owner-declared channel premise"
            );
            let Body::Protocol { causal_edges, .. } = &linked.declaration().body else {
                unreachable!("the linked declaration was checked as a protocol");
            };
            assert_eq!(
                linked.causal_edges(),
                causal_edges,
                "the linked graph retains QSL's exact causal edges without deriving arrival order"
            );
            let Body::Protocol { controls, .. } = &linked.declaration().body else {
                unreachable!("the linked declaration was checked as a protocol");
            };
            let (parallel_index, parallel_children) = controls
                .iter()
                .enumerate()
                .find_map(|(index, control)| match &control.operation {
                    ControlOperation::Parallel { branches, .. } => Some((
                        u32::try_from(index).expect("published control index fits"),
                        branches
                            .iter()
                            .map(|branch| branch.body.index)
                            .collect::<BTreeSet<_>>(),
                    )),
                    ControlOperation::Sequence { .. }
                    | ControlOperation::Choice { .. }
                    | ControlOperation::Repeat { .. }
                    | ControlOperation::Await { .. }
                    | ControlOperation::Event { .. }
                    | ControlOperation::Check { .. }
                    | ControlOperation::Commit { .. } => None,
                })
                .expect("published handoff has a parallel control");
            let parallel_nodes = parallel_children
                .iter()
                .copied()
                .chain([parallel_index])
                .collect::<BTreeSet<_>>();
            let parallel_edges = linked
                .causal_edges()
                .iter()
                .filter(|edge| {
                    edge.from.node.declaration == declaration_handle
                        && edge.to.node.declaration == declaration_handle
                        && parallel_nodes.contains(&edge.from.node.index)
                        && parallel_nodes.contains(&edge.to.node.index)
                })
                .collect::<Vec<_>>();
            let orders = parallel_orders(&parallel_edges);
            assert!(
                orders.len() > 1,
                "the owner-declared parallel subgraph admits more than one legal interleaving"
            );
            for order in &orders {
                for edge in &parallel_edges {
                    let from = causal_endpoint(edge, true);
                    let to = causal_endpoint(edge, false);
                    let from_index = order
                        .iter()
                        .position(|endpoint| *endpoint == from)
                        .expect("every retained edge source is ordered");
                    let to_index = order
                        .iter()
                        .position(|endpoint| *endpoint == to)
                        .expect("every retained edge target is ordered");
                    assert!(
                        from_index < to_index,
                        "every legal interleaving preserves the owner-declared edge"
                    );
                }
            }
            assert!(
        controls.iter().any(|control| matches!(
            control.operation,
            ControlOperation::Choice { .. }
        )),
        "a choice remains an owner-declared alternative and is never passed to the parallel enumerator"
    );
            assert_eq!(
                linked.fifo_partition(ChannelHandle::new(0), b"owner-evaluated-key"),
                Err(FifoPartitionRefusal::NotFifo(ChannelHandle::new(0))),
                "an admitted unordered channel cannot be silently partitioned as FIFO"
            );
            assert_eq!(
                linked.fifo_partition(ChannelHandle::new(u32::MAX), b"owner-evaluated-key"),
                Err(FifoPartitionRefusal::UnknownChannel(ChannelHandle::new(
                    u32::MAX
                ))),
                "an unselected channel coordinate cannot be repaired into an admitted FIFO channel"
            );

            let mut contradictory_roles = roles
                .iter()
                .enumerate()
                .map(|(index, _)| RoleClassification {
                    role_handle: u32::try_from(index).expect("published role index fits"),
                    internality: Internality::Internal,
                })
                .collect::<Vec<_>>();
            contradictory_roles[0].internality = Internality::External;
            let contradictory_definition = StaticBindingDefinition::new(
                STATIC_BINDING_CONTRACT,
                selection(),
                producer.key().clone(),
                declaration_handle,
                contradictory_roles.clone(),
                vec![StaticSlot {
                    handle: SlotHandle::new(subject_slot),
                    compatibility: SlotCompatibility::Subject {
                        kind: SubjectKind::new(ORDER_KIND.to_vec()).expect("non-empty kind"),
                    },
                }],
            )
            .expect("different static content remains an individually valid selection");
            let contradictory_runtime = RuntimeBindingSet::new(
                RUNTIME_BINDING_CONTRACT,
                contradictory_definition.selection(),
                &contradictory_definition,
                &producer,
                vec![RuntimeBinding::subject(
                    SlotHandle::new(subject_slot),
                    subject(&producer),
                )],
            )
            .expect("same runtime value satisfies the changed static selection");
            let contradictory_context =
                LinkedAssessment::new(&package, &contradictory_definition, &contradictory_runtime)
                    .expect("same identity with different content reaches the registry");
            assert_eq!(
                registry.register(&contradictory_context),
                Err(RegistryRefusal::ContentContradiction)
            );

            let revised_definition = StaticBindingDefinition::new(
                STATIC_BINDING_CONTRACT,
                StaticSelection {
                    identity: selection().identity,
                    revision: "2".into(),
                    digest: [2; 32],
                },
                producer.key().clone(),
                declaration_handle,
                contradictory_roles,
                vec![StaticSlot {
                    handle: SlotHandle::new(subject_slot),
                    compatibility: SlotCompatibility::Subject {
                        kind: SubjectKind::new(ORDER_KIND.to_vec()).expect("non-empty kind"),
                    },
                }],
            )
            .expect("changed static content must arrive under a new selected definition");
            let revised_runtime = RuntimeBindingSet::new(
                RUNTIME_BINDING_CONTRACT,
                revised_definition.selection(),
                &revised_definition,
                &producer,
                vec![RuntimeBinding::subject(
                    SlotHandle::new(subject_slot),
                    subject(&producer),
                )],
            )
            .expect("the new selected static definition accepts its runtime value");
            let revised_context =
                LinkedAssessment::new(&package, &revised_definition, &revised_runtime)
                    .expect("the revised static definition still targets the admitted protocol");
            assert_ne!(
                revised_context
                    .definition_identity()
                    .expect("the revised static definition has an FR-288 identity"),
                identity,
                "a changed accepted static binding definition changes FR-288"
            );
            registry
                .register(&revised_context)
                .expect("a changed static definition is not a contradictory replay");
            assert_eq!(registry.len(), 2);
        },
    );
    assert!(matches!(
        completed,
        ProducerReadiness::Executable(OperationOutcome::Completed(()))
    ));
}

/// A compensation bridge receives only QObs's complete FR-287 subject at a
/// QSL `effect` requirement.  A static coordinate never becomes a replacement
/// runtime identity, and the subject's opaque owner key remains unchanged.
#[test]
#[trace("TC-002", "TC-003", "FR-002-AC-1", "FR-003-AC-3")]
fn tc_002_effect_projection_requires_an_owner_effect_slot() {
    let package = admitted_published_v2();
    let (declaration_handle, effect_slot) = effect_coordinates(&package);
    let Body::Protocol {
        roles,
        compensations,
        ..
    } = &package.inherited().declarations
        [usize::try_from(declaration_handle).expect("published index fits")]
    .body
    else {
        unreachable!("selected declaration was found as a protocol")
    };
    let first_compensation = compensations
        .first()
        .expect("published protocol has one compensation definition");
    let producer = producer();
    let definition = StaticBindingDefinition::new(
        STATIC_BINDING_CONTRACT,
        selection(),
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
        vec![
            StaticSlot {
                handle: SlotHandle::new(effect_slot),
                compatibility: SlotCompatibility::Subject {
                    kind: SubjectKind::new(ORDER_KIND.to_vec()).expect("non-empty kind"),
                },
            },
            StaticSlot {
                handle: SlotHandle::new(first_compensation.attempt_instance),
                compatibility: SlotCompatibility::Subject {
                    kind: SubjectKind::new(ORDER_KIND.to_vec()).expect("non-empty kind"),
                },
            },
            StaticSlot {
                handle: SlotHandle::new(first_compensation.effect_instance),
                compatibility: SlotCompatibility::Subject {
                    kind: SubjectKind::new(ORDER_KIND.to_vec()).expect("non-empty kind"),
                },
            },
        ],
    )
    .expect("an exact selected QObs subject kind is admitted for the effect slot");
    let observed = subject(&producer);
    let runtime = RuntimeBindingSet::new(
        RUNTIME_BINDING_CONTRACT,
        definition.selection(),
        &definition,
        &producer,
        vec![
            RuntimeBinding::subject(SlotHandle::new(effect_slot), observed.clone()),
            RuntimeBinding::subject(
                SlotHandle::new(first_compensation.attempt_instance),
                QualifiedSubject::new(
                    &producer,
                    SubjectKind::new(ORDER_KIND.to_vec()).expect("non-empty kind"),
                    SubjectIdentity::new(b"compensation-attempt:1".to_vec())
                        .expect("non-empty identity"),
                ),
            ),
            RuntimeBinding::subject(
                SlotHandle::new(first_compensation.effect_instance),
                QualifiedSubject::new(
                    &producer,
                    SubjectKind::new(ORDER_KIND.to_vec()).expect("non-empty kind"),
                    SubjectIdentity::new(b"compensation-effect:1".to_vec())
                        .expect("non-empty identity"),
                ),
            ),
        ],
    )
    .expect("the owner subject satisfies the selected effect slot");
    let context = LinkedAssessment::new(&package, &definition, &runtime)
        .expect("the admitted selected effect context links");

    let effects = context.observed_effects();
    assert_eq!(effects.len(), 1);
    assert_eq!(effects[0].slot(), SlotHandle::new(effect_slot));
    assert_eq!(effects[0].subject(), &observed);

    let registrations = context.compensation_registration_inputs();
    assert_eq!(registrations.len(), 2);
    assert_eq!(registrations[0].compensation_handle(), 0);
    assert_eq!(registrations[1].compensation_handle(), 1);
    assert_eq!(registrations[0].effect().subject(), &observed);
    assert_eq!(registrations[1].effect().subject(), &observed);
    assert_eq!(context.compensation_attempt_inputs().len(), 1);
    assert_eq!(context.compensation_effect_inputs().len(), 1);

    let producer_request = ProducerResolutionRequest::for_assessment(
        package.digest(),
        definition.selection().digest,
        &[Claim::Conformance],
        select_runtime_bindings(context.runtime(), 500_000)
            .expect("fixture runtime selection is bounded"),
    )
    .expect("the closed claim selection is representable");
    let retargeted_runtime = RuntimeBindingSet::new(
        RUNTIME_BINDING_CONTRACT,
        definition.selection(),
        &definition,
        &producer,
        runtime
            .bindings()
            .iter()
            .map(|binding| {
                if binding.slot() == SlotHandle::new(first_compensation.attempt_instance) {
                    RuntimeBinding::subject(
                        binding.slot(),
                        QualifiedSubject::new(
                            &producer,
                            SubjectKind::new(ORDER_KIND.to_vec()).expect("non-empty kind"),
                            SubjectIdentity::new(b"compensation-attempt:2".to_vec())
                                .expect("non-empty identity"),
                        ),
                    )
                } else {
                    binding.clone()
                }
            })
            .collect(),
    )
    .expect("one non-activation QObs runtime substitution remains valid");
    let retargeted_context = LinkedAssessment::new(&package, &definition, &retargeted_runtime)
        .expect("the substituted compensation runtime still links");
    assert!(matches!(
        with_assessment_after_resolving(
            &producer_request,
            &producer_manifest(),
            producer_budget(),
            SystemMonotonicClock,
            |permit| AssessmentInputs::new(
                permit,
                retargeted_context,
                vec![Claim::Conformance],
            ),
            |_| (),
        ),
        ProducerReadiness::Executable(OperationOutcome::Refused(refusal))
            if refusal.code().as_str() == "version-lock.capability.request-mismatch"
    ));
    let completed = with_assessment_after_resolving(
        &producer_request,
        &producer_manifest(),
        producer_budget(),
        SystemMonotonicClock,
        |permit| AssessmentInputs::new(permit, context, vec![Claim::Conformance]),
        |engine| {
            let Body::Protocol { controls, .. } = &engine.linked().declaration().body else {
                panic!("the linked assessment retains the selected protocol declaration")
            };
            assert_eq!(engine.compensation_inputs().len(), 2);
            // Task-002 projects eligible exact inputs; Task-003 alone may turn them
            // into state after the admitted activation guard completes true.
            assert!(engine.compensation().registrations().is_empty());
            assert!(engine.conformance().registrations().is_empty());
            assert_eq!(
        engine.conformance().obligations().len(),
        controls.len(),
        "the conformance kernel retains every linked control without turning compensation inputs into registrations"
    );
            assert_eq!(
                engine.conformance().compensations(),
                engine.linked().compensations()
            );
            assert_eq!(
                engine
                    .conformance()
                    .compensation(0)
                    .expect("the first linked compensation is retained")
                    .definition(),
                engine.linked().compensations()[0].definition()
            );
            assert!(engine.conformance().compensation(u32::MAX).is_none());
            assert_eq!(
                engine
                    .conformance()
                    .obligation(0)
                    .expect("the first linked control is a conformance obligation")
                    .operation(),
                &controls[0].operation
            );
            assert!(engine.conformance().obligation(u32::MAX).is_none());
            assert_eq!(
                engine.conformance().causal_edges(),
                engine.linked().causal_edges()
            );
            assert!(engine.conformance().attempts().is_empty());
            assert!(engine.conformance().effects().is_empty());
            assert_eq!(engine.compensation_inputs().len(), 2);
            assert_eq!(
                engine.compensation_inputs()[0].effect().subject(),
                &observed
            );
            assert!(engine.compensation().registrations().is_empty());
            assert!(engine.conformance().registrations().is_empty());
            assert_eq!(engine.conformance().obligations().len(), controls.len());
            assert_eq!(
                engine.conformance().compensations(),
                engine.linked().compensations()
            );
            assert_eq!(
                engine.conformance().causal_edges(),
                engine.linked().causal_edges()
            );
            assert!(engine.conformance().attempts().is_empty());
            assert!(engine.conformance().effects().is_empty());
        },
    );
    assert!(matches!(
        completed,
        ProducerReadiness::Executable(OperationOutcome::Completed(()))
    ));
}
