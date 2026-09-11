// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-006/FR-001-AC-4: producer digest domains stay separate from the
//! consumer's RFC 8785 protocol-result domain, in both directions.
//!
//! These cases need no producer fixture, so they run in the default lane rather
//! than behind `QUIRE_PROTOCOL_A_FIXTURE`.

use ix_trace_rs::trace;
use quire_protocol::{
    digest::{
        jcs::{canonicalize, JcsError, ResultDigest},
        producer::{DigestDomain, DigestSlot, OfferedDomain, ProducerDigestRef},
    },
    inventory::{
        axis::SelectionAxis, DigestClassification, ResultDigestRefusal, SelectedInventory,
    },
    wire, ByteDigest, Expected,
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

/// A producer reference offered where the JCS protocol-result digest belongs.
///
/// Tracing: TC-006, FR-001-AC-4.
#[test]
#[trace("TC-006", "FR-001-AC-4")]
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
            assert_eq!(violation.selected, DigestDomain::ProtocolResultJcs);
            assert_eq!(
                violation.offered,
                OfferedDomain::Identified(DigestDomain::ProducerArtifactBytes)
            );
            assert_eq!(violation.value, offered);
        }
        other => panic!("expected a cross-domain refusal, got {other:?}"),
    }
}

/// A JCS protocol-result digest offered where a producer reference belongs.
///
/// Tracing: TC-006, FR-001-AC-4.
#[test]
#[trace("TC-006", "FR-001-AC-4")]
fn tc_006_result_digest_in_a_producer_slot_refuses() {
    let selections = Selections::new();
    let inventory = selections.inventory();
    let offered = inventory
        .result_domain_text(DigestSlot::Baseline)
        .expect("baseline is a selected slot");
    // Well formed in every syntactic sense: the same algorithm label, the same
    // length, the same alphabet as the value it replaces.
    assert!(offered.starts_with("sha256:"));
    assert_eq!(offered.len(), selections.baseline.digest.to_string().len());
    match inventory.classify_offered(DigestSlot::Baseline, &offered) {
        DigestClassification::CrossDomain(violation) => {
            assert_eq!(violation.slot, DigestSlot::Baseline);
            assert_eq!(violation.selected, DigestDomain::ProducerArtifactBytes);
            assert_eq!(
                violation.offered,
                OfferedDomain::Identified(DigestDomain::ProtocolResultJcs)
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
#[trace("TC-006", "FR-001-AC-4")]
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
#[trace("TC-006", "FR-001-AC-4")]
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
#[trace("TC-006", "FR-006-AC-4")]
fn tc_006_producer_reference_is_opaque_and_copied_exactly() {
    let text = "sha256:0000000000000000000000000000000000000000000000000000000000000001";
    let reference = ProducerDigestRef::copy_exact(DigestDomain::ProducerSource, text)
        .expect("producer-source is a producer-owned domain");
    assert_eq!(reference.as_producer_text(), text);
    assert_eq!(reference.domain(), DigestDomain::ProducerSource);
    assert!(
        ProducerDigestRef::copy_exact(DigestDomain::ProtocolResultJcs, text).is_none(),
        "a protocol-result digest must not be constructible as a producer reference"
    );
}

/// RFC 8785 canonicalisation is member-order invariant, and refuses a number
/// rather than approximating ECMAScript number serialisation.
///
/// Tracing: TC-006, FR-006-AC-4.
#[test]
#[trace("TC-006", "FR-006-AC-4")]
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
    let axes = SelectionAxis::ALL;
    for (index, left) in axes.iter().enumerate() {
        for right in &axes[index + 1..] {
            assert_ne!(left, right);
            assert_ne!(left.as_str(), right.as_str());
        }
    }
}

/// Every A refusal published at the pinned revision is named by B's mapping.
///
/// A's enums are `#[non_exhaustive]`, so the mapping is forced to carry one
/// final arm per enum. That arm produces `UnclassifiedSource`, and this test
/// fails if any currently published A variant reaches it — so the arm can only
/// ever absorb a genuinely new A variant, loudly, never an existing one.
///
/// Tracing: TC-006, FR-008-AC-1.
#[test]
#[trace("TC-006", "FR-008-AC-1")]
fn tc_006_classification_names_every_known_source_variant() {
    use quire_protocol::inventory::map::{classify_source, AdmissionRefusal};
    use quire_protocol::{Error, Invalid, Unsupported};

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
        let refusal = classify_source(&Error::Invalid(code));
        assert!(
            !matches!(refusal, AdmissionRefusal::UnclassifiedSource { .. }),
            "{code:?} is unnamed"
        );
    }
    for code in UNSUPPORTED {
        let refusal = classify_source(&Error::Unsupported(code));
        assert!(
            !matches!(refusal, AdmissionRefusal::UnclassifiedSource { .. }),
            "{code:?} is unnamed"
        );
    }
    assert!(!matches!(
        classify_source(&Error::Allocation),
        AdmissionRefusal::UnclassifiedSource { .. }
    ));
    assert!(!matches!(
        classify_source(&Error::Json { line: 1, column: 2 }),
        AdmissionRefusal::UnclassifiedSource { .. }
    ));
}
