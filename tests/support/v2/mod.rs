// SPDX-License-Identifier: AGPL-3.0-or-later
//! IT-001 SC-06..SC-09: the B-owned `/2` fixture loader and mutation corpus.
//!
//! This module is authored ahead of A's `protocol_artifact::v2` delivery. It
//! deliberately names no A type: it indexes B's own retained evidence and
//! enumerates the scenarios the `/2` lane must run. It parses no compiled
//! artifact — `b-v2-delivery-index.json` is B's evidence index, written by B at pin
//! time, and the offered bytes are carried through untouched.

use std::{
    env, fmt, fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;

/// Directory holding A's `/2` delivery together with B's evidence index.
pub const FIXTURE_ENV: &str = "QUIRE_PROTOCOL_A_V2_FIXTURE";

/// B's evidence index inside that directory. Not emitted by A.
pub const DELIVERY_INDEX_FILE: &str = "b-v2-delivery-index.json";

/// What `quire-spec-language#40` still owes before this lane can execute.
///
/// A's `/2` is implemented and under final integration; its public surface is
/// settled and recorded in IT-001 Preconditions, so it is not listed here. What
/// remains is the final revision and its fixtures. The lane refuses to report a
/// `/2` outcome until every one of these is in hand, and none may be
/// substituted by prose, a native source file, an emitted sidecar, a
/// hand-authored byte string, or a `/1` value.
///
/// B pins no intermediate commit: only A's one final immutable revision.
pub const REQUIRED_A_DELIVERABLES: [&str; 5] = [
    "A's one final immutable quire-spec-language revision to pin",
    "the raw /2 artifact bytes and their external artifact reference",
    "the derivation recipe B implements to construct v2::Expected, \
     v2::ExpectedDefinition and v2::ExpectedTemporal from the original inputs \
     -- a recipe B executes, never a sidecar B reads",
    "the original, unrecanonicalized producer input corpus",
    "the per-axis mutation-fixture paths and their documented refusal identities",
];

// ---------------------------------------------------------------------------
// Absence
// ---------------------------------------------------------------------------

/// Why the `/2` lane could not be assembled. Every variant is reported; none is
/// treated as a pass.
#[derive(Debug)]
pub enum Absence {
    /// `QUIRE_PROTOCOL_A_V2_FIXTURE` is unset.
    EnvUnset,
    /// The named directory does not exist.
    MissingRoot(PathBuf),
    /// B has not written its evidence index for this delivery yet.
    MissingDeliveryIndex(PathBuf),
    /// The index names a member that is not present.
    MissingMember { role: String, path: PathBuf },
    /// The index does not deserialise into B's own record shape.
    UnreadableDeliveryIndex { path: PathBuf, cause: String },
    /// One scenario has more than one registered mutation fixture.
    AmbiguousMutation { scenario: String },
    /// Two bindings name one declaration, which A's rule forbids: bindings
    /// "cannot repeat".
    RepeatedBindingDeclaration { declaration: u32 },
    /// The indexed bindings are not sorted by `declaration`, which A's rule
    /// requires.
    UnsortedBindings { at: u32, after: u32 },
}

impl fmt::Display for Absence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EnvUnset => write!(f, "{FIXTURE_ENV} is unset"),
            Self::MissingRoot(path) => write!(f, "fixture root {} does not exist", path.display()),
            Self::MissingDeliveryIndex(path) => {
                write!(f, "B evidence index {} is absent", path.display())
            }
            Self::MissingMember { role, path } => {
                write!(f, "indexed {role} member {} is absent", path.display())
            }
            Self::UnreadableDeliveryIndex { path, cause } => {
                write!(
                    f,
                    "B evidence index {} is unreadable: {cause}",
                    path.display()
                )
            }
            Self::AmbiguousMutation { scenario } => {
                write!(f, "scenario {scenario} has more than one mutation fixture")
            }
            Self::RepeatedBindingDeclaration { declaration } => {
                write!(f, "declaration {declaration} is bound more than once")
            }
            Self::UnsortedBindings { at, after } => {
                write!(f, "declaration {at} is indexed after {after}")
            }
        }
    }
}

// ---------------------------------------------------------------------------
// B's evidence index
// ---------------------------------------------------------------------------

/// The roles B retains independently of the offer. Each is an original producer
/// input; B copies its bytes and never recanonicalises them.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SelectionRole {
    Artifact,
    Contract,
    Baseline,
    Producer,
    Source,
    Model,
    Dependency,
    Definition,
    Configuration,
    Result,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndexedInput {
    pub role: SelectionRole,
    pub path: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndexedOffer {
    /// Path to A's exact emitted `/2` bytes. B never rewrites this file.
    pub path: String,
    /// A's external artifact reference for those bytes, copied verbatim.
    pub reference: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndexedTemporal {
    pub profile: String,
    pub clock_alternative: String,
    pub clock_parameters: Vec<String>,
    pub bindings: Vec<IndexedBinding>,
}

/// One temporal binding, addressed by A's `declaration` field.
///
/// A's `/2` wire contract states the whole rule in one sentence: "There is
/// exactly one binding for every temporal declaration and none for other
/// declaration families. Bindings sort by `declaration` and cannot repeat."
/// That is cardinality, uniqueness and order, all keyed on `declaration`.
///
/// B does not restate it as a "declared key" or a "declared total order": A uses
/// neither phrase, and the inflation would attribute to A a declaration it did
/// not make. `quire-specification` settles none of it — FR-252 gives a clock
/// binding a *value* identity whose key carries no declaration component, so two
/// bindings on two different declarations compare equal upstream.
///
/// The checks below enforce A's sentence over B's own delivery index. They are
/// B checking its retained evidence against A's published rule, not B deriving
/// a rule of its own.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndexedBinding {
    pub owner: String,
    pub declaration: u32,
    pub path: String,
}

/// One A-declared `/2` resource limit and its exact bound.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndexedLimit {
    pub name: String,
    pub exact: u64,
}

/// One A-supplied mutation fixture and the axis it moves.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndexedMutation {
    pub id: String,
    pub scenario: String,
    pub path: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeliveryIndex {
    /// The immutable A revision these bytes were emitted at.
    pub producer_revision: String,
    pub offer: IndexedOffer,
    pub inputs: Vec<IndexedInput>,
    pub temporal: IndexedTemporal,
    pub limits: Vec<IndexedLimit>,
    pub mutations: Vec<IndexedMutation>,
}

// ---------------------------------------------------------------------------
// Loader
// ---------------------------------------------------------------------------

/// A's exact offered bytes plus the reference they arrived under.
pub struct OfferedArtifact {
    bytes: Vec<u8>,
    reference: String,
}

impl OfferedArtifact {
    /// The exact bytes A emitted, unmodified.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// A's external artifact reference, copied verbatim.
    pub fn reference(&self) -> &str {
        &self.reference
    }
}

/// B's assembled view of one `/2` delivery.
#[derive(Debug)]
pub struct Fixture {
    root: PathBuf,
    delivery_index: DeliveryIndex,
    offer: OfferedArtifact,
}

impl Fixture {
    /// Assemble the lane from the environment, or report exactly what is absent.
    pub fn load() -> Result<Self, Absence> {
        let root = match env::var_os(FIXTURE_ENV) {
            Some(value) => PathBuf::from(value),
            None => return Err(Absence::EnvUnset),
        };
        Self::load_from(&root)
    }

    /// Assemble the lane from an explicit root.
    pub fn load_from(root: &Path) -> Result<Self, Absence> {
        if !root.is_dir() {
            return Err(Absence::MissingRoot(root.to_path_buf()));
        }
        let index_path = root.join(DELIVERY_INDEX_FILE);
        let manifest_bytes = match fs::read(&index_path) {
            Ok(bytes) => bytes,
            Err(_) => return Err(Absence::MissingDeliveryIndex(index_path)),
        };
        let delivery_index: DeliveryIndex =
            serde_json::from_slice(&manifest_bytes).map_err(|error| {
                Absence::UnreadableDeliveryIndex {
                    path: index_path.clone(),
                    cause: error.to_string(),
                }
            })?;

        for input in &delivery_index.inputs {
            require(root, &format!("{:?}", input.role), &input.path)?;
        }
        require(root, "temporal-profile", &delivery_index.temporal.profile)?;
        // A's rule: bindings sort by `declaration` and cannot repeat. Without
        // it holding over B's retained index, "duplicate" and "reordered" name
        // nothing.
        let mut previous: Option<u32> = None;
        for binding in &delivery_index.temporal.bindings {
            require(root, "temporal-binding", &binding.path)?;
            if let Some(after) = previous {
                if binding.declaration == after {
                    return Err(Absence::RepeatedBindingDeclaration {
                        declaration: binding.declaration,
                    });
                }
                if binding.declaration < after {
                    return Err(Absence::UnsortedBindings {
                        at: binding.declaration,
                        after,
                    });
                }
            }
            previous = Some(binding.declaration);
        }
        for mutation in &delivery_index.mutations {
            require(root, &mutation.scenario, &mutation.path)?;
        }

        let offer_path = root.join(&delivery_index.offer.path);
        let bytes = fs::read(&offer_path).map_err(|_| Absence::MissingMember {
            role: "offered-artifact".to_owned(),
            path: offer_path,
        })?;
        let offer = OfferedArtifact {
            bytes,
            reference: delivery_index.offer.reference.clone(),
        };

        Ok(Self {
            root: root.to_path_buf(),
            delivery_index,
            offer,
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn delivery_index(&self) -> &DeliveryIndex {
        &self.delivery_index
    }

    /// The exact `/2` bytes, for submission and for the byte-preservation check.
    pub fn offer(&self) -> &OfferedArtifact {
        &self.offer
    }

    /// Read one indexed producer input byte for byte. No canonicalisation.
    ///
    /// An unindexed role and an unreadable file are distinct outcomes: the
    /// second is reported, never collapsed into the first.
    pub fn input_bytes(&self, role: SelectionRole) -> Result<Option<Vec<u8>>, Absence> {
        let Some(input) = self
            .delivery_index
            .inputs
            .iter()
            .find(|input| input.role == role)
        else {
            return Ok(None);
        };
        let path = self.root.join(&input.path);
        fs::read(&path)
            .map(Some)
            .map_err(|_| Absence::MissingMember {
                role: format!("{:?}", input.role),
                path,
            })
    }

    /// Read the A-supplied mutation fixture registered for a scenario id.
    ///
    /// A scenario with more than one registered mutation is a delivery defect,
    /// not a first-match lookup.
    pub fn mutation_bytes(&self, scenario: &str) -> Result<Option<Vec<u8>>, Absence> {
        let mut matches = self
            .delivery_index
            .mutations
            .iter()
            .filter(|mutation| mutation.scenario == scenario);
        let Some(mutation) = matches.next() else {
            return Ok(None);
        };
        if matches.next().is_some() {
            return Err(Absence::AmbiguousMutation {
                scenario: scenario.to_owned(),
            });
        }
        let path = self.root.join(&mutation.path);
        fs::read(&path)
            .map(Some)
            .map_err(|_| Absence::MissingMember {
                role: format!("mutation for {scenario}"),
                path,
            })
    }

    /// Every scenario this delivery owes that no indexed mutation supplies.
    ///
    /// The loader checks delivered mutations against owed scenarios in both
    /// directions: an under-delivery must not read as an assembled lane.
    pub fn undelivered_scenarios(&self) -> Vec<String> {
        let delivered: Vec<&str> = self
            .delivery_index
            .mutations
            .iter()
            .map(|mutation| mutation.scenario.as_str())
            .collect();
        scenarios_for(&self.delivery_index)
            .into_iter()
            .filter(|row| row.disposition == Disposition::RefusesBeforeLinking)
            .map(|row| row.id)
            .filter(|id| !delivered.contains(&id.as_str()))
            .collect()
    }
}

impl fmt::Debug for OfferedArtifact {
    /// Report the size and reference, never the bytes: a diagnostic must not
    /// become a second, differently shaped copy of the offer.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OfferedArtifact")
            .field("bytes", &self.bytes.len())
            .field("reference", &self.reference)
            .finish()
    }
}

fn require(root: &Path, role: &str, path: &str) -> Result<(), Absence> {
    let resolved = root.join(path);
    if resolved.is_file() {
        Ok(())
    } else {
        Err(Absence::MissingMember {
            role: role.to_owned(),
            path: resolved,
        })
    }
}

// ---------------------------------------------------------------------------
// Scenario corpus
// ---------------------------------------------------------------------------

/// A selection axis B retains independently and mutates one at a time.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectionAxis {
    Artifact,
    Contract,
    Baseline,
    Producer,
    Source,
    Model,
    Dependency,
    Definition,
    Configuration,
    Result,
    TemporalProfile,
    ClockAlternative,
    ClockParameter,
}

impl SelectionAxis {
    pub const ALL: [Self; 13] = [
        Self::Artifact,
        Self::Contract,
        Self::Baseline,
        Self::Producer,
        Self::Source,
        Self::Model,
        Self::Dependency,
        Self::Definition,
        Self::Configuration,
        Self::Result,
        Self::TemporalProfile,
        Self::ClockAlternative,
        Self::ClockParameter,
    ];

    pub fn slug(self) -> &'static str {
        match self {
            Self::Artifact => "artifact",
            Self::Contract => "contract",
            Self::Baseline => "baseline",
            Self::Producer => "producer",
            Self::Source => "source",
            Self::Model => "model",
            Self::Dependency => "dependency",
            Self::Definition => "definition",
            Self::Configuration => "configuration",
            Self::Result => "result",
            Self::TemporalProfile => "temporal-profile",
            Self::ClockAlternative => "clock-alternative",
            Self::ClockParameter => "clock-parameter",
        }
    }
}

/// An axis of the *offered artifact* that IT-001 mutates for both lanes.
///
/// Distinct from `SelectionAxis`: those change what B independently selected,
/// these change what A offered. Both must refuse, for different reasons.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OfferedAxis {
    WireVersion,
    Seal,
    Feature,
    ReferencedBinding,
    NumericRepresentation,
    NumericDomain,
}

impl OfferedAxis {
    pub const ALL: [Self; 6] = [
        Self::WireVersion,
        Self::Seal,
        Self::Feature,
        Self::ReferencedBinding,
        Self::NumericRepresentation,
        Self::NumericDomain,
    ];

    pub fn slug(self) -> &'static str {
        match self {
            Self::WireVersion => "wire-version",
            Self::Seal => "seal",
            Self::Feature => "feature",
            Self::ReferencedBinding => "referenced-binding",
            Self::NumericRepresentation => "numeric-representation",
            Self::NumericDomain => "numeric-domain",
        }
    }
}

/// The four digest domains that must stay separate across the `/2` selection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DigestDomain {
    ProducerCanonical,
    NativeRaw,
    CompiledArtifact,
    ProtocolResult,
}

impl DigestDomain {
    pub const ALL: [Self; 4] = [
        Self::ProducerCanonical,
        Self::NativeRaw,
        Self::CompiledArtifact,
        Self::ProtocolResult,
    ];

    pub fn slug(self) -> &'static str {
        match self {
            Self::ProducerCanonical => "producer-canonical",
            Self::NativeRaw => "native-raw",
            Self::CompiledArtifact => "compiled-artifact",
            Self::ProtocolResult => "protocol-result",
        }
    }
}

/// A producer input B must copy rather than recanonicalise.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProducerInput {
    Source,
    Model,
    Dependency,
    Configuration,
}

impl ProducerInput {
    pub const ALL: [Self; 4] = [
        Self::Source,
        Self::Model,
        Self::Dependency,
        Self::Configuration,
    ];

    pub fn slug(self) -> &'static str {
        match self {
            Self::Source => "source",
            Self::Model => "model",
            Self::Dependency => "dependency",
            Self::Configuration => "configuration",
        }
    }
}

/// How a `/2` temporal binding set is made wrong, one way at a time.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TemporalBindingMutation {
    Missing,
    Surplus,
    Duplicate,
    Reordered,
    ForeignOwner,
    OutOfRange,
}

impl TemporalBindingMutation {
    pub const ALL: [Self; 6] = [
        Self::Missing,
        Self::Surplus,
        Self::Duplicate,
        Self::Reordered,
        Self::ForeignOwner,
        Self::OutOfRange,
    ];

    pub fn slug(self) -> &'static str {
        match self {
            Self::Missing => "missing",
            Self::Surplus => "surplus",
            Self::Duplicate => "duplicate",
            Self::Reordered => "reordered",
            Self::ForeignOwner => "foreign-owner",
            Self::OutOfRange => "out-of-range",
        }
    }
}

/// Submitting one contract version's bytes at the other version's entry point.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CrossVersion {
    V1BytesAtV2Entry,
    V2BytesAtV1Entry,
}

impl CrossVersion {
    pub const ALL: [Self; 2] = [Self::V1BytesAtV2Entry, Self::V2BytesAtV1Entry];

    pub fn slug(self) -> &'static str {
        match self {
            Self::V1BytesAtV2Entry => "v1-bytes-at-v2-entry",
            Self::V2BytesAtV1Entry => "v2-bytes-at-v1-entry",
        }
    }
}

/// Where the *budget* sits relative to what one unchanged input consumes.
///
/// The input is held constant and only the budget moves. This is a budget
/// probe, not an input probe: an input below an inclusive maximum would
/// legitimately admit, which is a different scenario and not this one.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LimitBound {
    /// Exactly at the declared bound: must admit.
    Exact,
    /// One unit short of the declared bound: must return that limit's refusal.
    OneShort,
    /// The exact-bound input resubmitted with a fresh limit after a refusal.
    FreshRetry,
}

impl LimitBound {
    pub const ALL: [Self; 3] = [Self::Exact, Self::OneShort, Self::FreshRetry];

    pub fn slug(self) -> &'static str {
        match self {
            Self::Exact => "exact",
            Self::OneShort => "one-short",
            Self::FreshRetry => "fresh-retry",
        }
    }
}

/// What a scenario submits.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScenarioKind {
    /// The exact offered bytes against the independently built selection.
    Valid,
    CrossVersion(CrossVersion),
    ChangedSelection(SelectionAxis),
    /// A resealed mutation of the offered artifact itself.
    ChangedOffer(OfferedAxis),
    /// A digest of `offered` presented in `slot`'s position.
    DigestDomainSubstitution {
        offered: DigestDomain,
        slot: DigestDomain,
    },
    /// A producer input B recanonicalised instead of copying.
    Recanonicalized(ProducerInput),
    TemporalBinding(TemporalBindingMutation),
    ResourceLimit {
        limit: String,
        bound: LimitBound,
    },
}

/// What the scenario's outcome must be. Only `Valid`, the exactly sufficient
/// budget, and the fresh-budget resubmission may admit; everything else must
/// refuse before a package exists.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Disposition {
    Admits,
    RefusesBeforeLinking,
}

/// The refusal identity a scenario expects, as a B-owned label.
///
/// A two-valued admits/refuses oracle is satisfied by one shared error code
/// across every negative row, which is the defect SR-027 recorded on `/1`. Each
/// refusing scenario therefore carries a distinct expected identity here. These
/// labels are B's own; mapping them onto A's typed refusal enum is part of the
/// pin step, and that mapping must stay injective.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExpectedRefusal {
    /// Bytes of one contract version offered at another version's entry point.
    WrongContractVersion(CrossVersion),
    /// A B-retained selection identity that does not match the offer.
    ChangedSelection(SelectionAxis),
    /// An offered-artifact axis A mutated after sealing.
    ChangedOffer(OfferedAxis),
    /// A digest of one domain presented in another domain's slot.
    CrossDomainDigest {
        offered: DigestDomain,
        slot: DigestDomain,
    },
    /// A producer input B recanonicalised instead of copying byte for byte.
    RecanonicalizedInput(ProducerInput),
    /// A temporal binding set that is wrong in one named way.
    TemporalBinding(TemporalBindingMutation),
    /// A budget one unit short of what the unchanged input consumes.
    ///
    /// Named a refusal identity here only because this corpus enumerates
    /// non-admitting outcomes under one type. The outcome itself is NFR-002's
    /// resource-incomplete kind, which FR-016 keeps distinct from a refusal.
    ExhaustedBudget { limit: String },
}

#[derive(Clone, Debug)]
pub struct Scenario {
    pub id: String,
    /// The IT-001 scenario criterion this row discharges.
    pub criterion: &'static str,
    pub kind: ScenarioKind,
    pub disposition: Disposition,
    /// `None` exactly when `disposition` is `Admits`.
    pub expected_refusal: Option<ExpectedRefusal>,
}

/// The scenarios that do not depend on A's declared limit set.
///
/// One valid admission, both cross-version submissions, every changed-selection
/// axis, every offered-artifact axis, every ordered digest-domain substitution,
/// every recanonicalised producer input, and every temporal-binding mutation.
/// Every refusing row carries a distinct expected refusal identity.
pub fn static_scenarios() -> Vec<Scenario> {
    let mut scenarios = vec![Scenario {
        id: "v2-valid".to_owned(),
        criterion: "IT-001-SC-06",
        kind: ScenarioKind::Valid,
        disposition: Disposition::Admits,
        expected_refusal: None,
    }];

    for cross in CrossVersion::ALL {
        scenarios.push(refusing(
            format!("v2-cross-{}", cross.slug()),
            "IT-001-SC-06",
            ScenarioKind::CrossVersion(cross),
            ExpectedRefusal::WrongContractVersion(cross),
        ));
    }

    for axis in SelectionAxis::ALL {
        scenarios.push(refusing(
            format!("v2-changed-{}", axis.slug()),
            "IT-001-SC-06",
            ScenarioKind::ChangedSelection(axis),
            ExpectedRefusal::ChangedSelection(axis),
        ));
    }

    for axis in OfferedAxis::ALL {
        scenarios.push(refusing(
            format!("v2-offered-{}", axis.slug()),
            "IT-001-SC-06",
            ScenarioKind::ChangedOffer(axis),
            ExpectedRefusal::ChangedOffer(axis),
        ));
    }

    for offered in DigestDomain::ALL {
        for slot in DigestDomain::ALL {
            if offered == slot {
                continue;
            }
            scenarios.push(refusing(
                format!("v2-digest-{}-as-{}", offered.slug(), slot.slug()),
                "IT-001-SC-07",
                ScenarioKind::DigestDomainSubstitution { offered, slot },
                ExpectedRefusal::CrossDomainDigest { offered, slot },
            ));
        }
    }

    for input in ProducerInput::ALL {
        scenarios.push(refusing(
            format!("v2-recanonicalized-{}", input.slug()),
            "IT-001-SC-07",
            ScenarioKind::Recanonicalized(input),
            ExpectedRefusal::RecanonicalizedInput(input),
        ));
    }

    for mutation in TemporalBindingMutation::ALL {
        scenarios.push(refusing(
            format!("v2-temporal-{}", mutation.slug()),
            "IT-001-SC-08",
            ScenarioKind::TemporalBinding(mutation),
            ExpectedRefusal::TemporalBinding(mutation),
        ));
    }

    scenarios
}

fn refusing(
    id: String,
    criterion: &'static str,
    kind: ScenarioKind,
    expected_refusal: ExpectedRefusal,
) -> Scenario {
    Scenario {
        id,
        criterion,
        kind,
        disposition: Disposition::RefusesBeforeLinking,
        expected_refusal: Some(expected_refusal),
    }
}

/// The exactly-sufficient, one-short and fresh-budget probe for each limit A
/// declares. The input is held constant; only the budget varies.
pub fn resource_scenarios(limits: &[IndexedLimit]) -> Vec<Scenario> {
    let mut scenarios = Vec::with_capacity(limits.len() * LimitBound::ALL.len());
    for limit in limits {
        for bound in LimitBound::ALL {
            let refuses = bound == LimitBound::OneShort;
            scenarios.push(Scenario {
                id: format!("v2-limit-{}-{}", limit.name, bound.slug()),
                criterion: "IT-001-SC-09",
                kind: ScenarioKind::ResourceLimit {
                    limit: limit.name.clone(),
                    bound,
                },
                disposition: if refuses {
                    Disposition::RefusesBeforeLinking
                } else {
                    Disposition::Admits
                },
                expected_refusal: refuses.then(|| ExpectedRefusal::ExhaustedBudget {
                    limit: limit.name.clone(),
                }),
            });
        }
    }
    scenarios
}

/// Every scenario the `/2` lane owes for one delivery.
pub fn scenarios_for(delivery_index: &DeliveryIndex) -> Vec<Scenario> {
    let mut scenarios = static_scenarios();
    scenarios.extend(resource_scenarios(&delivery_index.limits));
    scenarios
}
