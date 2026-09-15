// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-011: shared operation/claim outcomes and typed refusal foundations.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use super::cause::EmptySourceContractField;
use crate::bounds::inventory::{
    production_inventory, Budget, BudgetDimension, OperationOutcome as KernelOutcome,
    ResourceAccountant,
};
use crate::refusal::catalog::{
    audit, lookup, CatalogDigest, CatalogDigestUnavailable, CatalogSelection, RetainedBytes,
    REGISTRY, UNREGISTERED_CODE, UNSUPPORTED_SOURCE_CONTRACT,
};
use crate::refusal::{
    AffectedIdentity, CauseDetail, InvertedSpan, Location, PartialOutput, Prerequisite,
    PrerequisiteState, ProtocolCause, ProtocolCauseKind, ProtocolRefusal, ProtocolStage,
    RefusalAxis, RefusalCause, RefusalDomain, Rendering, ResourceUsage, SourceCause,
    SourceContract, SourceField, SourceLocation, StageDomainMismatch, Unavailability,
    UnsupportedSourceContract, CONTRACT as REFUSAL_CONTRACT,
};
use crate::repro::ToolchainIdentity;
use crate::{ClaimOutcome, OperationOutcome};
use ix_trace_rs::trace;

fn axis(value: &[u8]) -> RefusalAxis {
    RefusalAxis::new(value).expect("test axis is non-empty")
}

fn missing_definition(value: &[u8]) -> RefusalCause {
    ProtocolCause::MissingDefinition {
        definition: RetainedBytes::new(value),
    }
    .into()
}

fn usage() -> ResourceUsage {
    ResourceUsage::protocol("quire.protocol/test-usage/1", &[("operations", 1)])
}

fn representation_refusal(axis_value: &[u8]) -> ProtocolRefusal {
    let code = lookup(UNREGISTERED_CODE.code).expect("registered foundation code");
    ProtocolRefusal::builder(
        code,
        CatalogSelection::local(),
        ProtocolStage::Representation,
        axis(axis_value),
        missing_definition(b"definition/7"),
        usage(),
    )
    .expect("representation code and stage agree")
    .identity(AffectedIdentity::retained(b"definition/7"))
    .unavailable_location(Unavailability::NotReportedBySource)
    .emitting_nothing()
}

fn hash_of<T: Hash>(value: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

fn resource_incomplete() -> crate::bounds::inventory::ResourceIncomplete {
    let inventory = production_inventory().expect("production resource inventory");
    let budget = Budget::nothing_granted()
        .grant(BudgetDimension::InputByte, 1)
        .expect("positive grant");
    let mut resources = ResourceAccountant::new(budget, &inventory);
    match ToolchainIdentity::new_bounded(b"more-than-one-byte", "toolchain", &mut resources) {
        KernelOutcome::ResourceIncomplete(incomplete) => incomplete,
        other => panic!("one-short fixture must exhaust input bytes: {other:?}"),
    }
}

/// Tracing: TC-011, FR-008-AC-1, FR-036-AC-4, Task-017.
#[test]
#[trace("TC-011", "FR-008-AC-1", "FR-036-AC-4")]
fn tc_011_domains_and_stages_are_closed_and_exact() {
    assert_eq!(REFUSAL_CONTRACT, "quire.protocol/refusal/1");
    assert_eq!(
        RefusalDomain::ALL.map(RefusalDomain::as_str),
        [
            "resolution",
            "admission",
            "representation",
            "assessment",
            "result",
            "projection",
            "evidence",
        ]
    );
    assert_eq!(
        ProtocolStage::ALL.map(ProtocolStage::as_str),
        [
            "contract-resolution",
            "admission",
            "representation",
            "compensation",
            "conformance",
            "claim-dispatch",
            "result-assembly",
            "result-read",
            "projection",
            "evidence-handoff",
        ]
    );
    assert_eq!(
        ProtocolStage::ALL.map(ProtocolStage::domain),
        [
            RefusalDomain::Resolution,
            RefusalDomain::Admission,
            RefusalDomain::Representation,
            RefusalDomain::Assessment,
            RefusalDomain::Assessment,
            RefusalDomain::Assessment,
            RefusalDomain::Result,
            RefusalDomain::Result,
            RefusalDomain::Projection,
            RefusalDomain::Evidence,
        ]
    );
}

/// Tracing: TC-011, FR-036-AC-4, Task-017.
#[test]
#[trace("TC-011", "FR-036-AC-4")]
fn tc_011_every_invalid_stage_domain_pair_refuses() {
    for stage in ProtocolStage::ALL {
        for domain in RefusalDomain::ALL {
            let result = stage.validate_domain(domain);
            if domain == stage.domain() {
                assert_eq!(result, Ok(()));
            } else {
                assert_eq!(
                    result,
                    Err(StageDomainMismatch {
                        stage,
                        expected: stage.domain(),
                        offered: domain,
                    })
                );
            }
        }
    }
}

/// Tracing: TC-011, FR-036-AC-2, FR-036-AC-4, Task-017.
#[test]
#[trace("TC-011", "FR-036-AC-2", "FR-036-AC-4")]
fn tc_011_envelope_construction_validates_stage_domain_and_axis() {
    let code = lookup(UNREGISTERED_CODE.code).expect("registered foundation code");
    let refusal = representation_refusal(b"unregistered-source-code");
    assert_eq!(refusal.domain(), RefusalDomain::Representation);
    assert_eq!(refusal.stage(), ProtocolStage::Representation);
    assert_eq!(refusal.axis().as_bytes(), b"unregistered-source-code");
    assert_eq!(refusal.partial_output(), PartialOutput::NoneEmitted);
    assert_eq!(refusal.causes().len(), 1);
    assert!(matches!(
        refusal.causes().declared_order(),
        [RefusalCause::Protocol(ProtocolCause::MissingDefinition { definition })]
            if definition.as_bytes() == b"definition/7"
    ));

    assert!(RefusalAxis::new(b"").is_err());
    assert_eq!(
        ProtocolRefusal::builder(
            code,
            CatalogSelection::local(),
            ProtocolStage::Admission,
            axis(b"wrong-stage"),
            missing_definition(b"definition/7"),
            usage(),
        )
        .expect_err("a representation code cannot be raised at admission"),
        StageDomainMismatch {
            stage: ProtocolStage::Admission,
            expected: RefusalDomain::Admission,
            offered: RefusalDomain::Representation,
        }
    );
}

/// Tracing: TC-011, FR-014-AC-1, FR-014-AC-2, FR-014-AC-4, Task-017.
#[test]
#[trace("TC-011", "FR-014-AC-1", "FR-014-AC-2", "FR-014-AC-4")]
fn tc_011_axis_keys_do_not_hide_an_interior_collision() {
    let first = representation_refusal(b"first-axis");
    let second = representation_refusal(b"second-axis");
    assert_ne!(first, second, "the typed outcome identities differ by axis");
    assert!(
        first.has_same_interior_as(&second),
        "the producer interior collision remains independently observable"
    );

    let same_axis = representation_refusal(b"first-axis");
    assert_eq!(first, same_axis);
    assert!(first.has_same_interior_as(&same_axis));
}

/// Tracing: TC-011, FR-039-AC-1, Task-017.
#[test]
#[trace("TC-011", "FR-039-AC-1")]
fn tc_011_local_cause_members_and_payloads_do_not_collapse() {
    assert_eq!(ProtocolCauseKind::ALL.len(), 10);
    let labels: HashSet<_> = ProtocolCauseKind::ALL
        .into_iter()
        .map(ProtocolCauseKind::as_str)
        .collect();
    assert_eq!(labels.len(), 10);
    assert!(!labels.contains("resource-incomplete"));

    let causes = [
        ProtocolCause::MissingDefinition {
            definition: RetainedBytes::new(b"d"),
        },
        ProtocolCause::ConflictingDefinition {
            definition: RetainedBytes::new(b"d"),
            conflicting_definition: RetainedBytes::new(b"d2"),
        },
        ProtocolCause::WrongAnchor {
            expected: RetainedBytes::new(b"a"),
            observed: RetainedBytes::new(b"b"),
        },
        ProtocolCause::MissingObservation {
            observation: RetainedBytes::new(b"o"),
        },
        ProtocolCause::UnprovedDefinedness {
            expression: RetainedBytes::new(b"e"),
        },
        ProtocolCause::LogicalViolation {
            obligation: RetainedBytes::new(b"p"),
        },
        ProtocolCause::IdentityMismatch {
            expected: RetainedBytes::new(b"i"),
            observed: RetainedBytes::new(b"j"),
        },
        ProtocolCause::IncompleteEvidence {
            evidence: RetainedBytes::new(b"v"),
        },
        ProtocolCause::UnsupportedCapability {
            capability: RetainedBytes::new(b"c"),
        },
        ProtocolCause::ProjectionFindingLoss {
            finding: RetainedBytes::new(b"f"),
        },
    ];
    assert_eq!(
        causes.each_ref().map(ProtocolCause::kind),
        ProtocolCauseKind::ALL
    );
}

/// Tracing: TC-011, FR-038-AC-1, FR-038-AC-2, FR-038-AC-3, Task-017.
#[test]
#[trace("TC-011", "FR-038-AC-1", "FR-038-AC-2", "FR-038-AC-3")]
fn tc_011_source_causes_require_exact_contract_before_interpretation() {
    assert_eq!(
        SourceContract::new(b"", b"1"),
        Err(EmptySourceContractField::Identity)
    );
    assert_eq!(
        SourceContract::new(b"owner/contract", b""),
        Err(EmptySourceContractField::Revision)
    );

    let selected = SourceContract::new(b"owner/contract", b"1").expect("selected contract");
    let exact = SourceCause::retain(
        &selected,
        selected.clone(),
        b"owner-code",
        SourceField::Available(RetainedBytes::new(b"owner-stage")),
        SourceField::Unavailable,
        CauseDetail::Retained(RetainedBytes::new(b"opaque-payload")),
    )
    .expect("exact contract admits");
    assert_eq!(exact.contract(), &selected);
    assert_eq!(exact.code().as_bytes(), b"owner-code");

    let changed = SourceContract::new(b"owner/contract", b"2").expect("changed contract");
    assert!(SourceCause::retain(
        &selected,
        changed,
        b"owner-code",
        SourceField::Unavailable,
        SourceField::Unavailable,
        CauseDetail::NoneOffered,
    )
    .is_err());

    let unknown_contract =
        SourceContract::new(b"future/contract", b"99").expect("well-formed unknown contract");
    let unknown = UnsupportedSourceContract::retain(
        unknown_contract.clone(),
        b"future-code",
        SourceField::Available(RetainedBytes::new(b"future-stage")),
        SourceField::Unavailable,
        CauseDetail::Retained(RetainedBytes::new(b"future-payload")),
    );
    assert_eq!(unknown.contract(), &unknown_contract);
    assert_eq!(unknown.code().as_bytes(), b"future-code");
    assert!(matches!(
        RefusalCause::from(unknown),
        RefusalCause::UnsupportedSourceContract(_)
    ));
}

/// Tracing: TC-011, FR-038-AC-4, FR-039-AC-2, FR-039-AC-3, Task-017.
#[test]
#[trace("TC-011", "FR-038-AC-4", "FR-039-AC-2", "FR-039-AC-3")]
fn tc_011_typed_causes_retain_order_multiplicity_and_unknown_interiors() {
    let code = lookup(UNREGISTERED_CODE.code).expect("registered foundation code");
    let unknown_contract =
        SourceContract::new(b"future/contract", b"99").expect("unknown contract is well formed");
    let unknown = UnsupportedSourceContract::retain(
        unknown_contract,
        b"future-code",
        SourceField::Unavailable,
        SourceField::Unavailable,
        CauseDetail::WithheldByDetailBudget,
    );
    let second = ProtocolCause::ConflictingDefinition {
        definition: RetainedBytes::new(b"definition/7"),
        conflicting_definition: RetainedBytes::new(b"definition/8"),
    };
    let refusal = ProtocolRefusal::builder(
        code,
        CatalogSelection::local(),
        ProtocolStage::Representation,
        axis(b"unknown-source-contract"),
        RefusalCause::from(unknown.clone()),
        usage(),
    )
    .expect("representation code and stage agree")
    .cause(second.clone())
    .cause(RefusalCause::from(unknown.clone()))
    .emitting_nothing();

    assert_eq!(refusal.causes().len(), 3);
    assert_eq!(
        refusal.causes().declared_order(),
        &[
            RefusalCause::from(unknown.clone()),
            RefusalCause::from(second),
            RefusalCause::from(unknown),
        ],
        "causes remain in supplied order and duplicates are not erased"
    );
    assert_eq!(refusal.partial_output(), PartialOutput::NoneEmitted);
}

/// Tracing: TC-011, FR-014-AC-5, FR-016-AC-5, FR-040-AC-1, Task-017.
#[test]
#[trace("TC-011", "FR-014-AC-5", "FR-016-AC-5", "FR-040-AC-1")]
fn tc_011_operation_outcomes_are_three_non_substitutable_states() {
    let completed: OperationOutcome<Vec<u8>> = OperationOutcome::Completed(Vec::new());
    let refused: OperationOutcome<Vec<u8>> =
        OperationOutcome::Refused(representation_refusal(b"invalid-representation"));
    let incomplete: OperationOutcome<Vec<u8>> =
        OperationOutcome::ResourceIncomplete(resource_incomplete());

    assert_ne!(completed, refused);
    assert_ne!(completed, incomplete);
    assert_ne!(refused, incomplete);
    assert!(matches!(completed, OperationOutcome::Completed(value) if value.is_empty()));
    assert!(matches!(refused, OperationOutcome::Refused(_)));
    assert!(matches!(
        incomplete,
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::InputByte
    ));

    let mapped = OperationOutcome::Completed(2_u8).map(|value| value + 1);
    assert_eq!(mapped, OperationOutcome::Completed(3));
}

/// Tracing: TC-011, FR-036-AC-5, FR-040-AC-2, Task-017.
#[test]
#[trace("TC-011", "FR-036-AC-5", "FR-040-AC-2")]
fn tc_011_partial_output_is_explicit_and_requires_a_retained_sibling() {
    let code = lookup(UNREGISTERED_CODE.code).expect("registered foundation code");
    let retained = ProtocolRefusal::builder(
        code,
        CatalogSelection::local(),
        ProtocolStage::Representation,
        axis(b"claim-local-construction"),
        missing_definition(b"definition/7"),
        usage(),
    )
    .expect("representation code and stage agree")
    .retaining_independent("first-row", vec!["second-row"]);

    assert_eq!(
        retained.refusal().partial_output(),
        PartialOutput::IndependentResultsRetained
    );
    assert_eq!(retained.independent_results(), &["first-row", "second-row"]);
    assert_ne!(
        retained.refusal(),
        &representation_refusal(b"claim-local-construction")
    );
}

/// Tracing: TC-011, FR-040-AC-2, Task-017.
#[test]
#[trace("TC-011", "FR-040-AC-2")]
fn tc_011_claim_outcomes_keep_sibling_rows_and_refusals_independent() {
    let outcomes = [
        ClaimOutcome::Canonical("conformance-row"),
        ClaimOutcome::Refused(representation_refusal(b"projection-row-construction")),
        ClaimOutcome::Canonical("monitorability-row"),
    ];
    assert_eq!(outcomes[0].canonical(), Some(&"conformance-row"));
    assert!(outcomes[0].refusal().is_none());
    assert!(outcomes[1].canonical().is_none());
    assert_eq!(
        outcomes[1]
            .refusal()
            .expect("claim-local refusal")
            .axis()
            .as_bytes(),
        b"projection-row-construction"
    );
    assert_eq!(outcomes[2].canonical(), Some(&"monitorability-row"));
}

/// Tracing: TC-011, FR-008-AC-2, FR-008-AC-7, Task-017.
#[test]
#[trace("TC-011", "FR-008-AC-2", "FR-008-AC-7")]
fn tc_011_rendering_neither_changes_identity_nor_invents_location() {
    let refusal = representation_refusal(b"missing-location");
    assert_eq!(
        refusal.location(),
        &Location::Unavailable(Unavailability::NotReportedBySource)
    );
    let rendered = refusal.rendered_as(Rendering {
        message: "unit-a:12:19".to_owned(),
        locale: "fr-FR".to_owned(),
        display_name: "unit-a".to_owned(),
        path: "unit-a:12:19".to_owned(),
    });
    assert_eq!(rendered, refusal);
    assert_eq!(rendered.location(), refusal.location());
    assert_eq!(rendered.axis(), refusal.axis());
}

/// Tracing: TC-011, FR-008-AC-1, Task-017.
#[test]
#[trace("TC-011", "FR-008-AC-1")]
fn tc_011_local_refusal_registry_remains_collision_free() {
    assert_eq!(audit(REGISTRY), Vec::new());
}

/// Tracing: TC-011, FR-008-AC-1, FR-036-AC-1, Task-017.
#[test]
#[trace("TC-011", "FR-008-AC-1", "FR-036-AC-1")]
fn tc_011_every_foundation_envelope_field_is_typed_and_readable() {
    let code = lookup(UNREGISTERED_CODE.code).expect("registered foundation code");
    let location = SourceLocation::reported(b"unit-a", 12, 19).expect("well-formed span");
    let refusal = ProtocolRefusal::builder(
        code,
        CatalogSelection::local(),
        ProtocolStage::Representation,
        axis(b"missing-definition"),
        missing_definition(b"definition/7"),
        usage(),
    )
    .expect("representation code and stage agree")
    .prerequisite(Prerequisite::new(
        ProtocolStage::Admission,
        PrerequisiteState::Established,
    ))
    .identity(AffectedIdentity::retained(b"producer/definition@7"))
    .located_at(location.clone())
    .rendering(Rendering {
        message: "missing definition".to_owned(),
        locale: "en-GB".to_owned(),
        display_name: "definition seven".to_owned(),
        path: "sources/a.quire".to_owned(),
    })
    .emitting_nothing();

    assert_eq!(refusal.code(), code);
    assert_eq!(refusal.domain(), RefusalDomain::Representation);
    assert_eq!(refusal.stage(), ProtocolStage::Representation);
    assert_eq!(refusal.axis().as_bytes(), b"missing-definition");
    assert_eq!(
        refusal.prerequisites(),
        &[Prerequisite::new(
            ProtocolStage::Admission,
            PrerequisiteState::Established,
        )]
    );
    assert_eq!(
        refusal.identity(),
        &AffectedIdentity::retained(b"producer/definition@7")
    );
    assert_eq!(refusal.location(), &Location::Available(location));
    assert_eq!(refusal.causes().len(), 1);
    assert_eq!(
        refusal
            .resource_usage()
            .protocol_record()
            .owner()
            .as_bytes(),
        b"agent-ix/quire-protocol"
    );
    assert_eq!(
        refusal
            .resource_usage()
            .protocol_record()
            .contract()
            .as_bytes(),
        b"quire.protocol/test-usage/1"
    );
    assert_eq!(
        refusal.resource_usage().protocol_record().measures()[0]
            .dimension()
            .as_bytes(),
        b"operations"
    );
    assert_eq!(
        refusal.resource_usage().protocol_record().measures()[0].consumed(),
        1
    );
    assert!(refusal.resource_usage().external_records().is_empty());
    assert_eq!(refusal.partial_output(), PartialOutput::NoneEmitted);
    assert_eq!(refusal.retry(), code.retry());
    assert_eq!(refusal.rendering().locale, "en-GB");
    assert_eq!(
        refusal.catalog().digest(),
        &CatalogDigest::Unavailable(CatalogDigestUnavailable::NotPublished)
    );
}

/// Tracing: TC-011, FR-008-AC-2, FR-037-AC-1, Task-017.
#[test]
#[trace("TC-011", "FR-008-AC-2", "FR-037-AC-1")]
fn tc_011_rendering_changes_neither_equality_hash_nor_retry() {
    let refusal = representation_refusal(b"missing-definition");
    let rendered = refusal.rendered_as(Rendering {
        message: "completely different".to_owned(),
        locale: "de-DE".to_owned(),
        display_name: "anderer Name".to_owned(),
        path: "anderer/pfad.quire".to_owned(),
    });
    assert_eq!(rendered, refusal);
    assert_eq!(hash_of(&rendered), hash_of(&refusal));
    assert_eq!(rendered.retry(), refusal.retry());
    assert_eq!(rendered.location(), refusal.location());
    assert_eq!(rendered.causes(), refusal.causes());
}

/// Tracing: TC-011, FR-008-AC-2, FR-014-AC-1, Task-017.
#[test]
#[trace("TC-011", "FR-008-AC-2", "FR-014-AC-1")]
fn tc_011_each_refusal_identity_component_changes_equality() {
    let code = lookup(UNREGISTERED_CODE.code).expect("registered foundation code");
    let build = || {
        ProtocolRefusal::builder(
            code,
            CatalogSelection::local(),
            ProtocolStage::Representation,
            axis(b"axis-a"),
            missing_definition(b"definition/7"),
            usage(),
        )
        .expect("representation code and stage agree")
    };
    let base = build().emitting_nothing();

    let other_code = ProtocolRefusal::builder(
        lookup(UNSUPPORTED_SOURCE_CONTRACT.code).expect("spec-named code is registered"),
        CatalogSelection::local(),
        ProtocolStage::Representation,
        axis(b"axis-a"),
        missing_definition(b"definition/7"),
        usage(),
    )
    .expect("representation code and stage agree")
    .emitting_nothing();
    assert_ne!(other_code, base, "code is part of identity");

    let other_catalog = ProtocolRefusal::builder(
        code,
        CatalogSelection::declared(
            RetainedBytes::new(b"owner/catalog"),
            RetainedBytes::new(b"2"),
            CatalogDigest::Unavailable(CatalogDigestUnavailable::NotDeclaredBySource),
        ),
        ProtocolStage::Representation,
        axis(b"axis-a"),
        missing_definition(b"definition/7"),
        usage(),
    )
    .expect("representation code and stage agree")
    .emitting_nothing();
    assert_ne!(other_catalog, base, "catalog is part of identity");

    let result_code = lookup("result.invalid-json").expect("result code is registered");
    let at_assembly = ProtocolRefusal::builder(
        result_code,
        CatalogSelection::local(),
        ProtocolStage::ResultAssembly,
        axis(b"axis-a"),
        missing_definition(b"definition/7"),
        usage(),
    )
    .expect("result assembly belongs to result")
    .emitting_nothing();
    let at_read = ProtocolRefusal::builder(
        result_code,
        CatalogSelection::local(),
        ProtocolStage::ResultRead,
        axis(b"axis-a"),
        missing_definition(b"definition/7"),
        usage(),
    )
    .expect("result read belongs to result")
    .emitting_nothing();
    assert_ne!(at_assembly, at_read, "stage is part of identity");

    let other_usage = ProtocolRefusal::builder(
        code,
        CatalogSelection::local(),
        ProtocolStage::Representation,
        axis(b"axis-a"),
        missing_definition(b"definition/7"),
        ResourceUsage::protocol("quire.protocol/test-usage/1", &[("operations", 2)]),
    )
    .expect("representation code and stage agree")
    .emitting_nothing();
    assert_ne!(other_usage, base, "resource usage is part of identity");
    assert_ne!(
        hash_of(&other_usage),
        hash_of(&base),
        "resource usage changes the identity hash"
    );

    assert_ne!(
        build()
            .prerequisite(Prerequisite::new(
                ProtocolStage::Admission,
                PrerequisiteState::Established,
            ))
            .emitting_nothing(),
        base,
        "prerequisites are part of identity"
    );
    assert_ne!(
        build()
            .identity(AffectedIdentity::retained(b"definition/8"))
            .emitting_nothing(),
        base,
        "affected identity is part of identity"
    );
    assert_ne!(
        build()
            .located_at(SourceLocation::reported(b"unit-a", 1, 2).expect("valid span"))
            .emitting_nothing(),
        base,
        "location is part of identity"
    );
    assert_ne!(
        build()
            .cause(ProtocolCause::ConflictingDefinition {
                definition: RetainedBytes::new(b"definition/7"),
                conflicting_definition: RetainedBytes::new(b"definition/8"),
            })
            .emitting_nothing(),
        base,
        "cause chain is part of identity"
    );
    assert_ne!(
        ProtocolRefusal::builder(
            code,
            CatalogSelection::local(),
            ProtocolStage::Representation,
            axis(b"axis-b"),
            missing_definition(b"definition/7"),
            usage(),
        )
        .expect("representation code and stage agree")
        .emitting_nothing(),
        base,
        "axis is part of identity"
    );
    assert_ne!(
        build().retaining_independent(1_u8, Vec::new()).refusal(),
        &base,
        "partial-output state is part of identity"
    );
}

/// Tracing: TC-011, FR-037-AC-3, FR-037-AC-5, NFR-008-AC-6, Task-017.
#[test]
#[trace("TC-011", "FR-037-AC-3", "FR-037-AC-5", "NFR-008-AC-6")]
fn tc_011_unavailability_and_prerequisite_states_do_not_collapse() {
    let code = lookup(UNREGISTERED_CODE.code).expect("registered foundation code");
    let locations = [
        Unavailability::NotReportedBySource,
        Unavailability::NotApplicableToStage,
        Unavailability::WithheldByDetailBudget,
    ];
    let location_refusal = |reason| {
        ProtocolRefusal::builder(
            code,
            CatalogSelection::local(),
            ProtocolStage::Representation,
            axis(b"axis-a"),
            missing_definition(b"definition/7"),
            usage(),
        )
        .expect("representation code and stage agree")
        .unavailable_location(reason)
        .emitting_nothing()
    };
    for (index, reason) in locations.iter().enumerate() {
        for (other_index, other) in locations.iter().enumerate() {
            assert_eq!(
                index == other_index,
                location_refusal(*reason) == location_refusal(*other)
            );
        }
    }

    let states = [
        PrerequisiteState::Established,
        PrerequisiteState::NotEstablished,
        PrerequisiteState::Unavailable,
    ];
    let prerequisite_refusal = |state| {
        ProtocolRefusal::builder(
            code,
            CatalogSelection::local(),
            ProtocolStage::Representation,
            axis(b"axis-a"),
            missing_definition(b"definition/7"),
            usage(),
        )
        .expect("representation code and stage agree")
        .prerequisite(Prerequisite::new(ProtocolStage::Admission, state))
        .emitting_nothing()
    };
    for (index, state) in states.iter().enumerate() {
        for (other_index, other) in states.iter().enumerate() {
            assert_eq!(
                index == other_index,
                prerequisite_refusal(*state) == prerequisite_refusal(*other)
            );
        }
    }
}

/// Tracing: TC-011, FR-037-AC-3, Task-017.
#[test]
#[trace("TC-011", "FR-037-AC-3")]
fn tc_011_source_location_refuses_inverted_spans() {
    assert_eq!(
        SourceLocation::reported(b"unit-a", 19, 12),
        Err(InvertedSpan { start: 19, end: 12 })
    );
    let empty = SourceLocation::reported(b"unit-a", 12, 12).expect("empty span is valid");
    assert_eq!((empty.start(), empty.end()), (12, 12));
    assert_eq!(empty.unit().as_bytes(), b"unit-a");
}

/// Tracing: TC-011, FR-014-AC-5, FR-016-AC-5, Task-017.
#[test]
#[trace("TC-011", "FR-014-AC-5", "FR-016-AC-5")]
fn tc_011_mapping_preserves_both_non_success_variants() {
    let refused: OperationOutcome<u8> =
        OperationOutcome::Refused(representation_refusal(b"invalid-representation"));
    let mapped_refused = refused.map(u16::from);
    assert!(matches!(mapped_refused, OperationOutcome::Refused(_)));

    let incomplete: OperationOutcome<u8> =
        OperationOutcome::ResourceIncomplete(resource_incomplete());
    let mapped_incomplete = incomplete.map(u16::from);
    assert!(matches!(
        mapped_incomplete,
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::InputByte
    ));
}

/// Tracing: TC-011, FR-008-AC-1, FR-008-AC-5, FR-036-AC-1, FR-039-AC-1.
#[test]
#[trace("TC-011", "FR-008-AC-1", "FR-008-AC-5", "FR-036-AC-1", "FR-039-AC-1")]
fn tc_011_result_constructors_retain_the_actual_local_stage() {
    let assembly = crate::result::Refusal::assembly(
        crate::result::ErrorCode::InvalidSelection,
        "assembly fixture",
        crate::result::Usage::default(),
        None,
    );
    let read = crate::result::Refusal::read(
        crate::result::ErrorCode::InvalidJson,
        "read fixture",
        crate::result::Usage::default(),
        None,
    );
    assert_eq!(assembly.envelope().stage(), ProtocolStage::ResultAssembly);
    assert_eq!(read.envelope().stage(), ProtocolStage::ResultRead);
    assert!(matches!(
        assembly.envelope().causes().declared_order(),
        [RefusalCause::Protocol(cause)] if cause.kind() == ProtocolCauseKind::IncompleteEvidence
    ));
    assert!(matches!(
        read.envelope().causes().declared_order(),
        [RefusalCause::Protocol(cause)] if cause.kind() == ProtocolCauseKind::IncompleteEvidence
    ));
    assert_eq!(
        assembly.envelope().prerequisites(),
        [Prerequisite::new(
            ProtocolStage::ClaimDispatch,
            PrerequisiteState::Established
        )]
    );
    assert_eq!(
        read.envelope().prerequisites(),
        [Prerequisite::new(
            ProtocolStage::ResultAssembly,
            PrerequisiteState::Unavailable
        )]
    );
}
