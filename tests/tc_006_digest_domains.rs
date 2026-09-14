// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-006/FR-001-AC-4: producer digest domains stay separate from the
//! consumer's RFC 8785 protocol-result domain, in both directions.
//!
//! These cases need no producer fixture, so they run in the default lane rather
//! than behind `QUIRE_PROTOCOL_A_FIXTURE`.

use ix_trace_rs::trace;
use quire_protocol::{
    bounds::inventory::{
        production_inventory, Budget, BudgetDimension, OperationOutcome, ResourceAccountant,
    },
    digest::{
        jcs::{canonicalize, JcsError, ResultDigest},
        producer::{DigestDomain, DigestSlot, OfferedDomain, ProducerDigestRef},
    },
    inventory::{
        axis::DivergedSelection, map::admit_discriminating_bounded, DigestClassification,
        ResultDigestRefusal, SelectedInventory,
    },
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
        }
    }

    fn inventory(&self) -> SelectedInventory {
        SelectedInventory::from_expected(&self.expected()).expect("selections canonicalise")
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

/// A producer reference offered where the JCS protocol-result digest belongs.
///
/// Tracing: TC-006, FR-001-AC-4.
#[test]
#[trace("TC-006", "FR-001-AC-4", "FR-015-AC-3")]
fn tc_006_producer_reference_in_the_result_slot_refuses() {
    let selections = Selections::new();
    let inventory = selections.inventory();
    let result = json!({ "outcome": "complete", "closure": "global" });
    let offered = selections.baseline.digest.to_string();
    let refusal = inventory
        .accept_result_digest(&result, &offered)
        .expect_err("a producer reference is not a protocol-result digest");
    match refusal {
        ResultDigestRefusal::CrossDomain(violation) => {
            assert_eq!(violation.slot, DigestSlot::ProtocolResult);
            assert_eq!(violation.selected, DigestDomain::VerificationJcs);
            assert_eq!(
                violation.offered,
                OfferedDomain::Identified(DigestDomain::RawArtifact)
            );
            assert_eq!(violation.value, offered);
        }
        other => panic!("expected a cross-domain refusal, got {other:?}"),
    }
}

/// A JCS protocol-result digest offered where a producer reference belongs.
///
/// Tracing: TC-001, TC-006, FR-001-AC-4.
#[test]
#[trace("TC-001", "TC-006", "FR-001-AC-4", "FR-015-AC-3")]
fn tc_006_result_digest_in_a_producer_slot_refuses() {
    let selections = Selections::new();
    let inventory = selections.inventory();
    let offered = inventory
        .result_domain_text(DigestSlot::Baseline)
        .expect("baseline is a selected slot");
    // The published vocabulary requires this domain to be spelled `sha256-jcs:`
    // and states that a raw `sha256` digest cannot substitute for it, so the
    // label itself separates the two domains. The hex payload is still the same
    // length and alphabet as the value it replaces, so nothing about the digest
    // *value* distinguishes them.
    assert!(offered.starts_with("sha256-jcs:"));
    assert!(!offered.starts_with("sha256:"));
    let producer_text = selections.baseline.digest.to_string();
    assert_eq!(
        offered.rsplit_once(':').expect("labelled").1.len(),
        producer_text.rsplit_once(':').expect("labelled").1.len()
    );
    match inventory.classify_offered(DigestSlot::Baseline, &offered) {
        DigestClassification::CrossDomain(violation) => {
            assert_eq!(violation.slot, DigestSlot::Baseline);
            assert_eq!(violation.selected, DigestDomain::RawArtifact);
            assert_eq!(
                violation.offered,
                OfferedDomain::Identified(DigestDomain::VerificationJcs)
            );
        }
        other => panic!("expected a cross-domain classification, got {other:?}"),
    }
}

/// The selected producer value itself is in domain, for every producer slot.
///
/// Tracing: TC-006, FR-001-AC-4.
#[test]
#[trace("TC-006", "FR-001-AC-4")]
fn tc_006_selected_producer_values_are_in_domain() {
    let selections = Selections::new();
    let inventory = selections.inventory();
    for (slot, selected) in [
        (DigestSlot::Contract, &selections.contract),
        (DigestSlot::Baseline, &selections.baseline),
        (DigestSlot::ProducerBinary, &selections.producer.binary),
    ] {
        assert_eq!(
            inventory.classify_offered(slot, &selected.digest.to_string()),
            DigestClassification::InSelectedDomain,
            "slot {slot}"
        );
    }
}

/// A wrong value inside the producer's own domain is divergence, not a domain
/// error. The two must not share a code, or the digest-domain guard would fire
/// for every ordinary mismatch and prove nothing.
///
/// Tracing: TC-006, FR-001-AC-4.
#[test]
#[trace("TC-006", "FR-001-AC-4")]
fn tc_006_same_domain_divergence_is_not_cross_domain() {
    let selections = Selections::new();
    let inventory = selections.inventory();
    let other = ByteDigest::of(b"some other producer artifact bytes").to_string();
    assert_eq!(
        inventory.classify_offered(DigestSlot::Baseline, &other),
        DigestClassification::Divergent
    );
}

/// An unselected algorithm label is a foreign domain, refused as such rather
/// than as a malformed value.
///
/// Tracing: TC-006, FR-001-AC-4.
#[test]
#[trace("TC-006", "FR-001-AC-4")]
fn tc_006_foreign_algorithm_label_is_cross_domain() {
    let selections = Selections::new();
    let inventory = selections.inventory();
    let body = selections
        .baseline
        .digest
        .to_string()
        .strip_prefix("sha256:")
        .expect("A spells producer references with the sha256 label")
        .to_owned();
    match inventory.classify_offered(DigestSlot::Baseline, &format!("sha512:{body}")) {
        DigestClassification::CrossDomain(violation) => assert_eq!(
            violation.offered,
            OfferedDomain::ForeignLabel("sha512".to_owned())
        ),
        other => panic!("expected a cross-domain classification, got {other:?}"),
    }
}

/// A slot the inventory holds no producer selection for says so, rather than
/// reporting the offered value as a divergence it never compared.
///
/// Tracing: TC-006, FR-001-AC-4.
#[test]
#[trace("TC-006", "FR-001-AC-4", "FR-015-AC-4")]
fn tc_006_unselected_slot_is_explicit() {
    let selections = Selections::new();
    let inventory = selections.inventory();
    assert_eq!(
        inventory.classify_offered(
            DigestSlot::ProtocolResult,
            &selections.baseline.digest.to_string()
        ),
        DigestClassification::UnselectedSlot
    );
    assert_eq!(
        inventory.result_domain_text(DigestSlot::ProtocolResult),
        None
    );
}

/// A value with no algorithm label is its own outcome, not an empty label.
///
/// Tracing: TC-006, FR-001-AC-4.
#[test]
#[trace("TC-006", "FR-001-AC-4", "FR-015-AC-4")]
fn tc_006_unlabelled_value_is_cross_domain() {
    let selections = Selections::new();
    let inventory = selections.inventory();
    match inventory.classify_offered(DigestSlot::Baseline, "6f9cb8bf8521ca51") {
        DigestClassification::CrossDomain(violation) => {
            assert_eq!(violation.offered, OfferedDomain::Unlabelled);
        }
        other => panic!("expected a cross-domain classification, got {other:?}"),
    }
}

/// A producer reference is copied exactly and cannot be built for the
/// consumer's result domain at all.
///
/// Tracing: TC-006, FR-006-AC-4.
#[test]
#[trace("TC-006", "FR-006-AC-4", "FR-015-AC-2", "FR-031-AC-6")]
fn tc_006_producer_reference_is_opaque_and_copied_exactly() {
    let text = "sha256:0000000000000000000000000000000000000000000000000000000000000001";
    let reference = ProducerDigestRef::copy_exact(DigestDomain::RawArtifact, text)
        .expect("producer-source is a producer-owned domain");
    assert_eq!(reference.as_producer_text(), text);
    assert_eq!(reference.domain(), DigestDomain::RawArtifact);
    assert!(
        ProducerDigestRef::copy_exact(DigestDomain::VerificationJcs, text).is_none(),
        "a protocol-result digest must not be constructible as a producer reference"
    );
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
        ResultDigest::of_result(&left).expect("canonicalises"),
        ResultDigest::of_result(&right).expect("canonicalises")
    );
    let digest = ResultDigest::of_result(&left)
        .expect("canonicalises")
        .text();
    assert!(digest.starts_with("sha256-jcs:"));
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
/// published `Producer` and version-2 families at this pin. Retaining the
/// complete `Error` makes the compulsory catch-all typed and future-safe.
///
/// Tracing: TC-001, FR-001-AC-2, Task-001.
#[test]
#[trace("TC-001", "FR-001-AC-2", "Task-001")]
fn tc_001_mapping_retains_every_published_source_error() {
    use quire_protocol::inventory::map::classify_source;
    use quire_protocol::{v2, Error, Invalid, NumberError, Unsupported};
    use quire_spec_language::linking::composed::producer::ProducerModelRefusal;
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
    const UNSUPPORTED: [Unsupported; 7] = [
        Unsupported::Wire,
        Unsupported::Feature,
        Unsupported::Definition,
        Unsupported::Profile,
        Unsupported::ProducerCorrespondence,
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
        Error::Producer(ProducerModelRefusal::Interface),
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
