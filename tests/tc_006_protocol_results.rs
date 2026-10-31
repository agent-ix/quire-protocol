// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-006: canonical result encoding and typed refusal mapping.
//!
//! These cases need no producer fixture, so they run in the default lane rather
//! than behind `QUIRE_PROTOCOL_A_FIXTURE`.

use ix_trace_rs::trace;
use quire_protocol::{
    bounds::inventory::{
        production_inventory, Budget, BudgetDimension, OperationOutcome, ResourceAccountant,
    },
    digest::jcs::{canonicalize, JcsError},
    inventory::{axis::DivergedSelection, map::admit_discriminating_bounded},
    wire, ByteDigest, Expected, Limits,
};
use serde_json::json;

const BASELINE_BYTES: &[u8] = b"the exact selected baseline artifact bytes";
const CONTRACT_BYTES: &[u8] = b"the exact selected contract artifact bytes";
const BINARY_BYTES: &[u8] = b"the exact selected producer binary bytes";
const PACKAGE_BYTES: &[u8] = b"the exact selected linked package bytes";

fn revision(namespace: &str, value: &str) -> wire::Revision {
    wire::Revision {
        namespace: namespace.to_owned(),
        value: value.to_owned(),
    }
}

fn reference(identity: &str, kind: wire::ArtifactKind, bytes: &[u8]) -> wire::ArtifactRef {
    wire::ArtifactRef {
        ref_version: "ix.artifact-ref/3-draft".to_owned(),
        kind,
        authority: "ix://agent-ix/quire-protocol".to_owned(),
        identity: identity.to_owned(),
        revision: revision("test-selection", "1"),
        digest: ByteDigest::of(bytes),
        wire: wire::Wire {
            identity: "text/markdown".to_owned(),
            version: "1".to_owned(),
        },
    }
}

/// One complete independently selected acceptance context.
struct Selections {
    artifact: wire::ArtifactRef,
    contract: wire::ArtifactRef,
    baseline: wire::ArtifactRef,
    producer: wire::Producer,
    language: wire::Language,
}

impl Selections {
    fn new() -> Self {
        Self {
            artifact: reference(
                "selected-package",
                wire::ArtifactKind::LinkedPackage,
                PACKAGE_BYTES,
            ),
            contract: reference(
                "quire.compiled-protocol/1",
                wire::ArtifactKind::Source,
                CONTRACT_BYTES,
            ),
            baseline: reference("ix:native", wire::ArtifactKind::Source, BASELINE_BYTES),
            producer: wire::Producer {
                implementation: "selected-producer".to_owned(),
                revision: revision("crate-version", "0.1.0"),
                binary: reference(
                    "selected-binary",
                    wire::ArtifactKind::GeneratedArtifact,
                    BINARY_BYTES,
                ),
            },
            language: wire::Language {
                identity: "ix:native".to_owned(),
                edition: "1-draft".to_owned(),
            },
        }
    }

    fn expected(&self) -> Expected<'_> {
        Expected {
            artifact: &self.artifact,
            contract: &self.contract,
            baseline: &self.baseline,
            producer: &self.producer,
            language: &self.language,
            sources: &[],
            dependencies: &[],
            models: &[],
            domain_packages: &[],
        }
    }
}

#[test]
#[trace("TC-009", "NFR-002-AC-1", "FR-016-AC-2", "FR-016-AC-4")]
fn tc_009_admission_charges_all_input_bytes_before_decode() {
    let selections = Selections::new();
    let bytes = b"{}";
    let inventory = production_inventory().expect("production inventory");
    let one_short = Budget::nothing_granted()
        .grant(BudgetDimension::InputByte, bytes.len() - 1)
        .expect("positive one-short grant");
    let mut one_short_resources = ResourceAccountant::new(one_short, &inventory);
    let incomplete = admit_discriminating_bounded(
        bytes,
        &selections.expected(),
        Limits::default(),
        &mut one_short_resources,
    );
    assert!(matches!(
        incomplete,
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::InputByte
                && value.consumed() == 0
                && value.requested() == bytes.len()
    ));
    assert_eq!(
        one_short_resources
            .usage()
            .consumed(BudgetDimension::InputByte),
        0
    );

    let mut owner_resources = ResourceAccountant::new(Budget::nothing_granted(), &inventory);
    let owner_short = Limits {
        payload_bytes: bytes.len() - 1,
        ..Limits::default()
    };
    let owner_incomplete = admit_discriminating_bounded(
        bytes,
        &selections.expected(),
        owner_short,
        &mut owner_resources,
    );
    assert!(matches!(
        owner_incomplete,
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::InputByte
                && value.granted() == bytes.len() - 1
                && value.requested() == bytes.len()
    ));
    assert_eq!(
        owner_resources.usage().consumed(BudgetDimension::InputByte),
        0
    );

    let exact = Budget::nothing_granted()
        .grant(BudgetDimension::InputByte, bytes.len())
        .expect("positive exact grant");
    let mut exact_resources = ResourceAccountant::new(exact, &inventory);
    let reached_reader = admit_discriminating_bounded(
        bytes,
        &selections.expected(),
        Limits::default(),
        &mut exact_resources,
    );
    assert!(matches!(reached_reader, OperationOutcome::DomainError(_)));
    assert_eq!(
        exact_resources.usage().consumed(BudgetDimension::InputByte),
        bytes.len()
    );

    let one_allocation = Budget::nothing_granted()
        .grant(BudgetDimension::AccountedAllocation, 1)
        .expect("one admitted-package allocation");
    let mut allocation_resources = ResourceAccountant::new(one_allocation, &inventory);
    assert!(matches!(
        admit_discriminating_bounded(
            bytes,
            &selections.expected(),
            Limits::default(),
            &mut allocation_resources,
        ),
        OperationOutcome::DomainError(_)
    ));
    assert!(matches!(
        admit_discriminating_bounded(
            bytes,
            &selections.expected(),
            Limits::default(),
            &mut allocation_resources,
        ),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::AccountedAllocation
                && value.consumed() == 1
                && value.requested() == 1
    ));
}

/// RFC 8785 canonicalisation is member-order invariant, and refuses a number
/// rather than approximating ECMAScript number serialisation.
///
/// Tracing: TC-006, FR-006-AC-4.
#[test]
#[trace(
    "TC-006",
    "TC-008",
    "FR-006-AC-4",
    "FR-031-AC-4",
    "NFR-001-AC-1",
    "NFR-001-AC-3"
)]
fn tc_006_jcs_is_member_order_invariant_and_refuses_numbers() {
    let left = json!({ "b": "two", "a": "one", "c": [true, null] });
    let right = json!({ "c": [true, null], "a": "one", "b": "two" });
    assert_eq!(
        canonicalize(&left).expect("string members canonicalise"),
        canonicalize(&right).expect("string members canonicalise")
    );
    assert_eq!(
        canonicalize(&json!({ "n": 1 })),
        Err(JcsError::UnsupportedNumber("1".to_owned()))
    );
}

/// No two selection axes share a code.
///
/// Tracing: TC-006, FR-001-AC-2.
#[test]
#[trace("TC-006", "FR-001-AC-2")]
fn tc_006_no_two_selection_axes_share_a_code() {
    let axes = DivergedSelection::ALL;
    for (index, left) in axes.iter().enumerate() {
        for right in &axes[index + 1..] {
            assert_ne!(left, right);
            assert_ne!(left.as_str(), right.as_str());
        }
    }
}

/// Every A refusal published at the pinned revision survives B's mapping as
/// the exact typed source value and retains A's stable code.
///
/// A's enums are `#[non_exhaustive]`. A debug-string catch-all therefore loses
/// type identity exactly when A adds a family, and it already loses the
/// published version-2 family at this pin. Retaining the complete `Error`
/// makes the compulsory catch-all typed and future-safe. `Error::Producer`
/// (and `Target::Producer`) no longer exist as of this pin: QSL dropped the
/// Producer 1.2/Filament correspondence adapter entirely (QSL #131/#137), so
/// this enumeration no longer lists it either. `Unsupported::ProducerCorrespondence`
/// went with the wire `Correspondence` record in QSL-63 (FR-042).
///
/// Tracing: TC-001, FR-001-AC-2, Task-001.
#[test]
#[trace("TC-001", "FR-001-AC-2", "Task-001")]
fn tc_001_mapping_retains_every_published_source_error() {
    use quire_protocol::inventory::map::classify_source;
    use quire_protocol::{v2, Error, Invalid, NumberError, Unsupported};
    use quire_spec_language::protocol_artifact::{Dimension, Exhaustion};

    const INVALID: [Invalid; 26] = [
        Invalid::Selection,
        Invalid::Seal,
        Invalid::Name,
        Invalid::StructuralInteger,
        Invalid::WrongNumericKind,
        Invalid::NumericDomain,
        Invalid::Inventory,
        Invalid::Order,
        Invalid::Duplicate,
        Invalid::Dependency,
        Invalid::Definition,
        Invalid::Model,
        Invalid::ForeignLocus,
        Invalid::Locus,
        Invalid::Reference,
        Invalid::Owner,
        Invalid::Scope,
        Invalid::Type,
        Invalid::Profile,
        Invalid::Call,
        Invalid::Binding,
        Invalid::Control,
        Invalid::Cycle,
        Invalid::Feature,
        Invalid::Canonical,
        Invalid::Encoding,
    ];
    const UNSUPPORTED: [Unsupported; 6] = [
        Unsupported::Wire,
        Unsupported::Feature,
        Unsupported::Definition,
        Unsupported::Profile,
        Unsupported::FamilyProof,
        Unsupported::Export,
    ];

    for code in INVALID {
        let source = Error::Invalid(code);
        let refusal = classify_source(source.clone());
        assert_eq!(refusal.source_error(), Some(&source));
        assert_eq!(refusal.source_code(), Some(source.code()));
    }
    for code in UNSUPPORTED {
        let source = Error::Unsupported(code);
        let refusal = classify_source(source.clone());
        assert_eq!(refusal.source_error(), Some(&source));
        assert_eq!(refusal.source_code(), Some(source.code()));
    }
    for source in [
        Error::Allocation,
        Error::Json { line: 1, column: 2 },
        Error::Numeric(NumberError::NonPositiveDenominator),
        Error::V2(v2::Refusal::OfferOrder),
        Error::Incomplete(Exhaustion {
            dimension: Dimension::Entries,
            used: 8,
            requested: 1,
            limit: 8,
            locus: None,
        }),
    ] {
        let refusal = classify_source(source.clone());
        assert_eq!(refusal.source_error(), Some(&source));
        assert_eq!(refusal.source_code(), Some(source.code()));
    }
}
