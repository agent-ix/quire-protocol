// SPDX-License-Identifier: AGPL-3.0-or-later
//! IT-001 SC-06..SC-09: B's reader for A's `quire.compiled-protocol/2` handoff.
//!
//! A publishes the handoff as a directory: the offered `/2` bytes, their
//! external artifact reference, the original unrecanonicalized producer inputs,
//! an independent selection sidecar, and a machine-readable mutation corpus.
//!
//! Every `Expected` value this module builds comes from the sidecar and the
//! original input files. Nothing is read out of the offered `/2` bytes: the
//! offer is carried to `admit_and_link_v2` untouched and has no authority to
//! select or repair any expectation.
//!
//! The record shapes below are the *envelope* of A's sidecar and mutation
//! manifest, which A publishes as JSON but not as a Rust type. Every field
//! inside them is an A-owned published type (`protocol_artifact::wire::*`,
//! `protocol_artifact::v2::wire::*`); this module declares no protocol
//! vocabulary of its own. See `missing_a_api()` for what B owes A here.

use std::{
    env, fmt, fs,
    path::{Path, PathBuf},
};

use quire_contract_ir as ir;
use quire_spec_language::{
    formal_source::FormalSource,
    model_source::{self, ModelSourceLimits},
    native_model::{ModelLimits, NativeModel},
    protocol_artifact::{self as artifact, v2, wire as w},
    ByteDigest, Source, SourceIdentity,
};
use serde::Deserialize;

/// Directory holding A's `/2` handoff, as published by its release producer.
pub const FIXTURE_ENV: &str = "QUIRE_PROTOCOL_A_V2_HANDOFF";

/// Files A's handoff publishes at a fixed name.
pub const OFFER_FILE: &str = "compiled-protocol-v2.json";
pub const REFERENCE_FILE: &str = "compiled-protocol-v2.ref.json";
pub const SELECTION_FILE: &str = "expected-v2.json";
pub const MUTATION_MANIFEST_FILE: &str = "mutations/manifest.json";

/// The mutation manifest envelope version this reader accepts.
pub const MUTATION_FORMAT: &str = "quire.protocol.v2-mutations/1";

// ---------------------------------------------------------------------------
// Absence
// ---------------------------------------------------------------------------

/// Why the `/2` lane could not be assembled. Every variant is reported; none is
/// treated as a pass.
#[derive(Debug)]
pub enum Absence {
    /// `QUIRE_PROTOCOL_A_V2_HANDOFF` is unset.
    EnvUnset,
    /// The named directory does not exist.
    MissingRoot(PathBuf),
    /// A published member is absent.
    MissingMember { role: String, path: PathBuf },
    /// A published member does not deserialise into its envelope shape.
    Unreadable { path: PathBuf, cause: String },
    /// The mutation manifest declares an envelope version B does not read.
    UnknownMutationFormat { format: String },
    /// The mutation manifest's base members name files other than the ones B
    /// loaded, so its cases are stated against a different handoff.
    ManifestBase { member: String, named: String },
    /// The selection names an accounting contract whose counters B does not
    /// interpret, so its ceilings cannot be carried over.
    AccountingVersion { named: String },
    /// Two temporal selections name one clock identity.
    RepeatedClockIdentity { identity: String },
    /// A retained original input does not hash to the identity the sidecar
    /// independently selected for it. The handoff is internally inconsistent
    /// and no admission outcome may be reported from it.
    InputDigest {
        role: String,
        path: PathBuf,
        selected: String,
        actual: String,
    },
    /// The model source could not be re-admitted through A's public reader, so
    /// B cannot construct the `AdmittedModel` the expectation requires.
    Model { cause: String },
}

impl fmt::Display for Absence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EnvUnset => write!(f, "{FIXTURE_ENV} is unset"),
            Self::MissingRoot(path) => write!(f, "handoff root {} does not exist", path.display()),
            Self::MissingMember { role, path } => {
                write!(f, "published {role} member {} is absent", path.display())
            }
            Self::Unreadable { path, cause } => {
                write!(f, "{} is unreadable: {cause}", path.display())
            }
            Self::UnknownMutationFormat { format } => {
                write!(
                    f,
                    "mutation manifest declares format {format}, not {MUTATION_FORMAT}"
                )
            }
            Self::ManifestBase { member, named } => write!(
                f,
                "mutation manifest's {member} names {named}, which is not the \
                 member B loaded"
            ),
            Self::AccountingVersion { named } => write!(
                f,
                "the independent selection names accounting contract {named}, \
                 not {}",
                artifact::ACCOUNTING_VERSION
            ),
            Self::InputDigest {
                role,
                path,
                selected,
                actual,
            } => write!(
                f,
                "retained {role} input {} hashes to {actual}, but the independent \
                 selection names {selected}",
                path.display()
            ),
            Self::RepeatedClockIdentity { identity } => {
                write!(f, "two temporal selections name clock identity {identity}")
            }
            Self::Model { cause } => {
                write!(f, "the model source could not be re-admitted: {cause}")
            }
        }
    }
}

// ---------------------------------------------------------------------------
// A's sidecar envelope
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectedDeclaration {
    pub name: String,
    pub span: w::Span,
    pub requirement: w::Requirement,
    pub clause: String,
    pub execution: w::Execution,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectedSource {
    pub file: String,
    pub source: w::Source,
    pub declarations: Vec<SelectedDeclaration>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectedDependency {
    pub artifact: w::ArtifactRef,
    pub file: String,
    pub requires: Vec<w::ArtifactRef>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectedModel {
    pub artifact: w::ArtifactRef,
    pub source: w::Source,
    pub source_file: String,
    pub source_format: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Selection {
    pub artifact: w::ArtifactRef,
    pub contract: w::ArtifactRef,
    pub baseline: w::ArtifactRef,
    pub producer: w::Producer,
    pub language: w::Language,
    pub sources: Vec<SelectedSource>,
    pub dependencies: Vec<SelectedDependency>,
    pub model: SelectedModel,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectedClockInput {
    pub identity: String,
    pub digest: String,
    pub file: String,
    pub configuration: v2::wire::ClockConfiguration,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectedTemporal {
    pub source: w::ArtifactRef,
    pub declaration: SelectedDeclaration,
    pub definition_identity: String,
    pub definition_revision: w::Revision,
    pub definition_artifact: w::ArtifactRef,
    pub clock_input: SelectedClockInput,
}

/// A's selected artifact ceilings. Deserialised into A's own `Limits`; the
/// `accounting_version` is carried so a changed accounting contract is visible
/// rather than silently reinterpreted.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectedLimits {
    pub accounting_version: String,
    pub payload_bytes: usize,
    pub output_bytes: usize,
    pub source_bytes: usize,
    pub content_bytes: usize,
    pub sources: usize,
    pub dependencies: usize,
    pub definitions: usize,
    pub models: usize,
    pub declarations: usize,
    pub entries: usize,
    pub references: usize,
    pub byte_work: usize,
    pub depth: usize,
}

impl SelectedLimits {
    /// A's accepted ceiling record, with no member left to a local default.
    pub fn limits(&self) -> artifact::Limits {
        artifact::Limits {
            payload_bytes: self.payload_bytes,
            output_bytes: self.output_bytes,
            source_bytes: self.source_bytes,
            content_bytes: self.content_bytes,
            sources: self.sources,
            dependencies: self.dependencies,
            definitions: self.definitions,
            models: self.models,
            declarations: self.declarations,
            entries: self.entries,
            references: self.references,
            byte_work: self.byte_work,
            depth: self.depth,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectionV2 {
    pub inherited: Selection,
    pub temporal: Vec<SelectedTemporal>,
    pub limits: SelectedLimits,
}

// ---------------------------------------------------------------------------
// A's mutation corpus envelope
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub enum MutationInput {
    /// Submit these bytes in place of the base offer, under the named
    /// independently selected external artifact reference.
    Offer {
        file: String,
        artifact: w::ArtifactRef,
    },
    /// Replace one retained original input's bytes, then submit the unchanged
    /// base offer.
    ReplaceOriginal { target: String, file: String },
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MutationCase {
    pub identity: String,
    pub axis: String,
    pub input: MutationInput,
    pub expected_refusal_code: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MutationManifest {
    pub format: String,
    pub base_offer: String,
    pub base_artifact: String,
    pub independent_selection: String,
    pub cases: Vec<MutationCase>,
}

// ---------------------------------------------------------------------------
// The loaded handoff
// ---------------------------------------------------------------------------

/// A's handoff, with every original input retained as its published bytes.
#[derive(Debug)]
pub struct Handoff {
    root: PathBuf,
    offer: Vec<u8>,
    reference: w::ArtifactRef,
    selection: SelectionV2,
    mutations: MutationManifest,
    /// Original source text per `selection.inherited.sources`, in that order.
    source_text: Vec<String>,
    /// Original dependency bytes per `selection.inherited.dependencies`.
    dependency_bytes: Vec<Vec<u8>>,
    /// The model source's original bytes and its re-admitted native model.
    model_source_bytes: Vec<u8>,
    model: NativeModel,
}

impl Handoff {
    /// Load the handoff named by `FIXTURE_ENV`.
    pub fn load() -> Result<Self, Absence> {
        let root = env::var_os(FIXTURE_ENV).ok_or(Absence::EnvUnset)?;
        Self::load_from(Path::new(&root))
    }

    /// Load the handoff published under `root`.
    pub fn load_from(root: &Path) -> Result<Self, Absence> {
        if !root.is_dir() {
            return Err(Absence::MissingRoot(root.to_path_buf()));
        }
        let offer = read_member(root, OFFER_FILE, "offer")?;
        let reference: w::ArtifactRef = read_json(root, REFERENCE_FILE, "artifact reference")?;
        let selection: SelectionV2 = read_json(root, SELECTION_FILE, "independent selection")?;
        let mutations: MutationManifest =
            read_json(root, MUTATION_MANIFEST_FILE, "mutation manifest")?;
        if mutations.format != MUTATION_FORMAT {
            return Err(Absence::UnknownMutationFormat {
                format: mutations.format,
            });
        }
        // The manifest states its cases against named base members. If those are
        // not the members B loaded, the declared refusal identities belong to a
        // different handoff and cannot be asserted against this one.
        for (member, named, loaded) in [
            ("base_offer", &mutations.base_offer, OFFER_FILE),
            ("base_artifact", &mutations.base_artifact, REFERENCE_FILE),
            (
                "independent_selection",
                &mutations.independent_selection,
                SELECTION_FILE,
            ),
        ] {
            if named != loaded {
                return Err(Absence::ManifestBase {
                    member: member.to_owned(),
                    named: named.clone(),
                });
            }
        }
        // The ceilings below are counted under a named accounting contract. A
        // different contract reinterprets every counter, so it is refused rather
        // than carried across.
        if selection.limits.accounting_version != artifact::ACCOUNTING_VERSION {
            return Err(Absence::AccountingVersion {
                named: selection.limits.accounting_version.clone(),
            });
        }

        // The sidecar is an independent selection, not an oracle: every identity
        // it names is checked against the original bytes A also published. A
        // disagreement is the handoff's defect, reported before any admission.
        let mut source_text = Vec::new();
        for selected in &selection.inherited.sources {
            let bytes = read_member(root, &selected.file, "source")?;
            check_digest(
                root,
                &selected.file,
                "source",
                &bytes,
                selected.source.artifact.digest,
            )?;
            source_text.push(
                String::from_utf8(bytes).map_err(|error| Absence::Unreadable {
                    path: root.join(&selected.file),
                    cause: error.to_string(),
                })?,
            );
        }

        let mut dependency_bytes = Vec::new();
        for selected in &selection.inherited.dependencies {
            let bytes = read_member(root, &selected.file, "dependency")?;
            check_digest(
                root,
                &selected.file,
                "dependency",
                &bytes,
                selected.artifact.digest,
            )?;
            dependency_bytes.push(bytes);
        }

        let model_file = selection.inherited.model.source_file.clone();
        let model_source_bytes = read_member(root, &model_file, "model source")?;
        check_digest(
            root,
            &model_file,
            "model source",
            &model_source_bytes,
            selection.inherited.model.source.artifact.digest,
        )?;

        let mut clock_identities: Vec<&str> = Vec::new();
        for selected in &selection.temporal {
            let bytes = read_member(root, &selected.clock_input.file, "clock input")?;
            let actual = ByteDigest::of(&bytes).to_string();
            if actual != selected.clock_input.digest {
                return Err(Absence::InputDigest {
                    role: "clock input".to_owned(),
                    path: root.join(&selected.clock_input.file),
                    selected: selected.clock_input.digest.clone(),
                    actual,
                });
            }
            // Two temporal rows selecting one clock identity would let one row's
            // clock outcome stand in for another's.
            let identity = selected.clock_input.identity.as_str();
            if clock_identities.contains(&identity) {
                return Err(Absence::RepeatedClockIdentity {
                    identity: identity.to_owned(),
                });
            }
            clock_identities.push(identity);
        }

        let model = admit_model(&selection.inherited.model, &model_source_bytes, &selection)?;

        Ok(Self {
            root: root.to_path_buf(),
            offer,
            reference,
            selection,
            mutations,
            source_text,
            dependency_bytes,
            model_source_bytes,
            model,
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The offered `/2` bytes, exactly as published.
    pub fn offer(&self) -> &[u8] {
        &self.offer
    }

    /// A's external reference for the base offer.
    pub fn reference(&self) -> &w::ArtifactRef {
        &self.reference
    }

    pub fn selection(&self) -> &SelectionV2 {
        &self.selection
    }

    pub fn mutations(&self) -> &MutationManifest {
        &self.mutations
    }

    /// A's accepted ceilings for this handoff.
    pub fn limits(&self) -> artifact::Limits {
        self.selection.limits.limits()
    }

    /// Read a mutation fixture's bytes.
    pub fn mutation_bytes(&self, file: &str) -> Result<Vec<u8>, Absence> {
        read_member(&self.root, file, "mutation")
    }

    /// Stage one: declaration tables and the foreign model source, borrowed
    /// from the retained originals.
    pub fn declarations(&self) -> Declarations<'_> {
        let sources = self
            .selection
            .inherited
            .sources
            .iter()
            .map(|selected| {
                selected
                    .declarations
                    .iter()
                    .map(expected_declaration)
                    .collect()
            })
            .collect();
        let temporal = self
            .selection
            .temporal
            .iter()
            .map(|selected| expected_declaration(&selected.declaration))
            .collect();
        Declarations {
            sources,
            temporal,
            foreign: artifact::ExpectedForeignSource {
                artifact: &self.selection.inherited.model.source.artifact,
                formal: &self.selection.inherited.model.source.formal,
                bytes: &self.model_source_bytes,
            },
        }
    }

    /// Stage two: the inventories `Expected` borrows.
    pub fn inventories<'a>(&'a self, declarations: &'a Declarations<'a>) -> Inventories<'a> {
        let sources = self
            .selection
            .inherited
            .sources
            .iter()
            .zip(&self.source_text)
            .zip(&declarations.sources)
            .map(|((selected, text), declared)| artifact::ExpectedSource {
                artifact: &selected.source.artifact,
                native: &selected.source.native,
                path: &selected.source.path,
                formal: &selected.source.formal,
                text,
                declarations: declared,
            })
            .collect();
        let dependencies = self
            .selection
            .inherited
            .dependencies
            .iter()
            .zip(&self.dependency_bytes)
            .map(|(selected, bytes)| artifact::SuppliedDependency {
                artifact: &selected.artifact,
                bytes,
                requires: &selected.requires,
            })
            .collect();
        let models = vec![artifact::AdmittedModel {
            artifact: &self.selection.inherited.model.artifact,
            model: &self.model,
            source: &declarations.foreign,
        }];
        let temporal = self
            .selection
            .temporal
            .iter()
            .zip(&declarations.temporal)
            .map(|(selected, declared)| v2::ExpectedTemporal {
                source: &selected.source,
                declaration: declared,
                definition: v2::ExpectedDefinition {
                    identity: &selected.definition_identity,
                    revision: &selected.definition_revision,
                    artifact: &selected.definition_artifact,
                },
                clock: &selected.clock_input.configuration,
            })
            .collect();
        Inventories {
            sources,
            dependencies,
            models,
            temporal,
        }
    }

    /// Stage three: the complete independent expectation for the base offer.
    pub fn expected<'a>(&'a self, inventories: &'a Inventories<'a>) -> v2::Expected<'a> {
        self.expected_for(inventories, &self.selection.inherited.artifact)
    }

    /// The same expectation with one substituted external artifact reference,
    /// as a mutation case's independently selected seal requires.
    pub fn expected_for<'a>(
        &'a self,
        inventories: &'a Inventories<'a>,
        seal: &'a w::ArtifactRef,
    ) -> v2::Expected<'a> {
        v2::Expected {
            inherited: artifact::Expected {
                artifact: seal,
                contract: &self.selection.inherited.contract,
                baseline: &self.selection.inherited.baseline,
                producer: &self.selection.inherited.producer,
                language: &self.selection.inherited.language,
                sources: &inventories.sources,
                dependencies: &inventories.dependencies,
                models: &inventories.models,
            },
            temporal: &inventories.temporal,
        }
    }

    /// The same inventories with one retained original replaced, for a
    /// `replace_original` mutation. Returns `None` when the target names no
    /// published dependency.
    pub fn replaced_dependency_bytes(&self, target: &str, bytes: Vec<u8>) -> Option<Vec<Vec<u8>>> {
        let index = self
            .selection
            .inherited
            .dependencies
            .iter()
            .position(|selected| selected.file == target)?;
        let mut replaced = self.dependency_bytes.clone();
        replaced[index] = bytes;
        Some(replaced)
    }

    /// Dependency inventory over caller-supplied bytes, used by
    /// `replace_original` cases.
    pub fn dependencies_over<'a>(
        &'a self,
        bytes: &'a [Vec<u8>],
    ) -> Vec<artifact::SuppliedDependency<'a>> {
        self.selection
            .inherited
            .dependencies
            .iter()
            .zip(bytes)
            .map(|(selected, bytes)| artifact::SuppliedDependency {
                artifact: &selected.artifact,
                bytes,
                requires: &selected.requires,
            })
            .collect()
    }
}

/// Stage-one storage: nothing here borrows from stage two.
#[derive(Debug)]
pub struct Declarations<'a> {
    pub sources: Vec<Vec<artifact::ExpectedDeclaration<'a>>>,
    pub temporal: Vec<artifact::ExpectedDeclaration<'a>>,
    pub foreign: artifact::ExpectedForeignSource<'a>,
}

/// Stage-two storage: the four inventories `v2::Expected` borrows.
#[derive(Debug)]
pub struct Inventories<'a> {
    pub sources: Vec<artifact::ExpectedSource<'a>>,
    pub dependencies: Vec<artifact::SuppliedDependency<'a>>,
    pub models: Vec<artifact::AdmittedModel<'a>>,
    pub temporal: Vec<v2::ExpectedTemporal<'a>>,
}

// ---------------------------------------------------------------------------
// Refusal identity
// ---------------------------------------------------------------------------

/// Whether `error` is the refusal A's manifest declares as `code`.
///
/// `v2::Refusal` publishes its own stable `code()`. The two codes A's corpus
/// declares outside that enum — a closed-shape JSON refusal and a seal refusal —
/// have no published accessor on `artifact::Error`, so they are matched against
/// A's typed variants here rather than against a rendered message. This is a
/// reading of A's declared identities, not a B-owned code vocabulary; see
/// `missing_a_api()`.
pub fn matches_declared_code(error: &artifact::Error, code: &str) -> bool {
    match code {
        "json" => matches!(error, artifact::Error::Json { .. }),
        "invalid.seal" => matches!(error, artifact::Error::Invalid(artifact::Invalid::Seal)),
        _ => matches!(error, artifact::Error::V2(refusal) if refusal.code() == code),
    }
}

/// The refusal identity B can report for `error`, for a failure message.
pub fn reported_code(error: &artifact::Error) -> String {
    match error {
        artifact::Error::V2(refusal) => refusal.code().to_owned(),
        artifact::Error::Json { .. } => "json".to_owned(),
        artifact::Error::Invalid(artifact::Invalid::Seal) => "invalid.seal".to_owned(),
        other => format!("{other:?}"),
    }
}

/// What B needs from A that the handoff does not publish. Reported to A rather
/// than filled in locally.
pub const fn missing_a_api() -> [&'static str; 2] {
    [
        "expected-v2.json and mutations/manifest.json have no published Rust \
         type: their envelope is declared only inside A's example, so every \
         consumer re-declares it. B asks A to publish the two envelope records \
         from the library.",
        "protocol_artifact::Error publishes no stable code accessor, so the \
         `json` and `invalid.seal` identities A's corpus declares can only be \
         matched by variant. B asks A for `Error::code()` alongside \
         `v2::Refusal::code()`.",
    ]
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn expected_declaration(selected: &SelectedDeclaration) -> artifact::ExpectedDeclaration<'_> {
    artifact::ExpectedDeclaration {
        name: &selected.name,
        span: &selected.span,
        requirement: &selected.requirement,
        clause: &selected.clause,
        execution: &selected.execution,
    }
}

fn read_member(root: &Path, relative: &str, role: &str) -> Result<Vec<u8>, Absence> {
    let path = root.join(relative);
    fs::read(&path).map_err(|_| Absence::MissingMember {
        role: role.to_owned(),
        path,
    })
}

fn read_json<T: serde::de::DeserializeOwned>(
    root: &Path,
    relative: &str,
    role: &str,
) -> Result<T, Absence> {
    let bytes = read_member(root, relative, role)?;
    serde_json::from_slice(&bytes).map_err(|error| Absence::Unreadable {
        path: root.join(relative),
        cause: error.to_string(),
    })
}

fn check_digest(
    root: &Path,
    relative: &str,
    role: &str,
    bytes: &[u8],
    selected: ByteDigest,
) -> Result<(), Absence> {
    let actual = ByteDigest::of(bytes);
    if actual == selected {
        return Ok(());
    }
    Err(Absence::InputDigest {
        role: role.to_owned(),
        path: root.join(relative),
        selected: selected.to_string(),
        actual: actual.to_string(),
    })
}

/// Re-admit the model through A's public reader. B holds no model of its own:
/// the `AdmittedModel` an expectation needs is constructor-private to A, so the
/// only way to supply it is to execute A's admission over the original bytes.
fn admit_model(
    selected: &SelectedModel,
    bytes: &[u8],
    selection: &SelectionV2,
) -> Result<NativeModel, Absence> {
    let source = Source::read(
        SourceIdentity {
            identity: selected.source.native.identity.clone(),
            revision: selected.source.native.revision.clone(),
        },
        selected.source.path.clone(),
        bytes,
        selection.limits.source_bytes,
    )
    .map_err(|error| Absence::Model {
        cause: error.to_string(),
    })?;
    let document =
        ir::SourceDocumentId::new(selected.source.formal.document.clone()).map_err(|error| {
            Absence::Model {
                cause: error.to_string(),
            }
        })?;
    let revision = selected
        .source
        .formal
        .revision
        .value
        .parse::<u64>()
        .map_err(|error| Absence::Model {
            cause: format!(
                "formal revision {} is not a positive integer: {error}",
                selected.source.formal.revision.value
            ),
        })?;
    let revision = ir::SourceRevision::new(revision).map_err(|error| Absence::Model {
        cause: error.to_string(),
    })?;
    let formal = FormalSource::new(source, ir::SourceIdentity::new(document, revision));
    model_source::read(
        formal,
        &selected.source_format,
        ModelSourceLimits::default(),
    )
    .map_err(|error| Absence::Model {
        cause: error.to_string(),
    })?
    .admit(ModelLimits::default())
    .map_err(|error| Absence::Model {
        cause: error.to_string(),
    })
}
