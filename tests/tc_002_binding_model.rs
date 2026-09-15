// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-002: versioned static/runtime binding capability admission.

use std::collections::BTreeSet;

use agent_ix_baseline_producer::StaticProducerBundle;
use ix_trace_rs::trace;
use quire_observation::{
    QualifiedSubject, Relationship, RelationshipIdentity, SubjectIdentity, SubjectKind,
};
use quire_protocol::model::v2::{
    link as link_v2, ChannelHandle, DefinitionRegistry, FifoPartitionRefusal, Internality,
    LinkedAssessment, RegistryRefusal, RoleClassification, RuntimeBinding, RuntimeBindingSet,
    RuntimeRefusal, SlotCompatibility, SlotHandle, StaticBindingDefinition, StaticRefusal,
    StaticSelection, StaticSlot, RUNTIME_BINDING_CONTRACT, STATIC_BINDING_CONTRACT,
};
use quire_protocol::wire::{BindingKind, Body, CausalEdge, ControlOperation, EdgeKind, Port};

// The shared handoff reader carries all public mutation helpers; this focused
// binding lane exercises only `Handoff`, so the remaining helper surface is
// intentionally unused here.
#[allow(dead_code, unused_imports)]
#[path = "support/v2_handoff/mod.rs"]
mod handoff;

const ORDER_KIND: &[u8] = b"ix://agent-ix/commerce/type/Order";
const SHIPMENT_KIND: &[u8] = b"ix://agent-ix/commerce/type/Shipment";
const ORDER_SHIPMENT: &str = "ix://agent-ix/commerce/relationship/Order-shipment";

fn admitted_published_v2() -> quire_spec_language::protocol_artifact::v2::AdmittedPackage {
    let loaded = handoff::Handoff::load().expect("published v2 handoff");
    let declarations = loaded.declarations();
    let inventories = loaded.inventories(&declarations);
    let expected = loaded.expected(&inventories);
    quire_protocol::intake::v2::admit(loaded.offer(), &expected, loaded.limits())
        .into_result()
        .expect("published v2 offer admits")
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

/// The full-assessment boundary accepts only an admitted `/2` protocol and
/// retains its static and runtime capabilities without retargeting either one.
#[test]
#[trace("TC-002", "FR-002-AC-1", "FR-002-AC-3")]
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
    let runtime = RuntimeBindingSet::new(
        RUNTIME_BINDING_CONTRACT,
        definition.selection(),
        &definition,
        &producer,
        vec![RuntimeBinding::subject(
            SlotHandle::new(subject_slot),
            subject(&producer),
        )],
    )
    .expect("selected runtime capability admits");
    let context = LinkedAssessment::new(&package, &definition, &runtime)
        .expect("same admitted package and selected capabilities link");
    assert_eq!(context.package().digest(), package.digest());
    assert_eq!(context.definition().selection(), definition.selection());
    assert_eq!(context.runtime().target(), definition.selection());

    let mut registry = DefinitionRegistry::default();
    registry
        .register(&context)
        .expect("first static definition registers");
    registry
        .register(&context)
        .expect("exact static definition replay is idempotent");
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
        contradictory_roles,
        vec![StaticSlot {
            handle: SlotHandle::new(u32::try_from(subject_slot).expect("published slot fits")),
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
            SlotHandle::new(u32::try_from(subject_slot).expect("published slot fits")),
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
}
