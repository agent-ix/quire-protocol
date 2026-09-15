// SPDX-License-Identifier: AGPL-3.0-or-later
// Small owner-typed FCD producer used by QObs integration fixtures.
//
// This constructs the public producer records and invokes their real admission
// boundary. It is not a copied external JSON fixture and it does not mirror an
// admitted type.

use std::collections::{BTreeMap, BTreeSet};

use agent_ix_baseline_producer as producer;

pub const SUBJECT_KIND: &[u8] = b"ix://agent-ix/quire-protocol/test-subject";
pub const TARGET_KIND: &[u8] = b"ix://agent-ix/quire-protocol/test-target";
pub const RELATIONSHIP: &str = "ix://agent-ix/quire-protocol/test-relationship";

pub fn admitted(tag: &str) -> producer::AdmittedStaticBundle {
    let profile_identity = format!("profile:{tag}");
    let model_identity = format!("model:{tag}");
    let configuration_identity = format!("configuration:{tag}");
    let inventory_identity = format!("inventory:{tag}");

    let mut configuration = producer::ConfigurationDocument {
        configuration_identity: configuration_identity.clone(),
        baseline_version: producer::INTERFACE_VERSION.into(),
        digest: producer::DigestSelection::canonical(format!("sha256:{}", "0".repeat(64))),
        model_authority: "test:quire-protocol".into(),
        profile_identities: BTreeSet::from([profile_identity.clone()]),
        adapter_identities: BTreeSet::from(["test:qobs-adapter".into()]),
        mapping_targets: BTreeSet::from([profile_identity.clone()]),
        loss_policy: "test:refuse-loss".into(),
        resource_limits: producer::ResourceLimits {
            numeric_resource_limit: Some(producer::NumericResourceLimit::new(4096, 6144)),
            declared_bounds: BTreeMap::new(),
        },
        digest_selections: producer::DigestDomainSelection::baseline(),
        revision_namespaces: producer::ADMISSIBLE_REVISION_NAMESPACES
            .iter()
            .map(|namespace| (*namespace).to_owned())
            .collect(),
        trusted_references: BTreeSet::from(["test:quire-protocol-registry".into()]),
    };
    configuration.digest =
        producer::configuration_digest(&configuration).expect("test configuration is canonical");

    let model = producer::ModelSelection {
        model_identity: model_identity.clone(),
        model_revision: producer::Revision::producer("1"),
        digest: producer::DigestSelection::canonical(format!("sha256:{}", "1".repeat(64))),
    };
    let profile = producer::ProfileSelection {
        profile_identity: profile_identity.clone(),
        profile_revision: producer::Revision::producer("1"),
        digest: producer::DigestSelection::canonical(format!("sha256:{}", "2".repeat(64))),
    };
    let source = producer::ArtifactReference {
        ref_version: "ix.artifact-ref/3-draft".into(),
        kind: producer::ArtifactKind::Source,
        authority: "test:quire-protocol".into(),
        identity: format!("source:{tag}"),
        revision: producer::Revision::producer("1"),
        digest: producer::RawByteDigest::new(format!("sha256:{}", "3".repeat(64)))
            .expect("test raw digest"),
        wire: producer::WireReference {
            identity: "quire-protocol/test-source".into(),
            version: "1".into(),
        },
    };
    let formal = producer::FormalDocument {
        document: format!("formal:{tag}"),
        revision: producer::Revision::producer("1"),
    };
    let source_locus = producer::SourceLocus {
        source: source.clone(),
        formal: formal.clone(),
        span: producer::Span { start: 0, end: 1 },
    };
    let membership = producer::InventoryMembership::new(
        inventory_identity.clone(),
        producer::InventoryCompleteness::Complete,
    );
    let component_identity = format!("component:{tag}");
    let source_endpoint_identity = format!("endpoint:{tag}:source");
    let target_endpoint_identity = format!("endpoint:{tag}:target");
    let source_type = String::from_utf8(SUBJECT_KIND.to_vec()).expect("UTF-8 test kind");
    let target_type = String::from_utf8(TARGET_KIND.to_vec()).expect("UTF-8 test kind");
    let component = producer::ComponentDeclaration {
        component_identity: component_identity.clone(),
        component_revision: producer::Revision::producer("1"),
        digest: producer::DigestSelection::canonical(format!("sha256:{}", "4".repeat(64))),
        repository_identity: format!("repository:{tag}"),
        repository_revision: producer::Revision::producer("1"),
        role_identities: BTreeSet::from(["test-role".into()]),
        owning_type_identity: source_type.clone(),
        source_locus: Some(source_locus.clone()),
        inventory_membership: membership.clone(),
    };
    let multiplicity = producer::Multiplicity {
        lower: 0,
        upper: Some(1),
        ordered: false,
        unique: true,
    };
    let source_endpoint = producer::EndpointDeclaration {
        endpoint_identity: source_endpoint_identity.clone(),
        endpoint_revision: producer::Revision::producer("1"),
        digest: producer::DigestSelection::canonical(format!("sha256:{}", "5".repeat(64))),
        component_identity: component_identity.clone(),
        type_identity: source_type.clone(),
        role: "source".into(),
        multiplicity: Some(multiplicity.clone()),
        source_locus: Some(source_locus.clone()),
        inventory_membership: membership.clone(),
    };
    let target_endpoint = producer::EndpointDeclaration {
        endpoint_identity: target_endpoint_identity.clone(),
        endpoint_revision: producer::Revision::producer("1"),
        digest: producer::DigestSelection::canonical(format!("sha256:{}", "6".repeat(64))),
        component_identity: component_identity.clone(),
        type_identity: target_type.clone(),
        role: "target".into(),
        multiplicity: Some(multiplicity.clone()),
        source_locus: Some(source_locus.clone()),
        inventory_membership: membership.clone(),
    };
    let relationship = producer::RelationshipDeclaration {
        relationship_identity: RELATIONSHIP.into(),
        relationship_name: "TestRelationship".into(),
        relationship_revision: Some(producer::Revision::producer("1")),
        digest: Some(producer::DigestSelection::canonical(format!(
            "sha256:{}",
            "7".repeat(64)
        ))),
        source: producer::RelationshipEndpoint {
            endpoint_identity: source_endpoint_identity.clone(),
            type_identity: source_type.clone(),
            role: "source".into(),
            multiplicity: Some(multiplicity.clone()),
        },
        target: producer::RelationshipEndpoint {
            endpoint_identity: target_endpoint_identity.clone(),
            type_identity: target_type.clone(),
            role: "target".into(),
            multiplicity: Some(multiplicity),
        },
        semantics: producer::RelationshipSemantics {
            category: "association".into(),
            direction: producer::RelationshipDirection::SourceToTarget,
            composite: false,
            lifecycle: "independent".into(),
            ownership: "none".into(),
        },
        ownership: Some(producer::RelationshipOwnership {
            model_identity: model_identity.clone(),
            profile_identity: profile_identity.clone(),
            configuration_identity: configuration_identity.clone(),
        }),
        inventory_membership: Some(membership),
    };
    let closure = producer::StaticClosure {
        configuration_identity: configuration_identity.clone(),
        configuration_digest: configuration.digest.clone(),
        model_identity: model_identity.clone(),
        model_digest: model.digest.clone(),
        profile_identity: profile_identity.clone(),
        profile_digest: profile.digest.clone(),
        declaration_sources: vec![producer::DeclarationSource {
            source,
            native: producer::NativeSourceLabel::new(format!("native:{tag}"), "1"),
            path: format!("tests/{tag}.quire"),
            formal,
        }],
    };
    let inventory = producer::InventoryDeclaration {
        inventory_identity,
        completeness: producer::InventoryCompleteness::Complete,
        component_identities: BTreeSet::from([component_identity.clone()]),
        endpoint_identities: BTreeSet::from([
            source_endpoint_identity.clone(),
            target_endpoint_identity.clone(),
        ]),
        relationship_identities: BTreeSet::from([RELATIONSHIP.into()]),
    };
    let export = |kind: producer::ExportKind, identity: &str, path: &str| producer::ExportRecord {
        kind,
        producer_object_identity: model_identity.clone(),
        export_identity: identity.into(),
        export_path: vec![path.into()],
        locus: Some(source_locus.clone()),
    };
    let correspondence = producer::ProducerNativeCorrespondence {
        binding_relation_identity: format!("relation:{tag}"),
        producer: producer::ProducerObjectReference {
            object_kind: "model".into(),
            authority: "test:quire-protocol".into(),
            identity: model_identity.clone(),
            revision: producer::Revision::producer("1"),
            digest: model.digest.clone(),
        },
        native: producer::NativeArtifactReference {
            identity: format!("native-model:{tag}"),
            revision: producer::Revision::native("1"),
            raw_byte_digest: producer::DigestSelection::native_bytes(format!(
                "sha256:{}",
                "8".repeat(64)
            )),
        },
        native_definition_closure: Vec::new(),
        required_native_definition_identities: BTreeSet::new(),
        configuration_identity: Some(configuration_identity),
        exports: vec![
            export(producer::ExportKind::Object, &source_type, "TestSubject"),
            export(producer::ExportKind::Record, &target_type, "TestTarget"),
            export(
                producer::ExportKind::Component,
                &component_identity,
                "TestComponent",
            ),
            export(
                producer::ExportKind::Endpoint,
                &source_endpoint_identity,
                "TestSourceEndpoint",
            ),
            export(
                producer::ExportKind::Endpoint,
                &target_endpoint_identity,
                "TestTargetEndpoint",
            ),
            export(
                producer::ExportKind::Relationship,
                RELATIONSHIP,
                "TestRelationship",
            ),
        ],
    };
    let mut offered = producer::StaticProducerBundle {
        bundle_identity: Some(format!("producer:{tag}")),
        bundle_revision: Some(producer::Revision::producer("1")),
        digest: None,
        interface_version: Some(producer::INTERFACE_VERSION.into()),
        model: Some(model),
        profile: Some(profile),
        components: vec![component],
        endpoints: vec![source_endpoint, target_endpoint],
        relationships: vec![relationship],
        inventory: Some(inventory),
        configuration: Some(configuration),
        static_closure: Some(closure),
        correspondences: vec![correspondence],
    };
    offered.digest = Some(
        offered
            .canonical_digest_selection()
            .expect("test producer digest computes"),
    );
    offered.admit().expect("test producer admits")
}
