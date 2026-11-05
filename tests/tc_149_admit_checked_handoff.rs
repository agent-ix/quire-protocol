// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-149: admission of the checked-package handoff (FR-043).

mod support {
    pub mod handoff;
}

use std::path::{Path, PathBuf};

use ix_trace_rs::trace;
use qsl_package::{EqualityFileRefusal, IdentityFileRefusal};
use quire_contract_model::{
    read_checked_package, CheckedPackageDispatchResult, CheckedPackageEvidence,
    CheckedPackageLimit, CheckedPackageReadLimits, CheckedPackageRefusal,
    CheckedPackageRefusalCode,
};
use quire_protocol::checked_package::{admit, HandoffFiles, Refusal};
use quire_spec_language::checked_handoff::{Corpus, SelectionAxis};
use serde_json::Value;
use support::handoff::{written, Handoff};

fn files_of<'a>(handoff: &'a Handoff, package: &'a [u8]) -> HandoffFiles<'a> {
    HandoffFiles {
        checked_package: package,
        selection: &handoff.files["selection.json"],
        identities: &handoff.files["identities.json"],
        equality: &handoff.files["equality.json"],
    }
}

fn bounded() -> CheckedPackageReadLimits {
    CheckedPackageReadLimits::bounded()
}

fn direct(
    bytes: &[u8],
    limits: CheckedPackageReadLimits,
    evidence: &CheckedPackageEvidence,
) -> CheckedPackageDispatchResult {
    read_checked_package(bytes, limits, evidence)
}

fn spell(value: &impl serde::Serialize) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_default()
}

fn reader_refusal(refusal: Result<impl std::fmt::Debug, Refusal>) -> CheckedPackageRefusal {
    match refusal {
        Err(Refusal::Reader(refusal)) => *refusal,
        other => panic!("expected a reader refusal, got {other:?}"),
    }
}

/// FR-043-AC-1: the three packages of recipe R are admitted with every
/// selection identity unchanged, and the admitted package is what
/// `read_checked_package` returns.
#[test]
#[trace("FR-043-AC-1", "TC-149")]
fn the_three_packages_of_recipe_r_are_admitted_unchanged() {
    let expected_rows = [
        ("event_position", 4),
        ("fixed_sample", 1),
        ("timestamped_event", 1),
    ];
    for (kind, rows) in expected_rows {
        let handoff = written(kind, Corpus::None);
        let bytes = &handoff.files["checked-package.json"];
        let admitted = admit(&files_of(&handoff, bytes), &handoff.evidence, bounded())
            .unwrap_or_else(|refusal| panic!("{kind}: {refusal}"));
        let CheckedPackageDispatchResult::AdmittedV2(read) =
            direct(bytes, bounded(), &handoff.evidence)
        else {
            panic!("{kind}: the reader refused its own handoff");
        };
        assert_eq!(
            admitted.package(),
            &*read,
            "{kind}: the admitted package is the reader's value"
        );
        assert_eq!(admitted.package().name(), "r");
        assert_eq!(
            admitted.package().temporal_bindings().map(<[_]>::len),
            Some(rows),
            "{kind}: one binding row per selected clock"
        );
        assert_eq!(admitted.identities().package_name(), "r");
        assert_eq!(admitted.equality().package_name(), "r");
    }
}

fn manifest(handoff: &Handoff) -> Vec<Value> {
    let manifest: Value =
        serde_json::from_slice(&handoff.files["mutations/manifest.json"]).expect("manifest JSON");
    assert_eq!(manifest["format"], "quire.checked-handoff.mutations/1");
    manifest["cases"].as_array().expect("cases").clone()
}

/// FR-043-AC-2: every `reader` case of the standard corpus returns the code,
/// loci, cause and node locus its manifest row names, unchanged.
/// FR-043-AC-3: every `selection` case is admitted by the reader and refused by
/// the selection with its manifest axis, unchanged.
#[test]
#[trace("FR-043-AC-2", "FR-043-AC-3", "TC-149")]
fn the_standard_corpus_gets_each_cases_declared_refusal() {
    let mut reader_cases = 0;
    let mut selection_cases = 0;
    for (kind, total) in [
        ("event_position", 22),
        ("fixed_sample", 5),
        ("timestamped_event", 3),
    ] {
        let handoff = written(kind, Corpus::Standard);
        let cases = manifest(&handoff);
        assert_eq!(cases.len(), total, "{kind}: the corpus the writer wrote");
        for case in cases {
            let identity = case["identity"].as_str().expect("identity");
            let file = case["input"]["file"].as_str().expect("file");
            let bytes = &handoff.files[file];
            let result = admit(&files_of(&handoff, bytes), &handoff.evidence, bounded());
            match case["stage"].as_str().expect("stage") {
                "reader" => {
                    reader_cases += 1;
                    let refusal = reader_refusal(result);
                    let expected = &case["expected"];
                    assert_eq!(
                        spell(&refusal.code),
                        expected["code"].as_str().expect("code"),
                        "{identity}"
                    );
                    assert_eq!(
                        refusal.cause.as_ref().map(spell).as_deref(),
                        expected["cause"].as_str(),
                        "{identity}"
                    );
                    assert_eq!(
                        refusal
                            .loci
                            .first()
                            .and_then(|locus| locus.path.as_ref())
                            .map(ToString::to_string),
                        expected["pointer"].as_str().map(str::to_owned),
                        "{identity}"
                    );
                    let CheckedPackageDispatchResult::Refused(owner) =
                        direct(bytes, bounded(), &handoff.evidence)
                    else {
                        panic!("{identity}: the reader admitted a reader case");
                    };
                    assert_eq!(
                        refusal, owner,
                        "{identity}: carried unchanged, loci and node locus included"
                    );
                }
                "selection" => {
                    selection_cases += 1;
                    let Err(Refusal::Selection(refusal)) = result else {
                        panic!("{identity}: expected a selection refusal, got {result:?}");
                    };
                    assert_eq!(
                        refusal.axis.name(),
                        case["expected"]["axis"].as_str().expect("axis"),
                        "{identity}"
                    );
                    assert!(
                        matches!(
                            direct(bytes, bounded(), &handoff.evidence),
                            CheckedPackageDispatchResult::AdmittedV2(_)
                        ),
                        "{identity}: the reader admits a selection case"
                    );
                }
                other => panic!("{identity}: unknown stage {other}"),
            }
        }
    }
    assert_eq!((reader_cases, selection_cases), (20, 10));
}

/// FR-043-AC-3: a changed caller selection identity over unchanged offered
/// bytes is a selection refusal and no admitted package.
#[test]
#[trace("FR-043-AC-3", "TC-149")]
fn a_changed_selection_identity_over_unchanged_bytes_is_refused() {
    let handoff = written("event_position", Corpus::None);
    let mut selection: Value =
        serde_json::from_slice(&handoff.files["selection.json"]).expect("JSON");
    selection["clauses"][0]["profile"]["identity"] = Value::from("other.profile.identity");
    let changed = serde_json::to_vec(&selection).expect("encode");
    let bytes = &handoff.files["checked-package.json"];
    let mut files = files_of(&handoff, bytes);
    files.selection = &changed;
    let refusal = match admit(&files, &handoff.evidence, bounded()) {
        Err(Refusal::Selection(refusal)) => refusal,
        other => panic!("expected a selection refusal, got {other:?}"),
    };
    assert_eq!(refusal.axis, SelectionAxis::DefinitionIdentity);
    assert_eq!(refusal.clause.as_deref(), Some("Due"));
}

fn rust_files(directory: &Path, found: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(directory).expect("directory") {
        let path = entry.expect("entry").path();
        if path.is_dir() {
            rust_files(&path, found);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            found.push(path);
        }
    }
}

/// FR-043-AC-4: the only call that reads checked-package bytes is
/// `read_checked_package`, and no source declares a refusal code, axis or
/// format string of the checked package.
#[test]
#[trace("FR-043-AC-4", "TC-149")]
fn no_source_reads_declares_or_names_the_checked_package_format() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rust_files(&root, &mut files);
    assert!(!files.is_empty());
    let mut readers = Vec::new();
    for file in files {
        let text = std::fs::read_to_string(&file).expect("source");
        let name = file
            .strip_prefix(&root)
            .expect("below src")
            .display()
            .to_string();
        if text.contains("read_checked_package(") {
            readers.push(name.clone());
        }
        for forbidden in [
            "enum CheckedPackageRefusalCode",
            "enum SelectionAxis",
            "enum CheckedPackageLimit",
            "quire.checked-package/",
            "quire.checked-handoff.",
        ] {
            assert!(!text.contains(forbidden), "{name} names {forbidden}");
        }
    }
    assert_eq!(readers, ["checked_package.rs"]);
}

/// FR-043-AC-5: bytes of the retired compiled-protocol formats offered at the
/// checked-package entry point are a typed reader refusal that names the
/// version it read; nothing is detected, upgraded or repaired.
#[test]
#[trace("FR-043-AC-5", "TC-149")]
fn retired_format_bytes_are_refused_and_never_upgraded() {
    let handoff = written("event_position", Corpus::None);
    let mut package: Value =
        serde_json::from_slice(&handoff.files["checked-package.json"]).expect("JSON");
    for retired in ["quire.compiled-protocol/1", "quire.compiled-protocol/2"] {
        package["contract_version"] = Value::from(retired);
        let bytes = serde_json::to_vec(&package).expect("encode");
        let refusal = reader_refusal(admit(
            &files_of(&handoff, &bytes),
            &handoff.evidence,
            bounded(),
        ));
        assert_eq!(
            refusal.code,
            CheckedPackageRefusalCode::UnknownContractVersion
        );
        assert_eq!(refusal.contract_version.as_deref(), Some(retired));
    }
}

/// FR-043-AC-6: a bare JSON number outside the safe-integer interval, a
/// signed-64 overflow written as a bare number and a non-canonical rational in a
/// package field each return the reader's refusal unchanged.
#[test]
#[trace("FR-043-AC-6", "TC-149")]
fn numeric_defects_are_the_readers_refusals() {
    let handoff = written("event_position", Corpus::None);
    let text = String::from_utf8(handoff.files["checked-package.json"].clone()).expect("UTF-8");
    for (label, number) in [
        ("unsafe bare number", "9007199254740993"),
        ("signed-64 overflow", "9223372036854775808"),
    ] {
        let bytes = text
            .replacen("\"start\":1184", &format!("\"start\":{number}"), 1)
            .into_bytes();
        let refusal = reader_refusal(admit(
            &files_of(&handoff, &bytes),
            &handoff.evidence,
            bounded(),
        ));
        assert_eq!(
            refusal.code,
            CheckedPackageRefusalCode::NoncanonicalWire,
            "{label}"
        );
        let CheckedPackageDispatchResult::Refused(owner) =
            direct(&bytes, bounded(), &handoff.evidence)
        else {
            panic!("{label}: the reader admitted it");
        };
        assert_eq!(refusal, owner, "{label}: carried unchanged");
    }

    let fixed = written("fixed_sample", Corpus::None);
    let mut package: Value =
        serde_json::from_slice(&fixed.files["checked-package.json"]).expect("JSON");
    *package
        .pointer_mut("/temporal_bindings/0/clock/period/numerator")
        .expect("field") = Value::from("02");
    let bytes = serde_json::to_vec(&package).expect("encode");
    let refusal = reader_refusal(admit(&files_of(&fixed, &bytes), &fixed.evidence, bounded()));
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MalformedWire);
    let CheckedPackageDispatchResult::Refused(owner) = direct(&bytes, bounded(), &fixed.evidence)
    else {
        panic!("non-canonical rational: the reader admitted it");
    };
    assert_eq!(refusal, owner, "non-canonical rational: carried unchanged");
}

fn with_limit(limit: CheckedPackageLimit, value: u64) -> CheckedPackageReadLimits {
    let mut limits = bounded();
    match limit {
        CheckedPackageLimit::Bytes => limits.bytes = value,
        CheckedPackageLimit::Nodes => limits.nodes = value,
        CheckedPackageLimit::Edges => limits.edges = value,
        CheckedPackageLimit::Occurrences => limits.occurrences = value,
        CheckedPackageLimit::Diagnostics => limits.diagnostics = value,
        CheckedPackageLimit::Work => limits.work = value,
    }
    limits
}

fn ceiling(limit: CheckedPackageLimit) -> u64 {
    match limit {
        CheckedPackageLimit::Bytes => bounded().bytes,
        CheckedPackageLimit::Nodes => bounded().nodes,
        CheckedPackageLimit::Edges => bounded().edges,
        CheckedPackageLimit::Occurrences => bounded().occurrences,
        CheckedPackageLimit::Diagnostics => bounded().diagnostics,
        CheckedPackageLimit::Work => bounded().work,
    }
}

/// FR-043-AC-7: a limit one below the measured usage returns the reader's
/// incomplete result naming that limit, with no admitted package; the exact
/// limit admits.
#[test]
#[trace("FR-043-AC-7", "TC-149")]
fn a_limit_one_below_usage_is_incomplete_and_the_exact_limit_admits() {
    let handoff = written("event_position", Corpus::None);
    let bytes = &handoff.files["checked-package.json"];
    let mut nonzero = 0;
    for limit in [
        CheckedPackageLimit::Bytes,
        CheckedPackageLimit::Nodes,
        CheckedPackageLimit::Edges,
        CheckedPackageLimit::Occurrences,
        CheckedPackageLimit::Diagnostics,
        CheckedPackageLimit::Work,
    ] {
        // The measured usage is the smallest limit that still admits.
        let (mut low, mut high) = (0, ceiling(limit));
        while low < high {
            let middle = low + (high - low) / 2;
            if admit(
                &files_of(&handoff, bytes),
                &handoff.evidence,
                with_limit(limit, middle),
            )
            .is_ok()
            {
                high = middle;
            } else {
                low = middle + 1;
            }
        }
        let usage = low;
        if usage == 0 {
            continue;
        }
        nonzero += 1;
        match admit(
            &files_of(&handoff, bytes),
            &handoff.evidence,
            with_limit(limit, usage - 1),
        ) {
            Err(Refusal::Incomplete(incomplete)) => {
                assert_eq!(incomplete.limit_kind, limit);
                assert_eq!(incomplete.limit, usage - 1);
            }
            other => panic!("{limit:?}: expected an incomplete read, got {other:?}"),
        }
        assert!(admit(
            &files_of(&handoff, bytes),
            &handoff.evidence,
            with_limit(limit, usage)
        )
        .is_ok());
    }
    assert!(
        nonzero >= 4,
        "bytes, nodes, edges and work are measured: {nonzero}"
    );
}

/// FR-043-AC-8: an identity or equality file of another format, or one whose
/// row names a node the package does not hold, returns QSL's file refusal
/// unchanged and no admitted handoff.
#[test]
#[trace("FR-043-AC-8", "TC-149")]
fn identity_and_equality_files_are_refused_with_their_owners_refusals() {
    let handoff = written("event_position", Corpus::None);
    let bytes = &handoff.files["checked-package.json"];

    let mut identities: Value =
        serde_json::from_slice(&handoff.files["identities.json"]).expect("JSON");
    identities["format"] = Value::from("quire.checked-handoff.identities/2");
    let other = serde_json::to_vec(&identities).expect("encode");
    let mut files = files_of(&handoff, bytes);
    files.identities = &other;
    match admit(&files, &handoff.evidence, bounded()) {
        Err(Refusal::Identities(IdentityFileRefusal::Format(format))) => {
            assert_eq!(format, "quire.checked-handoff.identities/2");
        }
        other => panic!("expected the identity file's format refusal, got {other:?}"),
    }

    identities["format"] = Value::from("quire.checked-handoff.identities/1");
    identities["declarations"][0]["structural_node"]["package"] = Value::from("elsewhere");
    let unjoined = serde_json::to_vec(&identities).expect("encode");
    files.identities = &unjoined;
    match admit(&files, &handoff.evidence, bounded()) {
        Err(Refusal::Identities(IdentityFileRefusal::Unjoined { index: 0, .. })) => {}
        other => panic!("expected an unjoined identity row, got {other:?}"),
    }

    let mut equality: Value =
        serde_json::from_slice(&handoff.files["equality.json"]).expect("JSON");
    equality["format"] = Value::from("quire.checked-handoff.equality/2");
    let other = serde_json::to_vec(&equality).expect("encode");
    let mut files = files_of(&handoff, bytes);
    files.equality = &other;
    match admit(&files, &handoff.evidence, bounded()) {
        Err(Refusal::Equality(EqualityFileRefusal::Format(format))) => {
            assert_eq!(format, "quire.checked-handoff.equality/2");
        }
        other => panic!("expected the equality file's format refusal, got {other:?}"),
    }
}
