// SPDX-License-Identifier: AGPL-3.0-or-later
//! IT-001/Task-009: actual A-owned four-unit compiled-protocol handoff.

use std::{fs, path::Path};

use ix_trace_rs::trace;
use quire_contract_ir as ir;
use quire_protocol::{
    digest::producer::{CrossDomainDigest, DigestDomain, DigestSlot, OfferedDomain},
    inventory::{
        axis::DivergedSelection,
        map::{admit_discriminating, AdmissionRefusal},
        SelectedInventory,
    },
    model, wire, AdmittedModel, AdmittedPackage, ByteDigest, Expected, ExpectedDeclaration,
    ExpectedForeignSource, ExpectedSource, Invalid, Limits, NumberComponent, NumberError,
    SuppliedDependency,
};
use quire_spec_language::{
    formal_source::FormalSource,
    model_source::{self, ModelSourceLimits},
    native_model::ModelLimits,
    protocol_artifact::{
        handoff::{SelectedDependency, Selection, PUBLISHED_V1_HANDOFF, PUBLISHED_V1_OFFER_FILE},
        NumberWire,
    },
    Source, SourceIdentity,
};

struct Fixture {
    selection: Selection,
    dependency_bytes: Vec<Vec<u8>>,
    model: quire_spec_language::native_model::NativeModel,
}

impl Fixture {
    fn load(root: &Path) -> Self {
        // This B-owned immutable selection fixture is loaded before the offer.
        // It is distinct from A's emitted `expected.json`, which the harness
        // never reads. The offered package can therefore not select its own
        // acceptance context.
        let selection: Selection =
            serde_json::from_slice(include_bytes!("fixtures/a-four-unit-selections.json"))
                .expect("B selection fixture shape");
        // Each selected source names a file in the handoff whose bytes hash to
        // the digest the selection carries. A non-empty-string check passed on
        // any fixture ever committed; this one fails whenever the selection and
        // the handoff are derived from different producer runs, which is how a
        // hybrid fixture went unnoticed across two A revisions.
        for source in &selection.sources {
            let bytes = fs::read(root.join(&source.file))
                .unwrap_or_else(|error| panic!("selected source {}: {error}", source.file));
            assert_eq!(
                ByteDigest::of(&bytes),
                source.source.artifact.digest,
                "selected source {} does not hash to its selected digest",
                source.file
            );
        }
        let model_source = &selection.model.source;
        let model_bytes = fs::read(root.join(&selection.model.source_file)).expect("model bytes");
        assert_eq!(ByteDigest::of(&model_bytes), model_source.artifact.digest);
        let source = Source::read(
            SourceIdentity {
                identity: model_source.native.identity.clone(),
                revision: model_source.native.revision.clone(),
            },
            &model_source.path,
            &model_bytes,
            quire_spec_language::Limits::default().source_bytes,
        )
        .expect("model source");
        let formal = FormalSource::new(
            source,
            ir::SourceIdentity::new(
                ir::SourceDocumentId::new(&model_source.formal.document).expect("formal document"),
                ir::SourceRevision::new(
                    model_source
                        .formal
                        .revision
                        .value
                        .parse()
                        .expect("formal revision"),
                )
                .expect("revision"),
            ),
        );
        let model = model_source::read(
            formal,
            &selection.model.source_format,
            ModelSourceLimits::default(),
        )
        .expect("admitted model draft")
        .admit(ModelLimits::default())
        .expect("admitted model");
        assert_eq!(
            model.artifact_bytes(),
            dependency_bytes(root, &selection.dependencies, &selection.model.artifact)
        );
        let dependency_bytes = selection
            .dependencies
            .iter()
            .map(|dependency| fs::read(root.join(&dependency.file)).expect("dependency bytes"))
            .collect();
        Self {
            selection,
            dependency_bytes,
            model,
        }
    }

    fn admit(&self, bytes: &[u8]) -> quire_protocol::Report<quire_protocol::AdmittedPackage> {
        self.admit_selected(&self.selection, bytes)
    }

    fn admit_selected(
        &self,
        selection: &Selection,
        bytes: &[u8],
    ) -> quire_protocol::Report<quire_protocol::AdmittedPackage> {
        self.offer(selection, bytes, |bytes, expected| {
            quire_protocol::intake::v1::admit(bytes, expected, Limits::default())
        })
    }

    /// Offer the same bytes through B's discriminating consumer boundary, which
    /// resolves A's single shared `Invalid::Selection` code against B's own
    /// independently selected inventory and guards the digest domains first.
    fn discriminate(
        &self,
        selection: &Selection,
        bytes: &[u8],
    ) -> Result<AdmittedPackage, AdmissionRefusal> {
        self.offer(selection, bytes, |bytes, expected| {
            admit_discriminating(bytes, expected, Limits::default())
        })
    }

    /// Build the independently selected acceptance context once and hand it to
    /// the chosen consumer entry point.
    fn offer<T>(
        &self,
        selection: &Selection,
        bytes: &[u8],
        entry: impl FnOnce(&[u8], &Expected<'_>) -> T,
    ) -> T {
        assert_eq!(
            selection.dependencies.len(),
            self.dependency_bytes.len(),
            "each selected dependency must have supplied bytes"
        );
        let dependencies = selection
            .dependencies
            .iter()
            .zip(&self.dependency_bytes)
            .map(|(dependency, bytes)| SuppliedDependency {
                artifact: &dependency.artifact,
                bytes,
                requires: &dependency.requires,
            })
            .collect::<Vec<_>>();
        let foreign = ExpectedForeignSource {
            artifact: &selection.model.source.artifact,
            formal: &selection.model.source.formal,
            bytes: self.model.source().source().text().as_bytes(),
        };
        let models = [AdmittedModel {
            artifact: &selection.model.artifact,
            model: &self.model,
            source: &foreign,
        }];
        let sources = selection
            .sources
            .iter()
            .map(|source| {
                let declarations = source
                    .declarations
                    .iter()
                    .map(|item| ExpectedDeclaration {
                        name: &item.name,
                        span: &item.span,
                        requirement: &item.requirement,
                        clause: &item.clause,
                        execution: &item.execution,
                    })
                    .collect::<Vec<_>>();
                (source, declarations)
            })
            .collect::<Vec<_>>();
        let expected_sources = sources
            .iter()
            .map(|(source, declarations)| ExpectedSource {
                artifact: &source.source.artifact,
                native: &source.source.native,
                path: &source.source.path,
                formal: &source.source.formal,
                text: &source.source.text,
                declarations,
            })
            .collect::<Vec<_>>();
        entry(
            bytes,
            &Expected {
                artifact: &selection.artifact,
                contract: &selection.contract,
                baseline: &selection.baseline,
                producer: &selection.producer,
                language: &selection.language,
                sources: &expected_sources,
                dependencies: &dependencies,
                models: &models,
            },
        )
    }
}

fn dependency_bytes(
    root: &Path,
    dependencies: &[SelectedDependency],
    expected: &wire::ArtifactRef,
) -> Vec<u8> {
    let dependency = dependencies
        .iter()
        .find(|item| item.artifact == *expected)
        .expect("selected model dependency");
    fs::read(root.join(&dependency.file)).expect("selected model bytes")
}

fn fixture_root() -> std::path::PathBuf {
    std::path::PathBuf::from(PUBLISHED_V1_HANDOFF)
}

fn offered_bytes(root: &Path) -> Vec<u8> {
    fs::read(root.join(PUBLISHED_V1_OFFER_FILE)).expect("A compiled bytes")
}

fn resealed(selection: &Selection, bytes: &[u8]) -> Selection {
    let mut selection = selection.clone();
    selection.artifact.digest = ByteDigest::of(bytes);
    selection
}

fn replace_once(bytes: &[u8], before: &[u8], after: &[u8]) -> Vec<u8> {
    assert_eq!(
        before.len(),
        after.len(),
        "mutation preserves compact wire length"
    );
    let index = bytes
        .windows(before.len())
        .position(|window| window == before)
        .expect("fixture contains mutation target");
    let mut changed = bytes.to_vec();
    changed[index..index + before.len()].copy_from_slice(after);
    changed
}

/// The complete top-level `baseline` field as A's producer emits it. Scoping a
/// mutation to this one field keeps the axis under test on the baseline
/// selection: the same artifact also appears in the dependency inventory, and a
/// mutation that moved both could be refused by either check.
///
/// It is rendered through A's own `Serialize` impl for `wire::ArtifactRef`
/// rather than mirrored as a literal here. `CLAUDE.md` forbids restating an
/// owned wire shape, "not even in tests", and the literal this replaced spelled
/// out `refVersion`, `kind`, `authority`, `identity`, `revision`, `digest` and
/// `wire` in A's declared member order — seven names and an ordering B would
/// have had to re-learn from A every time A changed one.
///
/// `replace_unique` is what proves the rendering is byte-exact: it requires the
/// result to occur exactly once in the payload A emitted.
fn baseline_field(selection: &Selection) -> String {
    format!(
        r#""baseline":{}"#,
        serde_json::to_string(&selection.baseline)
            .expect("A serialises its own artifact reference")
    )
}

/// Replace a target that must occur exactly once, so a future A fixture cannot
/// silently widen a one-axis mutation across additional fields.
fn replace_unique(bytes: &[u8], before: &[u8], after: &[u8]) -> Vec<u8> {
    assert_eq!(
        before.len(),
        after.len(),
        "mutation preserves compact wire length"
    );
    let occurrences = bytes
        .windows(before.len())
        .filter(|window| *window == before)
        .count();
    assert_eq!(occurrences, 1, "mutation target must identify one field");
    replace_once(bytes, before, after)
}

/// One baseline field with exactly one substring respelled in place.
///
/// The uniqueness check is a precondition on this test's own construction, not
/// a gate on the library: it fails if the mutation target stops naming one
/// field of the rendered reference. Equal length is not checked here —
/// `replace_unique` enforces it against the whole payload, which is the level
/// at which it matters.
fn mutated_baseline_field(baseline: &str, before: &str, after: &str) -> String {
    assert_eq!(
        baseline.matches(before).count(),
        1,
        "`{before}` must name exactly one field of the baseline reference"
    );
    baseline.replace(before, after)
}

/// The same string with its final character replaced, so a one-axis mutation
/// stays one axis wide and the payload keeps its length.
///
/// Deriving the target from the selection rather than spelling A's emitted
/// value into this file means a regenerated fixture moves the mutation with it
/// instead of leaving a literal that no longer occurs.
fn last_character_changed(value: &str) -> String {
    let (head, last) = value
        .split_at_checked(value.len() - 1)
        .expect("a non-empty selected value on a character boundary");
    let replacement = if last == "7" { "6" } else { "7" };
    format!("{head}{replacement}")
}

/// An exact integer as A spells it on the wire.
fn integer(decimal: &str) -> wire::Integer {
    wire::Integer(NumberWire::Integer {
        decimal: decimal.to_owned(),
    })
}

/// The `operation` field of one offered value, rendered through A's own
/// `ValueOperation` and `Number` impls.
///
/// The anchors below were literals spelling `operation`, `kind`, `number`,
/// `value`, `integer`, `decimal`, `rational`, `numerator` and `denominator` —
/// nine A-owned names and their nesting, restated in a repository `CLAUDE.md`
/// forbids from mirroring an owned wire shape "not even in tests". Rendering
/// them keeps exactly one owner. `respell_unique` proves the rendering is
/// byte-exact: it requires the anchor to occur exactly once in A's payload.
fn value_operation(operation: &wire::ValueOperation) -> String {
    format!(
        r#""operation":{}"#,
        serde_json::to_string(operation).expect("A serialises its own value operation")
    )
}

/// An offered integer value in the declaration graph, typed by the model scalar
/// `Tally` (domain 0..=5). Anchoring on the enclosing value operation keeps the
/// numeric mutations on a *value*; the same spelling also appears as a type
/// bound, where a different check would refuse it.
fn integer_value() -> String {
    value_operation(&wire::ValueOperation::Number {
        value: wire::Number(NumberWire::Integer {
            decimal: "5".to_owned(),
        }),
    })
}

/// The offered rational value, typed by the model scalar `Ratio`.
fn rational_value() -> String {
    value_operation(&wire::ValueOperation::Number {
        value: wire::Number(NumberWire::Rational {
            numerator: "1".to_owned(),
            denominator: "1".to_owned(),
        }),
    })
}

/// `Tally`'s declared representation bound, which must equal the selected
/// model's own bound rather than merely admit the values that occur.
///
/// A's `Representation::Integer` carries the two bounds inside a tagged object;
/// the anchor is its member list without that wrapper, so that a mutation lands
/// on a bound rather than on the alternative tag.
fn integer_bound() -> String {
    let representation = serde_json::to_string(&wire::Representation::Integer {
        minimum: integer("0"),
        maximum: integer("5"),
    })
    .expect("A serialises its own representation");
    let members = representation
        .strip_prefix(r#"{"kind":"integer","#)
        .and_then(|rest| rest.strip_suffix('}'))
        .unwrap_or_else(|| panic!("A's integer representation is tagged: {representation}"));
    members.to_owned()
}

/// Replace a target that must occur exactly once, without the compact-length
/// constraint: a numeric spelling's exact decimal width is the property under
/// test, so the mutation may resize the payload.
fn respell_unique(bytes: &[u8], before: &[u8], after: &[u8]) -> Vec<u8> {
    let occurrences = bytes
        .windows(before.len())
        .filter(|window| *window == before)
        .count();
    assert_eq!(occurrences, 1, "mutation target must identify one field");
    let index = bytes
        .windows(before.len())
        .position(|window| window == before)
        .expect("fixture contains mutation target");
    let mut changed = Vec::with_capacity(bytes.len() - before.len() + after.len());
    changed.extend_from_slice(&bytes[..index]);
    changed.extend_from_slice(after);
    changed.extend_from_slice(&bytes[index + before.len()..]);
    changed
}

/// Respell one numeric field inside an anchored, unique enclosing context.
///
/// As in `mutated_baseline_field`, the uniqueness check is a precondition on
/// the anchor this test supplies. The gate is `respell_unique`, which requires
/// the anchor to occur exactly once in the offered payload.
fn respell_within(bytes: &[u8], anchor: &str, before: &str, after: &str) -> Vec<u8> {
    assert_eq!(
        anchor.matches(before).count(),
        1,
        "`{before}` must occur once inside the supplied anchor"
    );
    respell_unique(
        bytes,
        anchor.as_bytes(),
        anchor.replace(before, after).as_bytes(),
    )
}

/// Re-encode an admitted package through A's accepted canonical encoder and
/// require the exact offered bytes back.
///
/// A's reader compares its own canonical re-encoding before admitting, so the
/// first two assertions below restate a check A has already made — they fail
/// only if A's intake path and A's `encode_candidate` disagree, which is a
/// defect in A that this lane would surface rather than one in B. They are
/// recorded as corroboration, not as B's gate, and the doc says so because a
/// test that reads as a gate and is not one is worse than no test.
///
/// The last assertion is B's. It hashes the re-encoded bytes in B's own raw
/// artifact domain and requires A's reported digest back, so a digest A
/// computed over anything but exactly these bytes fails here. That is also
/// what pins the canonical numeric spelling: every admitted number is
/// re-spelled from its validated signed-64 components rather than echoed, so
/// byte identity over the re-encoding is a statement about the numbers.
fn assert_round_trips(admitted: &quire_protocol::AdmittedPackage, offered: &[u8]) {
    let candidate = quire_spec_language::protocol_artifact::encode_candidate(
        admitted.package(),
        Limits::default(),
    )
    .into_result()
    .expect("canonical re-encoding");
    assert_eq!(candidate.bytes(), offered, "re-encoded bytes must be exact");
    assert_eq!(candidate.digest(), admitted.digest());
    assert_eq!(
        ByteDigest::of(candidate.bytes()),
        candidate.digest(),
        "A's reported digest must be B's own hash of the re-encoded bytes"
    );
}

fn numeric_error(report: quire_protocol::Report<quire_protocol::AdmittedPackage>) -> NumberError {
    match report.into_result().unwrap_err() {
        quire_protocol::Error::Numeric(error) => error,
        other => panic!("expected a numeric refusal, got {other:?}"),
    }
}

/// Return the first Cargo.lock package paragraph whose `name` exactly matches
/// `name`; this is a static dependency-boundary assertion, not a runtime
/// package reader.
fn locked_package(name: &str) -> &str {
    include_str!("../Cargo.lock")
        .split("[[package]]")
        .find(|paragraph| {
            paragraph
                .lines()
                .any(|line| line == format!(r#"name = "{name}""#))
        })
        .expect("locked package")
}

/// Every `.rs` file under `src/`, as (repository-relative path, contents),
/// sorted by path so a failure names the same file on every run.
///
/// `include_str!` can only name files a human already listed, which is exactly
/// the property under test: a containment claim that inspects one hand-named
/// file is satisfied by putting the second reader in any other file. This walks
/// the tree instead, so a new module is covered the moment it is added.
fn b_sources() -> Vec<(String, String)> {
    fn walk(dir: &Path, prefix: &str, into: &mut Vec<(String, String)>) {
        let mut entries = fs::read_dir(dir)
            .unwrap_or_else(|error| panic!("read {}: {error}", dir.display()))
            .map(|entry| entry.expect("directory entry").path())
            .collect::<Vec<_>>();
        entries.sort();
        for path in entries {
            let name = path
                .file_name()
                .expect("named entry")
                .to_str()
                .expect("UTF-8 file name")
                .to_owned();
            let relative = format!("{prefix}{name}");
            if path.is_dir() {
                walk(&path, &format!("{relative}/"), into);
            } else if name.ends_with(".rs") {
                let contents = fs::read_to_string(&path).expect("UTF-8 source file");
                into.push((relative, contents));
            }
        }
    }

    let mut sources = Vec::new();
    walk(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src").as_path(),
        "src/",
        &mut sources,
    );
    sources
}

/// A's reading entry points, each of which B may delegate to from exactly one
/// place. `intake::v1::admit` and `intake::v2::admit` are those places.
const A_READER_CALLS: [(&str, &str); 2] = [
    (
        "quire_spec_language::protocol_artifact::read(",
        "src/intake/v1.rs",
    ),
    (
        "quire_spec_language::protocol_artifact::v2::read(",
        "src/intake/v2.rs",
    ),
];

/// Tracing: IT-001, Task-009.
#[test]
#[trace("IT-001", "Task-009")]
fn it_001_inventory_identifies_one_b_owned_reader_path() {
    // This is intentionally source/dependency inspection: the absence of a
    // second B-owned parser is a containment property with no runtime input
    // that can exercise it. It does not make a claim about A's compiler
    // internals, which remain A-owned behind these public reader calls.
    let sources = b_sources();
    assert!(
        sources.len() > 20,
        "the source walk found {} files, so it is not inspecting the tree",
        sources.len()
    );

    for (call, expected_path) in A_READER_CALLS {
        let sites = sources
            .iter()
            .filter(|(_, contents)| contents.contains(call))
            .map(|(path, contents)| (path.as_str(), contents.matches(call).count()))
            .collect::<Vec<_>>();
        assert_eq!(
            sites,
            [(expected_path, 1)],
            "`{call}` must be reached from exactly one versioned intake module"
        );
    }

    // A B-owned parser would arrive as a module, not as a call: name the shapes
    // `AD-001` forbids rather than only the two that were once written down.
    for (path, contents) in &sources {
        for forbidden in ["mod parser", "mod frontend", "mod lexer", "mod grammar"] {
            assert!(
                !contents.contains(forbidden),
                "{path} declares `{forbidden}`, which this repository does not own"
            );
        }
    }

    let manifest = include_str!("../Cargo.toml");
    let production = manifest
        .split("[dependencies]\n")
        .nth(1)
        .expect("production dependency section")
        .split("\n[dev-dependencies]")
        .next()
        .expect("production dependency terminator");
    let dependencies = production
        .lines()
        .filter_map(|line| line.split_once('=').map(|(name, _)| name.trim()))
        .collect::<Vec<_>>();
    assert_eq!(
        dependencies,
        [
            "quire-spec-language",
            "quire-observation",
            "quire-verification-contracts",
            "serde_json",
            "serde",
            "jsonschema",
            "sha2"
        ]
    );

    let consumer = locked_package("quire-protocol");
    assert!(consumer.contains(r#""quire-spec-language""#));
    assert!(consumer.contains(r#""quire-observation""#));
    assert!(consumer.contains(r#""quire-verification-contracts""#));
    assert!(consumer.contains(r#""sha2 0.10.9""#));
    assert!(consumer.contains(r#""serde_json""#));
    assert!(!consumer.contains(r#""logos""#));
}

/// Tracing: IT-001, FR-001-AC-1, Task-009.
///
/// This scenario statically links the handoff, so `FR-002-AC-1` would be the
/// natural tag. It is deliberately absent: the owner has ruled AC-1 unresolved
/// until runtime instance identities exist (issue #4), and A carries binding
/// requirements rather than instances, so nothing here can fail when AC-1 is
/// violated. Tagging it would count the criterion as backed by a test that
/// cannot check it.
#[test]
#[trace("IT-001", "FR-001-AC-1", "Task-009")]
fn it_001_accepts_and_links_the_actual_four_unit_handoff() {
    let root = fixture_root();
    let fixture = Fixture::load(&root);
    let bytes = offered_bytes(&root);
    let admitted = fixture
        .admit(&bytes)
        .into_result()
        .expect("positive admission");
    assert_eq!(admitted.package().sources.len(), 4);
    assert_eq!(admitted.package().declarations.len(), 7);
    let linked = model::v1::link_static(&admitted, 6).expect("static version-1 link");
    assert_eq!(linked.role_instances().len(), 2);
    assert_eq!(linked.channels().len(), 1);
    assert_eq!(linked.controls().len(), 16);
    assert_eq!(linked.edges().len(), 36);
}

/// Tracing: IT-001, Task-009.
#[test]
#[trace("IT-001", "Task-009")]
fn it_001_refuses_changed_artifact_bytes_before_linking() {
    let root = fixture_root();
    let fixture = Fixture::load(&root);
    let mut bytes = offered_bytes(&root);
    bytes.push(b' ');
    let error = fixture.admit(&bytes).into_result().unwrap_err();
    assert!(
        matches!(error, quire_protocol::Error::Invalid(Invalid::Seal)),
        "got {error:?}"
    );
}

/// Tracing: IT-001, Task-009.
#[test]
#[trace("IT-001", "Task-009")]
fn it_001_refuses_one_axis_producer_identity_change() {
    let root = fixture_root();
    let fixture = Fixture::load(&root);
    let bytes = offered_bytes(&root);
    // `native_protocol_handoff` also spells the producer binary identity and a
    // dependency identity; anchor on the implementation field so this stays the
    // producer-identity axis whatever order A emits the payload in.
    let changed = replace_unique(
        &bytes,
        br#""implementation":"quire-spec-language/native_protocol_handoff""#,
        br#""implementation":"quire-spec-language/native_protocol_handovv""#,
    );
    let selection = resealed(&fixture.selection, &changed);
    // The producer implementation label is its own axis: A collapses it into
    // the shared `Invalid::Selection`, so this asserts B's own producer-identity
    // code, which no other landed mutation can produce.
    let error = fixture.discriminate(&selection, &changed).unwrap_err();
    assert_eq!(
        error,
        AdmissionRefusal::Selection {
            axis: DivergedSelection::ProducerIdentity,
            source: quire_protocol::Error::Invalid(Invalid::Selection),
        },
        "got {error:?}"
    );
}

/// Tracing: IT-001, Task-009.
#[test]
#[trace("IT-001", "Task-009")]
fn it_001_refuses_one_axis_unknown_feature() {
    let root = fixture_root();
    let fixture = Fixture::load(&root);
    let bytes = offered_bytes(&root);
    let changed = replace_once(
        &bytes,
        b"quire.protocol.values/1",
        b"quire.protocol.valuex/1",
    );
    let selection = resealed(&fixture.selection, &changed);
    let error = fixture
        .admit_selected(&selection, &changed)
        .into_result()
        .unwrap_err();
    assert!(
        matches!(error, quire_protocol::Error::Unsupported(_)),
        "got {error:?}"
    );
}

/// Tracing: IT-001, FR-001-AC-2, Task-009.
#[test]
#[trace("IT-001", "FR-001-AC-2", "Task-009")]
fn it_001_refuses_a_mutated_referenced_operation_binding_before_linking() {
    let root = fixture_root();
    let fixture = Fixture::load(&root);
    let bytes = offered_bytes(&root);
    // BeforeApply's execution binding actually references Workflow::apply at
    // model export 14. Pointing it at Workflow (export 13) keeps the compact
    // representation well-formed but disagrees with B's independently selected
    // declaration inventory, so selection must refuse before package exposure.
    let changed = replace_unique(
        &bytes,
        br#""execution":{"kind":"pre","operation":{"model":0,"export":14}}"#,
        br#""execution":{"kind":"pre","operation":{"model":0,"export":13}}"#,
    );
    let selection = resealed(&fixture.selection, &changed);
    let error = fixture
        .admit_selected(&selection, &changed)
        .into_result()
        .unwrap_err();
    assert!(
        matches!(error, quire_protocol::Error::Invalid(Invalid::Selection)),
        "got {error:?}"
    );
}

/// Tracing: IT-001, Task-009.
#[test]
#[trace("IT-001", "Task-009")]
fn it_001_refuses_one_axis_non_numeric_integer_component() {
    let root = fixture_root();
    let fixture = Fixture::load(&root);
    let bytes = offered_bytes(&root);
    // This alters one exact numeric component but leaves compact JSON framing
    // untouched. Resealing proves that numeric validation, rather than only the
    // outer artifact digest, refuses the value before a linked package exists.
    let changed = respell_within(
        &bytes,
        &integer_value(),
        r#""decimal":"5""#,
        r#""decimal":"x""#,
    );
    let selection = resealed(&fixture.selection, &changed);
    let error = fixture
        .admit_selected(&selection, &changed)
        .into_result()
        .unwrap_err();
    assert!(
        matches!(error, quire_protocol::Error::Numeric(_)),
        "got {error:?}"
    );
}

/// Tracing: IT-001, Task-009.
#[test]
#[trace("IT-001", "Task-009")]
fn it_001_refuses_one_axis_wire_version_change() {
    let root = fixture_root();
    let fixture = Fixture::load(&root);
    let bytes = offered_bytes(&root);
    let changed = replace_once(
        &bytes,
        b"\"wire\":\"quire.compiled-protocol/1\"",
        b"\"wire\":\"quire.compiled-protocol/2\"",
    );
    let selection = resealed(&fixture.selection, &changed);
    let error = fixture
        .admit_selected(&selection, &changed)
        .into_result()
        .unwrap_err();
    assert!(
        matches!(
            error,
            quire_protocol::Error::Unsupported(quire_protocol::Unsupported::Wire)
        ),
        "got {error:?}"
    );
}

/// Tracing: IT-001, Task-009.
#[test]
#[trace("IT-001", "Task-009")]
fn it_001_refuses_one_axis_stale_baseline_selection() {
    let root = fixture_root();
    let fixture = Fixture::load(&root);
    let bytes = offered_bytes(&root);
    // Only the top-level baseline revision moves. B's frozen selection, not the
    // offer, decides which semantic baseline is accepted.
    let baseline = baseline_field(&fixture.selection);
    let selected = &fixture.selection.baseline.revision.value;
    let changed = replace_unique(
        &bytes,
        baseline.as_bytes(),
        mutated_baseline_field(
            &baseline,
            &format!(r#""value":"{selected}""#),
            &format!(r#""value":"{}""#, last_character_changed(selected)),
        )
        .as_bytes(),
    );
    let selection = resealed(&fixture.selection, &changed);
    // A superseded revision of the selected baseline identity: the baseline
    // selection state moved, not the baseline's identity and not its digest.
    let error = fixture.discriminate(&selection, &changed).unwrap_err();
    assert_eq!(
        error,
        AdmissionRefusal::Selection {
            axis: DivergedSelection::BaselineSelectionState,
            source: quire_protocol::Error::Invalid(Invalid::Selection),
        },
        "got {error:?}"
    );
}

/// Tracing: IT-001, Task-009.
#[test]
#[trace("IT-001", "Task-009")]
fn it_001_refuses_one_axis_changed_baseline_digest() {
    let root = fixture_root();
    let fixture = Fixture::load(&root);
    let bytes = offered_bytes(&root);
    // The same artifact stays correct in the dependency inventory, so only the
    // baseline digest check can refuse this offer.
    let baseline = baseline_field(&fixture.selection);
    let selected = fixture.selection.baseline.digest.to_string();
    let changed = replace_unique(
        &bytes,
        baseline.as_bytes(),
        mutated_baseline_field(&baseline, &selected, &last_character_changed(&selected)).as_bytes(),
    );
    let selection = resealed(&fixture.selection, &changed);
    // The value diverges inside the baseline's own producer digest domain, so
    // this is the digest axis and not the cross-domain axis below.
    let error = fixture.discriminate(&selection, &changed).unwrap_err();
    assert_eq!(
        error,
        AdmissionRefusal::Selection {
            axis: DivergedSelection::BaselineDigest,
            source: quire_protocol::Error::Invalid(Invalid::Selection),
        },
        "got {error:?}"
    );
}

/// Tracing: IT-001, Task-009.
#[test]
#[trace("IT-001", "Task-009")]
fn it_001_refuses_one_axis_cross_domain_baseline_digest() {
    let root = fixture_root();
    let fixture = Fixture::load(&root);
    let bytes = offered_bytes(&root);
    // Only the baseline digest's domain label moves; its hexadecimal body and
    // every other digest in the offer are unchanged. `sha512:` is not A's
    // vocabulary and is not claimed to be: it is a well-formed label B chose
    // precisely because A does not know it.
    let baseline = baseline_field(&fixture.selection);
    let selected = fixture.selection.baseline.digest.to_string();
    let foreign = selected.replacen("sha256:", "sha512:", 1);
    assert_ne!(foreign, selected, "the selected digest must carry a label");
    let changed = replace_unique(
        &bytes,
        baseline.as_bytes(),
        mutated_baseline_field(&baseline, &selected, &foreign).as_bytes(),
    );
    let selection = resealed(&fixture.selection, &changed);
    // A parse failure would prove nothing here: it would only say the label is
    // unknown to A's `ByteDigest`, which a well-formed substitution avoids
    // entirely. B must refuse this on the digest-domain axis itself.
    let error = fixture.discriminate(&selection, &changed).unwrap_err();
    assert_eq!(
        error,
        AdmissionRefusal::CrossDomainDigest(CrossDomainDigest {
            slot: DigestSlot::Baseline,
            selected: DigestDomain::RawArtifact,
            offered: OfferedDomain::ForeignLabel("sha512".to_owned()),
            value: "sha512:6f9cb8bf8521ca51a5e6340511b545db43001879446a59c5c87efe4985acdb06"
                .to_owned(),
        }),
        "got {error:?}"
    );
}

/// The load-bearing case: a cross-domain substitution B must *identify*.
///
/// The offered value is B's own RFC 8785/JCS protocol-result digest of the same
/// baseline reference, copied into the slot that accepts only the producer's
/// exact-bytes digest. It renders `sha256-jcs:`, the label its own domain
/// requires, so B has to name the domain it was given rather than report a bare
/// mismatch: the refusal below carries `Identified(VerificationJcs)`, not the
/// `ForeignLabel` an unknown algorithm produces.
///
/// That labelled spelling is itself the guard, and it is recent. `ResultDigest`
/// deliberately does not delegate to the producer's `Display` — see
/// `src/digest/jcs.rs`, "rendering both domains identically is what let a
/// cross-domain value pass the algorithm-label test as a merely divergent one".
/// Before that change this test asserted the offered value began `sha256:`, and
/// the assertion survived the change only because the lane does not run under
/// the gate. The inventory is still what tells the two domains apart; the label
/// now says which one is present.
///
/// Tracing: IT-001, FR-001-AC-4.
#[test]
#[trace("IT-001", "FR-001-AC-4")]
fn it_001_refuses_well_formed_cross_domain_baseline_digest() {
    let root = fixture_root();
    let fixture = Fixture::load(&root);
    let bytes = offered_bytes(&root);
    let result_domain = fixture.offer(&fixture.selection, &bytes, |_, expected| {
        SelectedInventory::from_expected(expected)
            .expect("B selections canonicalise")
            .result_domain_text(DigestSlot::Baseline)
            .expect("baseline is a selected slot")
    });
    let selected_digest = fixture.selection.baseline.digest.to_string();
    assert_ne!(result_domain, selected_digest);
    assert!(
        result_domain.starts_with("sha256-jcs:"),
        "the result domain must render under its own label, got {result_domain}"
    );
    assert!(
        selected_digest.starts_with("sha256:"),
        "the producer's exact-bytes domain renders bare, got {selected_digest}"
    );
    // The two domains no longer render at the same width — `sha256:` against
    // `sha256-jcs:` — so this substitution resizes the payload and cannot go
    // through `replace_unique`. The width difference is part of what is under
    // test, not an accident to be normalised away.
    let baseline = baseline_field(&fixture.selection);
    let changed = respell_within(&bytes, &baseline, &selected_digest, &result_domain);
    let selection = resealed(&fixture.selection, &changed);
    let error = fixture.discriminate(&selection, &changed).unwrap_err();
    assert_eq!(
        error,
        AdmissionRefusal::CrossDomainDigest(CrossDomainDigest {
            slot: DigestSlot::Baseline,
            selected: DigestDomain::RawArtifact,
            offered: OfferedDomain::Identified(DigestDomain::VerificationJcs),
            value: result_domain,
        }),
        "got {error:?}"
    );
}

/// Tracing: IT-001, FR-001-AC-4, Task-009.
#[test]
#[trace("IT-001", "FR-001-AC-4", "Task-009")]
fn it_001_round_trips_the_admitted_numeric_wire_profile() {
    let root = fixture_root();
    let fixture = Fixture::load(&root);
    let bytes = offered_bytes(&root);
    let admitted = fixture
        .admit(&bytes)
        .into_result()
        .expect("positive admission");
    assert_round_trips(&admitted, &bytes);
}

/// Tracing: IT-001, FR-001-AC-4, Task-009.
#[test]
#[trace("IT-001", "FR-001-AC-4", "Task-009")]
fn it_001_round_trips_signed_64_field_boundaries() {
    let root = fixture_root();
    let fixture = Fixture::load(&root);
    let bytes = offered_bytes(&root);
    // These literals occur in the reachable `Workflow.wide` path through
    // Healthy -> Bounded, not merely in an unused scalar declaration.
    for decimal in ["-9223372036854775808", "9223372036854775807"] {
        let operation = format!(
            r#""operation":{{"kind":"number","value":{{"kind":"integer","decimal":"{decimal}"}}}}"#
        );
        assert!(
            bytes
                .windows(operation.len())
                .any(|window| window == operation.as_bytes()),
            "fixture must carry {decimal} as a compiled numeric operation"
        );
    }
    let admitted = fixture
        .admit(&bytes)
        .into_result()
        .expect("endpoint admission");
    assert_round_trips(&admitted, &bytes);
}

/// Tracing: IT-001, FR-001-AC-4, Task-009.
#[test]
#[trace("IT-001", "FR-001-AC-4", "Task-009")]
fn it_001_round_trips_exact_rational_boundaries() {
    let root = fixture_root();
    let fixture = Fixture::load(&root);
    let bytes = offered_bytes(&root);
    // The independent model selection gives Exact its signed-64 numerator and
    // denominator domains; Bounded references both emitted endpoint pairs.
    for value in [
        r#"{"kind":"rational","numerator":"9223372036854775807","denominator":"9223372036854775806"}"#,
        r#"{"kind":"rational","numerator":"-9223372036854775808","denominator":"9223372036854775807"}"#,
    ] {
        let operation = format!(r#""operation":{{"kind":"number","value":{value}}}"#);
        assert!(
            bytes
                .windows(operation.len())
                .any(|window| window == operation.as_bytes()),
            "fixture must carry reduced exact boundary {value} as a compiled numeric operation"
        );
    }
    let admitted = fixture
        .admit(&bytes)
        .into_result()
        .expect("endpoint admission");
    assert_round_trips(&admitted, &bytes);
}

/// Tracing: IT-001, Task-009.
#[test]
#[trace("IT-001", "Task-009")]
fn it_001_refuses_one_axis_non_canonical_decimal() {
    let root = fixture_root();
    let fixture = Fixture::load(&root);
    let bytes = offered_bytes(&root);
    // Every spelling below denotes a value the field admits. Only the spelling
    // is wrong, so a reader that repaired text instead of refusing it would
    // admit each one.
    for spelling in ["05", "+5", "5.0", " 5", "-0"] {
        let changed = respell_within(
            &bytes,
            &integer_value(),
            r#""decimal":"5""#,
            &format!(r#""decimal":"{spelling}""#),
        );
        let selection = resealed(&fixture.selection, &changed);
        assert_eq!(
            numeric_error(fixture.admit_selected(&selection, &changed)),
            NumberError::NonCanonicalDecimal {
                component: NumberComponent::Decimal,
            },
            "spelling {spelling:?}"
        );
    }
}

/// Tracing: IT-001, Task-009.
#[test]
#[trace("IT-001", "Task-009")]
fn it_001_refuses_one_axis_unsafe_json_number() {
    let root = fixture_root();
    let fixture = Fixture::load(&root);
    let bytes = offered_bytes(&root);
    // A bare JSON number is the form that would silently acquire a float or
    // double interpretation on the way in. Each is refused at shape, before any
    // numeric component is admitted, so none reaches the exact-decimal codec:
    // a bare integer fails the required string shape, and every other bare form
    // fails the structural-integer rule that keeps JSON numbers out of the
    // exact numeric domain entirely.
    let changed = respell_within(
        &bytes,
        &integer_value(),
        r#""decimal":"5""#,
        r#""decimal":5"#,
    );
    let selection = resealed(&fixture.selection, &changed);
    let error = fixture
        .admit_selected(&selection, &changed)
        .into_result()
        .unwrap_err();
    assert!(
        matches!(error, quire_protocol::Error::Json { .. }),
        "bare 5: got {error:?}"
    );

    for bare in ["5.0", "1e3", "-0", "9223372036854775808"] {
        let changed = respell_within(
            &bytes,
            &integer_value(),
            r#""decimal":"5""#,
            &format!(r#""decimal":{bare}"#),
        );
        let selection = resealed(&fixture.selection, &changed);
        let error = fixture
            .admit_selected(&selection, &changed)
            .into_result()
            .unwrap_err();
        assert!(
            matches!(
                error,
                quire_protocol::Error::Invalid(Invalid::StructuralInteger)
            ),
            "bare {bare}: got {error:?}"
        );
    }
}

/// Tracing: IT-001, Task-009.
#[test]
#[trace("IT-001", "Task-009")]
fn it_001_refuses_one_axis_out_of_range_component() {
    let root = fixture_root();
    let fixture = Fixture::load(&root);
    let bytes = offered_bytes(&root);
    // One past each signed-64 endpoint: canonically spelled, outside the domain.
    for spelling in ["9223372036854775808", "-9223372036854775809"] {
        let changed = respell_within(
            &bytes,
            &integer_value(),
            r#""decimal":"5""#,
            &format!(r#""decimal":"{spelling}""#),
        );
        let selection = resealed(&fixture.selection, &changed);
        assert_eq!(
            numeric_error(fixture.admit_selected(&selection, &changed)),
            NumberError::ComponentOutOfRange {
                component: NumberComponent::Decimal,
            },
            "spelling {spelling:?}"
        );
    }
}

/// Tracing: IT-001, Task-009.
#[test]
#[trace("IT-001", "Task-009")]
fn it_001_labels_the_refused_rational_component() {
    let root = fixture_root();
    let fixture = Fixture::load(&root);
    let bytes = offered_bytes(&root);
    // The refusal names the component that was wrong, so a caller can act on it
    // without reading diagnostic text. A numerator failure is not a decimal one.
    let changed = respell_within(
        &bytes,
        &rational_value(),
        r#""numerator":"1""#,
        r#""numerator":"01""#,
    );
    let selection = resealed(&fixture.selection, &changed);
    assert_eq!(
        numeric_error(fixture.admit_selected(&selection, &changed)),
        NumberError::NonCanonicalDecimal {
            component: NumberComponent::Numerator,
        }
    );

    let changed = respell_within(
        &bytes,
        &rational_value(),
        r#""denominator":"1""#,
        r#""denominator":"9223372036854775808""#,
    );
    let selection = resealed(&fixture.selection, &changed);
    assert_eq!(
        numeric_error(fixture.admit_selected(&selection, &changed)),
        NumberError::ComponentOutOfRange {
            component: NumberComponent::Denominator,
        }
    );
}

/// Tracing: IT-001, Task-009.
#[test]
#[trace("IT-001", "Task-009")]
fn it_001_refuses_one_axis_non_positive_denominator() {
    let root = fixture_root();
    let fixture = Fixture::load(&root);
    let bytes = offered_bytes(&root);
    for denominator in ["0", "-1"] {
        let changed = respell_within(
            &bytes,
            &rational_value(),
            r#""denominator":"1""#,
            &format!(r#""denominator":"{denominator}""#),
        );
        let selection = resealed(&fixture.selection, &changed);
        assert_eq!(
            numeric_error(fixture.admit_selected(&selection, &changed)),
            NumberError::NonPositiveDenominator,
            "denominator {denominator:?}"
        );
    }
}

/// Tracing: IT-001, Task-009.
#[test]
#[trace("IT-001", "Task-009")]
fn it_001_refuses_one_axis_unreduced_rational() {
    let root = fixture_root();
    let fixture = Fixture::load(&root);
    let bytes = offered_bytes(&root);
    // 2/4 denotes the admitted 1/2 and 0/2 denotes the admitted 0/1. Both are
    // inside the field's domain, so only the reduction rule can refuse them,
    // and the reader must refuse rather than reduce.
    for (numerator, denominator) in [("2", "4"), ("0", "2")] {
        let changed = respell_within(
            &bytes,
            &rational_value(),
            r#""numerator":"1","denominator":"1""#,
            &format!(r#""numerator":"{numerator}","denominator":"{denominator}""#),
        );
        let selection = resealed(&fixture.selection, &changed);
        assert_eq!(
            numeric_error(fixture.admit_selected(&selection, &changed)),
            NumberError::UnreducedRational,
            "{numerator}/{denominator}"
        );
    }
}

/// Tracing: IT-001, Task-009.
#[test]
#[trace("IT-001", "Task-009")]
fn it_001_refuses_a_value_outside_its_declared_field_domain() {
    let root = fixture_root();
    let fixture = Fixture::load(&root);
    let bytes = offered_bytes(&root);
    // Well-formed under the numeric wire profile, and outside the domain the
    // declaration's own type gives the field. Numeric well-formedness alone
    // therefore never admits a value.
    let integer = respell_within(
        &bytes,
        &integer_value(),
        r#""decimal":"5""#,
        r#""decimal":"9223372036854775807""#,
    );
    let selection = resealed(&fixture.selection, &integer);
    let error = fixture
        .admit_selected(&selection, &integer)
        .into_result()
        .unwrap_err();
    assert!(
        matches!(
            error,
            quire_protocol::Error::Invalid(Invalid::NumericDomain)
        ),
        "integer: got {error:?}"
    );

    let rational = respell_within(
        &bytes,
        &rational_value(),
        r#""numerator":"1","denominator":"1""#,
        r#""numerator":"-9223372036854775808","denominator":"9223372036854775807""#,
    );
    let selection = resealed(&fixture.selection, &rational);
    let error = fixture
        .admit_selected(&selection, &rational)
        .into_result()
        .unwrap_err();
    assert!(
        matches!(
            error,
            quire_protocol::Error::Invalid(Invalid::NumericDomain)
        ),
        "rational: got {error:?}"
    );
}

/// Tracing: IT-001, Task-009.
#[test]
#[trace("IT-001", "Task-009")]
fn it_001_refuses_a_substituted_numeric_kind() {
    let root = fixture_root();
    let fixture = Fixture::load(&root);
    let bytes = offered_bytes(&root);
    // Integer 5 and rational 5/1 denote one quantity in two admitted forms.
    // In a value position the substitution leaves the field's declared integer
    // domain; in an integer-only bound position it is refused as a wrong kind.
    let value = respell_within(
        &bytes,
        &integer_value(),
        r#"{"kind":"integer","decimal":"5"}"#,
        r#"{"kind":"rational","numerator":"5","denominator":"1"}"#,
    );
    let selection = resealed(&fixture.selection, &value);
    let error = fixture
        .admit_selected(&selection, &value)
        .into_result()
        .unwrap_err();
    assert!(
        matches!(
            error,
            quire_protocol::Error::Invalid(Invalid::NumericDomain)
        ),
        "value: got {error:?}"
    );

    let bound = respell_within(
        &bytes,
        &integer_bound(),
        r#""maximum":{"kind":"integer","decimal":"5"}"#,
        r#""maximum":{"kind":"rational","numerator":"5","denominator":"1"}"#,
    );
    let selection = resealed(&fixture.selection, &bound);
    let error = fixture
        .admit_selected(&selection, &bound)
        .into_result()
        .unwrap_err();
    assert!(
        matches!(
            error,
            quire_protocol::Error::Invalid(Invalid::WrongNumericKind)
        ),
        "bound: got {error:?}"
    );
}

/// Tracing: IT-001, Task-009.
#[test]
#[trace("IT-001", "Task-009")]
fn it_001_refuses_a_type_bound_that_disagrees_with_the_selected_model() {
    let root = fixture_root();
    let fixture = Fixture::load(&root);
    let bytes = offered_bytes(&root);
    // Widening a declared representation bound keeps every value in the package
    // inside it, so only agreement with the independently selected model bytes
    // can refuse this. The offer cannot relax its own type domains.
    let changed = respell_within(
        &bytes,
        &integer_bound(),
        r#""maximum":{"kind":"integer","decimal":"5"}"#,
        r#""maximum":{"kind":"integer","decimal":"9223372036854775807"}"#,
    );
    let selection = resealed(&fixture.selection, &changed);
    let error = fixture
        .admit_selected(&selection, &changed)
        .into_result()
        .unwrap_err();
    assert!(
        matches!(error, quire_protocol::Error::Invalid(Invalid::Type)),
        "got {error:?}"
    );
}
