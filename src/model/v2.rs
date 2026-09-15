// SPDX-License-Identifier: AGPL-3.0-or-later
//! Constructor-validated binding capabilities for `quire.compiled-protocol/2`.
//!
//! A static binding definition selects the consumer-owned role classifications
//! and slot compatibility facts. A runtime binding set then associates those
//! selected slots with QObs-owned FR-287 subjects or FR-262 relationships. The
//! split is intentional: no static coordinate becomes a runtime identity, and
//! concrete runtime values do not alter the static definition selection.

use std::collections::BTreeSet;

use quire_observation::{
    AdmittedBundleKey, AdmittedStaticBundle, QualifiedSubject, Relationship, SubjectKind,
};
use quire_spec_language::protocol_artifact::{
    v2 as artifact_v2,
    wire::{ArtifactRef, BindingKind, Body, Requirement},
};

/// The only static binding-definition contract accepted by this model.
pub const STATIC_BINDING_CONTRACT: &str = "quire.protocol.binding-definition/1";
/// The only runtime-binding contract accepted by this model.
pub const RUNTIME_BINDING_CONTRACT: &str = "quire.protocol.runtime-binding/1";

/// Exact selected static binding-definition artifact identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StaticSelection {
    /// Immutable selected artifact identity.
    pub identity: Box<str>,
    /// Exact selected artifact revision.
    pub revision: Box<str>,
    /// Exact raw-byte digest of the selected artifact.
    pub digest: [u8; 32],
}

/// The role classification selected by FR-217.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Internality {
    /// The role is internal to the assessed protocol.
    Internal,
    /// The role is external to the assessed protocol.
    External,
}

/// One exact static role classification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleClassification {
    /// Exact source-qualified static role handle.
    pub role_handle: u32,
    /// Selected FR-217 classification.
    pub internality: Internality,
}

/// Exact static binding-requirement coordinate.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SlotHandle(u32);

impl SlotHandle {
    /// Constructs one selected binding-requirement handle.
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the exact source-qualified coordinate.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// Exact declaration-local static channel coordinate.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ChannelHandle(u32);

impl ChannelHandle {
    /// Constructs one selected channel coordinate.
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the exact source-qualified channel coordinate.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// Consumer-selected compatibility for one imported QObs runtime value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SlotCompatibility {
    /// This slot accepts one exact producer-local subject kind.
    Subject { kind: SubjectKind },
    /// This slot accepts one exact FCD relationship declaration and its
    /// ordered producer-local endpoint kinds.
    Relationship {
        /// Exact FCD relationship declaration identity.
        declaration_identity: Box<str>,
        /// Exact producer-local kind required at the authored source endpoint.
        source_kind: SubjectKind,
        /// Exact producer-local kind required at the authored target endpoint.
        target_kind: SubjectKind,
    },
}

/// One static slot and its independently selected compatibility fact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StaticSlot {
    /// Exact binding-requirement coordinate.
    pub handle: SlotHandle,
    /// No producer spelling is inferred beyond this explicit fact.
    pub compatibility: SlotCompatibility,
}

/// One accepted static binding definition for an exact version-2 declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StaticBindingDefinition {
    selection: StaticSelection,
    producer: AdmittedBundleKey,
    declaration_handle: u32,
    roles: Vec<RoleClassification>,
    slots: Vec<StaticSlot>,
}

/// Why a selected static definition cannot be used.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StaticRefusal {
    /// The selected contract was absent, older, or unknown.
    UnsupportedContract,
    /// An identity or revision was empty.
    EmptySelection,
    /// A role classification was selected more than once.
    DuplicateRole,
    /// A static slot was selected more than once.
    DuplicateSlot,
    /// No runtime slot was selected.
    EmptySlots,
    /// A relationship compatibility omitted its declaration identity.
    EmptyRelationshipDeclaration,
}

impl StaticBindingDefinition {
    /// Validates and retains one exact static binding selection.
    pub fn new(
        contract: &str,
        selection: StaticSelection,
        producer: AdmittedBundleKey,
        declaration_handle: u32,
        roles: Vec<RoleClassification>,
        slots: Vec<StaticSlot>,
    ) -> Result<Self, StaticRefusal> {
        if contract != STATIC_BINDING_CONTRACT {
            return Err(StaticRefusal::UnsupportedContract);
        }
        if selection.identity.is_empty() || selection.revision.is_empty() {
            return Err(StaticRefusal::EmptySelection);
        }
        let mut role_handles = BTreeSet::new();
        for role in &roles {
            if !role_handles.insert(role.role_handle) {
                return Err(StaticRefusal::DuplicateRole);
            }
        }
        if slots.is_empty() {
            return Err(StaticRefusal::EmptySlots);
        }
        let mut slot_handles = BTreeSet::new();
        for slot in &slots {
            if !slot_handles.insert(slot.handle) {
                return Err(StaticRefusal::DuplicateSlot);
            }
            if matches!(
                &slot.compatibility,
                SlotCompatibility::Relationship {
                    declaration_identity,
                    ..
                } if declaration_identity.is_empty()
            ) {
                return Err(StaticRefusal::EmptyRelationshipDeclaration);
            }
        }
        Ok(Self {
            selection,
            producer,
            declaration_handle,
            roles,
            slots,
        })
    }

    /// Returns the exact selected static artifact.
    #[must_use]
    pub const fn selection(&self) -> &StaticSelection {
        &self.selection
    }

    /// Returns the exact selected QObs producer authority.
    #[must_use]
    pub const fn producer(&self) -> &AdmittedBundleKey {
        &self.producer
    }

    /// Returns the exact selected version-2 declaration coordinate.
    #[must_use]
    pub const fn declaration_handle(&self) -> u32 {
        self.declaration_handle
    }

    /// Returns each static role classification in selected order.
    #[must_use]
    pub fn roles(&self) -> &[RoleClassification] {
        &self.roles
    }

    /// Returns every selected static slot in selected order.
    #[must_use]
    pub fn slots(&self) -> &[StaticSlot] {
        &self.slots
    }
}

/// One offered runtime binding for one selected static slot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeBinding {
    /// The static slot that this runtime value satisfies.
    slot: SlotHandle,
    /// Imported QObs owner value; Protocol never reconstructs it from a slot.
    value: RuntimeValue,
}

/// A QObs owner value usable at one static slot.
#[derive(Clone, Debug, Eq, PartialEq)]
enum RuntimeValue {
    /// One complete authority-qualified FR-287 subject.
    Subject(Box<QualifiedSubject>),
    /// One complete authority-qualified FR-262 relationship.
    Relationship(Box<Relationship>),
}

impl RuntimeBinding {
    /// Supplies one complete QObs subject for the selected static slot.
    #[must_use]
    pub fn subject(slot: SlotHandle, subject: QualifiedSubject) -> Self {
        Self {
            slot,
            value: RuntimeValue::Subject(Box::new(subject)),
        }
    }

    /// Supplies one complete QObs relationship for the selected static slot.
    #[must_use]
    pub fn relationship(slot: SlotHandle, relationship: Relationship) -> Self {
        Self {
            slot,
            value: RuntimeValue::Relationship(Box::new(relationship)),
        }
    }

    /// Returns the exact selected static slot coordinate.
    #[must_use]
    pub const fn slot(&self) -> SlotHandle {
        self.slot
    }

    /// Returns the imported QObs subject when this binding carries one.
    #[must_use]
    pub fn subject_value(&self) -> Option<&QualifiedSubject> {
        match &self.value {
            RuntimeValue::Subject(subject) => Some(subject),
            RuntimeValue::Relationship(_) => None,
        }
    }

    /// Returns the imported QObs relationship when this binding carries one.
    #[must_use]
    pub fn relationship_value(&self) -> Option<&Relationship> {
        match &self.value {
            RuntimeValue::Subject(_) => None,
            RuntimeValue::Relationship(relationship) => Some(relationship),
        }
    }
}

/// Why a runtime binding set cannot be admitted.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuntimeRefusal {
    /// The selected runtime-binding contract was absent, older, or unknown.
    UnsupportedContract,
    /// The runtime set targets a different static binding selection.
    ForeignStaticSelection,
    /// The supplied static bundle differs from the producer selected by the
    /// static binding definition.
    ForeignStaticProducer,
    /// One binding names no selected static slot.
    UnknownSlot(SlotHandle),
    /// More than one offered value names one static slot.
    DuplicateSlot(SlotHandle),
    /// A selected static slot has no offered runtime value.
    MissingSlot(SlotHandle),
    /// A QObs subject or relationship belongs to another producer authority.
    ForeignAuthority(SlotHandle),
    /// The offered QObs value is not the selected value family.
    WrongValueKind(SlotHandle),
    /// A subject's opaque producer-local kind differs from the selected fact.
    IncompatibleSubjectKind(SlotHandle),
    /// A relationship declaration differs from the selected fact.
    IncompatibleRelationship(SlotHandle),
    /// One complete FR-262 relationship identity was offered with different
    /// declaration or endpoint facts for one required static slot.
    RelationshipContradiction(SlotHandle),
    /// More than one distinct FR-262 relationship satisfies one required
    /// static slot.
    AmbiguousRelationship(SlotHandle),
}

/// Complete admitted runtime binding set for one static definition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeBindingSet {
    target: StaticSelection,
    bindings: Vec<RuntimeBinding>,
}

impl RuntimeBindingSet {
    /// Validates one exact runtime set against an accepted static definition.
    ///
    /// `target` is compared componentwise so a same-spelled external artifact
    /// cannot retarget a runtime set to another static selection.
    pub fn new(
        contract: &str,
        target: &StaticSelection,
        definition: &StaticBindingDefinition,
        producer: &AdmittedStaticBundle,
        bindings: Vec<RuntimeBinding>,
    ) -> Result<Self, RuntimeRefusal> {
        if contract != RUNTIME_BINDING_CONTRACT {
            return Err(RuntimeRefusal::UnsupportedContract);
        }
        if target != definition.selection() {
            return Err(RuntimeRefusal::ForeignStaticSelection);
        }
        if producer.key() != definition.producer() {
            return Err(RuntimeRefusal::ForeignStaticProducer);
        }
        let mut admitted = Vec::with_capacity(bindings.len());
        for binding in bindings {
            let Some(slot) = definition
                .slots()
                .iter()
                .find(|slot| slot.handle == binding.slot())
            else {
                return Err(RuntimeRefusal::UnknownSlot(binding.slot()));
            };
            validate_value(
                binding.slot(),
                &slot.compatibility,
                producer,
                &binding.value,
            )?;
            if let Some(previous) = admitted
                .iter()
                .find(|previous: &&RuntimeBinding| previous.slot() == binding.slot())
            {
                match (&previous.value, &binding.value) {
                    (RuntimeValue::Relationship(previous), RuntimeValue::Relationship(current))
                        if previous.authority() == current.authority()
                            && previous.identity() == current.identity() =>
                    {
                        if previous == current {
                            // Exact FR-262 replay for this required slot is
                            // idempotent; retain the first complete value.
                            continue;
                        }
                        return Err(RuntimeRefusal::RelationshipContradiction(binding.slot()));
                    }
                    (RuntimeValue::Relationship(_), RuntimeValue::Relationship(_)) => {
                        return Err(RuntimeRefusal::AmbiguousRelationship(binding.slot()));
                    }
                    _ => return Err(RuntimeRefusal::DuplicateSlot(binding.slot())),
                }
            }
            admitted.push(binding);
        }
        for slot in definition.slots() {
            if !admitted
                .iter()
                .any(|binding: &RuntimeBinding| binding.slot() == slot.handle)
            {
                return Err(RuntimeRefusal::MissingSlot(slot.handle));
            }
        }
        Ok(Self {
            target: target.clone(),
            bindings: admitted,
        })
    }

    /// Returns the exact static selection this set targets.
    #[must_use]
    pub const fn target(&self) -> &StaticSelection {
        &self.target
    }

    /// Returns each admitted binding in caller-selected order.
    #[must_use]
    pub fn bindings(&self) -> &[RuntimeBinding] {
        &self.bindings
    }
}

/// Why one version-2 full-assessment context cannot link.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextRefusal {
    /// The selected declaration coordinate is not in the admitted package.
    UnknownDeclaration,
    /// The selected declaration is not a protocol declaration.
    NotProtocol,
    /// The runtime set targets another static selection.
    RuntimeTargetMismatch,
    /// A selected role coordinate is outside the protocol role table.
    UnknownRole,
    /// A protocol role has no static FR-217 classification.
    MissingRoleClassification,
    /// A static slot is outside the declaration binding table.
    UnknownSlot,
    /// A relationship requirement was selected as a subject, or vice versa.
    WrongSlotFamily,
}

/// One non-retargetable, fully typed assessment input boundary.
///
/// The context borrows the admitted package and its two independently admitted
/// binding capabilities. None of its fields are mutable or replaceable, so a
/// later Engine cannot retarget the request while retaining incremental state.
#[derive(Clone, Copy, Debug)]
pub struct LinkedAssessment<'a> {
    package: &'a artifact_v2::AdmittedPackage,
    definition: &'a StaticBindingDefinition,
    runtime: &'a RuntimeBindingSet,
}

impl<'a> LinkedAssessment<'a> {
    /// Validates that exact static and runtime selections target one admitted
    /// version-2 protocol declaration.
    pub fn new(
        package: &'a artifact_v2::AdmittedPackage,
        definition: &'a StaticBindingDefinition,
        runtime: &'a RuntimeBindingSet,
    ) -> Result<Self, ContextRefusal> {
        if runtime.target() != definition.selection() {
            return Err(ContextRefusal::RuntimeTargetMismatch);
        }
        let declaration = package
            .inherited()
            .declarations
            .get(usize::try_from(definition.declaration_handle()).unwrap_or(usize::MAX))
            .ok_or(ContextRefusal::UnknownDeclaration)?;
        let Body::Protocol { roles, .. } = &declaration.body else {
            return Err(ContextRefusal::NotProtocol);
        };
        validate_roles(roles.len(), definition.roles())?;
        validate_slots(&declaration.bindings, definition.slots())?;
        Ok(Self {
            package,
            definition,
            runtime,
        })
    }

    /// Returns the immutable admitted version-2 package.
    #[must_use]
    pub const fn package(&self) -> &'a artifact_v2::AdmittedPackage {
        self.package
    }

    /// Returns the selected static binding definition.
    #[must_use]
    pub const fn definition(&self) -> &'a StaticBindingDefinition {
        self.definition
    }

    /// Returns the exact admitted runtime binding set.
    #[must_use]
    pub const fn runtime(&self) -> &'a RuntimeBindingSet {
        self.runtime
    }
}

fn validate_roles(
    role_count: usize,
    classifications: &[RoleClassification],
) -> Result<(), ContextRefusal> {
    let mut selected = BTreeSet::new();
    for role in classifications {
        let index = usize::try_from(role.role_handle).unwrap_or(usize::MAX);
        if index >= role_count {
            return Err(ContextRefusal::UnknownRole);
        }
        selected.insert(index);
    }
    if selected.len() != role_count {
        return Err(ContextRefusal::MissingRoleClassification);
    }
    Ok(())
}

fn validate_slots(
    requirements: &[quire_spec_language::protocol_artifact::wire::BindingRequirement],
    slots: &[StaticSlot],
) -> Result<(), ContextRefusal> {
    for slot in slots {
        let index = usize::try_from(slot.handle.get()).unwrap_or(usize::MAX);
        let Some(requirement) = requirements.get(index) else {
            return Err(ContextRefusal::UnknownSlot);
        };
        let relationship = requirement.kind == BindingKind::Relationship;
        if relationship != matches!(&slot.compatibility, SlotCompatibility::Relationship { .. }) {
            return Err(ContextRefusal::WrongSlotFamily);
        }
    }
    Ok(())
}

/// One exact owner-typed package-table member with its original coordinate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Indexed<T> {
    index: u32,
    value: T,
}

impl<T> Indexed<T> {
    /// Returns the original package-table coordinate.
    #[must_use]
    pub const fn index(&self) -> u32 {
        self.index
    }

    /// Returns the complete owner-typed selected member.
    #[must_use]
    pub const fn value(&self) -> &T {
        &self.value
    }
}

/// Complete semantic selection closure for one FR-288 identity.
///
/// This is a componentwise closure, not a graph fingerprint: every retained
/// item is the QSL owner's typed value and its original package coordinate.
/// Concrete QObs runtime values and resource ceilings are deliberately absent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticSelectionClosure {
    language: quire_spec_language::protocol_artifact::wire::Language,
    profile: Indexed<quire_spec_language::protocol_artifact::wire::Definition>,
    profile_artifact: ArtifactRef,
    definitions: Vec<Indexed<quire_spec_language::protocol_artifact::wire::Definition>>,
    dependencies: Vec<Indexed<quire_spec_language::protocol_artifact::wire::Dependency>>,
    models: Vec<Indexed<quire_spec_language::protocol_artifact::wire::Model>>,
    types: Vec<Indexed<quire_spec_language::protocol_artifact::wire::Type>>,
    temporal_bindings: Vec<artifact_v2::wire::TemporalBinding>,
}

impl SemanticSelectionClosure {
    /// Returns the selected native language and edition.
    #[must_use]
    pub const fn language(&self) -> &quire_spec_language::protocol_artifact::wire::Language {
        &self.language
    }

    /// Returns the declaration's selected profile definition.
    #[must_use]
    pub const fn profile(
        &self,
    ) -> &Indexed<quire_spec_language::protocol_artifact::wire::Definition> {
        &self.profile
    }

    /// Returns the exact artifact supplying the selected profile.
    #[must_use]
    pub const fn profile_artifact(&self) -> &ArtifactRef {
        &self.profile_artifact
    }

    /// Returns the recursive selected definition closure.
    #[must_use]
    pub fn definitions(
        &self,
    ) -> &[Indexed<quire_spec_language::protocol_artifact::wire::Definition>] {
        &self.definitions
    }

    /// Returns the exact dependency closure supporting the selection.
    #[must_use]
    pub fn dependencies(
        &self,
    ) -> &[Indexed<quire_spec_language::protocol_artifact::wire::Dependency>] {
        &self.dependencies
    }

    /// Returns selected native model correspondences.
    #[must_use]
    pub fn models(&self) -> &[Indexed<quire_spec_language::protocol_artifact::wire::Model>] {
        &self.models
    }

    /// Returns the complete selected type closure.
    #[must_use]
    pub fn types(&self) -> &[Indexed<quire_spec_language::protocol_artifact::wire::Type>] {
        &self.types
    }

    /// Returns selected temporal bindings in declared requirement order.
    #[must_use]
    pub fn temporal_bindings(&self) -> &[artifact_v2::wire::TemporalBinding] {
        &self.temporal_bindings
    }
}

/// Componentwise FR-288 identity for one selected static protocol definition.
///
/// It retains structured owner values rather than hashing the graph. Concrete
/// runtime values are absent: they are carried by [`RuntimeBindingSet`] and do
/// not select this identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DefinitionIdentity {
    source_document: ArtifactRef,
    declaration_handle: u32,
    formal_requirement: Requirement,
    clause: Box<str>,
    static_binding: StaticSelection,
    closure: SemanticSelectionClosure,
}

impl DefinitionIdentity {
    /// Returns the source-document selection that owns this declaration.
    #[must_use]
    pub const fn source_document(&self) -> &ArtifactRef {
        &self.source_document
    }

    /// Returns the exact source-inventory declaration coordinate.
    #[must_use]
    pub const fn declaration_handle(&self) -> u32 {
        self.declaration_handle
    }

    /// Returns the selected formal owner/revision pair.
    #[must_use]
    pub const fn formal_requirement(&self) -> &Requirement {
        &self.formal_requirement
    }

    /// Returns the exact formal clause identity.
    #[must_use]
    pub fn clause(&self) -> &str {
        &self.clause
    }

    /// Returns the selected static binding definition identity.
    #[must_use]
    pub const fn static_binding(&self) -> &StaticSelection {
        &self.static_binding
    }

    /// Returns the complete selected semantic closure.
    #[must_use]
    pub const fn closure(&self) -> &SemanticSelectionClosure {
        &self.closure
    }

    /// Returns the exact declaration-ordered temporal selections in this
    /// static definition closure.
    #[must_use]
    pub fn temporal_bindings(&self) -> &[artifact_v2::wire::TemporalBinding] {
        self.closure.temporal_bindings()
    }
}

/// Why a linked definition has no valid FR-288 identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DefinitionRefusal {
    /// The declaration's source coordinate is outside the admitted source table.
    UnknownSource,
    /// The selected declaration coordinate is not in this admitted package.
    UnknownDeclaration,
    /// A protocol-selected temporal declaration has no admitted binding.
    MissingTemporalBinding,
    /// More than one admitted binding names one selected temporal declaration.
    AmbiguousTemporalBinding,
}

impl LinkedAssessment<'_> {
    /// Derives the componentwise static definition identity from this exact
    /// admitted package and static binding selection.
    pub fn definition_identity(&self) -> Result<DefinitionIdentity, DefinitionRefusal> {
        let declaration = self
            .package
            .inherited()
            .declarations
            .get(usize::try_from(self.definition.declaration_handle()).unwrap_or(usize::MAX))
            .ok_or(DefinitionRefusal::UnknownDeclaration)?;
        let source = self
            .package
            .inherited()
            .sources
            .get(usize::try_from(declaration.locus.source).unwrap_or(usize::MAX))
            .ok_or(DefinitionRefusal::UnknownSource)?;
        if !matches!(&declaration.body, Body::Protocol { .. }) {
            return Err(DefinitionRefusal::UnknownDeclaration);
        }
        let closure =
            selection_closure(self.package, declaration).map_err(|error| match error {
                SelectionClosureRefusal::MissingTemporalBinding => {
                    DefinitionRefusal::MissingTemporalBinding
                }
                SelectionClosureRefusal::AmbiguousTemporalBinding => {
                    DefinitionRefusal::AmbiguousTemporalBinding
                }
                SelectionClosureRefusal::InvalidSelection => DefinitionRefusal::UnknownDeclaration,
            })?;
        Ok(DefinitionIdentity {
            source_document: source.artifact.clone(),
            declaration_handle: self.definition.declaration_handle(),
            formal_requirement: declaration.requirement.clone(),
            clause: declaration.clause.clone().into(),
            static_binding: self.definition.selection().clone(),
            closure,
        })
    }
}

/// Complete consistency content retained for one FR-288 identity.
#[derive(Clone, Debug, Eq, PartialEq)]
struct DefinitionContent {
    declaration: quire_spec_language::protocol_artifact::wire::Declaration,
    static_definition: StaticBindingDefinition,
}

/// One registered static definition and its complete consistency content.
#[derive(Clone, Debug, Eq, PartialEq)]
struct RegisteredDefinition {
    identity: DefinitionIdentity,
    content: DefinitionContent,
}

/// Caller-owned registry that detects contradictory static definitions.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DefinitionRegistry {
    entries: Vec<RegisteredDefinition>,
}

/// Why a definition registry cannot retain an offered linked assessment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RegistryRefusal {
    /// One static identity was offered with different complete typed content.
    ContentContradiction,
    /// The selected declaration has no valid source-document coordinate.
    InvalidIdentity,
}

impl DefinitionRegistry {
    /// Retains one linked static definition or refuses a contradictory replay.
    ///
    /// Replaying equal typed content is idempotent. The registry never merges
    /// or repairs a same-identity conflict.
    pub fn register(&mut self, context: &LinkedAssessment<'_>) -> Result<(), RegistryRefusal> {
        let identity = context
            .definition_identity()
            .map_err(|_| RegistryRefusal::InvalidIdentity)?;
        let declaration = context
            .package()
            .inherited()
            .declarations
            .get(usize::try_from(identity.declaration_handle()).unwrap_or(usize::MAX))
            .ok_or(RegistryRefusal::InvalidIdentity)?;
        let content = DefinitionContent {
            declaration: semantic_declaration(declaration),
            static_definition: context.definition.clone(),
        };
        if let Some(existing) = self.entries.iter().find(|entry| entry.identity == identity) {
            return if existing.content == content {
                Ok(())
            } else {
                Err(RegistryRefusal::ContentContradiction)
            };
        }
        self.entries
            .push(RegisteredDefinition { identity, content });
        Ok(())
    }

    /// Returns the number of distinct registered static definition identities.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the registry has no retained static definition identity.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Removes presentation-only source members from typed consistency content.
///
/// This deliberately walks QSL's closed wire types instead of serializing a
/// declaration through generic JSON or inventing a reduced mirror. All semantic
/// values remain in the owner type, while names, source spans and operator
/// locations cannot create a false FR-288 contradiction.
fn semantic_declaration(
    declaration: &quire_spec_language::protocol_artifact::wire::Declaration,
) -> quire_spec_language::protocol_artifact::wire::Declaration {
    use quire_spec_language::protocol_artifact::wire::{
        Body, ControlOperation, Locus, Nullable, Span,
    };

    fn empty_locus() -> Locus {
        Locus {
            source: 0,
            span: Span { start: 0, end: 0 },
        }
    }

    fn normalize_control(operation: &mut ControlOperation) {
        match operation {
            ControlOperation::Choice { cases, .. } => {
                for case in cases {
                    case.label.clear();
                    case.locus = empty_locus();
                }
            }
            ControlOperation::Parallel { branches, .. } => {
                for branch in branches {
                    branch.label.clear();
                    branch.locus = empty_locus();
                }
            }
            ControlOperation::Event { related, .. } => {
                for relation in related {
                    relation.locus = empty_locus();
                }
            }
            ControlOperation::Sequence { .. }
            | ControlOperation::Repeat { .. }
            | ControlOperation::Await { .. }
            | ControlOperation::Check { .. }
            | ControlOperation::Commit { .. } => {}
        }
    }

    let mut declaration = declaration.clone();
    declaration.name.clear();
    declaration.locus = empty_locus();
    for scope in &mut declaration.scopes {
        scope.locus = empty_locus();
    }
    for anchor in &mut declaration.anchors {
        anchor.locus = empty_locus();
    }
    for binder in &mut declaration.binders {
        binder.name.clear();
        binder.locus = empty_locus();
    }
    for value in &mut declaration.values {
        value.locus = empty_locus();
        value.operator_locus = Nullable(None);
    }
    for temporal in &mut declaration.temporal {
        temporal.locus = empty_locus();
    }
    for binding in &mut declaration.bindings {
        binding.name.clear();
        binding.locus = empty_locus();
    }
    if let Body::Protocol {
        roles,
        relationships,
        channels,
        compensations,
        controls,
        finish,
        ..
    } = &mut declaration.body
    {
        for role in roles {
            role.name.clear();
            role.locus = empty_locus();
        }
        for relationship in relationships {
            relationship.name.clear();
            relationship.locus = empty_locus();
        }
        for channel in channels {
            channel.name.clear();
            channel.locus = empty_locus();
        }
        for compensation in compensations {
            compensation.name.clear();
            compensation.locus = empty_locus();
        }
        for control in controls {
            control.name.clear();
            control.locus = empty_locus();
            normalize_control(&mut control.operation);
        }
        finish.name.clear();
        finish.locus = empty_locus();
    }
    declaration
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SelectionClosureRefusal {
    InvalidSelection,
    MissingTemporalBinding,
    AmbiguousTemporalBinding,
}

fn selection_closure(
    package: &artifact_v2::AdmittedPackage,
    declaration: &quire_spec_language::protocol_artifact::wire::Declaration,
) -> Result<SemanticSelectionClosure, SelectionClosureRefusal> {
    use quire_spec_language::protocol_artifact::wire::Body;

    let inherited = package.inherited();
    let profile = inherited
        .definitions
        .get(usize::try_from(declaration.profile).unwrap_or(usize::MAX))
        .ok_or(SelectionClosureRefusal::InvalidSelection)?;
    let profile_artifact = inherited
        .dependencies
        .get(usize::try_from(profile.artifact).unwrap_or(usize::MAX))
        .ok_or(SelectionClosureRefusal::InvalidSelection)?;

    let mut definitions = BTreeSet::from([declaration.profile]);
    definitions.extend(declaration.requires.iter().copied());
    close_definitions(&mut definitions, &inherited.definitions)?;

    let mut models = BTreeSet::new();
    let mut types = BTreeSet::new();
    collect_declaration_selections(declaration, &mut models, &mut types);
    close_types(&mut types, &mut models, &inherited.types)?;

    let mut dependencies = BTreeSet::new();
    for index in &definitions {
        let definition = inherited
            .definitions
            .get(usize::try_from(*index).unwrap_or(usize::MAX))
            .ok_or(SelectionClosureRefusal::InvalidSelection)?;
        dependencies.insert(definition.artifact);
        dependencies.extend(definition.rules.iter().copied());
    }
    // Binding contracts are dependency-table coordinates, not definition-table
    // coordinates. Treating them as definitions silently rejects valid `/2`
    // packages once a contract index exceeds the definition population.
    dependencies.extend(declaration.bindings.iter().map(|binding| binding.contract));
    for index in &models {
        let model = inherited
            .models
            .get(usize::try_from(*index).unwrap_or(usize::MAX))
            .ok_or(SelectionClosureRefusal::InvalidSelection)?;
        dependencies.insert(model.artifact);
        if let Some(correspondence) = &model.correspondence.0 {
            dependencies.insert(correspondence.producer.interface);
            dependencies.insert(correspondence.native);
            dependencies.insert(correspondence.relation);
            dependencies.extend(correspondence.exports.iter().copied());
        }
    }
    close_dependencies(&mut dependencies, &inherited.dependencies)?;

    let Body::Protocol {
        temporal_requirements,
        ..
    } = &declaration.body
    else {
        return Err(SelectionClosureRefusal::InvalidSelection);
    };
    let mut temporal_bindings = Vec::with_capacity(temporal_requirements.len());
    for required in temporal_requirements {
        let mut matches = package
            .package()
            .temporal_bindings
            .iter()
            .filter(|binding| binding.declaration == *required);
        let binding = matches
            .next()
            .ok_or(SelectionClosureRefusal::MissingTemporalBinding)?;
        if matches.next().is_some() {
            return Err(SelectionClosureRefusal::AmbiguousTemporalBinding);
        }
        temporal_bindings.push(binding.clone());
    }

    Ok(SemanticSelectionClosure {
        language: inherited.language.clone(),
        profile: Indexed {
            index: declaration.profile,
            value: profile.clone(),
        },
        profile_artifact: profile_artifact.artifact.clone(),
        definitions: indexed_values(&definitions, &inherited.definitions)?,
        dependencies: indexed_values(&dependencies, &inherited.dependencies)?,
        models: indexed_values(&models, &inherited.models)?,
        types: indexed_values(&types, &inherited.types)?,
        temporal_bindings,
    })
}

fn indexed_values<T: Clone>(
    indices: &BTreeSet<u32>,
    values: &[T],
) -> Result<Vec<Indexed<T>>, SelectionClosureRefusal> {
    indices
        .iter()
        .map(|index| {
            values
                .get(usize::try_from(*index).unwrap_or(usize::MAX))
                .cloned()
                .map(|value| Indexed {
                    index: *index,
                    value,
                })
                .ok_or(SelectionClosureRefusal::InvalidSelection)
        })
        .collect()
}

fn close_definitions(
    indices: &mut BTreeSet<u32>,
    values: &[quire_spec_language::protocol_artifact::wire::Definition],
) -> Result<(), SelectionClosureRefusal> {
    let mut pending: Vec<_> = indices.iter().copied().collect();
    while let Some(index) = pending.pop() {
        let definition = values
            .get(usize::try_from(index).unwrap_or(usize::MAX))
            .ok_or(SelectionClosureRefusal::InvalidSelection)?;
        for required in &definition.requires {
            if indices.insert(*required) {
                pending.push(*required);
            }
        }
    }
    Ok(())
}

fn close_dependencies(
    indices: &mut BTreeSet<u32>,
    values: &[quire_spec_language::protocol_artifact::wire::Dependency],
) -> Result<(), SelectionClosureRefusal> {
    let mut pending: Vec<_> = indices.iter().copied().collect();
    while let Some(index) = pending.pop() {
        let dependency = values
            .get(usize::try_from(index).unwrap_or(usize::MAX))
            .ok_or(SelectionClosureRefusal::InvalidSelection)?;
        for required in &dependency.requires {
            if indices.insert(*required) {
                pending.push(*required);
            }
        }
    }
    Ok(())
}

fn close_types(
    indices: &mut BTreeSet<u32>,
    models: &mut BTreeSet<u32>,
    values: &[quire_spec_language::protocol_artifact::wire::Type],
) -> Result<(), SelectionClosureRefusal> {
    use quire_spec_language::protocol_artifact::wire::Type;

    let mut pending: Vec<_> = indices.iter().copied().collect();
    while let Some(index) = pending.pop() {
        let value = values
            .get(usize::try_from(index).unwrap_or(usize::MAX))
            .ok_or(SelectionClosureRefusal::InvalidSelection)?;
        match value {
            Type::Boolean {} => {}
            Type::Scalar { export, .. }
            | Type::Enum { export }
            | Type::Record { export }
            | Type::Object { export } => {
                models.insert(export.model);
            }
            Type::Reference {
                export,
                object,
                universe,
            } => {
                models.extend([export.model, object.model, universe.model]);
            }
            Type::Option { value } => {
                if indices.insert(*value) {
                    pending.push(*value);
                }
            }
            Type::Sequence { element, .. } => {
                if indices.insert(*element) {
                    pending.push(*element);
                }
            }
        }
    }
    Ok(())
}

fn collect_export(
    reference: &quire_spec_language::protocol_artifact::wire::ExportRef,
    models: &mut BTreeSet<u32>,
) {
    models.insert(reference.model);
}

fn collect_value_exports(
    operation: &quire_spec_language::protocol_artifact::wire::ValueOperation,
    models: &mut BTreeSet<u32>,
) {
    use quire_spec_language::protocol_artifact::wire::ValueOperation;

    match operation {
        ValueOperation::Enum { variant } => collect_export(variant, models),
        ValueOperation::Field { field, .. } => collect_export(field, models),
        ValueOperation::Parent { edge, universe, .. }
        | ValueOperation::Reaches { edge, universe, .. } => {
            collect_export(edge, models);
            collect_export(universe, models);
        }
        ValueOperation::Boolean { .. }
        | ValueOperation::Number { .. }
        | ValueOperation::Text { .. }
        | ValueOperation::Read { .. }
        | ValueOperation::Group { .. }
        | ValueOperation::Unary { .. }
        | ValueOperation::Binary { .. }
        | ValueOperation::If { .. }
        | ValueOperation::Let { .. }
        | ValueOperation::Pre { .. }
        | ValueOperation::Call { .. }
        | ValueOperation::Size { .. }
        | ValueOperation::Contains { .. }
        | ValueOperation::Query { .. } => {}
    }
}

fn collect_control_exports(
    operation: &quire_spec_language::protocol_artifact::wire::ControlOperation,
    models: &mut BTreeSet<u32>,
) {
    use quire_spec_language::protocol_artifact::wire::{ControlOperation, Event};

    if let ControlOperation::Event {
        event: Event::Attempt { operation, .. },
        ..
    } = operation
    {
        collect_export(operation, models);
    }
}

fn collect_declaration_selections(
    declaration: &quire_spec_language::protocol_artifact::wire::Declaration,
    models: &mut BTreeSet<u32>,
    types: &mut BTreeSet<u32>,
) {
    use quire_spec_language::protocol_artifact::wire::{Body, Execution};

    match &declaration.execution {
        Execution::Pre { operation } | Execution::Post { operation } => {
            collect_export(operation, models);
        }
        Execution::Initialization { .. } | Execution::Handler { .. } => {}
    }
    types.extend(declaration.binders.iter().map(|binder| binder.value_type));
    for value in &declaration.values {
        types.insert(value.value_type);
        collect_value_exports(&value.operation, models);
    }
    for binding in &declaration.bindings {
        types.extend(binding.value_type.0.iter().copied());
        if let Some(model) = &binding.model.0 {
            collect_export(model, models);
        }
    }
    match &declaration.body {
        Body::Predicate { .. } | Body::Temporal { .. } => {}
        Body::State {
            context, operation, ..
        } => {
            collect_export(context, models);
            if let Some(operation) = &operation.0 {
                collect_export(operation, models);
            }
        }
        Body::Protocol {
            roles,
            relationships,
            channels,
            compensations,
            controls,
            ..
        } => {
            for role in roles {
                collect_export(&role.model, models);
            }
            for relationship in relationships {
                collect_export(&relationship.model, models);
            }
            types.extend(channels.iter().map(|channel| channel.message_type));
            for compensation in compensations {
                collect_export(&compensation.operation, models);
                types.insert(compensation.attempt_type);
            }
            for control in controls {
                collect_control_exports(&control.operation, models);
            }
        }
    }
}

/// The v2-native finite protocol representation retained for assessment.
///
/// Unlike the legacy `/1` graph, this value keeps the complete admitted QSL
/// declaration beside the componentwise FR-288 identity and the exact QObs
/// runtime bindings. It does not re-encode or reduce the declaration into a
/// fingerprint, and it has no constructor that accepts unadmitted wire data.
#[derive(Clone, Debug)]
pub struct LinkedProtocol<'a> {
    context: LinkedAssessment<'a>,
    identity: DefinitionIdentity,
    channels: Vec<LinkedChannelDefinition>,
    compensations: Vec<LinkedCompensationDefinition>,
    causal_edges: Vec<quire_spec_language::protocol_artifact::wire::CausalEdge>,
}

/// One exact admitted compensation definition in declaration-table order.
///
/// This retains QSL's complete typed definition. It is not a runtime
/// registration: that additionally needs the concrete observed forward effect.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinkedCompensationDefinition {
    handle: u32,
    definition: quire_spec_language::protocol_artifact::wire::Compensation,
}

impl LinkedCompensationDefinition {
    #[must_use]
    pub const fn handle(&self) -> u32 {
        self.handle
    }

    #[must_use]
    pub const fn definition(&self) -> &quire_spec_language::protocol_artifact::wire::Compensation {
        &self.definition
    }
}

/// Complete typed FIFO definition resolved from one admitted channel.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FifoDefinition {
    binder: Indexed<quire_spec_language::protocol_artifact::wire::Binder>,
    key: Indexed<quire_spec_language::protocol_artifact::wire::Value>,
    anchor: Indexed<quire_spec_language::protocol_artifact::wire::Anchor>,
    types: Vec<Indexed<quire_spec_language::protocol_artifact::wire::Type>>,
}

impl FifoDefinition {
    #[must_use]
    pub const fn binder(&self) -> &Indexed<quire_spec_language::protocol_artifact::wire::Binder> {
        &self.binder
    }
    #[must_use]
    pub const fn key(&self) -> &Indexed<quire_spec_language::protocol_artifact::wire::Value> {
        &self.key
    }
    #[must_use]
    pub const fn anchor(&self) -> &Indexed<quire_spec_language::protocol_artifact::wire::Anchor> {
        &self.anchor
    }
    #[must_use]
    pub fn types(&self) -> &[Indexed<quire_spec_language::protocol_artifact::wire::Type>] {
        &self.types
    }
}

/// One exact admitted channel plus its optional FIFO expression definition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinkedChannelDefinition {
    handle: ChannelHandle,
    fifo: Option<FifoDefinition>,
}

impl LinkedChannelDefinition {
    #[must_use]
    pub const fn handle(&self) -> ChannelHandle {
        self.handle
    }
    #[must_use]
    pub const fn fifo(&self) -> Option<&FifoDefinition> {
        self.fifo.as_ref()
    }
}

/// Exact runtime FIFO partition for one admitted static channel.
///
/// `value` is supplied in its owner type and is never interpreted, encoded, or
/// used to derive a causal edge. The definition identity and channel coordinate
/// ensure that equal owner values on distinct static channels remain distinct.
#[derive(Debug, Eq, PartialEq)]
pub struct FifoPartition<'a, T> {
    definition: &'a DefinitionIdentity,
    channel: ChannelHandle,
    value: T,
}

impl<'a, T> FifoPartition<'a, T> {
    /// Returns the complete static definition identity selecting this channel.
    #[must_use]
    pub const fn definition(&self) -> &'a DefinitionIdentity {
        self.definition
    }

    /// Returns the exact selected static FIFO channel coordinate.
    #[must_use]
    pub const fn channel(&self) -> ChannelHandle {
        self.channel
    }

    /// Returns the exact owner-evaluated value without reinterpretation.
    #[must_use]
    pub const fn value(&self) -> &T {
        &self.value
    }
}

/// Why a runtime FIFO partition cannot be formed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FifoPartitionRefusal {
    /// The selected channel coordinate is outside the admitted declaration.
    UnknownChannel(ChannelHandle),
    /// The selected channel is admitted but its ordering is not FIFO.
    NotFifo(ChannelHandle),
}

/// Why a v2 assessment representation cannot link.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LinkRefusal {
    /// The admitted declaration does not identify one source document.
    InvalidIdentity,
    /// The registry already retains another complete content set at this key.
    ContentContradiction,
    /// An admitted FIFO reference could not be retained through its typed QSL table.
    InvalidFifoDefinition(ChannelHandle),
}

/// Links one admitted v2 assessment context through the caller-owned registry.
///
/// The registry check happens before a representation is returned, so an
/// identity/content contradiction cannot leak a partially linked graph.
pub fn link<'a>(
    context: LinkedAssessment<'a>,
    registry: &mut DefinitionRegistry,
) -> Result<LinkedProtocol<'a>, LinkRefusal> {
    let identity = context
        .definition_identity()
        .map_err(|_| LinkRefusal::InvalidIdentity)?;
    let channels = build_channels(context.package(), &identity)?;
    let compensations = build_compensations(context.package(), &identity)?;
    let causal_edges = build_causal_edges(context.package(), &identity)?;
    registry.register(&context).map_err(|error| match error {
        RegistryRefusal::ContentContradiction => LinkRefusal::ContentContradiction,
        RegistryRefusal::InvalidIdentity => LinkRefusal::InvalidIdentity,
    })?;
    Ok(LinkedProtocol {
        context,
        identity,
        channels,
        compensations,
        causal_edges,
    })
}

impl<'a> LinkedProtocol<'a> {
    /// Returns the componentwise static protocol definition identity.
    #[must_use]
    pub const fn identity(&self) -> &DefinitionIdentity {
        &self.identity
    }

    /// Returns the complete typed QSL declaration selected for assessment.
    #[must_use]
    pub fn declaration(&self) -> &quire_spec_language::protocol_artifact::wire::Declaration {
        &self.context.package().inherited().declarations
            [usize::try_from(self.identity.declaration_handle()).unwrap_or(usize::MAX)]
    }

    /// Returns the immutable exact assessment capability.
    #[must_use]
    pub const fn context(&self) -> LinkedAssessment<'a> {
        self.context
    }

    /// Returns exact channels in declaration-table order.
    #[must_use]
    pub fn channels(&self) -> &[LinkedChannelDefinition] {
        &self.channels
    }

    /// Returns exact compensation definitions in declaration-table order.
    #[must_use]
    pub fn compensations(&self) -> &[LinkedCompensationDefinition] {
        &self.compensations
    }

    /// Returns every admitted causal edge in its owner-declared order.
    ///
    /// These are QSL's complete typed edges, retained beside the linked
    /// definition for the later deterministic conformance kernel.  This
    /// representation neither sorts them nor infers an edge from arrival,
    /// channel ordering, or a FIFO partition.
    #[must_use]
    pub fn causal_edges(&self) -> &[quire_spec_language::protocol_artifact::wire::CausalEdge] {
        &self.causal_edges
    }

    /// Forms one exact partition for a selected admitted FIFO channel.
    ///
    /// The complete owner typed declaration remains retained by this linked
    /// representation; this method only selects it by its declaration-local
    /// coordinate. It never evaluates the FIFO key expression or creates a
    /// causal edge from a partition or arrival order.
    pub fn fifo_partition<T>(
        &self,
        channel: ChannelHandle,
        value: T,
    ) -> Result<FifoPartition<'_, T>, FifoPartitionRefusal> {
        let selected = self
            .channels
            .get(usize::try_from(channel.get()).unwrap_or(usize::MAX))
            .ok_or(FifoPartitionRefusal::UnknownChannel(channel))?;
        if selected.fifo().is_none() {
            return Err(FifoPartitionRefusal::NotFifo(channel));
        }
        Ok(FifoPartition {
            definition: &self.identity,
            channel,
            value,
        })
    }
}

fn build_channels(
    package: &artifact_v2::AdmittedPackage,
    identity: &DefinitionIdentity,
) -> Result<Vec<LinkedChannelDefinition>, LinkRefusal> {
    use quire_spec_language::protocol_artifact::wire::{Body, Ordering};

    let declaration = package
        .inherited()
        .declarations
        .get(usize::try_from(identity.declaration_handle()).unwrap_or(usize::MAX))
        .ok_or(LinkRefusal::InvalidIdentity)?;
    let Body::Protocol { channels, .. } = &declaration.body else {
        return Err(LinkRefusal::InvalidIdentity);
    };
    channels
        .iter()
        .enumerate()
        .map(|(index, channel)| {
            let handle = ChannelHandle::new(u32::try_from(index).unwrap_or(u32::MAX));
            let fifo = match &channel.ordering {
                Ordering::Unordered {} => None,
                Ordering::Fifo {
                    binder,
                    key,
                    anchor,
                } => Some(build_fifo(package, binder, key, anchor, handle)?),
            };
            Ok(LinkedChannelDefinition { handle, fifo })
        })
        .collect()
}

fn build_compensations(
    package: &artifact_v2::AdmittedPackage,
    identity: &DefinitionIdentity,
) -> Result<Vec<LinkedCompensationDefinition>, LinkRefusal> {
    let declaration = package
        .inherited()
        .declarations
        .get(usize::try_from(identity.declaration_handle()).unwrap_or(usize::MAX))
        .ok_or(LinkRefusal::InvalidIdentity)?;
    let Body::Protocol { compensations, .. } = &declaration.body else {
        return Err(LinkRefusal::InvalidIdentity);
    };
    compensations
        .iter()
        .enumerate()
        .map(|(index, definition)| {
            Ok(LinkedCompensationDefinition {
                handle: u32::try_from(index).map_err(|_| LinkRefusal::InvalidIdentity)?,
                definition: definition.clone(),
            })
        })
        .collect()
}

fn build_causal_edges(
    package: &artifact_v2::AdmittedPackage,
    identity: &DefinitionIdentity,
) -> Result<Vec<quire_spec_language::protocol_artifact::wire::CausalEdge>, LinkRefusal> {
    let declaration = package
        .inherited()
        .declarations
        .get(usize::try_from(identity.declaration_handle()).unwrap_or(usize::MAX))
        .ok_or(LinkRefusal::InvalidIdentity)?;
    let Body::Protocol { causal_edges, .. } = &declaration.body else {
        return Err(LinkRefusal::InvalidIdentity);
    };
    Ok(causal_edges.clone())
}

fn build_fifo(
    package: &artifact_v2::AdmittedPackage,
    binder: &quire_spec_language::protocol_artifact::wire::Handle,
    key: &quire_spec_language::protocol_artifact::wire::Handle,
    anchor: &quire_spec_language::protocol_artifact::wire::Handle,
    channel: ChannelHandle,
) -> Result<FifoDefinition, LinkRefusal> {
    let declaration = |handle: &quire_spec_language::protocol_artifact::wire::Handle| {
        package
            .inherited()
            .declarations
            .get(usize::try_from(handle.declaration).unwrap_or(usize::MAX))
    };
    let binder_value = declaration(binder)
        .and_then(|d| {
            d.binders
                .get(usize::try_from(binder.index).unwrap_or(usize::MAX))
        })
        .cloned()
        .ok_or(LinkRefusal::InvalidFifoDefinition(channel))?;
    let key_value = declaration(key)
        .and_then(|d| {
            d.values
                .get(usize::try_from(key.index).unwrap_or(usize::MAX))
        })
        .cloned()
        .ok_or(LinkRefusal::InvalidFifoDefinition(channel))?;
    let anchor_value = declaration(anchor)
        .and_then(|d| {
            d.anchors
                .get(usize::try_from(anchor.index).unwrap_or(usize::MAX))
        })
        .cloned()
        .ok_or(LinkRefusal::InvalidFifoDefinition(channel))?;
    let mut types = BTreeSet::from([key_value.value_type]);
    let mut ignored_models = BTreeSet::new();
    close_types(&mut types, &mut ignored_models, &package.inherited().types)
        .map_err(|_| LinkRefusal::InvalidFifoDefinition(channel))?;
    Ok(FifoDefinition {
        binder: Indexed {
            index: binder.index,
            value: binder_value,
        },
        key: Indexed {
            index: key.index,
            value: key_value,
        },
        anchor: Indexed {
            index: anchor.index,
            value: anchor_value,
        },
        types: indexed_values(&types, &package.inherited().types)
            .map_err(|_| LinkRefusal::InvalidFifoDefinition(channel))?,
    })
}

fn validate_value(
    handle: SlotHandle,
    compatibility: &SlotCompatibility,
    producer: &AdmittedStaticBundle,
    value: &RuntimeValue,
) -> Result<(), RuntimeRefusal> {
    match (compatibility, value) {
        (SlotCompatibility::Subject { kind }, RuntimeValue::Subject(subject)) => {
            if subject.authority() != producer.key() {
                return Err(RuntimeRefusal::ForeignAuthority(handle));
            }
            if subject.kind() != kind {
                return Err(RuntimeRefusal::IncompatibleSubjectKind(handle));
            }
        }
        (
            SlotCompatibility::Relationship {
                declaration_identity,
                source_kind,
                target_kind,
            },
            RuntimeValue::Relationship(relationship),
        ) => {
            if relationship.authority() != producer.key()
                || relationship.interface_version() != producer.interface_version()
            {
                return Err(RuntimeRefusal::ForeignAuthority(handle));
            }
            if relationship.declaration_identity() != declaration_identity.as_ref() {
                return Err(RuntimeRefusal::IncompatibleRelationship(handle));
            }
            if relationship.source().kind() != source_kind
                || relationship.target().kind() != target_kind
            {
                return Err(RuntimeRefusal::IncompatibleRelationship(handle));
            }
        }
        _ => return Err(RuntimeRefusal::WrongValueKind(handle)),
    }
    Ok(())
}
