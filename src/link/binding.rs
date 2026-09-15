// SPDX-License-Identifier: AGPL-3.0-or-later
//! Version-explicit static and runtime binding contracts.
//!
//! QSL binding requirements identify static slots. QObs qualified subjects and
//! relationships identify runtime values. The two are deliberately separate:
//! no table position, display name, or QSL binding-kind spelling can become a
//! runtime identity.

use std::collections::{BTreeMap, BTreeSet};

use quire_observation::{
    AdmittedBundleKey, QualifiedSubject, Relationship, RequiredRelationship, SubjectKind,
};
use quire_spec_language::protocol_artifact::wire::{
    BindingKind, Correspondence, Declaration, Locus, Revision, SelectedDigest,
};
use serde::Serialize;

use super::requirement_identity;

/// Exact supported static binding-definition contract.
pub const BINDING_DEFINITION_CONTRACT: &str = "quire.protocol.binding-definition/1";
/// Exact supported runtime binding-set contract.
pub const RUNTIME_BINDING_CONTRACT: &str = "quire.protocol.runtime-binding/1";

/// Exact identity of one independently selected static binding definition.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct BindingArtifactIdentity {
    /// Producer-authored artifact identity.
    pub identity: String,
    /// Exact selected revision and namespace.
    pub revision: Revision,
    /// Exact selected digest, including its domain and domain version.
    pub digest: SelectedDigest,
}

/// Closed FR-217 role-internality vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RoleInternality {
    /// Static role inside the assessed protocol.
    Internal,
    /// Static role outside the assessed protocol; no observability is implied.
    External,
}

impl RoleInternality {
    fn parse(role: u32, value: &str) -> Result<Self, BindingOfferRefusal> {
        match value {
            "internal" => Ok(Self::Internal),
            "external" => Ok(Self::External),
            _ => Err(BindingOfferRefusal::UnknownInternality {
                role,
                offered: value.to_owned(),
            }),
        }
    }
}

/// Offered role classification before the closed label is admitted.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleClassificationOffer {
    /// Declaration-local role-table index.
    pub role: u32,
    /// Exact offered spelling; only lowercase `internal` and `external` admit.
    pub internality: String,
}

/// One admitted role classification.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RoleClassification {
    /// Declaration-local role-table index.
    pub role: u32,
    /// Exact closed classification.
    pub internality: RoleInternality,
}

/// Independently selected compatibility for one QSL runtime slot.
///
/// Subject and relationship slots are different alternatives. Consequently a
/// relationship cannot be accepted merely because an opaque subject-kind byte
/// string happens to equal a QSL label.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SlotCompatibility {
    /// One slot filled by a QObs FR-287 subject.
    Subject {
        /// Declaration-local binding-requirement index.
        requirement: u32,
        /// Exact selected producer authority.
        authority: AdmittedBundleKey,
        /// Exact authority-local subject-kind discriminator.
        producer_kind: SubjectKind,
        /// Exact selected producer/native correspondence, when present.
        correspondence: Option<Correspondence>,
    },
    /// One slot filled by a QObs FR-262 relationship.
    Relationship {
        /// Declaration-local binding-requirement index.
        requirement: u32,
        /// Exact selected producer authority.
        authority: AdmittedBundleKey,
        /// Exact declaration and ordered endpoint premise.
        required: Box<RequiredRelationship>,
        /// Exact selected producer/native correspondence, when present.
        correspondence: Option<Correspondence>,
    },
}

impl SlotCompatibility {
    /// Declaration-local binding-requirement index.
    #[must_use]
    pub const fn requirement(&self) -> u32 {
        match self {
            Self::Subject { requirement, .. } | Self::Relationship { requirement, .. } => {
                *requirement
            }
        }
    }

    /// Exact selected QObs producer authority.
    #[must_use]
    pub const fn authority(&self) -> &AdmittedBundleKey {
        match self {
            Self::Subject { authority, .. } | Self::Relationship { authority, .. } => authority,
        }
    }

    /// Exact selected QSL producer/native correspondence, when one exists.
    #[must_use]
    pub const fn correspondence(&self) -> Option<&Correspondence> {
        match self {
            Self::Subject { correspondence, .. } | Self::Relationship { correspondence, .. } => {
                correspondence.as_ref()
            }
        }
    }
}

/// Offered static binding definition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BindingDefinitionOffer {
    /// Exact contract version label.
    pub contract: String,
    /// Exact selected artifact identity.
    pub artifact: BindingArtifactIdentity,
    /// Admitted compiled-protocol declaration this definition classifies.
    pub declaration: u32,
    /// Complete offered role classifications in role-table order.
    pub roles: Vec<RoleClassificationOffer>,
    /// Complete selected slot compatibility table in requirement order.
    pub slots: Vec<SlotCompatibility>,
}

/// Caller-selected expected static binding definition.
///
/// The offer has no authority to select these fields. Admission succeeds only
/// when every offered field agrees with this independently constructed value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpectedBindingDefinition {
    /// Exact selected artifact identity.
    pub artifact: BindingArtifactIdentity,
    /// Exact selected declaration.
    pub declaration: u32,
    /// Complete expected role classifications.
    pub roles: Vec<RoleClassification>,
    /// Complete expected slot compatibility table.
    pub slots: Vec<SlotCompatibility>,
}

/// Constructor-guarded accepted static binding definition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcceptedBindingDefinition {
    artifact: BindingArtifactIdentity,
    declaration: u32,
    roles: Vec<RoleClassification>,
    slots: Vec<SlotCompatibility>,
}

impl AcceptedBindingDefinition {
    /// Exact selected artifact identity.
    #[must_use]
    pub const fn artifact(&self) -> &BindingArtifactIdentity {
        &self.artifact
    }

    /// Exact target declaration.
    #[must_use]
    pub const fn declaration(&self) -> u32 {
        self.declaration
    }

    /// Complete role classifications.
    #[must_use]
    pub fn roles(&self) -> &[RoleClassification] {
        &self.roles
    }

    /// Complete selected slot compatibility table.
    #[must_use]
    pub fn slots(&self) -> &[SlotCompatibility] {
        &self.slots
    }
}

/// Why an offered static binding definition was not accepted.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BindingOfferRefusal {
    /// The contract was absent, old or unknown.
    UnsupportedContract { offered: String },
    /// One identity component was empty.
    EmptyIdentity,
    /// A role or slot handle occurred twice.
    DuplicateHandle { table: &'static str, handle: u32 },
    /// Internality lay outside FR-217's exact vocabulary.
    UnknownInternality { role: u32, offered: String },
    /// The offered definition disagreed with the independent selection.
    SelectionMismatch,
}

/// Admit one exact static binding definition against independent selections.
pub fn admit_binding_definition(
    offer: BindingDefinitionOffer,
    expected: &ExpectedBindingDefinition,
) -> Result<AcceptedBindingDefinition, BindingOfferRefusal> {
    if offer.contract != BINDING_DEFINITION_CONTRACT {
        return Err(BindingOfferRefusal::UnsupportedContract {
            offered: offer.contract,
        });
    }
    if offer.artifact.identity.is_empty()
        || offer.artifact.revision.namespace.is_empty()
        || offer.artifact.revision.value.is_empty()
        || offer.artifact.digest.domain.is_empty()
        || offer.artifact.digest.version.is_empty()
        || offer.artifact.digest.algorithm.is_empty()
        || offer.artifact.digest.value.is_empty()
    {
        return Err(BindingOfferRefusal::EmptyIdentity);
    }

    let mut role_handles = BTreeSet::new();
    let roles = offer
        .roles
        .into_iter()
        .map(|role| {
            if !role_handles.insert(role.role) {
                return Err(BindingOfferRefusal::DuplicateHandle {
                    table: "roles",
                    handle: role.role,
                });
            }
            Ok(RoleClassification {
                role: role.role,
                internality: RoleInternality::parse(role.role, &role.internality)?,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut slot_handles = BTreeSet::new();
    for slot in &offer.slots {
        if !slot_handles.insert(slot.requirement()) {
            return Err(BindingOfferRefusal::DuplicateHandle {
                table: "slots",
                handle: slot.requirement(),
            });
        }
    }

    if offer.artifact != expected.artifact
        || offer.declaration != expected.declaration
        || roles != expected.roles
        || offer.slots != expected.slots
    {
        return Err(BindingOfferRefusal::SelectionMismatch);
    }

    Ok(AcceptedBindingDefinition {
        artifact: offer.artifact,
        declaration: offer.declaration,
        roles,
        slots: offer.slots,
    })
}

/// One offered runtime subject assigned to one static binding requirement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubjectBinding {
    /// Declaration-local binding-requirement index.
    pub requirement: u32,
    /// Imported constructor-guarded FR-287 value.
    pub subject: QualifiedSubject,
}

/// One offered runtime relationship assigned to one static relationship slot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationshipBinding {
    /// Declaration-local relationship binding-requirement index.
    pub requirement: u32,
    /// Imported constructor-guarded FR-262 value.
    pub relationship: Relationship,
}

/// Finite runtime values offered to one accepted static binding definition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeBindingSet {
    /// Exact contract version label.
    pub contract: String,
    /// Exact accepted static definition this set fills.
    pub target: BindingArtifactIdentity,
    /// Runtime subject entries.
    pub subjects: Vec<SubjectBinding>,
    /// Runtime relationship entries.
    pub relationships: Vec<RelationshipBinding>,
}

/// One validated runtime subject. Its slot association is constructor-private.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundSubject {
    requirement: u32,
    subject: QualifiedSubject,
}

impl BoundSubject {
    /// Exact static slot filled by this value.
    #[must_use]
    pub const fn requirement(&self) -> u32 {
        self.requirement
    }

    /// Exact owner-constructed FR-287 subject.
    #[must_use]
    pub const fn subject(&self) -> &QualifiedSubject {
        &self.subject
    }
}

/// One validated runtime relationship. Its slot association is constructor-private.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundRelationship {
    requirement: u32,
    relationship: Relationship,
}

impl BoundRelationship {
    /// Exact static slot filled by this value.
    #[must_use]
    pub const fn requirement(&self) -> u32 {
        self.requirement
    }

    /// Exact owner-constructed FR-262 relationship.
    #[must_use]
    pub const fn relationship(&self) -> &Relationship {
        &self.relationship
    }
}

/// Complete validated runtime binding set.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinkedRuntimeBindings {
    subjects: Vec<BoundSubject>,
    relationships: Vec<BoundRelationship>,
}

impl LinkedRuntimeBindings {
    /// Validated subjects in static-slot order.
    #[must_use]
    pub fn subjects(&self) -> &[BoundSubject] {
        &self.subjects
    }

    /// Validated, replay-deduplicated relationships in static-slot order.
    #[must_use]
    pub fn relationships(&self) -> &[BoundRelationship] {
        &self.relationships
    }
}

/// Exact runtime/static binding refusal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BindingRefusal {
    /// Runtime contract was absent, old, or unknown.
    UnsupportedRuntimeContract { offered: String },
    /// Runtime set targeted another binding artifact.
    WrongRuntimeTarget,
    /// A declaration role had no static internality classification.
    MissingStaticRole { role: u32, locus: Locus },
    /// A static classification named no declaration role.
    SurplusStaticRole { ordinal: usize, role: u32 },
    /// Complete role classifications were not in declaration role-table order.
    MisorderedStaticRole {
        ordinal: usize,
        role: u32,
        expected_role: u32,
        locus: Locus,
    },
    /// A declaration binding requirement had no selected compatibility fact.
    MissingStaticSlot { requirement: u32, locus: Locus },
    /// A selected static slot is outside the declaration's binding table.
    NoSuchStaticSlot { requirement: u32 },
    /// Complete compatibility facts were not in declaration binding-table order.
    MisorderedStaticSlot {
        ordinal: usize,
        requirement: u32,
        expected_requirement: u32,
        locus: Locus,
    },
    /// A selected slot alternative disagrees with the QSL requirement kind.
    WrongStaticSlotKind {
        requirement: u32,
        expected_relationship: bool,
        found: BindingKind,
        locus: Locus,
    },
    /// A selected relationship premise belongs to another QObs authority.
    IncompatibleStaticRelationshipAuthority { requirement: u32, locus: Locus },
    /// The selected QSL producer/native correspondence is not the slot's one.
    IncompatibleStaticSlot { requirement: u32, locus: Locus },
    /// A selected runtime slot was not supplied.
    MissingRuntimeBinding { requirement: u32, locus: Locus },
    /// An offered runtime entry does not name a selected static slot.
    SurplusRuntimeBinding { ordinal: usize, requirement: u32 },
    /// A selected slot was supplied more than once.
    DuplicateRuntimeBinding {
        ordinal: usize,
        requirement: u32,
        locus: Locus,
    },
    /// A subject was offered to a relationship slot or the reverse.
    WrongRuntimeEntryKind {
        ordinal: usize,
        requirement: u32,
        locus: Locus,
    },
    /// A runtime value belongs to another admitted producer authority.
    ForeignRuntimeAuthority {
        ordinal: usize,
        requirement: u32,
        locus: Locus,
    },
    /// A runtime subject has another producer-local kind.
    WrongRuntimeSubjectKind {
        ordinal: usize,
        requirement: u32,
        locus: Locus,
    },
    /// A runtime identity was copied from the static slot coordinate.
    SlotDerivedRuntimeIdentity {
        ordinal: usize,
        requirement: u32,
        locus: Locus,
    },
    /// One exact runtime key was reused with incompatible static binding facts.
    ContradictoryRuntimeIdentity {
        ordinal: usize,
        requirement: u32,
        previous_requirement: u32,
        locus: Locus,
    },
    /// A relationship does not satisfy its exact declaration/endpoints premise.
    IncompatibleRelationship {
        ordinal: usize,
        requirement: u32,
        locus: Locus,
    },
    /// One authority-qualified relationship identity carried different facts.
    ContradictoryRelationship {
        ordinal: usize,
        requirement: u32,
        locus: Locus,
    },
    /// Two distinct relationship identities satisfied one required slot.
    AmbiguousRelationship { requirement: u32, locus: Locus },
}

/// Non-empty ordered binding diagnostics.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BindingDiagnostics {
    first: BindingRefusal,
    rest: Vec<BindingRefusal>,
}

impl BindingDiagnostics {
    fn one(first: BindingRefusal) -> Self {
        Self {
            first,
            rest: Vec::new(),
        }
    }

    fn from_vec(findings: Vec<BindingRefusal>) -> Option<Self> {
        let mut findings = findings.into_iter();
        let first = findings.next()?;
        Some(Self {
            first,
            rest: findings.collect(),
        })
    }

    /// First established finding.
    #[must_use]
    pub const fn first(&self) -> &BindingRefusal {
        &self.first
    }

    /// Every independent finding in deterministic inspection order.
    pub fn iter(&self) -> impl Iterator<Item = &BindingRefusal> {
        std::iter::once(&self.first).chain(&self.rest)
    }

    /// Number of retained findings.
    #[must_use]
    pub fn count(&self) -> usize {
        1 + self.rest.len()
    }
}

pub(super) fn validate_static_definition(
    declaration: &Declaration,
    package_models: &[quire_spec_language::protocol_artifact::wire::Model],
    binding: &AcceptedBindingDefinition,
) -> Result<(), BindingDiagnostics> {
    let mut findings = Vec::new();
    let protocol_roles = match &declaration.body {
        quire_spec_language::protocol_artifact::wire::Body::Protocol { roles, .. } => roles,
        quire_spec_language::protocol_artifact::wire::Body::Predicate { .. }
        | quire_spec_language::protocol_artifact::wire::Body::State { .. }
        | quire_spec_language::protocol_artifact::wire::Body::Temporal { .. } => return Ok(()),
    };

    let selected_role_handles: BTreeSet<u32> =
        binding.roles().iter().map(|role| role.role).collect();
    for (index, role) in protocol_roles.iter().enumerate() {
        let Ok(role_handle) = u32::try_from(index) else {
            continue;
        };
        if !selected_role_handles.contains(&role_handle) {
            findings.push(BindingRefusal::MissingStaticRole {
                role: role_handle,
                locus: role.locus.clone(),
            });
        }
    }
    for (ordinal, role) in binding.roles().iter().enumerate() {
        if usize::try_from(role.role)
            .ok()
            .is_none_or(|index| index >= protocol_roles.len())
        {
            findings.push(BindingRefusal::SurplusStaticRole {
                ordinal,
                role: role.role,
            });
        }
    }
    if binding.roles().len() == protocol_roles.len()
        && selected_role_handles.len() == protocol_roles.len()
        && selected_role_handles
            .iter()
            .enumerate()
            .all(|(index, handle)| usize::try_from(*handle).ok() == Some(index))
    {
        for (ordinal, role) in binding.roles().iter().enumerate() {
            let Ok(expected_role) = u32::try_from(ordinal) else {
                continue;
            };
            if role.role != expected_role {
                if let Some(source_role) = usize::try_from(role.role)
                    .ok()
                    .and_then(|index| protocol_roles.get(index))
                {
                    findings.push(BindingRefusal::MisorderedStaticRole {
                        ordinal,
                        role: role.role,
                        expected_role,
                        locus: source_role.locus.clone(),
                    });
                }
            }
        }
    }

    let selected_slot_handles: BTreeSet<u32> = binding
        .slots()
        .iter()
        .map(SlotCompatibility::requirement)
        .collect();
    for (index, requirement) in declaration.bindings.iter().enumerate() {
        let Ok(requirement_handle) = u32::try_from(index) else {
            continue;
        };
        if !selected_slot_handles.contains(&requirement_handle) {
            findings.push(BindingRefusal::MissingStaticSlot {
                requirement: requirement_handle,
                locus: requirement.locus.clone(),
            });
        }
    }
    if binding.slots().len() == declaration.bindings.len()
        && selected_slot_handles.len() == declaration.bindings.len()
        && selected_slot_handles
            .iter()
            .enumerate()
            .all(|(index, handle)| usize::try_from(*handle).ok() == Some(index))
    {
        for (ordinal, slot) in binding.slots().iter().enumerate() {
            let Ok(expected_requirement) = u32::try_from(ordinal) else {
                continue;
            };
            let requirement_index = slot.requirement();
            if requirement_index != expected_requirement {
                if let Some(requirement) = declaration
                    .bindings
                    .get(usize::try_from(requirement_index).unwrap_or(usize::MAX))
                {
                    findings.push(BindingRefusal::MisorderedStaticSlot {
                        ordinal,
                        requirement: requirement_index,
                        expected_requirement,
                        locus: requirement.locus.clone(),
                    });
                }
            }
        }
    }

    for slot in binding.slots() {
        let requirement_index = slot.requirement();
        let Some(requirement) = usize::try_from(requirement_index)
            .ok()
            .and_then(|index| declaration.bindings.get(index))
        else {
            findings.push(BindingRefusal::NoSuchStaticSlot {
                requirement: requirement_index,
            });
            continue;
        };
        let expected_relationship = matches!(slot, SlotCompatibility::Relationship { .. });
        let kind_agrees = (requirement.kind == BindingKind::Relationship) == expected_relationship;
        if !kind_agrees {
            findings.push(BindingRefusal::WrongStaticSlotKind {
                requirement: requirement_index,
                expected_relationship,
                found: requirement.kind,
                locus: requirement.locus.clone(),
            });
        }
        if let SlotCompatibility::Relationship {
            authority,
            required,
            ..
        } = slot
        {
            if required.source().authority() != authority
                || required.target().authority() != authority
            {
                findings.push(BindingRefusal::IncompatibleStaticRelationshipAuthority {
                    requirement: requirement_index,
                    locus: requirement.locus.clone(),
                });
            }
        }
        let selected = requirement.model.0.as_ref().and_then(|export| {
            usize::try_from(export.model)
                .ok()
                .and_then(|index| package_models.get(index))
                .and_then(|model| model.correspondence.0.as_ref())
        });
        if selected != slot.correspondence() {
            findings.push(BindingRefusal::IncompatibleStaticSlot {
                requirement: requirement_index,
                locus: requirement.locus.clone(),
            });
        }
    }

    BindingDiagnostics::from_vec(findings).map_or(Ok(()), Err)
}

pub(super) fn validate_runtime(
    declaration: &Declaration,
    scope: &[u8],
    binding: &AcceptedBindingDefinition,
    runtime: RuntimeBindingSet,
) -> Result<LinkedRuntimeBindings, BindingDiagnostics> {
    if runtime.contract != RUNTIME_BINDING_CONTRACT {
        return Err(BindingDiagnostics::one(
            BindingRefusal::UnsupportedRuntimeContract {
                offered: runtime.contract,
            },
        ));
    }
    if runtime.target != *binding.artifact() {
        return Err(BindingDiagnostics::one(BindingRefusal::WrongRuntimeTarget));
    }

    let mut findings = Vec::new();

    let slots: BTreeMap<u32, &SlotCompatibility> = binding
        .slots()
        .iter()
        .map(|slot| (slot.requirement(), slot))
        .collect();
    let mut seen_subjects = BTreeSet::new();
    let mut subject_keys: Vec<(QualifiedSubject, u32, BindingKind, Option<Correspondence>)> =
        Vec::new();
    let mut subjects = Vec::new();
    for (ordinal, offered) in runtime.subjects.into_iter().enumerate() {
        let Some(slot) = slots.get(&offered.requirement).copied() else {
            findings.push(BindingRefusal::SurplusRuntimeBinding {
                ordinal,
                requirement: offered.requirement,
            });
            continue;
        };
        let Some(requirement) = declaration
            .bindings
            .get(usize::try_from(offered.requirement).unwrap_or(usize::MAX))
        else {
            continue;
        };
        if !seen_subjects.insert(offered.requirement) {
            findings.push(BindingRefusal::DuplicateRuntimeBinding {
                ordinal,
                requirement: offered.requirement,
                locus: requirement.locus.clone(),
            });
            continue;
        }
        let SlotCompatibility::Subject {
            authority,
            producer_kind,
            correspondence,
            ..
        } = slot
        else {
            findings.push(BindingRefusal::WrongRuntimeEntryKind {
                ordinal,
                requirement: offered.requirement,
                locus: requirement.locus.clone(),
            });
            continue;
        };
        let mut valid = true;
        if offered.subject.authority() != authority {
            findings.push(BindingRefusal::ForeignRuntimeAuthority {
                ordinal,
                requirement: offered.requirement,
                locus: requirement.locus.clone(),
            });
            valid = false;
        }
        if offered.subject.kind() != producer_kind {
            findings.push(BindingRefusal::WrongRuntimeSubjectKind {
                ordinal,
                requirement: offered.requirement,
                locus: requirement.locus.clone(),
            });
            valid = false;
        }
        let slot_identity = requirement_identity(
            scope,
            binding.declaration(),
            offered.requirement,
            requirement,
        );
        if offered.subject.identity().as_bytes() == slot_identity {
            findings.push(BindingRefusal::SlotDerivedRuntimeIdentity {
                ordinal,
                requirement: offered.requirement,
                locus: requirement.locus.clone(),
            });
            valid = false;
        }
        if let Some((_, previous_requirement, previous_kind, previous_correspondence)) =
            subject_keys
                .iter()
                .find(|(known, _, _, _)| known == &offered.subject)
        {
            if subject_identity_is_single_binding(*previous_kind)
                || subject_identity_is_single_binding(requirement.kind)
                || previous_kind != &requirement.kind
                || previous_correspondence.as_ref() != correspondence.as_ref()
            {
                findings.push(BindingRefusal::ContradictoryRuntimeIdentity {
                    ordinal,
                    requirement: offered.requirement,
                    previous_requirement: *previous_requirement,
                    locus: requirement.locus.clone(),
                });
                valid = false;
            }
        } else {
            subject_keys.push((
                offered.subject.clone(),
                offered.requirement,
                requirement.kind,
                correspondence.clone(),
            ));
        }
        if valid {
            subjects.push(BoundSubject {
                requirement: offered.requirement,
                subject: offered.subject,
            });
        }
    }

    let subject_slots: BTreeSet<u32> = binding
        .slots()
        .iter()
        .filter_map(|slot| match slot {
            SlotCompatibility::Subject { requirement, .. } => Some(*requirement),
            SlotCompatibility::Relationship { .. } => None,
        })
        .collect();
    for requirement in subject_slots.difference(&seen_subjects) {
        if let Some(source) = declaration
            .bindings
            .get(usize::try_from(*requirement).unwrap_or(usize::MAX))
        {
            findings.push(BindingRefusal::MissingRuntimeBinding {
                requirement: *requirement,
                locus: source.locus.clone(),
            });
        }
    }

    let mut seen_relationship_slots = BTreeSet::new();
    let mut relationship_keys: Vec<(AdmittedBundleKey, Vec<u8>, Relationship)> = Vec::new();
    let mut relationships_by_slot: BTreeMap<u32, Vec<Relationship>> = BTreeMap::new();
    for (ordinal, offered) in runtime.relationships.into_iter().enumerate() {
        let Some(slot) = slots.get(&offered.requirement).copied() else {
            findings.push(BindingRefusal::SurplusRuntimeBinding {
                ordinal,
                requirement: offered.requirement,
            });
            continue;
        };
        let Some(requirement) = declaration
            .bindings
            .get(usize::try_from(offered.requirement).unwrap_or(usize::MAX))
        else {
            continue;
        };
        seen_relationship_slots.insert(offered.requirement);
        let SlotCompatibility::Relationship {
            authority,
            required,
            ..
        } = slot
        else {
            findings.push(BindingRefusal::WrongRuntimeEntryKind {
                ordinal,
                requirement: offered.requirement,
                locus: requirement.locus.clone(),
            });
            continue;
        };
        let actual = offered.relationship;
        let mut valid = true;
        if actual.authority() != authority {
            findings.push(BindingRefusal::ForeignRuntimeAuthority {
                ordinal,
                requirement: offered.requirement,
                locus: requirement.locus.clone(),
            });
            valid = false;
        }
        if actual.declaration_identity() != required.declaration_identity()
            || actual.source() != required.source()
            || actual.target() != required.target()
        {
            findings.push(BindingRefusal::IncompatibleRelationship {
                ordinal,
                requirement: offered.requirement,
                locus: requirement.locus.clone(),
            });
            valid = false;
        }
        if let Some((_, _, known)) =
            relationship_keys
                .iter()
                .find(|(known_authority, identity, _)| {
                    known_authority == actual.authority()
                        && identity.as_slice() == actual.identity().as_bytes()
                })
        {
            if known != &actual {
                findings.push(BindingRefusal::ContradictoryRelationship {
                    ordinal,
                    requirement: offered.requirement,
                    locus: requirement.locus.clone(),
                });
                valid = false;
            }
        } else {
            relationship_keys.push((
                actual.authority().clone(),
                actual.identity().as_bytes().to_vec(),
                actual.clone(),
            ));
        }
        if valid {
            let retained = relationships_by_slot
                .entry(offered.requirement)
                .or_default();
            if !retained.contains(&actual) {
                retained.push(actual);
            }
        }
    }

    let relationship_slots: BTreeSet<u32> = binding
        .slots()
        .iter()
        .filter_map(|slot| match slot {
            SlotCompatibility::Relationship { requirement, .. } => Some(*requirement),
            SlotCompatibility::Subject { .. } => None,
        })
        .collect();
    for requirement in relationship_slots.difference(&seen_relationship_slots) {
        if let Some(source) = declaration
            .bindings
            .get(usize::try_from(*requirement).unwrap_or(usize::MAX))
        {
            findings.push(BindingRefusal::MissingRuntimeBinding {
                requirement: *requirement,
                locus: source.locus.clone(),
            });
        }
    }

    let mut relationships = Vec::new();
    for (requirement, values) in relationships_by_slot {
        let Some(source) = declaration
            .bindings
            .get(usize::try_from(requirement).unwrap_or(usize::MAX))
        else {
            continue;
        };
        if values.len() > 1 {
            findings.push(BindingRefusal::AmbiguousRelationship {
                requirement,
                locus: source.locus.clone(),
            });
            continue;
        }
        if let Some(relationship) = values.into_iter().next() {
            relationships.push(BoundRelationship {
                requirement,
                relationship,
            });
        }
    }

    if let Some(diagnostics) = BindingDiagnostics::from_vec(findings) {
        return Err(diagnostics);
    }
    subjects.sort_by_key(BoundSubject::requirement);
    relationships.sort_by_key(BoundRelationship::requirement);
    Ok(LinkedRuntimeBindings {
        subjects,
        relationships,
    })
}

fn subject_identity_is_single_binding(kind: BindingKind) -> bool {
    matches!(
        kind,
        BindingKind::WorkflowInstance
            | BindingKind::RoleInstance
            | BindingKind::Message
            | BindingKind::Send
            | BindingKind::Receive
            | BindingKind::Delivery
            | BindingKind::Attempt
            | BindingKind::Effect
    )
}

#[cfg(test)]
mod tests {
    use ix_trace_rs::trace;
    use quire_observation::{QualifiedSubject, RelationshipIdentity, SubjectIdentity};
    use quire_spec_language::{protocol_artifact::wire as w, ByteDigest};

    use super::*;

    mod producer_owner {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/support/producer_owner.rs"
        ));
    }

    fn locus() -> w::Locus {
        w::Locus {
            source: 0,
            span: w::Span { start: 10, end: 20 },
        }
    }

    fn handle(index: u32) -> w::Handle {
        w::Handle {
            declaration: 0,
            index,
        }
    }

    fn artifact_ref() -> w::ArtifactRef {
        w::ArtifactRef {
            ref_version: "ix.artifact-ref/3-draft".into(),
            kind: w::ArtifactKind::Binding,
            authority: "test:authority".into(),
            identity: "test:binding-authority".into(),
            revision: w::Revision {
                namespace: "test:revision".into(),
                value: "1".into(),
            },
            digest: ByteDigest::of(b"test binding authority"),
            wire: w::Wire {
                identity: "test:binding".into(),
                version: "1".into(),
            },
        }
    }

    fn declaration() -> w::Declaration {
        w::Declaration {
            name: "Flow".into(),
            locus: locus(),
            requirement: w::Requirement {
                package: "test:requirements".into(),
                identity: "FR-test".into(),
                revision: w::Revision {
                    namespace: "test:requirement-revision".into(),
                    value: "1".into(),
                },
            },
            clause: "test-clause".into(),
            execution: w::Execution::Initialization {
                name: "test".into(),
            },
            profile: 0,
            requires: Vec::new(),
            scopes: Vec::new(),
            anchors: Vec::new(),
            binders: Vec::new(),
            values: Vec::new(),
            temporal: Vec::new(),
            bindings: vec![w::BindingRequirement {
                name: "subject-slot".into(),
                kind: w::BindingKind::Participant,
                value_type: w::Nullable(None),
                authority: artifact_ref(),
                contract: 0,
                model: w::Nullable(None),
                subject: w::Subject::Declaration { declaration: 0 },
                anchor: handle(0),
                scope: handle(0),
                relation: w::Nullable(None),
                requires: Vec::new(),
                locus: locus(),
            }],
            body: w::Body::Protocol {
                input: handle(0),
                activation: w::Activation::Origin { anchor: handle(0) },
                captures: Vec::new(),
                roles: Vec::new(),
                relationships: Vec::new(),
                channels: Vec::new(),
                compensations: Vec::new(),
                temporal_requirements: Vec::new(),
                controls: Vec::new(),
                causal_edges: Vec::new(),
                run: handle(0),
                finish: Box::new(w::Finish {
                    name: "finish".into(),
                    binder: handle(0),
                    constraint: handle(0),
                    closure: 0,
                    locus: locus(),
                }),
            },
        }
    }

    fn artifact() -> BindingArtifactIdentity {
        BindingArtifactIdentity {
            identity: "test:binding-definition".into(),
            revision: w::Revision {
                namespace: "test:binding-revision".into(),
                value: "1".into(),
            },
            digest: w::SelectedDigest {
                domain: "test:binding-digest".into(),
                version: "1".into(),
                algorithm: "sha256".into(),
                value: format!("sha256:{}", "a".repeat(64)),
            },
        }
    }

    fn accepted(authority: AdmittedBundleKey, kind: SubjectKind) -> AcceptedBindingDefinition {
        let slot = SlotCompatibility::Subject {
            requirement: 0,
            authority,
            producer_kind: kind,
            correspondence: None,
        };
        accepted_selection(Vec::new(), vec![slot])
    }

    fn accepted_selection(
        roles: Vec<RoleClassification>,
        slots: Vec<SlotCompatibility>,
    ) -> AcceptedBindingDefinition {
        let expected = ExpectedBindingDefinition {
            artifact: artifact(),
            declaration: 0,
            roles: roles.clone(),
            slots: slots.clone(),
        };
        admit_binding_definition(
            BindingDefinitionOffer {
                contract: BINDING_DEFINITION_CONTRACT.into(),
                artifact: artifact(),
                declaration: 0,
                roles: roles
                    .iter()
                    .map(|role| RoleClassificationOffer {
                        role: role.role,
                        internality: match role.internality {
                            RoleInternality::Internal => "internal",
                            RoleInternality::External => "external",
                        }
                        .into(),
                    })
                    .collect(),
                slots,
            },
            &expected,
        )
        .expect("exact static selection")
    }

    fn correspondence() -> Correspondence {
        Correspondence {
            producer: quire_spec_language::protocol_artifact::wire::ProducerObject {
                interface: 1,
                kind: "test:producer-kind".into(),
                authority: "test:producer-authority".into(),
                identity: "test:producer-object".into(),
                revision: w::Revision {
                    namespace: "test:producer-revision".into(),
                    value: "1".into(),
                },
                digest: w::SelectedDigest {
                    domain: "test:producer-digest".into(),
                    version: "1".into(),
                    algorithm: "sha256".into(),
                    value: format!("sha256:{}", "c".repeat(64)),
                },
            },
            native: 2,
            relation: 3,
            exports: vec![0],
        }
    }

    fn subject(
        producer: &quire_observation::AdmittedStaticBundle,
        kind: SubjectKind,
        identity: Vec<u8>,
    ) -> QualifiedSubject {
        QualifiedSubject::new(
            producer,
            kind,
            SubjectIdentity::new(identity).expect("non-empty identity"),
        )
    }

    fn relationship_declaration(slots: usize) -> w::Declaration {
        let mut declaration = declaration();
        declaration.bindings = (0..slots)
            .map(|index| {
                let mut binding = declaration.bindings[0].clone();
                binding.name = format!("relationship-slot-{index}");
                binding.kind = w::BindingKind::Relationship;
                binding
            })
            .collect();
        declaration
    }

    fn subject_declaration(kind: w::BindingKind, slots: usize) -> w::Declaration {
        let mut declaration = declaration();
        declaration.bindings = (0..slots)
            .map(|index| {
                let mut binding = declaration.bindings[0].clone();
                binding.name = format!("subject-slot-{index}");
                binding.kind = kind;
                binding
            })
            .collect();
        declaration
    }

    fn relationship_subjects(
        producer: &quire_observation::AdmittedStaticBundle,
        suffix: &str,
    ) -> (QualifiedSubject, QualifiedSubject) {
        (
            subject(
                producer,
                SubjectKind::new(producer_owner::SUBJECT_KIND).expect("source kind"),
                format!("source:{suffix}").into_bytes(),
            ),
            subject(
                producer,
                SubjectKind::new(producer_owner::TARGET_KIND).expect("target kind"),
                format!("target:{suffix}").into_bytes(),
            ),
        )
    }

    fn accepted_relationships(
        producer: &quire_observation::AdmittedStaticBundle,
        required: Vec<quire_observation::RequiredRelationship>,
    ) -> AcceptedBindingDefinition {
        let slots: Vec<_> = required
            .into_iter()
            .enumerate()
            .map(|(index, required)| SlotCompatibility::Relationship {
                requirement: u32::try_from(index).expect("test slot index"),
                authority: producer.key().clone(),
                required: Box::new(required),
                correspondence: None,
            })
            .collect();
        let expected = ExpectedBindingDefinition {
            artifact: artifact(),
            declaration: 0,
            roles: Vec::new(),
            slots: slots.clone(),
        };
        admit_binding_definition(
            BindingDefinitionOffer {
                contract: BINDING_DEFINITION_CONTRACT.into(),
                artifact: artifact(),
                declaration: 0,
                roles: Vec::new(),
                slots,
            },
            &expected,
        )
        .expect("relationship selection")
    }

    #[trace("TC-002", "FR-002-AC-3")]
    #[test]
    fn tc_002_static_binding_contract_internality_and_handles_are_exact() {
        let expected_empty = ExpectedBindingDefinition {
            artifact: artifact(),
            declaration: 0,
            roles: Vec::new(),
            slots: Vec::new(),
        };
        for contract in ["", "quire.protocol.binding-definition/0", "unknown/1"] {
            let refusal = admit_binding_definition(
                BindingDefinitionOffer {
                    contract: contract.into(),
                    artifact: artifact(),
                    declaration: 0,
                    roles: Vec::new(),
                    slots: Vec::new(),
                },
                &expected_empty,
            )
            .expect_err("absent, old, and unknown contracts refuse");
            assert_eq!(
                refusal,
                BindingOfferRefusal::UnsupportedContract {
                    offered: contract.into()
                }
            );
        }

        let expected_role = ExpectedBindingDefinition {
            artifact: artifact(),
            declaration: 0,
            roles: vec![RoleClassification {
                role: 0,
                internality: RoleInternality::Internal,
            }],
            slots: Vec::new(),
        };
        for internality in ["Internal", "external ", "observer"] {
            let refusal = admit_binding_definition(
                BindingDefinitionOffer {
                    contract: BINDING_DEFINITION_CONTRACT.into(),
                    artifact: artifact(),
                    declaration: 0,
                    roles: vec![RoleClassificationOffer {
                        role: 0,
                        internality: internality.into(),
                    }],
                    slots: Vec::new(),
                },
                &expected_role,
            )
            .expect_err("only the exact closed internality labels admit");
            assert_eq!(
                refusal,
                BindingOfferRefusal::UnknownInternality {
                    role: 0,
                    offered: internality.into()
                }
            );
        }

        let duplicate_role = admit_binding_definition(
            BindingDefinitionOffer {
                contract: BINDING_DEFINITION_CONTRACT.into(),
                artifact: artifact(),
                declaration: 0,
                roles: vec![
                    RoleClassificationOffer {
                        role: 0,
                        internality: "internal".into(),
                    },
                    RoleClassificationOffer {
                        role: 0,
                        internality: "external".into(),
                    },
                ],
                slots: Vec::new(),
            },
            &expected_empty,
        )
        .expect_err("duplicate role handle");
        assert_eq!(
            duplicate_role,
            BindingOfferRefusal::DuplicateHandle {
                table: "roles",
                handle: 0
            }
        );

        let producer = producer_owner::admitted("duplicate-static-slot");
        let slot = SlotCompatibility::Subject {
            requirement: 0,
            authority: producer.key().clone(),
            producer_kind: SubjectKind::new(producer_owner::SUBJECT_KIND).expect("kind"),
            correspondence: None,
        };
        let duplicate_slot = admit_binding_definition(
            BindingDefinitionOffer {
                contract: BINDING_DEFINITION_CONTRACT.into(),
                artifact: artifact(),
                declaration: 0,
                roles: Vec::new(),
                slots: vec![slot.clone(), slot],
            },
            &expected_empty,
        )
        .expect_err("duplicate slot handle");
        assert_eq!(
            duplicate_slot,
            BindingOfferRefusal::DuplicateHandle {
                table: "slots",
                handle: 0
            }
        );

        let empty_identity_mutations: [fn(&mut BindingArtifactIdentity); 7] = [
            |value| value.identity.clear(),
            |value| value.revision.namespace.clear(),
            |value| value.revision.value.clear(),
            |value| value.digest.domain.clear(),
            |value| value.digest.version.clear(),
            |value| value.digest.algorithm.clear(),
            |value| value.digest.value.clear(),
        ];
        for mutate in empty_identity_mutations {
            let mut offered = artifact();
            mutate(&mut offered);
            let refusal = admit_binding_definition(
                BindingDefinitionOffer {
                    contract: BINDING_DEFINITION_CONTRACT.into(),
                    artifact: offered,
                    declaration: 0,
                    roles: Vec::new(),
                    slots: Vec::new(),
                },
                &expected_empty,
            )
            .expect_err("every empty artifact-identity component refuses");
            assert_eq!(refusal, BindingOfferRefusal::EmptyIdentity);
        }

        let refusal = admit_binding_definition(
            BindingDefinitionOffer {
                contract: BINDING_DEFINITION_CONTRACT.into(),
                artifact: artifact(),
                declaration: 1,
                roles: Vec::new(),
                slots: Vec::new(),
            },
            &expected_empty,
        )
        .expect_err("the offer cannot retarget the independent selection");
        assert_eq!(refusal, BindingOfferRefusal::SelectionMismatch);
    }

    #[trace("TC-002", "FR-002-AC-3", "FR-002-AC-4")]
    #[test]
    fn tc_002_static_slot_kind_range_correspondence_and_role_set_are_guarded() {
        let mut declaration = declaration();
        declaration.bindings[0].model = w::Nullable(Some(w::ExportRef {
            model: 0,
            export: 0,
        }));
        let selected_correspondence = correspondence();
        let models = vec![w::Model {
            artifact: 0,
            profile: "test:model".into(),
            exports: Vec::new(),
            correspondence: w::Nullable(Some(selected_correspondence.clone())),
        }];
        let producer = producer_owner::admitted("static-slot-validation");
        let kind = SubjectKind::new(producer_owner::SUBJECT_KIND).expect("kind");
        let slot = SlotCompatibility::Subject {
            requirement: 0,
            authority: producer.key().clone(),
            producer_kind: kind.clone(),
            correspondence: Some(selected_correspondence),
        };
        let exact = accepted_selection(Vec::new(), vec![slot]);
        validate_static_definition(&declaration, &models, &exact).expect("exact static slot");

        let incompatible = accepted_selection(
            Vec::new(),
            vec![SlotCompatibility::Subject {
                requirement: 0,
                authority: producer.key().clone(),
                producer_kind: kind.clone(),
                correspondence: None,
            }],
        );
        let refusal = validate_static_definition(&declaration, &models, &incompatible)
            .expect_err("correspondence mismatch");
        assert!(matches!(
            refusal.first(),
            BindingRefusal::IncompatibleStaticSlot { requirement: 0, .. }
        ));

        let out_of_range = accepted_selection(
            Vec::new(),
            vec![SlotCompatibility::Subject {
                requirement: 9,
                authority: producer.key().clone(),
                producer_kind: kind.clone(),
                correspondence: None,
            }],
        );
        let refusal = validate_static_definition(&declaration, &models, &out_of_range)
            .expect_err("out-of-range slot");
        assert!(matches!(
            refusal.first(),
            BindingRefusal::MissingStaticSlot { requirement: 0, .. }
        ));
        assert!(refusal
            .iter()
            .any(|finding| finding == &BindingRefusal::NoSuchStaticSlot { requirement: 9 }));

        let mut wrong_kind = declaration.clone();
        wrong_kind.bindings[0].kind = w::BindingKind::Relationship;
        let refusal = validate_static_definition(&wrong_kind, &models, &exact)
            .expect_err("subject selection for relationship slot");
        assert!(matches!(
            refusal.first(),
            BindingRefusal::WrongStaticSlotKind {
                requirement: 0,
                expected_relationship: false,
                found: w::BindingKind::Relationship,
                ..
            }
        ));

        let mut two_slot_declaration = declaration.clone();
        let mut second_requirement = two_slot_declaration.bindings[0].clone();
        second_requirement.name = "second-subject-slot".into();
        second_requirement.locus.span = w::Span { start: 30, end: 40 };
        two_slot_declaration.bindings.push(second_requirement);
        let reordered_slots = accepted_selection(
            Vec::new(),
            vec![
                SlotCompatibility::Subject {
                    requirement: 1,
                    authority: producer.key().clone(),
                    producer_kind: kind.clone(),
                    correspondence: Some(correspondence()),
                },
                SlotCompatibility::Subject {
                    requirement: 0,
                    authority: producer.key().clone(),
                    producer_kind: kind.clone(),
                    correspondence: Some(correspondence()),
                },
            ],
        );
        let refusal = validate_static_definition(&two_slot_declaration, &models, &reordered_slots)
            .expect_err("selected slot facts must retain binding-table order");
        assert_eq!(refusal.count(), 2);
        assert!(matches!(
            refusal.first(),
            BindingRefusal::MisorderedStaticSlot {
                ordinal: 0,
                requirement: 1,
                expected_requirement: 0,
                ..
            }
        ));

        let mut role_declaration = declaration;
        let w::Body::Protocol { roles, .. } = &mut role_declaration.body else {
            panic!("protocol body")
        };
        roles.push(w::Role {
            name: "role".into(),
            model: w::ExportRef {
                model: 0,
                export: 0,
            },
            instance: 0,
            locus: locus(),
        });
        let refusal = validate_static_definition(&role_declaration, &models, &exact)
            .expect_err("missing role classification");
        assert_eq!(
            refusal.first(),
            &BindingRefusal::MissingStaticRole {
                role: 0,
                locus: locus()
            }
        );

        let surplus = accepted_selection(
            vec![
                RoleClassification {
                    role: 0,
                    internality: RoleInternality::Internal,
                },
                RoleClassification {
                    role: 9,
                    internality: RoleInternality::External,
                },
            ],
            exact.slots().to_vec(),
        );
        let refusal = validate_static_definition(&role_declaration, &models, &surplus)
            .expect_err("surplus role classification");
        assert_eq!(
            refusal.first(),
            &BindingRefusal::SurplusStaticRole {
                ordinal: 1,
                role: 9
            }
        );

        let w::Body::Protocol { roles, .. } = &mut role_declaration.body else {
            unreachable!("protocol body")
        };
        roles.push(w::Role {
            name: "second-role".into(),
            model: w::ExportRef {
                model: 0,
                export: 0,
            },
            instance: 0,
            locus: w::Locus {
                source: 0,
                span: w::Span { start: 30, end: 40 },
            },
        });
        let reordered = accepted_selection(
            vec![
                RoleClassification {
                    role: 1,
                    internality: RoleInternality::External,
                },
                RoleClassification {
                    role: 0,
                    internality: RoleInternality::Internal,
                },
            ],
            exact.slots().to_vec(),
        );
        let refusal = validate_static_definition(&role_declaration, &models, &reordered)
            .expect_err("reordered role classifications");
        assert_eq!(refusal.count(), 2);
        assert_eq!(
            refusal.first(),
            &BindingRefusal::MisorderedStaticRole {
                ordinal: 0,
                role: 1,
                expected_role: 0,
                locus: w::Locus {
                    source: 0,
                    span: w::Span { start: 30, end: 40 }
                }
            }
        );

        let relationship_declaration = relationship_declaration(1);
        let relationship_owner = producer_owner::admitted("static-relationship-owner");
        let foreign_owner = producer_owner::admitted("static-relationship-foreign");
        let (source, target) = relationship_subjects(&relationship_owner, "static-authority");
        let required = quire_observation::RequiredRelationship::new(
            &relationship_owner,
            producer_owner::RELATIONSHIP,
            source,
            target,
        )
        .expect("required relationship");
        let incompatible_relationship = accepted_selection(
            Vec::new(),
            vec![SlotCompatibility::Relationship {
                requirement: 0,
                authority: foreign_owner.key().clone(),
                required: Box::new(required),
                correspondence: None,
            }],
        );
        let refusal =
            validate_static_definition(&relationship_declaration, &[], &incompatible_relationship)
                .expect_err("relationship premise and selected authority must agree");
        assert!(matches!(
            refusal.first(),
            BindingRefusal::IncompatibleStaticRelationshipAuthority { requirement: 0, .. }
        ));
    }

    #[trace("TC-002", "FR-002-AC-1", "FR-002-AC-3")]
    #[test]
    fn tc_002_runtime_subject_uses_exact_authority_kind_and_identity() {
        let declaration = declaration();
        let producer = producer_owner::admitted("binding-valid");
        let kind = SubjectKind::new(producer_owner::SUBJECT_KIND).expect("kind");
        let binding = accepted(producer.key().clone(), kind.clone());
        validate_static_definition(&declaration, &[], &binding).expect("static definition");
        let scope = super::super::declaration_identity(0, &declaration);
        let runtime = RuntimeBindingSet {
            contract: RUNTIME_BINDING_CONTRACT.into(),
            target: artifact(),
            subjects: vec![SubjectBinding {
                requirement: 0,
                subject: subject(&producer, kind, b"runtime:participant".to_vec()),
            }],
            relationships: Vec::new(),
        };
        let linked = validate_runtime(&declaration, &scope, &binding, runtime)
            .expect("exact runtime binding");
        assert_eq!(linked.subjects()[0].requirement(), 0);
        assert_eq!(
            linked.subjects()[0].subject().identity().as_bytes(),
            b"runtime:participant"
        );
    }

    #[trace("TC-002", "FR-002-AC-3")]
    #[test]
    fn tc_002_runtime_subject_refusals_are_independent_and_located() {
        let declaration = declaration();
        let producer = producer_owner::admitted("binding-selected");
        let foreign = producer_owner::admitted("binding-foreign");
        let selected_kind = SubjectKind::new(producer_owner::SUBJECT_KIND).expect("kind");
        let binding = accepted(producer.key().clone(), selected_kind);
        let scope = super::super::declaration_identity(0, &declaration);
        let slot_identity =
            super::super::requirement_identity(&scope, 0, 0, &declaration.bindings[0]);
        let runtime = RuntimeBindingSet {
            contract: RUNTIME_BINDING_CONTRACT.into(),
            target: artifact(),
            subjects: vec![SubjectBinding {
                requirement: 0,
                subject: subject(
                    &foreign,
                    SubjectKind::new(b"wrong-kind".to_vec()).expect("kind"),
                    slot_identity,
                ),
            }],
            relationships: Vec::new(),
        };
        let refused = validate_runtime(&declaration, &scope, &binding, runtime)
            .expect_err("foreign, wrong-kind, slot-derived subject");
        assert_eq!(refused.count(), 3);
        assert!(matches!(
            refused.iter().next(),
            Some(BindingRefusal::ForeignRuntimeAuthority {
                requirement: 0,
                locus,
                ..
            }) if locus == &declaration.bindings[0].locus
        ));
        assert!(refused
            .iter()
            .any(|finding| matches!(finding, BindingRefusal::WrongRuntimeSubjectKind { .. })));
        assert!(refused
            .iter()
            .any(|finding| matches!(finding, BindingRefusal::SlotDerivedRuntimeIdentity { .. })));
    }

    #[trace("TC-002", "FR-002-AC-3")]
    #[test]
    fn tc_002_missing_duplicate_and_surplus_runtime_slots_refuse() {
        let declaration = declaration();
        let producer = producer_owner::admitted("binding-cardinality");
        let kind = SubjectKind::new(producer_owner::SUBJECT_KIND).expect("kind");
        let binding = accepted(producer.key().clone(), kind.clone());
        let scope = super::super::declaration_identity(0, &declaration);

        let missing = validate_runtime(
            &declaration,
            &scope,
            &binding,
            RuntimeBindingSet {
                contract: RUNTIME_BINDING_CONTRACT.into(),
                target: artifact(),
                subjects: Vec::new(),
                relationships: Vec::new(),
            },
        )
        .expect_err("selected slot is required");
        assert!(matches!(
            missing.first(),
            BindingRefusal::MissingRuntimeBinding { requirement: 0, .. }
        ));

        let exact = SubjectBinding {
            requirement: 0,
            subject: subject(&producer, kind.clone(), b"runtime:one".to_vec()),
        };
        let duplicate = validate_runtime(
            &declaration,
            &scope,
            &binding,
            RuntimeBindingSet {
                contract: RUNTIME_BINDING_CONTRACT.into(),
                target: artifact(),
                subjects: vec![exact.clone(), exact],
                relationships: Vec::new(),
            },
        )
        .expect_err("duplicate slot");
        assert!(matches!(
            duplicate.first(),
            BindingRefusal::DuplicateRuntimeBinding { requirement: 0, .. }
        ));

        let surplus = validate_runtime(
            &declaration,
            &scope,
            &binding,
            RuntimeBindingSet {
                contract: RUNTIME_BINDING_CONTRACT.into(),
                target: artifact(),
                subjects: vec![SubjectBinding {
                    requirement: 9,
                    subject: subject(&producer, kind, b"runtime:surplus".to_vec()),
                }],
                relationships: Vec::new(),
            },
        )
        .expect_err("unselected slot");
        assert!(matches!(
            surplus.first(),
            BindingRefusal::SurplusRuntimeBinding {
                ordinal: 0,
                requirement: 9
            }
        ));
        assert!(surplus
            .iter()
            .any(|finding| matches!(finding, BindingRefusal::MissingRuntimeBinding { .. })));
    }

    #[trace("TC-002", "FR-002-AC-3")]
    #[test]
    fn tc_002_runtime_contract_target_and_entry_alternative_are_exact() {
        let declaration = declaration();
        let producer = producer_owner::admitted("runtime-contract");
        let kind = SubjectKind::new(producer_owner::SUBJECT_KIND).expect("kind");
        let binding = accepted(producer.key().clone(), kind.clone());
        let scope = super::super::declaration_identity(0, &declaration);

        for contract in ["", "quire.protocol.runtime-binding/0", "unknown/1"] {
            let refusal = validate_runtime(
                &declaration,
                &scope,
                &binding,
                RuntimeBindingSet {
                    contract: contract.into(),
                    target: artifact(),
                    subjects: Vec::new(),
                    relationships: Vec::new(),
                },
            )
            .expect_err("absent, old, and unknown runtime contracts refuse");
            assert_eq!(
                refusal.first(),
                &BindingRefusal::UnsupportedRuntimeContract {
                    offered: contract.into()
                }
            );
            assert_eq!(refusal.count(), 1, "unsupported input is not inspected");
        }

        let mut target = artifact();
        target.identity.push_str(":other");
        let refusal = validate_runtime(
            &declaration,
            &scope,
            &binding,
            RuntimeBindingSet {
                contract: RUNTIME_BINDING_CONTRACT.into(),
                target,
                subjects: Vec::new(),
                relationships: Vec::new(),
            },
        )
        .expect_err("another static binding target");
        assert_eq!(refusal.first(), &BindingRefusal::WrongRuntimeTarget);
        assert_eq!(refusal.count(), 1, "wrong-target input is not inspected");

        let relationship_declaration = relationship_declaration(1);
        let (source, target) = relationship_subjects(&producer, "wrong-entry-kind");
        let required = quire_observation::RequiredRelationship::new(
            &producer,
            producer_owner::RELATIONSHIP,
            source,
            target,
        )
        .expect("required relationship");
        let relationship_binding = accepted_relationships(&producer, vec![required]);
        let relationship_scope = super::super::declaration_identity(0, &relationship_declaration);
        let refusal = validate_runtime(
            &relationship_declaration,
            &relationship_scope,
            &relationship_binding,
            RuntimeBindingSet {
                contract: RUNTIME_BINDING_CONTRACT.into(),
                target: artifact(),
                subjects: vec![SubjectBinding {
                    requirement: 0,
                    subject: subject(&producer, kind, b"runtime:not-relationship".to_vec()),
                }],
                relationships: Vec::new(),
            },
        )
        .expect_err("subject cannot fill relationship slot");
        assert!(matches!(
            refusal.first(),
            BindingRefusal::WrongRuntimeEntryKind {
                ordinal: 0,
                requirement: 0,
                ..
            }
        ));

        let (source, target) = relationship_subjects(&producer, "relationship-to-subject");
        let relationship = Relationship::new(
            &producer,
            producer_owner::RELATIONSHIP,
            RelationshipIdentity::new(b"relationship:not-subject".to_vec()).expect("identity"),
            source,
            target,
        )
        .expect("runtime relationship");
        let refusal = validate_runtime(
            &declaration,
            &scope,
            &binding,
            RuntimeBindingSet {
                contract: RUNTIME_BINDING_CONTRACT.into(),
                target: artifact(),
                subjects: Vec::new(),
                relationships: vec![RelationshipBinding {
                    requirement: 0,
                    relationship,
                }],
            },
        )
        .expect_err("relationship cannot fill a subject slot");
        assert!(matches!(
            refusal.first(),
            BindingRefusal::MissingRuntimeBinding { requirement: 0, .. }
        ));
        assert!(refusal.iter().any(|finding| matches!(
            finding,
            BindingRefusal::WrongRuntimeEntryKind {
                ordinal: 0,
                requirement: 0,
                ..
            }
        )));
    }

    #[trace("TC-002", "FR-002-AC-1", "FR-002-AC-3")]
    #[test]
    fn tc_002_occurrence_identities_refuse_reuse_but_participants_may_share() {
        let producer = producer_owner::admitted("runtime-key-reuse");
        let kind = SubjectKind::new(producer_owner::SUBJECT_KIND).expect("kind");
        let slots = |kind: &SubjectKind| {
            (0..2)
                .map(|requirement| SlotCompatibility::Subject {
                    requirement,
                    authority: producer.key().clone(),
                    producer_kind: kind.clone(),
                    correspondence: None,
                })
                .collect()
        };
        let shared = subject(&producer, kind.clone(), b"runtime:shared".to_vec());
        let runtime = || RuntimeBindingSet {
            contract: RUNTIME_BINDING_CONTRACT.into(),
            target: artifact(),
            subjects: vec![
                SubjectBinding {
                    requirement: 0,
                    subject: shared.clone(),
                },
                SubjectBinding {
                    requirement: 1,
                    subject: shared.clone(),
                },
            ],
            relationships: Vec::new(),
        };

        for binding_kind in [
            w::BindingKind::WorkflowInstance,
            w::BindingKind::RoleInstance,
            w::BindingKind::Message,
            w::BindingKind::Send,
            w::BindingKind::Receive,
            w::BindingKind::Delivery,
            w::BindingKind::Attempt,
            w::BindingKind::Effect,
        ] {
            let declaration = subject_declaration(binding_kind, 2);
            let binding = accepted_selection(Vec::new(), slots(&kind));
            let scope = super::super::declaration_identity(0, &declaration);
            let refusal = validate_runtime(&declaration, &scope, &binding, runtime())
                .expect_err("one occurrence key cannot fill two distinct slots");
            assert!(matches!(
                refusal.first(),
                BindingRefusal::ContradictoryRuntimeIdentity {
                    ordinal: 1,
                    requirement: 1,
                    previous_requirement: 0,
                    ..
                }
            ));
        }

        let participants = subject_declaration(w::BindingKind::Participant, 2);
        let participant_binding = accepted_selection(Vec::new(), slots(&kind));
        let participant_scope = super::super::declaration_identity(0, &participants);
        let linked = validate_runtime(
            &participants,
            &participant_scope,
            &participant_binding,
            runtime(),
        )
        .expect("one participant may fill two compatible slots");
        assert_eq!(linked.subjects().len(), 2);
        assert_eq!(
            linked.subjects()[0].subject(),
            linked.subjects()[1].subject()
        );

        let mut crossed_kinds = subject_declaration(w::BindingKind::Participant, 2);
        crossed_kinds.bindings[1].kind = w::BindingKind::Component;
        let crossed_binding = accepted_selection(Vec::new(), slots(&kind));
        let crossed_scope = super::super::declaration_identity(0, &crossed_kinds);
        let refusal = validate_runtime(&crossed_kinds, &crossed_scope, &crossed_binding, runtime())
            .expect_err("one key cannot cross two QSL binding kinds");
        assert!(matches!(
            refusal.first(),
            BindingRefusal::ContradictoryRuntimeIdentity {
                ordinal: 1,
                requirement: 1,
                previous_requirement: 0,
                ..
            }
        ));

        let incompatible_participant_binding = accepted_selection(
            Vec::new(),
            vec![
                SlotCompatibility::Subject {
                    requirement: 0,
                    authority: producer.key().clone(),
                    producer_kind: kind.clone(),
                    correspondence: None,
                },
                SlotCompatibility::Subject {
                    requirement: 1,
                    authority: producer.key().clone(),
                    producer_kind: kind.clone(),
                    correspondence: Some(correspondence()),
                },
            ],
        );
        let refusal = validate_runtime(
            &participants,
            &participant_scope,
            &incompatible_participant_binding,
            runtime(),
        )
        .expect_err("one participant cannot cross incompatible selected correspondences");
        assert!(matches!(
            refusal.first(),
            BindingRefusal::ContradictoryRuntimeIdentity {
                ordinal: 1,
                requirement: 1,
                previous_requirement: 0,
                ..
            }
        ));
    }

    #[trace("TC-002", "FR-002-AC-1")]
    #[test]
    fn tc_002_required_runtime_subject_families_retain_distinct_exact_keys() {
        let producer = producer_owner::admitted("runtime-distinct-families");
        let producer_kind = SubjectKind::new(producer_owner::SUBJECT_KIND).expect("kind");

        for (family, binding_kind) in [
            w::BindingKind::WorkflowInstance,
            w::BindingKind::RoleInstance,
            w::BindingKind::Participant,
            w::BindingKind::Component,
            w::BindingKind::Endpoint,
            w::BindingKind::Message,
            w::BindingKind::Send,
            w::BindingKind::Receive,
            w::BindingKind::Delivery,
            w::BindingKind::Attempt,
            w::BindingKind::Effect,
        ]
        .into_iter()
        .enumerate()
        {
            let declaration = subject_declaration(binding_kind, 2);
            let binding = accepted_selection(
                Vec::new(),
                (0..2)
                    .map(|requirement| SlotCompatibility::Subject {
                        requirement,
                        authority: producer.key().clone(),
                        producer_kind: producer_kind.clone(),
                        correspondence: None,
                    })
                    .collect(),
            );
            let scope = super::super::declaration_identity(0, &declaration);
            let first = subject(
                &producer,
                producer_kind.clone(),
                format!("runtime:family:{family}:first").into_bytes(),
            );
            let second = subject(
                &producer,
                producer_kind.clone(),
                format!("runtime:family:{family}:second").into_bytes(),
            );
            let linked = validate_runtime(
                &declaration,
                &scope,
                &binding,
                RuntimeBindingSet {
                    contract: RUNTIME_BINDING_CONTRACT.into(),
                    target: artifact(),
                    subjects: vec![
                        SubjectBinding {
                            requirement: 0,
                            subject: first.clone(),
                        },
                        SubjectBinding {
                            requirement: 1,
                            subject: second.clone(),
                        },
                    ],
                    relationships: Vec::new(),
                },
            )
            .expect("distinct exact keys remain distinct");

            assert_eq!(linked.subjects().len(), 2);
            assert_eq!(linked.subjects()[0].subject(), &first);
            assert_eq!(linked.subjects()[1].subject(), &second);
        }
    }

    #[trace("TC-002", "FR-002-AC-3")]
    #[test]
    fn tc_002_relationship_missing_surplus_and_foreign_authority_refuse() {
        let declaration = relationship_declaration(1);
        let producer = producer_owner::admitted("relationship-selected-authority");
        let foreign = producer_owner::admitted("relationship-foreign-authority");
        let (source, target) = relationship_subjects(&producer, "selected-authority");
        let required = quire_observation::RequiredRelationship::new(
            &producer,
            producer_owner::RELATIONSHIP,
            source,
            target,
        )
        .expect("required relationship");
        let binding = accepted_relationships(&producer, vec![required]);
        let scope = super::super::declaration_identity(0, &declaration);

        let missing = validate_runtime(
            &declaration,
            &scope,
            &binding,
            RuntimeBindingSet {
                contract: RUNTIME_BINDING_CONTRACT.into(),
                target: artifact(),
                subjects: Vec::new(),
                relationships: Vec::new(),
            },
        )
        .expect_err("missing relationship");
        assert!(matches!(
            missing.first(),
            BindingRefusal::MissingRuntimeBinding { requirement: 0, .. }
        ));

        let (foreign_source, foreign_target) = relationship_subjects(&foreign, "foreign");
        let foreign_relationship = quire_observation::Relationship::new(
            &foreign,
            producer_owner::RELATIONSHIP,
            RelationshipIdentity::new(b"relationship:foreign".to_vec()).expect("identity"),
            foreign_source,
            foreign_target,
        )
        .expect("foreign relationship");
        let refused = validate_runtime(
            &declaration,
            &scope,
            &binding,
            RuntimeBindingSet {
                contract: RUNTIME_BINDING_CONTRACT.into(),
                target: artifact(),
                subjects: Vec::new(),
                relationships: vec![
                    RelationshipBinding {
                        requirement: 0,
                        relationship: foreign_relationship.clone(),
                    },
                    RelationshipBinding {
                        requirement: 9,
                        relationship: foreign_relationship,
                    },
                ],
            },
        )
        .expect_err("foreign and surplus relationships");
        assert!(matches!(
            refused.first(),
            BindingRefusal::ForeignRuntimeAuthority {
                ordinal: 0,
                requirement: 0,
                ..
            }
        ));
        assert!(refused.iter().any(|finding| matches!(
            finding,
            BindingRefusal::SurplusRuntimeBinding {
                ordinal: 1,
                requirement: 9
            }
        )));
    }

    #[trace("TC-002", "FR-002-AC-3", "FR-002-AC-4")]
    #[test]
    fn tc_002_exact_relationship_replay_is_one_idempotent_value() {
        let declaration = relationship_declaration(1);
        let producer = producer_owner::admitted("relationship-replay");
        let (source, target) = relationship_subjects(&producer, "replay");
        let required = quire_observation::RequiredRelationship::new(
            &producer,
            producer_owner::RELATIONSHIP,
            source.clone(),
            target.clone(),
        )
        .expect("required relationship");
        let binding = accepted_relationships(&producer, vec![required]);
        let relationship = quire_observation::Relationship::new(
            &producer,
            producer_owner::RELATIONSHIP,
            RelationshipIdentity::new(b"relationship:one".to_vec()).expect("identity"),
            source,
            target,
        )
        .expect("runtime relationship");
        let scope = super::super::declaration_identity(0, &declaration);
        let linked = validate_runtime(
            &declaration,
            &scope,
            &binding,
            RuntimeBindingSet {
                contract: RUNTIME_BINDING_CONTRACT.into(),
                target: artifact(),
                subjects: Vec::new(),
                relationships: vec![
                    RelationshipBinding {
                        requirement: 0,
                        relationship: relationship.clone(),
                    },
                    RelationshipBinding {
                        requirement: 0,
                        relationship,
                    },
                ],
            },
        )
        .expect("exact replay is a no-op");
        assert_eq!(linked.relationships().len(), 1);
    }

    #[trace("TC-002", "FR-002-AC-3")]
    #[test]
    fn tc_002_relationship_conflict_and_ambiguity_refuse() {
        let declaration = relationship_declaration(1);
        let producer = producer_owner::admitted("relationship-conflict");
        let (source, target) = relationship_subjects(&producer, "selected");
        let (_, other_target) = relationship_subjects(&producer, "other");
        let required = quire_observation::RequiredRelationship::new(
            &producer,
            producer_owner::RELATIONSHIP,
            source.clone(),
            target.clone(),
        )
        .expect("required relationship");
        let binding = accepted_relationships(&producer, vec![required]);
        let relationship = |identity: &[u8], target: QualifiedSubject| {
            quire_observation::Relationship::new(
                &producer,
                producer_owner::RELATIONSHIP,
                RelationshipIdentity::new(identity.to_vec()).expect("identity"),
                source.clone(),
                target,
            )
            .expect("runtime relationship")
        };
        let scope = super::super::declaration_identity(0, &declaration);
        let conflict = validate_runtime(
            &declaration,
            &scope,
            &binding,
            RuntimeBindingSet {
                contract: RUNTIME_BINDING_CONTRACT.into(),
                target: artifact(),
                subjects: Vec::new(),
                relationships: vec![
                    RelationshipBinding {
                        requirement: 0,
                        relationship: relationship(b"same-key", target.clone()),
                    },
                    RelationshipBinding {
                        requirement: 0,
                        relationship: relationship(b"same-key", other_target),
                    },
                ],
            },
        )
        .expect_err("one key with different facts");
        assert!(conflict
            .iter()
            .any(|finding| matches!(finding, BindingRefusal::ContradictoryRelationship { .. })));

        let ambiguous = validate_runtime(
            &declaration,
            &scope,
            &binding,
            RuntimeBindingSet {
                contract: RUNTIME_BINDING_CONTRACT.into(),
                target: artifact(),
                subjects: Vec::new(),
                relationships: vec![
                    RelationshipBinding {
                        requirement: 0,
                        relationship: relationship(b"key-a", target.clone()),
                    },
                    RelationshipBinding {
                        requirement: 0,
                        relationship: relationship(b"key-b", target),
                    },
                ],
            },
        )
        .expect_err("two keys satisfy one slot");
        assert!(matches!(
            ambiguous.first(),
            BindingRefusal::AmbiguousRelationship { requirement: 0, .. }
        ));
    }

    #[trace("TC-002", "FR-002-AC-1", "FR-002-AC-3")]
    #[test]
    fn tc_002_two_relationship_slots_retain_distinct_values() {
        let declaration = relationship_declaration(2);
        let producer = producer_owner::admitted("relationship-distinct");
        let (source_a, target_a) = relationship_subjects(&producer, "a");
        let (source_b, target_b) = relationship_subjects(&producer, "b");
        let required_a = quire_observation::RequiredRelationship::new(
            &producer,
            producer_owner::RELATIONSHIP,
            source_a.clone(),
            target_a.clone(),
        )
        .expect("required relationship a");
        let required_b = quire_observation::RequiredRelationship::new(
            &producer,
            producer_owner::RELATIONSHIP,
            source_b.clone(),
            target_b.clone(),
        )
        .expect("required relationship b");
        let binding = accepted_relationships(&producer, vec![required_a, required_b]);
        let actual = |identity: &[u8], source, target| {
            quire_observation::Relationship::new(
                &producer,
                producer_owner::RELATIONSHIP,
                RelationshipIdentity::new(identity.to_vec()).expect("identity"),
                source,
                target,
            )
            .expect("runtime relationship")
        };
        let scope = super::super::declaration_identity(0, &declaration);
        let linked = validate_runtime(
            &declaration,
            &scope,
            &binding,
            RuntimeBindingSet {
                contract: RUNTIME_BINDING_CONTRACT.into(),
                target: artifact(),
                subjects: Vec::new(),
                relationships: vec![
                    RelationshipBinding {
                        requirement: 0,
                        relationship: actual(b"relationship:a", source_a, target_a),
                    },
                    RelationshipBinding {
                        requirement: 1,
                        relationship: actual(b"relationship:b", source_b, target_b),
                    },
                ],
            },
        )
        .expect("two exact slots");
        assert_eq!(linked.relationships().len(), 2);
        assert_ne!(
            linked.relationships()[0].relationship().identity(),
            linked.relationships()[1].relationship().identity()
        );
    }
}
