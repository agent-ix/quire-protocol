// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-002: exact versioned static/runtime protocol linking.

use ix_trace_rs::trace;
use quire_observation::{
    QualifiedSubject, Relationship, RelationshipIdentity, RequiredRelationship, SubjectIdentity,
    SubjectKind,
};
use quire_protocol::link::{
    admit_binding_definition, BindingArtifactIdentity, BindingDefinitionOffer,
    DefinitionRegistration, DefinitionRegistry, ExpectedBindingDefinition, RelationshipBinding,
    RoleClassification, RoleClassificationOffer, RoleInternality, RuntimeBindingSet,
    SlotCompatibility, SubjectBinding, BINDING_DEFINITION_CONTRACT, RUNTIME_BINDING_CONTRACT,
};
use quire_protocol::wire::{self, Body};

#[path = "support/v2_handoff/mod.rs"]
// The shared handoff module serves the larger IT-001 corpus; this focused
// public-API test intentionally consumes only its strict loader and fixture.
#[allow(dead_code, unused_imports)]
mod handoff;
#[path = "support/producer_owner.rs"]
mod producer_owner;

struct Fixture {
    package: quire_protocol::v2::AdmittedPackage,
    binding: quire_protocol::link::AcceptedBindingDefinition,
    runtime: RuntimeBindingSet,
}

fn artifact() -> BindingArtifactIdentity {
    BindingArtifactIdentity {
        identity: "ix://agent-ix/quire-protocol/test-binding".into(),
        revision: wire::Revision {
            namespace: "ix://agent-ix/quire-protocol/revision".into(),
            value: "1".into(),
        },
        digest: wire::SelectedDigest {
            domain: "test-static-binding".into(),
            version: "1".into(),
            algorithm: "sha256".into(),
            value: format!("sha256:{}", "a".repeat(64)),
        },
    }
}

fn accept_binding(
    artifact: BindingArtifactIdentity,
    declaration: u32,
    roles: Vec<RoleClassification>,
    slots: Vec<SlotCompatibility>,
) -> quire_protocol::link::AcceptedBindingDefinition {
    let expected = ExpectedBindingDefinition {
        artifact: artifact.clone(),
        declaration,
        roles: roles.clone(),
        slots: slots.clone(),
    };
    admit_binding_definition(
        BindingDefinitionOffer {
            contract: BINDING_DEFINITION_CONTRACT.into(),
            artifact,
            declaration,
            roles: roles
                .iter()
                .map(|role| RoleClassificationOffer {
                    role: role.role,
                    internality: match role.internality {
                        RoleInternality::Internal => "internal",
                        RoleInternality::External => "external",
                    }
                    .into(),
                })
                .collect(),
            slots,
        },
        &expected,
    )
    .expect("independently selected static binding")
}

fn fixture() -> Fixture {
    let loaded = handoff::Handoff::load().expect("published v2 handoff");
    let declarations = loaded.declarations();
    let inventories = loaded.inventories(&declarations);
    let expected = loaded.expected(&inventories);
    let package = quire_protocol::intake::v2::admit(loaded.offer(), &expected, loaded.limits())
        .into_result()
        .expect("strict v2 package");
    let (declaration, protocol) = package
        .inherited()
        .declarations
        .iter()
        .enumerate()
        .find(|(_, declaration)| matches!(declaration.body, Body::Protocol { .. }))
        .and_then(|(index, declaration)| Some((u32::try_from(index).ok()?, declaration)))
        .expect("published protocol declaration");
    let Body::Protocol { roles, .. } = &protocol.body else {
        unreachable!("selected above")
    };
    let producer = producer_owner::admitted("tc-002-link");
    let kind = SubjectKind::new(producer_owner::SUBJECT_KIND).expect("subject kind");
    let relationship_source_kind =
        SubjectKind::new(producer_owner::SUBJECT_KIND).expect("relationship source kind");
    let relationship_target_kind =
        SubjectKind::new(producer_owner::TARGET_KIND).expect("relationship target kind");
    let mut slots = Vec::with_capacity(protocol.bindings.len());
    let mut subjects = Vec::new();
    let mut relationships = Vec::new();
    for (index, requirement) in protocol.bindings.iter().enumerate() {
        let requirement_index = u32::try_from(index).expect("admitted binding index");
        let correspondence = requirement.model.0.as_ref().and_then(|model| {
            package
                .inherited()
                .models
                .get(usize::try_from(model.model).ok()?)
                .and_then(|model| model.correspondence.0.clone())
        });
        if requirement.kind == wire::BindingKind::Relationship {
            let source = QualifiedSubject::new(
                &producer,
                relationship_source_kind.clone(),
                SubjectIdentity::new(format!("runtime:source:{index}").into_bytes())
                    .expect("source identity"),
            );
            let target = QualifiedSubject::new(
                &producer,
                relationship_target_kind.clone(),
                SubjectIdentity::new(format!("runtime:target:{index}").into_bytes())
                    .expect("target identity"),
            );
            let required = RequiredRelationship::new(
                &producer,
                producer_owner::RELATIONSHIP,
                source.clone(),
                target.clone(),
            )
            .expect("selected relationship premise");
            let relationship = Relationship::new(
                &producer,
                producer_owner::RELATIONSHIP,
                RelationshipIdentity::new(format!("runtime:relationship:{index}").into_bytes())
                    .expect("relationship identity"),
                source,
                target,
            )
            .expect("runtime relationship");
            slots.push(SlotCompatibility::Relationship {
                requirement: requirement_index,
                authority: producer.key().clone(),
                required: Box::new(required),
                correspondence,
            });
            relationships.push(RelationshipBinding {
                requirement: requirement_index,
                relationship,
            });
        } else {
            slots.push(SlotCompatibility::Subject {
                requirement: requirement_index,
                authority: producer.key().clone(),
                producer_kind: kind.clone(),
                correspondence,
            });
            subjects.push(SubjectBinding {
                requirement: requirement_index,
                subject: QualifiedSubject::new(
                    &producer,
                    kind.clone(),
                    SubjectIdentity::new(format!("runtime:subject:{index}").into_bytes())
                        .expect("runtime identity"),
                ),
            });
        }
    }
    let roles: Vec<RoleClassification> = (0..roles.len())
        .map(|index| RoleClassification {
            role: u32::try_from(index).expect("admitted role index"),
            internality: if index % 2 == 0 {
                RoleInternality::Internal
            } else {
                RoleInternality::External
            },
        })
        .collect();
    let identity = artifact();
    let binding = accept_binding(identity.clone(), declaration, roles, slots);
    Fixture {
        package,
        binding,
        runtime: RuntimeBindingSet {
            contract: RUNTIME_BINDING_CONTRACT.into(),
            target: identity,
            subjects,
            relationships,
        },
    }
}

#[trace("TC-002", "FR-002-AC-1", "FR-002-AC-4")]
#[test]
fn tc_002_published_v2_handoff_links_and_replays_transactionally() {
    let fixture = fixture();
    let mut registry = DefinitionRegistry::default();
    let mut missing_runtime = fixture.runtime.clone();
    missing_runtime.subjects.clear();
    assert!(quire_protocol::model::v2::link(
        &fixture.package,
        &fixture.binding,
        missing_runtime,
        &mut registry,
    )
    .is_err());
    assert!(
        registry.is_empty(),
        "a binding refusal cannot commit a definition"
    );

    let first = quire_protocol::model::v2::link(
        &fixture.package,
        &fixture.binding,
        fixture.runtime.clone(),
        &mut registry,
    )
    .expect("role model types are not concrete runtime authorities");

    assert_eq!(first.registration(), DefinitionRegistration::Inserted);
    assert_eq!(first.static_binding().roles().len(), 2);
    assert_eq!(
        first.runtime().subjects().len() + first.runtime().relationships().len(),
        first.static_binding().slots().len(),
        "every static slot has one exact runtime value"
    );
    assert!(!first.runtime().subjects().is_empty());
    let quire_protocol::wire::Body::Protocol { roles, .. } = &first.content().declaration().body
    else {
        panic!("linked fixture must retain a static protocol declaration");
    };
    assert_eq!(roles.len(), 2);
    let selected_source = &fixture.package.package().inherited.declarations
        [usize::try_from(fixture.binding.declaration()).expect("declaration index")];
    let retained_source = first
        .presentation()
        .declarations()
        .iter()
        .find(|declaration| declaration.index == fixture.binding.declaration())
        .expect("selected declaration source context");
    assert_eq!(retained_source.value.locus, selected_source.locus);
    assert_eq!(first.channels().len(), 1);
    assert_eq!(registry.len(), 1);

    let replay = quire_protocol::model::v2::link(
        &fixture.package,
        &fixture.binding,
        fixture.runtime.clone(),
        &mut registry,
    )
    .expect("exact replay");
    assert_eq!(replay.registration(), DefinitionRegistration::Replayed);
    assert_eq!(first.identity(), replay.identity());
    assert_eq!(registry.len(), 1, "exact replay cannot add a registry row");

    let producer = producer_owner::admitted("tc-002-link");
    let changed_subject = fixture
        .runtime
        .subjects
        .first()
        .expect("one subject binding");
    let kind = match fixture
        .binding
        .slots()
        .iter()
        .find(|slot| slot.requirement() == changed_subject.requirement)
        .expect("selected subject slot")
    {
        SlotCompatibility::Subject { producer_kind, .. } => producer_kind.clone(),
        SlotCompatibility::Relationship { .. } => unreachable!("fixture subject slot"),
    };
    let mut changed_runtime = fixture.runtime.clone();
    changed_runtime.subjects[0].subject = QualifiedSubject::new(
        &producer,
        kind,
        SubjectIdentity::new(b"runtime:tc-002:other".to_vec()).expect("runtime identity"),
    );
    let changed_runtime = quire_protocol::model::v2::link(
        &fixture.package,
        &fixture.binding,
        changed_runtime,
        &mut registry,
    )
    .expect("concrete runtime identity is outside FR-288");
    assert_eq!(
        changed_runtime.registration(),
        DefinitionRegistration::Replayed
    );
    assert_eq!(first.identity(), changed_runtime.identity());
    assert_eq!(first.graph(), changed_runtime.graph());
    assert_ne!(
        first.runtime().subjects()[0].subject(),
        changed_runtime.runtime().subjects()[0].subject()
    );
    assert_eq!(registry.len(), 1);
}

#[trace("TC-002", "FR-002-AC-3", "FR-002-AC-4")]
#[test]
fn tc_002_changed_static_binding_content_requires_another_fr288_identity() {
    let fixture = fixture();
    let mut registry = DefinitionRegistry::default();
    let baseline = quire_protocol::model::v2::link(
        &fixture.package,
        &fixture.binding,
        fixture.runtime.clone(),
        &mut registry,
    )
    .expect("baseline definition");

    let mut changed_roles = fixture.binding.roles().to_vec();
    changed_roles[0].internality = match changed_roles[0].internality {
        RoleInternality::Internal => RoleInternality::External,
        RoleInternality::External => RoleInternality::Internal,
    };
    let changed_role_binding = accept_binding(
        fixture.binding.artifact().clone(),
        fixture.binding.declaration(),
        changed_roles.clone(),
        fixture.binding.slots().to_vec(),
    );
    let refusal = quire_protocol::model::v2::link(
        &fixture.package,
        &changed_role_binding,
        fixture.runtime.clone(),
        &mut registry,
    )
    .expect_err("same FR-288 key cannot carry changed role internality");
    assert!(matches!(
        refusal,
        quire_protocol::link::LinkDiagnostics::DefinitionContradiction(_)
    ));
    assert_eq!(registry.len(), 1, "contradiction is transactional");

    let producer = producer_owner::admitted("tc-002-link");
    let changed_kind =
        SubjectKind::new(producer_owner::TARGET_KIND).expect("second admitted subject kind");
    let mut changed_slots = fixture.binding.slots().to_vec();
    let (slot_index, requirement) = changed_slots
        .iter()
        .enumerate()
        .find_map(|(index, slot)| match slot {
            SlotCompatibility::Subject { requirement, .. } => Some((index, *requirement)),
            SlotCompatibility::Relationship { .. } => None,
        })
        .expect("one subject slot");
    let SlotCompatibility::Subject { producer_kind, .. } = &mut changed_slots[slot_index] else {
        unreachable!("selected above")
    };
    *producer_kind = changed_kind.clone();
    let changed_slot_binding = accept_binding(
        fixture.binding.artifact().clone(),
        fixture.binding.declaration(),
        fixture.binding.roles().to_vec(),
        changed_slots,
    );
    let mut changed_runtime = fixture.runtime.clone();
    let subject = changed_runtime
        .subjects
        .iter_mut()
        .find(|subject| subject.requirement == requirement)
        .expect("runtime subject for changed slot");
    let subject_identity = subject.subject.identity().as_bytes().to_vec();
    subject.subject = QualifiedSubject::new(
        &producer,
        changed_kind,
        SubjectIdentity::new(subject_identity).expect("retained non-empty identity"),
    );
    let refusal = quire_protocol::model::v2::link(
        &fixture.package,
        &changed_slot_binding,
        changed_runtime,
        &mut registry,
    )
    .expect_err("same FR-288 key cannot carry changed slot compatibility");
    assert!(matches!(
        refusal,
        quire_protocol::link::LinkDiagnostics::DefinitionContradiction(_)
    ));
    assert_eq!(registry.len(), 1, "contradiction is transactional");

    let mut new_artifact = fixture.binding.artifact().clone();
    new_artifact.identity.push_str(":reclassified");
    let reidentified_binding = accept_binding(
        new_artifact.clone(),
        fixture.binding.declaration(),
        changed_roles,
        fixture.binding.slots().to_vec(),
    );
    let mut reidentified_runtime = fixture.runtime;
    reidentified_runtime.target = new_artifact;
    let reidentified = quire_protocol::model::v2::link(
        &fixture.package,
        &reidentified_binding,
        reidentified_runtime,
        &mut registry,
    )
    .expect("changed static selection with another identity");
    assert_ne!(baseline.identity(), reidentified.identity());
    assert_eq!(registry.len(), 2);
}
