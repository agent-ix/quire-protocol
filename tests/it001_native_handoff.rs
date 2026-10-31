// SPDX-License-Identifier: AGPL-3.0-or-later
//! IT-001/Task-009: actual A-owned four-unit compiled-protocol handoff.

use std::{fs, path::Path};

use ix_trace_rs::trace;
use qsl_foundation::{Source, SourceIdentity};
use quire_contract_ir as ir;
use quire_protocol::{
    inventory::{
        axis::DivergedSelection,
        map::{admit_discriminating, AdmissionRefusal},
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
        handoff::{
            SelectedDependency, Selection, PUBLISHED_V1_OFFER_FILE, PUBLISHED_V1_SELECTION_FILE,
        },
        NumberWire,
    },
};

#[allow(dead_code, unused_imports)]
#[path = "support/v2_handoff/mod.rs"]
mod handoff;

use handoff::qsl_producer;

struct Fixture {
    selection: Selection,
    dependency_bytes: Vec<Vec<u8>>,
    model: quire_spec_language::native_model::NativeModel,
}

impl Fixture {
    fn load(root: &Path) -> Self {
        // Every member is listed in A's checksum manifest and hashes to it;
        // an unlisted, corrupt or symbolic-link member refuses.
        handoff::verify_checksums(root).expect("A's /1 handoff checksums");
        // A publishes the independent selection sidecar beside the offer as its
        // own member. It is read from the fresh output of A's producer (see
        // `qsl_producer`), before the offer, and never out of the offered
        // bytes, so the offered package cannot select its own acceptance
        // context. B keeps no copy of it, and checks every source, model and
        // dependency byte against the sidecar's digests below.
        let selection: Selection = serde_json::from_slice(
            &fs::read(root.join(PUBLISHED_V1_SELECTION_FILE)).expect("A selection sidecar"),
        )
        .expect("A selection sidecar shape");
        // Each selected source names a file in the handoff whose bytes hash to
        // the digest the selection carries. A non-empty-string check passed on
        // any fixture ever committed; this one fails whenever the selection and
        // the handoff are derived from different producer runs, which is how a
        // hybrid fixture went unnoticed across two A revisions.
        let qsl = qsl_producer::qsl_root().expect("pinned QSL source");
        for source in &selection.sources {
            let bytes = fs::read(root.join(&source.file))
                .unwrap_or_else(|error| panic!("selected source {}: {error}", source.file));
            assert_eq!(
                ByteDigest::of(&bytes),
                source.source.artifact.digest,
                "selected source {} does not hash to its selected digest",
                source.file
            );
            assert_authored_source(qsl, &source.source.path, &bytes);
        }
        let model_source = &selection.model.source;
        let model_bytes = fs::read(root.join(&selection.model.source_file)).expect("model bytes");
        assert_eq!(ByteDigest::of(&model_bytes), model_source.artifact.digest);
        // B's anchor, independent of the sidecar: the model source is the
        // authored file in the pinned QSL checkout, byte for byte.
        assert_eq!(
            model_bytes,
            fs::read(qsl.join(&model_source.path)).expect("authored model source"),
            "model source {} differs from the pinned QSL checkout",
            model_source.path
        );
        let source = Source::read(
            SourceIdentity {
                authority: model_source.native.authority.clone(),
                identity: model_source.native.identity.clone(),
                revision_namespace: model_source.native.revision_namespace.clone(),
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
            .map(|dependency| {
                let bytes = fs::read(root.join(&dependency.file)).expect("dependency bytes");
                assert_eq!(
                    ByteDigest::of(&bytes),
                    dependency.artifact.digest,
                    "selected dependency {} does not hash to its selected digest",
                    dependency.file
                );
                bytes
            })
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
    /// independently selected references before retaining the owner refusal.
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
                domain_packages: &[],
            },
        )
    }
}

/// B's anchor, independent of the sidecar: a selected native source is the
/// authored unit body in the pinned QSL checkout (`<unit>.body.native` beside the
/// selected path), preceded only by the import header A's producer prepends.
fn assert_authored_source(qsl: &Path, path: &str, bytes: &[u8]) {
    let stem = path
        .strip_suffix(".native")
        .unwrap_or_else(|| panic!("selected source {path} is not a .native unit"));
    let body = fs::read(qsl.join(format!("{stem}.body.native")))
        .unwrap_or_else(|error| panic!("authored body for {path}: {error}"));
    let header = bytes
        .strip_suffix(body.as_slice())
        .unwrap_or_else(|| panic!("selected source {path} does not end with its authored body"));
    let header = std::str::from_utf8(header).expect("UTF-8 import header");
    for line in header.lines() {
        assert!(
            ["language ", "profile ", "model "]
                .iter()
                .any(|prefix| line.starts_with(prefix)),
            "selected source {path} carries a non-import header line: {line:?}"
        );
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

/// A complete `/1` handoff freshly emitted by A's own producer from the exactly
/// pinned QSL source. The committed `PUBLISHED_V1_HANDOFF` copy omits
/// dependency bytes, so it is not admissible; see `qsl_producer`.
fn fixture_root() -> std::path::PathBuf {
    qsl_producer::v1()
        .unwrap_or_else(|cause| panic!("A's /1 producer did not run: {cause}"))
        .to_path_buf()
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

/// Find the dependency-list line inside a locked package paragraph that
/// references `name`. Cargo writes the bare form (`"name",`) while one version
/// of a crate resolves and a version-qualified form (`"name version",` or
/// `"name version (source)",`) once several do, so both forms are accepted:
/// the boundary under test is which crates are depended on, not how Cargo
/// happens to disambiguate them today.
fn locked_dependency<'a>(block: &'a str, name: &str) -> Option<&'a str> {
    let bare = format!(r#""{name}","#);
    let qualified_prefix = format!(r#""{name} "#);
    block
        .lines()
        .map(str::trim)
        .find(|line| *line == bare || line.starts_with(&qualified_prefix))
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
            "qsl-foundation",
            "quire-observation",
            "quire-verification-contracts",
            "engineering-assurance",
            "serde_json",
            "serde",
            "jsonschema",
            "sha2"
        ]
    );

    let consumer = locked_package("quire-protocol");
    // Match bare and version-qualified lockfile references alike: a hardcoded
    // `"sha2 0.10.9"` broke when the graph dropped to one sha2 version, and a
    // bare `"logos"` negative check would pass vacuously if a second logos
    // version made Cargo qualify the reference.
    for required in [
        "quire-spec-language",
        "qsl-foundation",
        "quire-observation",
        "quire-verification-contracts",
        "engineering-assurance",
        "serde_json",
        "sha2",
    ] {
        assert!(
            locked_dependency(consumer, required).is_some(),
            "`{required}` is not a locked dependency of quire-protocol in:\n{consumer}"
        );
    }
    assert_eq!(locked_dependency(consumer, "logos"), None);
    // One `sha2` is locked, so the lockfile names it without a version suffix;
    // the version is asserted on the locked package itself.
    assert!(locked_package("sha2").contains(r#"version = "0.10.9""#));
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
    // The selected baseline digest differs, so this is the digest axis.
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
