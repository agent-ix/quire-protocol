// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-153: the retired compiler surfaces and integrity lanes are gone
//! (FR-047, AC-1, AC-2 and AC-4).

use std::path::Path;

use ix_trace_rs::trace;

fn rust_files(dir: &Path, found: &mut Vec<(String, String)>) {
    for entry in std::fs::read_dir(dir).expect("directory") {
        let path = entry.expect("entry").path();
        if path.is_dir() {
            rust_files(&path, found);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            found.push((
                path.display().to_string(),
                std::fs::read_to_string(&path).expect("source"),
            ));
        }
    }
}

/// The crate's production sources and manifest, as (path, text) pairs. The
/// scan is non-vacuous: it reads the sources and both manifests.
fn crate_text() -> Vec<(String, String)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    rust_files(&root.join("src"), &mut files);
    assert!(files.len() > 10, "the scan reads the crate's sources");
    for manifest in ["Cargo.toml", "Cargo.lock"] {
        files.push((
            manifest.to_owned(),
            std::fs::read_to_string(root.join(manifest)).expect("manifest"),
        ));
    }
    files
}

fn code(text: &str) -> String {
    text.lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// FR-047-AC-1: after the repoint no source file or manifest names
/// `quire_spec_language::number`.
#[test]
#[trace("FR-047-AC-1", "TC-153")]
fn no_source_or_manifest_names_the_number_module() {
    let named = ["quire_spec_language", "number"].join("::");
    for (path, text) in crate_text() {
        assert!(!code(&text).contains(&named), "{path} names {named}");
    }
    // The control: the crate does name the QSL root it imports from.
    assert!(crate_text()
        .iter()
        .any(|(_, text)| text.contains("quire_spec_language::checked_handoff")));
}

/// FR-047-AC-2: no `handoff-writer` feature and no retired format identity
/// in production source or public API; the refusal fixtures of FR-043-AC-5
/// (under `tests/`) are the one exception and are not scanned.
#[test]
#[trace("FR-047-AC-2", "TC-153")]
fn no_retired_feature_or_format_identity_remains() {
    let identities = [
        ["quire.compiled-protocol", "/1"].concat(),
        ["quire.compiled-protocol", "/2"].concat(),
        ["quire.protocol.v2-mutations", "/1"].concat(),
    ];
    for (path, text) in crate_text() {
        let live = code(&text);
        assert!(
            !live.contains("handoff-writer"),
            "{path} names handoff-writer"
        );
        for identity in &identities {
            assert!(!live.contains(identity.as_str()), "{path} names {identity}");
        }
    }
    let manifest =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
            .expect("manifest");
    assert!(
        !manifest.contains("[features]"),
        "the crate has no features table"
    );
}

/// FR-047-AC-4: no checksum or seal lane, no `ArtifactRef` digest or revision
/// and no integrity digest of an admitted package: a package is identified
/// by its canonical bytes. The result and version-lock content digests of
/// FR-009 and FR-015 are other items and stay.
#[test]
#[trace("FR-047-AC-4", "TC-153")]
fn no_integrity_lane_or_artifact_digest_remains() {
    let sources: Vec<_> = crate_text()
        .into_iter()
        .filter(|(path, _)| path.ends_with(".rs"))
        .collect();
    for (path, text) in &sources {
        for line in code(text).lines() {
            let lower = line.to_ascii_lowercase();
            for word in lower.split(|c: char| !c.is_ascii_alphanumeric()) {
                assert!(
                    !matches!(word, "checksum" | "seal" | "sealed" | "artifactref"),
                    "{path} names {word}: {line}"
                );
            }
        }
    }
    // The control: the digests FR-047 keeps are still here.
    assert!(sources
        .iter()
        .any(|(path, _)| path.ends_with("src/version_lock.rs")));
    assert!(sources
        .iter()
        .any(|(path, _)| path.ends_with("src/digest/byte.rs")));
}
