// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-011: refusal-code registry mutation and source-inventory controls.

use std::fs;
use std::path::{Path, PathBuf};

use ix_trace_rs::trace;
use quire_protocol::refusal::catalog::{
    audit, is_code_shape, lookup, malformation, unregistered_literals, CatalogDefect, CodeEntry,
    CodeModule, MalformedCode, Provenance, RefusalCode, UnregisteredLiteral, ENVELOPE_CODES,
    EXEMPT_LITERALS, REGISTRY, UNSUPPORTED_SOURCE_CONTRACT,
};
use quire_protocol::refusal::{RefusalDomain, RetryDecision};

const FIRST: CodeModule = CodeModule {
    root: "test",
    owner: "Task-017-a",
    module_path: "quire_protocol::refusal",
    entries: &[CodeEntry {
        provenance: Provenance::Minted,
        code: "test.shared-code",
        domain: RefusalDomain::Representation,
        retry: RetryDecision::Never,
        summary: "first test code",
    }],
};

const SECOND: CodeModule = CodeModule {
    root: "test",
    owner: "Task-017-b",
    module_path: "quire_protocol::refusal",
    entries: &[CodeEntry {
        provenance: Provenance::Minted,
        code: "test.shared-code",
        domain: RefusalDomain::Representation,
        retry: RetryDecision::Never,
        summary: "colliding test code",
    }],
};

const STRAYING_ROOT: CodeModule = CodeModule {
    root: "assessment",
    owner: "Task-017-stray",
    module_path: "quire_protocol::refusal",
    entries: &[CodeEntry {
        provenance: Provenance::Minted,
        code: "result.claimed.elsewhere",
        domain: RefusalDomain::Assessment,
        retry: RetryDecision::Never,
        summary: "minted under a root the module did not reserve",
    }],
};

const MIXED_DOMAINS: CodeModule = CodeModule {
    root: "assessment",
    owner: "Task-017-mixed",
    module_path: "quire_protocol::refusal",
    entries: &[
        CodeEntry {
            provenance: Provenance::Minted,
            code: "assessment.first.code",
            domain: RefusalDomain::Assessment,
            retry: RetryDecision::Never,
            summary: "first domain",
        },
        CodeEntry {
            provenance: Provenance::Minted,
            code: "assessment.second.code",
            domain: RefusalDomain::Projection,
            retry: RetryDecision::Never,
            summary: "second domain",
        },
    ],
};

const MALFORMED_MODULE: CodeModule = CodeModule {
    root: "refusal",
    owner: "Task-017-malformed",
    module_path: "quire_protocol::refusal",
    entries: &[CodeEntry {
        provenance: Provenance::Minted,
        code: "refusal.Bad_Code",
        domain: RefusalDomain::Representation,
        retry: RetryDecision::Never,
        summary: "malformed test code",
    }],
};

fn rust_sources(directory: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(directory).expect("source directory is readable") {
        let path = entry.expect("source entry is readable").path();
        if path.is_dir() {
            rust_sources(&path, files);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
}

fn all_rust_sources() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rust_sources(&root, &mut files);
    files.sort();
    assert!(!files.is_empty(), "source scan must not pass vacuously");
    files
}

fn module_sources(module_path: &str) -> String {
    let relative = module_path
        .strip_prefix("quire_protocol::")
        .unwrap_or_else(|| panic!("{module_path} is not a path inside this crate"))
        .replace("::", "/");
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut sources = String::new();
    let file = root.join(format!("{relative}.rs"));
    if file.is_file() {
        sources.push_str(&fs::read_to_string(&file).expect("module source is readable UTF-8"));
    }
    let directory = root.join(relative);
    if directory.is_dir() {
        let mut files = Vec::new();
        rust_sources(&directory, &mut files);
        files.sort();
        for path in files {
            sources.push_str(&fs::read_to_string(path).expect("module source is readable UTF-8"));
        }
    }
    assert!(!sources.is_empty(), "{module_path} names no source module");
    sources
}

/// Tracing: TC-011, FR-008-AC-1, Task-017.
#[test]
#[trace("TC-011", "FR-008-AC-1")]
fn tc_011_registry_is_clean_and_every_entry_resolves() {
    assert_eq!(audit(REGISTRY), Vec::new());
    for module in REGISTRY {
        for entry in module.entries {
            let found = lookup(entry.code).expect("registered code resolves");
            assert_eq!(found.as_str(), entry.code);
            assert_eq!(found.domain(), entry.domain);
            assert_eq!(found.retry(), entry.retry);
        }
    }
}

/// Tracing: TC-011, FR-008-AC-1, Task-017.
#[test]
#[trace("TC-011", "FR-008-AC-1")]
fn tc_011_registry_rejects_duplicate_roots_and_codes() {
    let defects = audit(&[FIRST, SECOND]);
    assert!(defects.contains(&CatalogDefect::DuplicateRoot {
        root: "test",
        first_owner: "Task-017-a",
        second_owner: "Task-017-b",
    }));
    assert!(defects.contains(&CatalogDefect::DuplicateCode {
        code: "test.shared-code",
        first_owner: "Task-017-a",
        second_owner: "Task-017-b",
    }));
}

/// Tracing: TC-011, FR-008-AC-1, Task-017.
#[test]
#[trace("TC-011", "FR-008-AC-1")]
fn tc_011_code_grammar_mutations_remain_distinct() {
    assert_eq!(
        malformation("single"),
        Some(MalformedCode::FewerThanTwoSegments)
    );
    assert_eq!(
        malformation("test..code"),
        Some(MalformedCode::EmptySegment)
    );
    assert_eq!(
        malformation("test.1code"),
        Some(MalformedCode::SegmentDoesNotStartWithLetter)
    );
    assert_eq!(
        malformation("test.-code"),
        Some(MalformedCode::HyphenAtSegmentEdge)
    );
    assert_eq!(
        malformation("test.co_de"),
        Some(MalformedCode::IllegalCharacter)
    );
    assert_eq!(malformation("test.valid-code"), None);
}

/// Tracing: TC-011, FR-008-AC-1, Task-017.
#[test]
#[trace("TC-011", "FR-008-AC-1")]
fn tc_011_literal_scan_kills_an_unregistered_code() {
    let source = r#"fn refusal() -> &'static str { "refusal.not-registered" }"#;
    let findings = unregistered_literals(REGISTRY, source);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].literal, "refusal.not-registered");
    assert!(unregistered_literals(REGISTRY, r#"let path = "quire.verification.jcs";"#).is_empty());
}

/// Tracing: TC-011, FR-008-AC-1, Task-017.
#[test]
#[trace("TC-011", "FR-008-AC-1")]
fn tc_011_no_source_file_emits_an_unregistered_code() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rust_sources(&root, &mut files);
    files.sort();
    assert!(!files.is_empty());

    let mut findings = Vec::new();
    for file in files {
        let source = fs::read_to_string(&file).expect("Rust source is UTF-8");
        for unregistered in unregistered_literals(REGISTRY, &source) {
            findings.push(format!("{}: {}", file.display(), unregistered.literal));
        }
    }
    assert_eq!(findings, Vec::<String>::new());
}

/// Tracing: TC-011, FR-008-AC-1, Task-017.
#[test]
#[trace("TC-011", "FR-008-AC-1")]
fn tc_011_refusal_code_is_constructible_only_through_registry_lookup() {
    let code: RefusalCode = lookup("refusal.code.unregistered").expect("registered code");
    assert_eq!(code.as_str(), "refusal.code.unregistered");
    assert!(lookup("refusal.code.fabricated").is_none());
}

/// Tracing: TC-011, FR-008-AC-1, Task-017.
#[test]
#[trace("TC-011", "FR-008-AC-1")]
fn tc_011_registry_audit_rejects_root_domain_and_grammar_drift() {
    assert_eq!(
        audit(&[STRAYING_ROOT]),
        vec![CatalogDefect::RootMismatch {
            code: "result.claimed.elsewhere",
            root: "assessment",
            owner: "Task-017-stray",
        }]
    );
    assert_eq!(
        audit(&[MIXED_DOMAINS]),
        vec![CatalogDefect::MixedDomains {
            root: "assessment",
            owner: "Task-017-mixed",
            first: RefusalDomain::Assessment,
            second: RefusalDomain::Projection,
        }]
    );
    assert_eq!(
        audit(&[MALFORMED_MODULE]),
        vec![CatalogDefect::Malformed {
            code: "refusal.Bad_Code",
            owner: "Task-017-malformed",
            reason: MalformedCode::SegmentDoesNotStartWithLetter,
        }]
    );
}

/// Tracing: TC-011, FR-008-AC-1, Task-017.
#[test]
#[trace("TC-011", "FR-008-AC-1")]
fn tc_011_literal_scanner_survives_quotes_comments_and_unreserved_roots() {
    let hidden = "hidden.code.here";
    let after_char_literal =
        format!("pub const QUOTE: char = '\"';\npub const HIDDEN: &str = \"{hidden}\";");
    assert_eq!(
        unregistered_literals(REGISTRY, &after_char_literal),
        vec![UnregisteredLiteral {
            literal: hidden.to_owned(),
        }]
    );
    let line_comment = format!("// a quoted \"{hidden}\" is prose\nlet value = 1;");
    assert!(unregistered_literals(REGISTRY, &line_comment).is_empty());
    let block_comment = format!("/* outer /* nested \"{hidden}\" */ prose */\nlet value = 1;");
    assert!(unregistered_literals(REGISTRY, &block_comment).is_empty());

    let unreserved = "admission.offer.rejected";
    assert!(is_code_shape(unreserved));
    assert_eq!(
        unregistered_literals(
            REGISTRY,
            r#"pub const INVENTED: &str = "admission.offer.rejected";"#,
        ),
        vec![UnregisteredLiteral {
            literal: unreserved.to_owned(),
        }]
    );
}

/// Tracing: TC-011, FR-008-AC-1, FR-008-AC-3, Task-017.
#[test]
#[trace("TC-011", "FR-008-AC-1", "FR-008-AC-3")]
fn tc_011_spec_named_code_and_minted_grammar_remain_distinct() {
    assert_eq!(
        UNSUPPORTED_SOURCE_CONTRACT.provenance,
        Provenance::SpecNamed
    );
    assert_eq!(
        UNSUPPORTED_SOURCE_CONTRACT.code,
        "unsupported-source-contract"
    );
    assert!(lookup(UNSUPPORTED_SOURCE_CONTRACT.code).is_some());
    assert_eq!(
        malformation(UNSUPPORTED_SOURCE_CONTRACT.code),
        Some(MalformedCode::FewerThanTwoSegments)
    );
    assert_eq!(audit(REGISTRY), Vec::new());
}

/// Tracing: TC-011, FR-008-AC-1, Task-017.
#[test]
#[trace("TC-011", "FR-008-AC-1")]
fn tc_011_hyphen_edge_classification_does_not_regress() {
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

/// Tracing: TC-011, FR-008-AC-1, Task-017.
#[test]
#[trace("TC-011", "FR-008-AC-1")]
fn tc_011_catalog_exemptions_are_live_and_minted_entries_have_owners() {
    let sources = all_rust_sources()
        .into_iter()
        .map(|path| fs::read_to_string(path).expect("source is readable UTF-8"))
        .collect::<Vec<_>>()
        .join("\n");
    for exempt in EXEMPT_LITERALS {
        assert!(sources.contains(exempt), "stale exemption: {exempt}");
    }

    let mut adrift = Vec::new();
    for module in REGISTRY {
        let owning = module_sources(module.module_path);
        for entry in module.entries {
            if entry.provenance == Provenance::Minted && !owning.contains(entry.code) {
                adrift.push(entry.code);
            }
        }
    }
    assert_eq!(adrift, Vec::<&str>::new());
    assert_eq!(ENVELOPE_CODES.owner, "C-01");
}
