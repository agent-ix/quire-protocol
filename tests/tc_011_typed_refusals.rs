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
    audit, lookup, malformation, unregistered_literals, CatalogDefect, CatalogDigest,
    CatalogDigestUnavailable, CatalogSelection, CodeEntry, CodeModule, MalformedCode, RefusalCode,
    RetainedBytes, UnregisteredLiteral, ENVELOPE_CODES, REGISTRY, UNREGISTERED_CODE,
};
use quire_protocol::refusal::{
    AffectedIdentity, Cause, CauseChain, CauseDetail, Location, OriginatingStage, PartialOutput,
    Prerequisite, PrerequisiteState, ProtocolRefusal, RefusalDomain, Rendering, RetryDecision,
    SourceLocation, Unavailability,
};

// ---------------------------------------------------------------------------
// Shared material
// ---------------------------------------------------------------------------

/// The one code chunk C-01 mints, as a handle.
fn unregistered_code() -> RefusalCode {
    RefusalCode::of(&UNREGISTERED_CODE)
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
    .located_at(SourceLocation::reported(b"unit-a", 12, 19))
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
        &Location::Available(SourceLocation::reported(b"unit-a", 12, 19))
    );
    assert_eq!(refusal.causes().len(), 1);
    assert_eq!(refusal.partial_output(), PartialOutput::NoneEmitted);
    assert_eq!(refusal.retry(), RetryDecision::Never);
}

#[test]
#[trace("TC-011", "FR-008-AC-1")]
fn tc_011_prerequisite_states_are_three_distinct_facts() {
    // "the source said no" and "the source said nothing" are different facts,
    // and neither is the other.
    let states = [
        PrerequisiteState::Established,
        PrerequisiteState::NotEstablished,
        PrerequisiteState::Unavailable,
    ];
    for (index, state) in states.iter().enumerate() {
        for (other_index, other) in states.iter().enumerate() {
            assert_eq!(index == other_index, state == other);
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
    .located_at(SourceLocation::reported(b"unit-a", 12, 19))
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
    .located_at(SourceLocation::reported(b"unit-a", 12, 19))
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
    .located_at(SourceLocation::reported(b"unit-a", 12, 19))
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
        code: "zeta.cause.one",
        domain: RefusalDomain::Assessment,
        retry: RetryDecision::Never,
        summary: "first declared cause",
    },
    CodeEntry {
        code: "alpha.cause.two",
        domain: RefusalDomain::Assessment,
        retry: RetryDecision::Never,
        summary: "second declared cause",
    },
    CodeEntry {
        code: "mu.cause.three",
        domain: RefusalDomain::Assessment,
        retry: RetryDecision::Never,
        summary: "third declared cause",
    },
];

#[test]
#[trace("TC-011", "FR-008-AC-1")]
fn tc_011_cause_order_is_declared_not_sorted() {
    let mut chain = CauseChain::new();
    for entry in &ORDERED_CAUSES {
        chain.push(Cause::new(RefusalCode::of(entry)));
    }

    let observed: Vec<&str> = chain
        .declared_order()
        .iter()
        .map(|cause| cause.code().as_str())
        .collect();
    assert_eq!(
        observed,
        vec!["zeta.cause.one", "alpha.cause.two", "mu.cause.three"],
        "causes are retained in declared order"
    );

    let mut sorted = observed.clone();
    sorted.sort_unstable();
    assert_ne!(
        observed, sorted,
        "the fixture must not be accidentally sorted, or this proves nothing"
    );
    assert_eq!(
        chain.position_of(RefusalCode::of(&ORDERED_CAUSES[2])),
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
            code: "assessment.first.code",
            domain: RefusalDomain::Assessment,
            retry: RetryDecision::Never,
            summary: "one domain",
        },
        CodeEntry {
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
    for module in REGISTRY {
        for entry in module.entries {
            assert_eq!(
                malformation(entry.code),
                None,
                "{} is a registered code and must be well formed",
                entry.code
            );
        }
    }
}

const MALFORMED_MODULE: CodeModule = CodeModule {
    root: "refusal",
    owner: "C-WW",
    module_path: "quire_protocol::elsewhere",
    entries: &[CodeEntry {
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

    // A registered code is not a finding, and a dotted literal under a root no
    // module has reserved is not a finding either.
    let clean = r#"
        let code = "refusal.code.unregistered";
        let other = "outcome.open";
        let path = "cargo.toml";
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
