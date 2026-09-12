// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-011 (partial, chunk C-01): the B-owned typed refusal envelope.
//!
//! Covers FR-008-AC-1 (every field present and typed), FR-008-AC-2 (rendering
//! cannot change identity), FR-008-AC-7 (an unavailable location stays
//! unavailable) and the structural half of FR-008-AC-4 and FR-008-AC-8 (the
//! partial-output state is a separate field; a withheld detail cannot erase a
//! cause). The remaining criteria need the selected-catalog validation of
//! chunk C-02 (AC-3, AC-6) and the cause taxonomy of chunk C-03 (AC-5, and the
//! policy half of AC-4 and AC-8).
//!
//! It also carries the **repo-wide refusal-code audit**. The code namespace is
//! a catalog rather than one crate-wide enum (see `refusal`'s module docs), so
//! the uniqueness property a single enum would have given for free is proved
//! here instead: no two chunks may mint the same spelling, claim the same
//! reserved root, or emit a code they did not register. Every one of those
//! checks is exercised twice — once against the real registry, which must be
//! clean, and once against a deliberately defective registry, which must
//! produce that exact defect and no other. Without the second half the audit
//! would pass simply because the registry is small today.

use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use ix_trace_rs::trace;
use quire_protocol::refusal::catalog::{
    audit, is_code_shape, lookup, lookup_in, malformation, unregistered_literals, CatalogDefect,
    CatalogDigest, CatalogDigestUnavailable, CatalogSelection, CodeEntry, CodeModule,
    MalformedCode, Provenance, RefusalCode, RetainedBytes, UnregisteredLiteral, ENVELOPE_CODES,
    EXEMPT_LITERALS, REGISTRY, UNREGISTERED_CODE, UNSUPPORTED_SOURCE_CONTRACT,
};
use quire_protocol::refusal::{
    AffectedIdentity, Cause, CauseChain, CauseDetail, InvertedSpan, Location, OriginatingStage,
    PartialOutput, Prerequisite, PrerequisiteState, ProtocolRefusal, RefusalDomain, Rendering,
    RetryDecision, SourceLocation, Unavailability,
};

// ---------------------------------------------------------------------------
// Shared material
// ---------------------------------------------------------------------------

/// The one code chunk C-01 mints, as a handle.
fn unregistered_code() -> RefusalCode {
    lookup(UNREGISTERED_CODE.code).expect("C-01's own code is registered")
}

/// A refusal with every FR-008-AC-1 field populated.
fn fully_populated() -> ProtocolRefusal {
    ProtocolRefusal::builder(
        unregistered_code(),
        CatalogSelection::local(),
        OriginatingStage::SourceRecognition,
    )
    .prerequisite(Prerequisite::new(
        OriginatingStage::IdentityLinking,
        PrerequisiteState::Established,
    ))
    .identity(AffectedIdentity::retained(b"producer/definition@7"))
    .located_at(SourceLocation::reported(b"unit-a", 12, 19).expect("12..19 is a well-formed span"))
    .cause(Cause::new(unregistered_code()))
    .rendering(Rendering {
        message: "the offered code is not in the consumer catalog".to_owned(),
        locale: "en-GB".to_owned(),
        display_name: "definition seven".to_owned(),
        path: "sources/a.quire".to_owned(),
    })
    .emitting_nothing()
}

fn hash_of<T: Hash>(value: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

// ---------------------------------------------------------------------------
// FR-008-AC-1 — every field is typed and present
// ---------------------------------------------------------------------------

#[test]
#[trace("TC-011", "FR-008-AC-1")]
fn tc_011_domain_is_typed_not_parsed() {
    // Five domains, five distinct discriminants, five distinct spellings. The
    // spellings are one-way: `RefusalDomain` has no `FromStr` and no
    // `From<&str>`, proved by the `compile_fail` doctest on the type, so no
    // equality path in this crate can run through text.
    assert_eq!(
        RefusalDomain::ALL.len(),
        5,
        "FR-008 names exactly five domains"
    );
    for (index, domain) in RefusalDomain::ALL.iter().enumerate() {
        for (other_index, other) in RefusalDomain::ALL.iter().enumerate() {
            assert_eq!(
                index == other_index,
                domain == other,
                "{domain:?} and {other:?} must be distinguishable by discriminant"
            );
            assert_eq!(
                index == other_index,
                domain.as_str() == other.as_str(),
                "{domain:?} and {other:?} must have distinct stable spellings"
            );
        }
    }

    // The domain of a refusal is the domain its code is registered under, so
    // the two cannot disagree.
    let refusal = fully_populated();
    assert_eq!(refusal.domain(), RefusalDomain::Representation);
    assert_eq!(refusal.domain(), refusal.code().domain());
}

#[test]
#[trace("TC-011", "FR-008-AC-1")]
fn tc_011_every_envelope_field_is_readable_without_parsing() {
    let refusal = fully_populated();

    assert_eq!(refusal.code().as_str(), "refusal.code.unregistered");
    assert_eq!(
        refusal.catalog().identity().as_bytes(),
        b"agent-ix/quire-protocol/refusal-catalog"
    );
    assert_eq!(refusal.catalog().revision().as_bytes(), b"1");
    assert_eq!(
        refusal.catalog().digest(),
        &CatalogDigest::Unavailable(CatalogDigestUnavailable::NotPublished),
        "B's own catalog has no published digest, and inventing one is forbidden"
    );
    assert_eq!(refusal.stage(), OriginatingStage::SourceRecognition);
    assert_eq!(
        refusal.prerequisites(),
        &[Prerequisite::new(
            OriginatingStage::IdentityLinking,
            PrerequisiteState::Established
        )]
    );
    assert_eq!(
        refusal.identity(),
        &AffectedIdentity::retained(b"producer/definition@7"),
        "the producer's identity bytes are retained unchanged"
    );
    assert_eq!(
        refusal.location(),
        &Location::Available(
            SourceLocation::reported(b"unit-a", 12, 19).expect("12..19 is a well-formed span")
        )
    );
    assert_eq!(refusal.causes().len(), 1);
    assert_eq!(refusal.partial_output(), PartialOutput::NoneEmitted);
    assert_eq!(refusal.retry(), RetryDecision::Never);
}

#[test]
#[trace("TC-011", "FR-008-AC-1")]
fn tc_011_prerequisite_states_are_three_distinct_facts() {
    // Comparing the three fieldless variants to each other proves only that
    // `derive(PartialEq)` exists. Each state is routed through a real envelope
    // instead, so the test fails if a state stops reaching the envelope's
    // identity — which is the fact FR-008 actually needs.
    let refusal_with = |state| {
        ProtocolRefusal::builder(
            unregistered_code(),
            CatalogSelection::local(),
            OriginatingStage::SourceRecognition,
        )
        .prerequisite(Prerequisite::new(OriginatingStage::IdentityLinking, state))
        .emitting_nothing()
    };

    let states = [
        PrerequisiteState::Established,
        PrerequisiteState::NotEstablished,
        PrerequisiteState::Unavailable,
    ];
    for (position, state) in states.iter().enumerate() {
        let refusal = refusal_with(*state);
        assert_eq!(
            refusal.prerequisites(),
            &[Prerequisite::new(OriginatingStage::IdentityLinking, *state)],
            "the state is readable back without parsing"
        );
        for other in &states[..position] {
            assert_ne!(
                refusal,
                refusal_with(*other),
                "{state:?} and {other:?} must not produce one refusal identity"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// FR-008-AC-2 — rendering cannot change identity
// ---------------------------------------------------------------------------

#[test]
#[trace("TC-011", "FR-008-AC-2")]
fn tc_011_rendering_does_not_change_identity() {
    let refusal = fully_populated();
    let renderings = [
        Rendering::default(),
        Rendering {
            message: "totally different wording".to_owned(),
            locale: "fr-FR".to_owned(),
            display_name: "septième définition".to_owned(),
            path: "autre/chemin.quire".to_owned(),
        },
        Rendering {
            message: "unit-a:12:19".to_owned(),
            locale: "de-DE".to_owned(),
            display_name: "unit-a".to_owned(),
            path: "unit-a".to_owned(),
        },
    ];

    for rendering in renderings {
        let rendered = refusal.rendered_as(rendering);
        assert_eq!(rendered, refusal, "equality must ignore rendering");
        assert_eq!(
            hash_of(&rendered),
            hash_of(&refusal),
            "hashing must ignore rendering"
        );
        assert_eq!(rendered.code(), refusal.code());
        assert_eq!(rendered.stage(), refusal.stage());
        assert_eq!(rendered.retry(), refusal.retry());
        assert_eq!(rendered.location(), refusal.location());
        assert_eq!(rendered.causes(), refusal.causes());
    }
}

#[test]
#[trace("TC-011", "FR-008-AC-2")]
fn tc_011_identity_still_changes_when_a_typed_field_changes() {
    // The guard on the test above: equality that ignored everything would pass
    // it just as well. A one-field typed change must be observable.
    let refusal = fully_populated();

    let other_stage = ProtocolRefusal::builder(
        unregistered_code(),
        CatalogSelection::local(),
        OriginatingStage::ResourceExecution,
    )
    .prerequisite(Prerequisite::new(
        OriginatingStage::IdentityLinking,
        PrerequisiteState::Established,
    ))
    .identity(AffectedIdentity::retained(b"producer/definition@7"))
    .located_at(SourceLocation::reported(b"unit-a", 12, 19).expect("12..19 is a well-formed span"))
    .cause(Cause::new(unregistered_code()))
    .emitting_nothing();
    assert_ne!(other_stage, refusal, "the stage is part of identity");

    let other_identity = ProtocolRefusal::builder(
        unregistered_code(),
        CatalogSelection::local(),
        OriginatingStage::SourceRecognition,
    )
    .prerequisite(Prerequisite::new(
        OriginatingStage::IdentityLinking,
        PrerequisiteState::Established,
    ))
    .identity(AffectedIdentity::retained(b"producer/definition@8"))
    .located_at(SourceLocation::reported(b"unit-a", 12, 19).expect("12..19 is a well-formed span"))
    .cause(Cause::new(unregistered_code()))
    .emitting_nothing();
    assert_ne!(other_identity, refusal, "the identity is part of identity");

    let other_catalog = ProtocolRefusal::builder(
        unregistered_code(),
        CatalogSelection::declared(
            RetainedBytes::new(b"some/other-catalog"),
            RetainedBytes::new(b"3"),
            CatalogDigest::Unavailable(CatalogDigestUnavailable::NotDeclaredBySource),
        ),
        OriginatingStage::SourceRecognition,
    )
    .prerequisite(Prerequisite::new(
        OriginatingStage::IdentityLinking,
        PrerequisiteState::Established,
    ))
    .identity(AffectedIdentity::retained(b"producer/definition@7"))
    .located_at(SourceLocation::reported(b"unit-a", 12, 19).expect("12..19 is a well-formed span"))
    .cause(Cause::new(unregistered_code()))
    .emitting_nothing();
    assert_ne!(
        other_catalog, refusal,
        "the selected catalog is part of identity"
    );
}

// ---------------------------------------------------------------------------
// FR-008-AC-7 — an unavailable location stays unavailable
// ---------------------------------------------------------------------------

#[test]
#[trace("TC-011", "FR-008-AC-7")]
fn tc_011_absent_location_stays_absent() {
    // Everything a location could be reconstructed from is present and
    // deliberately span-shaped: a display name, a path, and a message that
    // reads exactly like `unit:line:column`. None of it is a location.
    let refusal = ProtocolRefusal::builder(
        unregistered_code(),
        CatalogSelection::local(),
        OriginatingStage::SourceRecognition,
    )
    .rendering(Rendering {
        message: "unit-a:12:19: offered code not in catalog".to_owned(),
        locale: "en-GB".to_owned(),
        display_name: "unit-a".to_owned(),
        path: "unit-a".to_owned(),
    })
    .emitting_nothing();

    assert_eq!(
        refusal.location(),
        &Location::Unavailable(Unavailability::NotReportedBySource),
        "no source location was reported, so none exists"
    );
    assert!(!refusal.location().is_available());

    // The display material is retained — it is just not a location.
    assert_eq!(refusal.rendering().display_name, "unit-a");
    assert_eq!(refusal.rendering().path, "unit-a");

    // Re-rendering with even more span-shaped text promotes nothing.
    let rerendered = refusal.rendered_as(Rendering {
        message: "unit-a:12:19".to_owned(),
        locale: "en-GB".to_owned(),
        display_name: "unit-a:12:19".to_owned(),
        path: "unit-a:12:19".to_owned(),
    });
    assert_eq!(rerendered.location(), refusal.location());
}

#[test]
#[trace("TC-011", "FR-008-AC-7")]
fn tc_011_unavailability_reasons_are_three_distinct_facts() {
    // A withheld location and an unreported one are not the same refusal, so
    // they may not compare equal.
    let build = |reason| {
        ProtocolRefusal::builder(
            unregistered_code(),
            CatalogSelection::local(),
            OriginatingStage::SourceRecognition,
        )
        .unavailable_location(reason)
        .emitting_nothing()
    };
    let reasons = [
        Unavailability::NotReportedBySource,
        Unavailability::NotApplicableToStage,
        Unavailability::WithheldByDetailBudget,
    ];
    for (index, reason) in reasons.iter().enumerate() {
        for (other_index, other) in reasons.iter().enumerate() {
            assert_eq!(index == other_index, build(*reason) == build(*other));
        }
    }
}

// ---------------------------------------------------------------------------
// Cause chain — declared order, retained existence
// ---------------------------------------------------------------------------

/// Three test-only codes whose declared order is deliberately not their
/// alphabetical order, so a chain that sorted would be caught.
const ORDERED_CAUSES: [CodeEntry; 3] = [
    CodeEntry {
        provenance: Provenance::Minted,
        code: "cause.zeta.one",
        domain: RefusalDomain::Assessment,
        retry: RetryDecision::Never,
        summary: "first declared cause",
    },
    CodeEntry {
        provenance: Provenance::Minted,
        code: "cause.alpha.two",
        domain: RefusalDomain::Assessment,
        retry: RetryDecision::Never,
        summary: "second declared cause",
    },
    CodeEntry {
        provenance: Provenance::Minted,
        code: "cause.mu.three",
        domain: RefusalDomain::Assessment,
        retry: RetryDecision::Never,
        summary: "third declared cause",
    },
];

/// The ordering codes as a registry, so the test reaches them the way
/// production code does — through a lookup — rather than through a constructor
/// that would accept an entry nothing registered.
const ORDERED_CAUSE_MODULE: CodeModule = CodeModule {
    root: "cause",
    owner: "C-01-test",
    module_path: "tests::tc_011_typed_refusals",
    entries: &ORDERED_CAUSES,
};

const ORDERED_CAUSE_REGISTRY: &[CodeModule] = &[ORDERED_CAUSE_MODULE];

/// One ordering code, by spelling.
fn ordered_cause(code: &str) -> RefusalCode {
    lookup_in(ORDERED_CAUSE_REGISTRY, code).expect("a registered test cause code")
}

#[test]
#[trace("TC-011", "FR-008-AC-1")]
fn tc_011_cause_order_is_declared_not_sorted() {
    let mut chain = CauseChain::new();
    for entry in &ORDERED_CAUSES {
        chain.push(Cause::new(ordered_cause(entry.code)));
    }

    let observed: Vec<&str> = chain
        .declared_order()
        .iter()
        .map(|cause| cause.code().as_str())
        .collect();
    assert_eq!(
        observed,
        vec!["cause.zeta.one", "cause.alpha.two", "cause.mu.three"],
        "causes are retained in declared order"
    );

    let mut sorted = observed.clone();
    sorted.sort_unstable();
    assert_ne!(
        observed, sorted,
        "the fixture must not be accidentally sorted, or this proves nothing"
    );
    assert_eq!(
        chain.position_of(ordered_cause(ORDERED_CAUSES[2].code)),
        Some(2)
    );
}

#[test]
#[trace("TC-011", "FR-008-AC-8")]
fn tc_011_withheld_detail_cannot_erase_a_cause() {
    // A zero diagnostic-detail budget withholds the detail. The cause itself
    // survives, and it stays distinguishable from a cause the source simply
    // offered no detail for.
    let withheld = Cause::with_detail(unregistered_code(), CauseDetail::WithheldByDetailBudget);
    let none_offered = Cause::new(unregistered_code());
    let retained = Cause::with_detail(
        unregistered_code(),
        CauseDetail::Retained(RetainedBytes::new(b"offered detail")),
    );

    assert_ne!(withheld, none_offered);
    assert_ne!(withheld, retained);
    assert_ne!(none_offered, retained);
    assert_eq!(withheld.detail(), &CauseDetail::WithheldByDetailBudget);

    let mut chain = CauseChain::new();
    chain.push(withheld.clone());
    assert_eq!(
        chain.len(),
        1,
        "a withheld detail does not remove the cause"
    );
    assert_eq!(chain.declared_order(), &[withheld]);
}

// ---------------------------------------------------------------------------
// FR-008-AC-4 — the partial-output state is an explicit separate field
// ---------------------------------------------------------------------------

#[test]
#[trace("TC-011", "FR-008-AC-4")]
fn tc_011_partial_output_state_is_explicit() {
    let fatal = fully_populated();
    assert_eq!(
        fatal.partial_output(),
        PartialOutput::NoneEmitted,
        "a fatal refusal emits no success value"
    );

    // The success values and the declared state are separate fields, and the
    // only constructor that declares retention requires at least one value, so
    // the two cannot disagree. No `Option<T>` doubles as both.
    let retained = ProtocolRefusal::builder(
        unregistered_code(),
        CatalogSelection::local(),
        OriginatingStage::CapabilitySelection,
    )
    .retaining_independent("first-claim-result", vec!["second-claim-result"]);

    assert_eq!(
        retained.refusal().partial_output(),
        PartialOutput::IndependentResultsRetained
    );
    assert_eq!(
        retained.independent_results(),
        &["first-claim-result", "second-claim-result"]
    );
    assert_ne!(
        retained.refusal().partial_output(),
        fatal.partial_output(),
        "the two partial-output states are independently observable"
    );
}

// ---------------------------------------------------------------------------
// The repo-wide refusal-code audit
// ---------------------------------------------------------------------------

#[test]
#[trace("TC-011", "FR-008-AC-1")]
fn tc_011_registry_is_free_of_code_defects() {
    assert_eq!(
        audit(REGISTRY),
        Vec::new(),
        "the crate's own code namespace must be clean"
    );
    assert_eq!(
        lookup("refusal.code.unregistered").map(RefusalCode::as_str),
        Some("refusal.code.unregistered")
    );
    assert_eq!(
        lookup("refusal.code.invented-by-nobody"),
        None,
        "an unregistered spelling resolves to nothing, it is not fabricated"
    );
}

/// Two modules minting the identical spelling: the defect a crate-wide closed
/// enum would have made impossible, and the reason this audit exists.
const COLLIDING_A: CodeModule = CodeModule {
    root: "refusal",
    owner: "C-XX",
    module_path: "quire_protocol::refusal",
    entries: &[CodeEntry {
        provenance: Provenance::Minted,
        code: "refusal.shared.spelling",
        domain: RefusalDomain::Representation,
        retry: RetryDecision::Never,
        summary: "first minting",
    }],
};

const COLLIDING_B: CodeModule = CodeModule {
    root: "refusal",
    owner: "C-YY",
    module_path: "quire_protocol::elsewhere",
    entries: &[CodeEntry {
        provenance: Provenance::Minted,
        code: "refusal.shared.spelling",
        domain: RefusalDomain::Representation,
        retry: RetryDecision::Never,
        summary: "second minting of the same spelling",
    }],
};

#[test]
#[trace("TC-011", "FR-008-AC-1")]
fn tc_011_audit_reports_a_code_minted_by_two_chunks() {
    let defects = audit(&[COLLIDING_A, COLLIDING_B]);
    assert!(
        defects.contains(&CatalogDefect::DuplicateCode {
            code: "refusal.shared.spelling",
            first_owner: "C-XX",
            second_owner: "C-YY",
        }),
        "a shared spelling must be reported as its own defect: {defects:?}"
    );
}

#[test]
#[trace("TC-011", "FR-008-AC-1")]
fn tc_011_audit_reports_a_root_claimed_by_two_chunks() {
    let defects = audit(&[COLLIDING_A, COLLIDING_B]);
    assert!(
        defects.contains(&CatalogDefect::DuplicateRoot {
            root: "refusal",
            first_owner: "C-XX",
            second_owner: "C-YY",
        }),
        "a shared reserved root must be reported as its own defect: {defects:?}"
    );
}

const STRAYING_ROOT: CodeModule = CodeModule {
    root: "assessment",
    owner: "C-ZZ",
    module_path: "quire_protocol::elsewhere",
    entries: &[CodeEntry {
        provenance: Provenance::Minted,
        code: "result.claimed.elsewhere",
        domain: RefusalDomain::Assessment,
        retry: RetryDecision::Never,
        summary: "minted under another chunk's root",
    }],
};

#[test]
#[trace("TC-011", "FR-008-AC-1")]
fn tc_011_audit_reports_a_code_under_another_root() {
    let defects = audit(&[STRAYING_ROOT]);
    assert_eq!(
        defects,
        vec![CatalogDefect::RootMismatch {
            code: "result.claimed.elsewhere",
            root: "assessment",
            owner: "C-ZZ",
        }],
        "and only that defect"
    );
}

const MIXED_DOMAINS: CodeModule = CodeModule {
    root: "assessment",
    owner: "C-ZZ",
    module_path: "quire_protocol::elsewhere",
    entries: &[
        CodeEntry {
            provenance: Provenance::Minted,
            code: "assessment.first.code",
            domain: RefusalDomain::Assessment,
            retry: RetryDecision::Never,
            summary: "one domain",
        },
        CodeEntry {
            provenance: Provenance::Minted,
            code: "assessment.second.code",
            domain: RefusalDomain::Projection,
            retry: RetryDecision::Never,
            summary: "another domain under the same owner",
        },
    ],
};

#[test]
#[trace("TC-011", "FR-008-AC-1")]
fn tc_011_audit_reports_a_module_spanning_two_domains() {
    let defects = audit(&[MIXED_DOMAINS]);
    assert_eq!(
        defects,
        vec![CatalogDefect::MixedDomains {
            root: "assessment",
            owner: "C-ZZ",
            first: RefusalDomain::Assessment,
            second: RefusalDomain::Projection,
        }]
    );
}

#[test]
#[trace("TC-011", "FR-008-AC-1")]
fn tc_011_malformed_spellings_are_each_their_own_reason() {
    // One case per reason, each asserting only its own: a single "malformed"
    // verdict shared by five different mistakes would be exactly the collapsed
    // guard this chunk exists to remove.
    assert_eq!(
        malformation("single"),
        Some(MalformedCode::FewerThanTwoSegments)
    );
    assert_eq!(
        malformation("refusal..code"),
        Some(MalformedCode::EmptySegment)
    );
    assert_eq!(
        malformation("refusal.1code"),
        Some(MalformedCode::SegmentDoesNotStartWithLetter)
    );
    assert_eq!(
        malformation("refusal.code-"),
        Some(MalformedCode::HyphenAtSegmentEdge)
    );
    assert_eq!(
        malformation("refusal.Code"),
        Some(MalformedCode::SegmentDoesNotStartWithLetter)
    );
    assert_eq!(
        malformation("refusal.co_de"),
        Some(MalformedCode::IllegalCharacter)
    );
    assert_eq!(malformation("refusal.code.unregistered"), None);
    // The minted grammar binds minted codes. A spec-named code's spelling is
    // the requirement's, and `unsupported-source-contract` deliberately does not
    // satisfy it — see `tc_011_the_spec_named_code_is_registered_and_audits_clean`.
    let mut minted = 0;
    let mut spec_named = 0;
    for module in REGISTRY {
        for entry in module.entries {
            match entry.provenance {
                Provenance::Minted => {
                    minted += 1;
                    assert_eq!(
                        malformation(entry.code),
                        None,
                        "{} is a registered minted code and must be well formed",
                        entry.code
                    );
                }
                Provenance::SpecNamed => spec_named += 1,
            }
        }
    }
    assert!(minted > 0, "the registry must carry minted codes");
    assert_eq!(spec_named, 1, "FR-008 names exactly one local code");
}

const MALFORMED_MODULE: CodeModule = CodeModule {
    root: "refusal",
    owner: "C-WW",
    module_path: "quire_protocol::elsewhere",
    entries: &[CodeEntry {
        provenance: Provenance::Minted,
        code: "refusal.Bad_Code",
        domain: RefusalDomain::Representation,
        retry: RetryDecision::Never,
        summary: "not a legal spelling",
    }],
};

#[test]
#[trace("TC-011", "FR-008-AC-1")]
fn tc_011_audit_reports_a_malformed_registration() {
    let defects = audit(&[MALFORMED_MODULE]);
    assert_eq!(
        defects,
        vec![CatalogDefect::Malformed {
            code: "refusal.Bad_Code",
            owner: "C-WW",
            reason: MalformedCode::SegmentDoesNotStartWithLetter,
        }]
    );
}

// ---------------------------------------------------------------------------
// The repo-wide half: no `src/` file may emit an unregistered code
// ---------------------------------------------------------------------------

/// Every tracked Rust source file under `src/`.
fn source_files() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    collect_rust_files(&root, &mut files);
    files.sort();
    assert!(
        !files.is_empty(),
        "the scan found no source files, so it would pass vacuously"
    );
    files
}

fn collect_rust_files(directory: &Path, files: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(directory).expect("src/ is readable");
    for entry in entries {
        let path = entry.expect("directory entry is readable").path();
        if path.is_dir() {
            collect_rust_files(&path, files);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
}

#[test]
#[trace("TC-011", "FR-008-AC-1")]
fn tc_011_no_src_file_emits_an_unregistered_code() {
    let mut findings = Vec::new();
    for file in source_files() {
        let source = fs::read_to_string(&file).expect("source file is UTF-8");
        for literal in unregistered_literals(REGISTRY, &source) {
            findings.push(format!("{}: {}", file.display(), literal.literal));
        }
    }
    assert_eq!(
        findings,
        Vec::<String>::new(),
        "every code-shaped literal under a reserved root must be registered"
    );
}

#[test]
#[trace("TC-011", "FR-008-AC-1")]
fn tc_011_literal_scan_detects_an_unregistered_code() {
    // The guard on the test above: a scanner that found nothing would pass it
    // no matter what the sources contained.
    let source = r#"
        fn refuse() -> &'static str {
            let file = "spec/tests.md";
            let version = "1.0";
            let _ = (file, version);
            "refusal.invented.here"
        }
    "#;
    assert_eq!(
        unregistered_literals(REGISTRY, source),
        vec![UnregisteredLiteral {
            literal: "refusal.invented.here".to_owned(),
        }],
        "an unregistered code under a reserved root is found, and nothing else is"
    );

    // A registered code is not a finding. A code-shaped literal that is not a
    // code IS a finding, because shape cannot tell `cargo.toml` from the real
    // two-segment code `outcome.open` — the sanctioned silencer is
    // `EXEMPT_LITERALS`, never a weaker shape test.
    assert_eq!(
        unregistered_literals(REGISTRY, r#"let path = "cargo.toml";"#),
        vec![UnregisteredLiteral {
            literal: "cargo.toml".to_owned()
        }],
        "a code-shaped non-code is reported rather than guessed at"
    );

    let clean = r#"
        let code = "refusal.code.unregistered";
        let other = "outcome.open";
        let path = "spec/tests.md";
    "#;
    assert_eq!(unregistered_literals(REGISTRY, clean), Vec::new());
}

#[test]
#[trace("TC-011", "FR-008-AC-1")]
fn tc_011_every_registered_module_is_reachable_by_lookup() {
    // A module registered but unreachable would silently drop out of both
    // halves of the audit.
    for module in REGISTRY {
        for entry in module.entries {
            let found = lookup(entry.code).unwrap_or_else(|| {
                panic!("{} is registered but does not resolve", entry.code);
            });
            assert_eq!(found.domain(), entry.domain);
            assert_eq!(found.retry(), entry.retry);
        }
    }
    assert_eq!(ENVELOPE_CODES.owner, "C-01");
}

/// A reported span whose end precedes its start is refused at construction.
///
/// The offsets a mapping chunk passes here come from another repository's
/// records, and transposing them is the mistake that type has no way to prevent.
/// Admitting the transposed pair would build a well-formed envelope whose span
/// underflows on `end - start` and panics on `&text[start..end]` in whatever
/// renders it, so the refusal belongs at the call site that holds the offsets.
#[trace("TC-011", "FR-008-AC-2")]
#[test]
fn tc_011_an_inverted_span_is_refused_at_construction() {
    assert_eq!(
        SourceLocation::reported(b"unit-a", 19, 12),
        Err(InvertedSpan { start: 19, end: 12 }),
        "a span ending before it starts is not a location"
    );

    // Off by exactly one, which is the transposition a fencepost error makes.
    assert_eq!(
        SourceLocation::reported(b"unit-a", 12, 11),
        Err(InvertedSpan { start: 12, end: 11 })
    );

    // The empty span at an offset is well formed: `end` is one past the span,
    // so `start == end` names zero bytes at `start` rather than an inversion.
    let empty = SourceLocation::reported(b"unit-a", 12, 12).expect("an empty span is a position");
    assert_eq!(empty.start(), 12);
    assert_eq!(empty.end(), 12);

    // A well-formed span is unchanged by the check.
    let span = SourceLocation::reported(b"unit-a", 12, 19).expect("12..19 is well formed");
    assert_eq!((span.start(), span.end()), (12, 19));
    assert_eq!(span.unit(), &RetainedBytes::new(b"unit-a"));

    // The refusal carries both offsets as reported, so the caller can see the
    // transposition rather than only that one happened.
    let inverted = SourceLocation::reported(b"unit-a", 40, 12).expect_err("inverted");
    assert_eq!((inverted.start, inverted.end), (40, 12));
}

// ---------------------------------------------------------------------------
// Findings from the C-01 review: each test below fails on the defect it names
// ---------------------------------------------------------------------------

/// Every field the envelope's identity is built from actually changes identity.
///
/// Before this test, `code`, `causes`, `prerequisites` and `partial_output`
/// could each be deleted from `identity_fields` with the whole suite still
/// green. Two refusals differing only in code would then compare equal and hash
/// equal, and a caller de-duplicating refusals into a set would lose one — and
/// act on the survivor's retry decision.
#[test]
#[trace("TC-011", "FR-008-AC-2")]
fn tc_011_every_identity_field_changes_identity() {
    let plain = || {
        ProtocolRefusal::builder(
            unregistered_code(),
            CatalogSelection::local(),
            OriginatingStage::SourceRecognition,
        )
    };
    let base = plain().emitting_nothing();

    // Code.
    let other_code = ProtocolRefusal::builder(
        ordered_cause("cause.zeta.one"),
        CatalogSelection::local(),
        OriginatingStage::SourceRecognition,
    )
    .emitting_nothing();
    assert_ne!(other_code, base, "the code is part of the identity");

    // Causes.
    assert_ne!(
        plain()
            .cause(Cause::new(ordered_cause("cause.zeta.one")))
            .emitting_nothing(),
        base,
        "the cause chain is part of the identity"
    );

    // Prerequisites.
    assert_ne!(
        plain()
            .prerequisite(Prerequisite::new(
                OriginatingStage::IdentityLinking,
                PrerequisiteState::NotEstablished,
            ))
            .emitting_nothing(),
        base,
        "prerequisites are part of the identity"
    );

    // Partial output: the same builder either emits nothing or retains an
    // independent value, and those are different refusals.
    let retaining = plain().retaining_independent(1_u8, Vec::new());
    assert_ne!(
        retaining.refusal(),
        &base,
        "the partial-output state is part of the identity"
    );
}

/// Every originating stage renders to its own spelling.
///
/// `RefusalDomain` had this; `OriginatingStage` did not, and `ALL` was unused.
/// Two stages spelled identically make FR-008's "a wrapper cannot erase or
/// replace that stage" unobservable in the rendered form.
#[test]
#[trace("TC-011", "FR-008-AC-1")]
fn tc_011_every_stage_has_its_own_spelling() {
    let stages = OriginatingStage::ALL;
    assert_eq!(stages.len(), 6, "FR-008 names six stages");
    for (position, stage) in stages.iter().enumerate() {
        assert!(!stage.as_str().is_empty());
        for other in &stages[..position] {
            assert_ne!(
                stage.as_str(),
                other.as_str(),
                "{stage:?} and {other:?} render identically"
            );
        }
        assert_ne!(stage, &stages[(position + 1) % stages.len()]);
    }
}

/// FR-008's own code is registrable, and registered.
///
/// The catalog's minted grammar requires at least two dot-separated segments.
/// `unsupported-source-contract` — the one code FR-008 names, and assigns to
/// this repository — has none, so registering it used to fail the audit with
/// `FewerThanTwoSegments` and `RootMismatch`. C-02 could not have registered
/// its only code without deleting C-01's audit.
#[test]
#[trace("TC-011", "FR-008-AC-3")]
fn tc_011_the_spec_named_code_is_registered_and_audits_clean() {
    assert_eq!(
        UNSUPPORTED_SOURCE_CONTRACT.provenance,
        Provenance::SpecNamed
    );
    assert_eq!(
        UNSUPPORTED_SOURCE_CONTRACT.code, "unsupported-source-contract",
        "the requirement's spelling is the spelling"
    );
    assert!(
        lookup("unsupported-source-contract").is_some(),
        "FR-008's own code must be reachable through the registry"
    );
    assert_eq!(audit(REGISTRY), Vec::new(), "the real registry is clean");

    // The minted grammar still rejects that shape, which is why the provenance
    // distinction exists rather than the grammar simply being loosened.
    assert_eq!(
        malformation("unsupported-source-contract"),
        Some(MalformedCode::FewerThanTwoSegments)
    );
}

/// A code-shaped literal is a finding whatever root it sits under.
///
/// The scan used to ask whether some module had already reserved the literal's
/// root, which made it a denylist: the first chunk to mint a code under any of
/// the five FR-008 domain roots was invisible to it.
#[test]
#[trace("TC-011", "FR-008-AC-1")]
fn tc_011_an_unreserved_root_is_still_a_finding() {
    let source = r#"pub const INVENTED: &str = "admission.offer.rejected";"#;
    let found = unregistered_literals(REGISTRY, source);
    assert_eq!(
        found,
        vec![UnregisteredLiteral {
            literal: "admission.offer.rejected".to_owned()
        }],
        "a code under a root nobody has reserved yet is still unregistered"
    );
    assert!(is_code_shape("admission.offer.rejected"));
}

/// A quote inside a char literal or a comment does not blind the scan.
///
/// The scan was a search for `"` bytes, so one `pub const QUOTE: char = '"';`
/// shifted quote pairing and hid every code literal after it. Comments were
/// read as source too, which made this module's own prose a finding for the
/// first chunk to reserve a root it mentions.
#[test]
#[trace("TC-011", "FR-008-AC-1")]
fn tc_011_a_quote_in_a_char_literal_or_comment_does_not_blind_the_scan() {
    let hidden = "hidden.code.here";

    let after_char_literal =
        format!("pub const QUOTE: char = '\"';\npub const HIDDEN: &str = \"{hidden}\";");
    assert_eq!(
        unregistered_literals(REGISTRY, &after_char_literal),
        vec![UnregisteredLiteral {
            literal: hidden.to_owned()
        }],
        "a char literal holding a quote must not hide what follows it"
    );

    // Prose is not code: a literal that appears only inside a comment is not a
    // finding, in either comment form.
    let line_comment = format!("// a chunk writing a bare \"{hidden}\" in src/\nlet x = 1;");
    assert_eq!(unregistered_literals(REGISTRY, &line_comment), Vec::new());

    let block_comment = format!("/* outer /* nested \"{hidden}\" */ still inside */\nlet x = 1;");
    assert_eq!(unregistered_literals(REGISTRY, &block_comment), Vec::new());

    // And a real literal after a comment is still found.
    let after_comment = format!("// prose\npub const HIDDEN: &str = \"{hidden}\";");
    assert_eq!(
        unregistered_literals(REGISTRY, &after_comment),
        vec![UnregisteredLiteral {
            literal: hidden.to_owned()
        }]
    );
}

/// A leading hyphen reports as a hyphen at an edge.
///
/// `HyphenAtSegmentEdge` documents "began or ended with a hyphen", but the
/// letter check ran first, so the leading half was unreachable and the reported
/// reason named the wrong defect.
#[test]
#[trace("TC-011", "FR-008-AC-1")]
fn tc_011_a_leading_hyphen_is_a_hyphen_at_an_edge() {
    assert_eq!(
        malformation("refusal.-leading.code"),
        Some(MalformedCode::HyphenAtSegmentEdge)
    );
    assert_eq!(
        malformation("refusal.trailing-.code"),
        Some(MalformedCode::HyphenAtSegmentEdge)
    );
    assert_eq!(
        malformation("refusal.9numeric.code"),
        Some(MalformedCode::SegmentDoesNotStartWithLetter)
    );
}

/// Every exemption still corresponds to a literal in `src/`.
///
/// An exemption that no longer matches anything is worse than none: it sits in
/// the allowlist silently pre-approving a spelling that a later chunk could
/// take for a real code.
#[test]
#[trace("TC-011", "FR-008-AC-1")]
fn tc_011_the_exemption_list_is_not_stale() {
    let sources: String = source_files()
        .into_iter()
        .map(|path| fs::read_to_string(&path).expect("a readable src file"))
        .collect::<Vec<_>>()
        .join("\n");
    for exempt in EXEMPT_LITERALS {
        assert!(
            sources.contains(exempt),
            "exemption {exempt} matches no literal in src/; remove it"
        );
    }
}
