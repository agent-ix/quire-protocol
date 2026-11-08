// SPDX-License-Identifier: AGPL-3.0-or-later
//! The handoff directory of recipe R, written in process by QSL's writer
//! (QSL FR-517) from the recipe inputs under `tests/fixtures/checked-handoff`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use qsl_package::NativeJoinLimits;
use quire_contract_model::{
    CheckedPackageEvidence, ClockConfiguration, ClockExactNumber, NativeJoinBudget,
};
use quire_spec_language::checked_handoff::{evidence, write, CheckedHandoffRequest, Corpus};
use quire_spec_language::emit_checked_v2::{ClockSelection, Request};

const DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/checked-handoff"
);

fn recipe(kind: &str) -> (&'static str, Vec<(&'static str, ClockConfiguration)>) {
    match kind {
        "event_position" => (
            "unit-event-position.native",
            ["Full", "Partial", "Due", "DueLate"]
                .into_iter()
                .map(|clause| {
                    (
                        clause,
                        ClockConfiguration::EventPosition {
                            sequence_authority: "events-authority".into(),
                        },
                    )
                })
                .collect(),
        ),
        "fixed_sample" => (
            "unit-fixed-sample.native",
            vec![(
                "DueSample",
                ClockConfiguration::FixedSample {
                    epoch: ClockExactNumber::Integer {
                        decimal: "0".into(),
                    },
                    period: ClockExactNumber::Rational {
                        numerator: "1".into(),
                        denominator: "10".into(),
                    },
                    unit: "milliseconds".into(),
                },
            )],
        ),
        _ => (
            "unit-timestamped-event.native",
            vec![(
                "DueTimestamp",
                ClockConfiguration::TimestampedEvent {
                    timestamp_unit: "rfc3339".into(),
                },
            )],
        ),
    }
}

/// The finite native-join ceilings every test pass states for itself: 2^20
/// work steps and 2^20 logical storage units, the values QSL's own handoff
/// example uses (QSL FR-531 gives a fixture no default).
const NATIVE_JOIN: NativeJoinLimits = NativeJoinLimits::new(1 << 20, 1 << 20);

/// A new native-join budget of [`NATIVE_JOIN`] for one pass.
pub fn join_budget() -> NativeJoinBudget {
    NATIVE_JOIN.budget()
}

/// The request for the package of `kind`.
pub fn request(kind: &str, corpus: Corpus) -> CheckedHandoffRequest {
    let (unit, clocks) = recipe(kind);
    CheckedHandoffRequest {
        emit: Request {
            source: PathBuf::from(format!("{DIR}/{unit}")),
            package_name: "r".into(),
            models: vec![PathBuf::from(format!("{DIR}/model.semantic-ir.json"))],
            libraries: vec![(
                "agent-ix/checked-handoff-library".into(),
                PathBuf::from(format!("{DIR}/library.native")),
            )],
            execution_points: None,
            temporal_selections: clocks
                .into_iter()
                .map(|(clause, clock)| ClockSelection {
                    clause: clause.into(),
                    clock,
                })
                .collect(),
            out: None,
        },
        corpus,
        native_join: NATIVE_JOIN,
    }
}

/// One written handoff: its files by relative path and the evidence the
/// writer assembled for it.
pub struct Handoff {
    _holder: tempfile::TempDir,
    /// Every file below the handoff root.
    pub files: BTreeMap<String, Vec<u8>>,
    /// The evidence for reading `checked-package.json`.
    pub evidence: CheckedPackageEvidence,
}

/// Write the handoff of `kind` with `corpus` into a fresh directory and read
/// every file back.
pub fn written(kind: &str, corpus: Corpus) -> Handoff {
    let holder = tempfile::tempdir().expect("temporary directory");
    let out = holder.path().join("handoff");
    let asked = request(kind, corpus);
    finish(holder, out, asked)
}

/// Write a handoff from `source` while retaining the other inputs of `kind`.
#[allow(dead_code)] // Only TC-150 needs a second handoff; other test binaries include this module.
pub fn written_source(kind: &str, source: &str, corpus: Corpus) -> Handoff {
    let holder = tempfile::tempdir().expect("temporary directory");
    let source_path = holder.path().join("unit.native");
    std::fs::write(&source_path, source).expect("write source fixture");
    let mut asked = request(kind, corpus);
    asked.emit.source = source_path;
    let out = holder.path().join("handoff");
    finish(holder, out, asked)
}

/// Write a handoff from an explicitly adjusted request.
#[allow(dead_code)] // Only TC-150 needs an alternate temporal selection; other test binaries include this module.
pub fn written_request(asked: CheckedHandoffRequest) -> Handoff {
    let holder = tempfile::tempdir().expect("temporary directory");
    let out = holder.path().join("handoff");
    finish(holder, out, asked)
}

fn finish(holder: tempfile::TempDir, out: PathBuf, asked: CheckedHandoffRequest) -> Handoff {
    write(&asked, &out).expect("the handoff is written");
    let files = tree(&out);
    let evidence = evidence(&asked).expect("evidence");
    Handoff {
        _holder: holder,
        files,
        evidence,
    }
}

fn tree(root: &Path) -> BTreeMap<String, Vec<u8>> {
    fn walk(root: &Path, directory: &Path, files: &mut BTreeMap<String, Vec<u8>>) {
        for entry in std::fs::read_dir(directory).expect("read directory") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                walk(root, &path, files);
            } else {
                let name = path
                    .strip_prefix(root)
                    .expect("below root")
                    .display()
                    .to_string();
                files.insert(name, std::fs::read(&path).expect("read file"));
            }
        }
    }
    let mut files = BTreeMap::new();
    walk(root, root, &mut files);
    files
}
