// SPDX-License-Identifier: AGPL-3.0-or-later
//! Componentwise FR-288 identity and exact typed static consistency content.

use std::collections::BTreeSet;

use quire_spec_language::{
    protocol_artifact::{v2, wire},
    ByteDigest,
};

use super::binding::{
    AcceptedBindingDefinition, BindingArtifactIdentity, RoleClassification, SlotCompatibility,
};

/// Exact authored source-document key.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceDocumentKey {
    /// Source artifact identity.
    pub identity: String,
    /// Exact revision namespace and value.
    pub revision: wire::Revision,
    /// Digest of the admitted original source bytes.
    pub digest: ByteDigest,
}

/// Source-inventory-qualified declaration handle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DeclarationHandle {
    /// Source inventory index.
    pub source: u32,
    /// Package declaration-table index.
    pub declaration: u32,
}

/// Exact formal owner and clause.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FormalOwnerClause {
    /// Formal requirement selection.
    pub requirement: wire::Requirement,
    /// Exact clause identity.
    pub clause: String,
}

/// One exact package-table member with its original handle retained.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Indexed<T> {
    /// Original package-table index.
    pub index: u32,
    /// Exact owner-typed table member.
    pub value: T,
}

/// One exact declaration-local table member with its full owner handle retained.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Handled<T> {
    /// Exact declaration and declaration-local table index.
    pub handle: wire::Handle,
    /// Exact owner-typed table member.
    pub value: T,
}

/// Exact selected protocol profile definition and its source artifact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedProfile {
    /// Definition-table index selected by the declaration.
    pub definition: u32,
    /// Exact definition identity, revision, source, rules, and prerequisites.
    pub value: wire::Definition,
    /// Exact selected source artifact, including its digest.
    pub artifact: wire::ArtifactRef,
}

/// Complete selected semantic closure retained without a flattened digest.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticSelectionClosure {
    /// Exact native language and edition.
    pub language: wire::Language,
    /// Exact selected profile definition and source artifact.
    pub profile: SelectedProfile,
    /// Recursive selected definition closure, with original indices.
    pub definitions: Vec<Indexed<wire::Definition>>,
    /// Exact dependency graph supplying selected definition/model artifacts.
    pub dependencies: Vec<Indexed<wire::Dependency>>,
    /// Selected model identities and producer/native correspondences.
    pub models: Vec<Indexed<wire::Model>>,
    /// Complete reachable selected type graph.
    pub types: Vec<Indexed<wire::Type>>,
    /// Relevant static temporal selections; runtime clock observations are absent.
    pub temporal_bindings: Vec<v2::wire::TemporalBinding>,
    /// Accepted static binding selection; no runtime value is included.
    pub binding_definition: BindingArtifactIdentity,
}

/// Complete componentwise FR-288 key.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DefinitionIdentity {
    source: SourceDocumentKey,
    declaration: DeclarationHandle,
    owner: FormalOwnerClause,
    closure: SemanticSelectionClosure,
}

impl DefinitionIdentity {
    pub(super) fn from_package(
        package: &v2::wire::Package,
        declaration: u32,
        binding: &AcceptedBindingDefinition,
    ) -> Option<Self> {
        let inherited = &package.inherited;
        let declared = inherited
            .declarations
            .get(usize::try_from(declaration).ok()?)?;
        let source = inherited
            .sources
            .get(usize::try_from(declared.locus.source).ok()?)?;
        let profile_value = inherited
            .definitions
            .get(usize::try_from(declared.profile).ok()?)?;
        let profile_artifact = inherited
            .dependencies
            .get(usize::try_from(profile_value.artifact).ok()?)?;
        let selected = selection_closure(package, declaration, declared)?;
        Some(Self {
            source: SourceDocumentKey {
                identity: source.artifact.identity.clone(),
                revision: source.artifact.revision.clone(),
                digest: source.artifact.digest,
            },
            declaration: DeclarationHandle {
                source: declared.locus.source,
                declaration,
            },
            owner: FormalOwnerClause {
                requirement: declared.requirement.clone(),
                clause: declared.clause.clone(),
            },
            closure: SemanticSelectionClosure {
                language: inherited.language.clone(),
                profile: SelectedProfile {
                    definition: declared.profile,
                    value: profile_value.clone(),
                    artifact: profile_artifact.artifact.clone(),
                },
                definitions: selected.definitions,
                dependencies: selected.dependencies,
                models: selected.models,
                types: selected.types,
                temporal_bindings: selected.temporal_bindings,
                binding_definition: binding.artifact().clone(),
            },
        })
    }

    /// Exact source-document key.
    #[must_use]
    pub const fn source(&self) -> &SourceDocumentKey {
        &self.source
    }

    /// Exact source-qualified declaration handle.
    #[must_use]
    pub const fn declaration(&self) -> DeclarationHandle {
        self.declaration
    }

    /// Exact formal owner and clause.
    #[must_use]
    pub const fn owner(&self) -> &FormalOwnerClause {
        &self.owner
    }

    /// Complete semantic selection closure.
    #[must_use]
    pub const fn closure(&self) -> &SemanticSelectionClosure {
        &self.closure
    }
}

struct SelectionClosure {
    definitions: Vec<Indexed<wire::Definition>>,
    dependencies: Vec<Indexed<wire::Dependency>>,
    models: Vec<Indexed<wire::Model>>,
    types: Vec<Indexed<wire::Type>>,
    temporal_bindings: Vec<v2::wire::TemporalBinding>,
}

fn selection_closure(
    package: &v2::wire::Package,
    declaration_index: u32,
    _declaration: &wire::Declaration,
) -> Option<SelectionClosure> {
    let inherited = &package.inherited;
    let declaration_indices = reachable_declarations(declaration_index, &inherited.declarations)?;
    let mut definition_indices = BTreeSet::new();
    let mut model_indices = BTreeSet::new();
    let mut type_indices = BTreeSet::new();
    let mut dependency_indices = BTreeSet::new();

    for index in &declaration_indices {
        let declaration = inherited.declarations.get(usize::try_from(*index).ok()?)?;
        collect_declaration_selections(
            declaration,
            &mut definition_indices,
            &mut model_indices,
            &mut type_indices,
        );
        for binding in &declaration.bindings {
            definition_indices.insert(binding_definition_index(binding, &inherited.definitions)?);
            dependency_indices.insert(binding.contract);
            dependency_indices.extend(binding.relation.0);
        }
    }
    for temporal in &package.temporal_bindings {
        if declaration_indices.contains(&temporal.declaration) {
            definition_indices.insert(temporal.definition);
        }
    }
    close_definitions(&mut definition_indices, &inherited.definitions)?;
    close_types(&mut type_indices, &mut model_indices, &inherited.types)?;

    for index in &definition_indices {
        let value = inherited.definitions.get(usize::try_from(*index).ok()?)?;
        dependency_indices.insert(value.artifact);
        dependency_indices.extend(value.rules.iter().copied());
    }
    for index in &model_indices {
        let value = inherited.models.get(usize::try_from(*index).ok()?)?;
        dependency_indices.insert(value.artifact);
        if let Some(correspondence) = &value.correspondence.0 {
            dependency_indices.insert(correspondence.producer.interface);
            dependency_indices.insert(correspondence.native);
            dependency_indices.insert(correspondence.relation);
        }
    }
    close_dependencies(&mut dependency_indices, &inherited.dependencies)?;

    Some(SelectionClosure {
        definitions: indexed_values(&definition_indices, &inherited.definitions)?,
        dependencies: indexed_values(&dependency_indices, &inherited.dependencies)?,
        models: semantic_models(&model_indices, &inherited.models)?,
        types: indexed_values(&type_indices, &inherited.types)?,
        temporal_bindings: package
            .temporal_bindings
            .iter()
            .filter(|binding| declaration_indices.contains(&binding.declaration))
            .cloned()
            .collect(),
    })
}

fn reachable_declarations(root: u32, declarations: &[wire::Declaration]) -> Option<BTreeSet<u32>> {
    let mut selected = BTreeSet::from([root]);
    let mut pending = vec![root];
    while let Some(index) = pending.pop() {
        let declaration = declarations.get(usize::try_from(index).ok()?)?;
        for required in &declaration.requires {
            if selected.insert(*required) {
                pending.push(*required);
            }
        }
    }
    Some(selected)
}

fn binding_definition_index(
    binding: &wire::BindingRequirement,
    definitions: &[wire::Definition],
) -> Option<u32> {
    let mut matches = definitions.iter().enumerate().filter(|(_, definition)| {
        definition.artifact == binding.contract
            && definition.identity == binding.authority.identity
            && definition.revision == binding.authority.revision
    });
    let (index, _) = matches.next()?;
    if matches.next().is_some() {
        return None;
    }
    u32::try_from(index).ok()
}

fn indexed_values<T: Clone>(indices: &BTreeSet<u32>, values: &[T]) -> Option<Vec<Indexed<T>>> {
    indices
        .iter()
        .map(|index| {
            Some(Indexed {
                index: *index,
                value: values.get(usize::try_from(*index).ok()?)?.clone(),
            })
        })
        .collect()
}

fn semantic_models(
    indices: &BTreeSet<u32>,
    values: &[wire::Model],
) -> Option<Vec<Indexed<wire::Model>>> {
    let mut selected = indexed_values(indices, values)?;
    for model in &mut selected {
        for export in &mut model.value.exports {
            export.locus.span = wire::Span { start: 0, end: 0 };
        }
    }
    Some(selected)
}

fn close_definitions(indices: &mut BTreeSet<u32>, values: &[wire::Definition]) -> Option<()> {
    let mut pending: Vec<u32> = indices.iter().copied().collect();
    while let Some(index) = pending.pop() {
        let definition = values.get(usize::try_from(index).ok()?)?;
        for required in &definition.requires {
            if indices.insert(*required) {
                pending.push(*required);
            }
        }
    }
    Some(())
}

fn close_dependencies(indices: &mut BTreeSet<u32>, values: &[wire::Dependency]) -> Option<()> {
    let mut pending: Vec<u32> = indices.iter().copied().collect();
    while let Some(index) = pending.pop() {
        let dependency = values.get(usize::try_from(index).ok()?)?;
        for required in &dependency.requires {
            if indices.insert(*required) {
                pending.push(*required);
            }
        }
    }
    Some(())
}

fn close_types(
    indices: &mut BTreeSet<u32>,
    models: &mut BTreeSet<u32>,
    values: &[wire::Type],
) -> Option<()> {
    let mut pending: Vec<u32> = indices.iter().copied().collect();
    while let Some(index) = pending.pop() {
        let value = values.get(usize::try_from(index).ok()?)?;
        match value {
            wire::Type::Boolean {} => {}
            wire::Type::Scalar { export, .. }
            | wire::Type::Enum { export }
            | wire::Type::Record { export }
            | wire::Type::Object { export } => {
                models.insert(export.model);
            }
            wire::Type::Reference {
                export,
                object,
                universe,
            } => {
                models.extend([export.model, object.model, universe.model]);
            }
            wire::Type::Option { value } => {
                if indices.insert(*value) {
                    pending.push(*value);
                }
            }
            wire::Type::Sequence { element, .. } => {
                if indices.insert(*element) {
                    pending.push(*element);
                }
            }
        }
    }
    Some(())
}

fn collect_export(reference: &wire::ExportRef, models: &mut BTreeSet<u32>) {
    models.insert(reference.model);
}

fn collect_value_exports(operation: &wire::ValueOperation, models: &mut BTreeSet<u32>) {
    match operation {
        wire::ValueOperation::Enum { variant } => collect_export(variant, models),
        wire::ValueOperation::Field { field, .. } => collect_export(field, models),
        wire::ValueOperation::Parent { edge, universe, .. }
        | wire::ValueOperation::Reaches { edge, universe, .. } => {
            collect_export(edge, models);
            collect_export(universe, models);
        }
        wire::ValueOperation::Boolean { .. }
        | wire::ValueOperation::Number { .. }
        | wire::ValueOperation::Text { .. }
        | wire::ValueOperation::Read { .. }
        | wire::ValueOperation::Group { .. }
        | wire::ValueOperation::Unary { .. }
        | wire::ValueOperation::Binary { .. }
        | wire::ValueOperation::If { .. }
        | wire::ValueOperation::Let { .. }
        | wire::ValueOperation::Pre { .. }
        | wire::ValueOperation::Call { .. }
        | wire::ValueOperation::Size { .. }
        | wire::ValueOperation::Contains { .. }
        | wire::ValueOperation::Query { .. } => {}
    }
}

fn collect_control_selections(
    operation: &wire::ControlOperation,
    definitions: &mut BTreeSet<u32>,
    models: &mut BTreeSet<u32>,
) {
    match operation {
        wire::ControlOperation::Event {
            event: wire::Event::Attempt { operation, .. },
            ..
        } => collect_export(operation, models),
        wire::ControlOperation::Await { profile, .. }
        | wire::ControlOperation::Check { profile, .. } => {
            definitions.insert(*profile);
        }
        wire::ControlOperation::Sequence { .. }
        | wire::ControlOperation::Choice { .. }
        | wire::ControlOperation::Parallel { .. }
        | wire::ControlOperation::Repeat { .. }
        | wire::ControlOperation::Event { .. }
        | wire::ControlOperation::Commit { .. } => {}
    }
}

fn collect_declaration_selections(
    declaration: &wire::Declaration,
    definitions: &mut BTreeSet<u32>,
    models: &mut BTreeSet<u32>,
    types: &mut BTreeSet<u32>,
) {
    definitions.insert(declaration.profile);
    match &declaration.execution {
        wire::Execution::Pre { operation } | wire::Execution::Post { operation } => {
            collect_export(operation, models);
        }
        wire::Execution::Initialization { .. } | wire::Execution::Handler { .. } => {}
    }
    types.extend(declaration.binders.iter().map(|binder| binder.value_type));
    for value in &declaration.values {
        definitions.insert(value.profile);
        types.insert(value.value_type);
        collect_value_exports(&value.operation, models);
        match &value.operation {
            wire::ValueOperation::Size { result, .. }
            | wire::ValueOperation::Query { result, .. } => {
                types.insert(*result);
            }
            wire::ValueOperation::Boolean { .. }
            | wire::ValueOperation::Number { .. }
            | wire::ValueOperation::Text { .. }
            | wire::ValueOperation::Enum { .. }
            | wire::ValueOperation::Read { .. }
            | wire::ValueOperation::Group { .. }
            | wire::ValueOperation::Field { .. }
            | wire::ValueOperation::Unary { .. }
            | wire::ValueOperation::Binary { .. }
            | wire::ValueOperation::If { .. }
            | wire::ValueOperation::Let { .. }
            | wire::ValueOperation::Pre { .. }
            | wire::ValueOperation::Call { .. }
            | wire::ValueOperation::Contains { .. }
            | wire::ValueOperation::Parent { .. }
            | wire::ValueOperation::Reaches { .. } => {}
        }
    }
    for binding in &declaration.bindings {
        types.extend(binding.value_type.0.iter().copied());
        if let Some(model) = &binding.model.0 {
            collect_export(model, models);
        }
    }
    match &declaration.body {
        wire::Body::Predicate { result, .. } => {
            types.insert(*result);
        }
        wire::Body::Temporal { .. } => {}
        wire::Body::State {
            context, operation, ..
        } => {
            collect_export(context, models);
            if let Some(operation) = &operation.0 {
                collect_export(operation, models);
            }
        }
        wire::Body::Protocol {
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
                definitions.insert(compensation.profile);
                collect_export(&compensation.operation, models);
                types.insert(compensation.attempt_type);
            }
            for control in controls {
                collect_control_selections(&control.operation, definitions, models);
            }
        }
    }
}

/// One semantic declaration plus its accepted static binding facts.
///
/// The declaration remains a QSL owner type. Presentation members and source
/// locations are neutralized by an exhaustive typed walk; no JSON projection,
/// key deletion, numeric fingerprint, or locally mirrored wire record exists.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticContent {
    declaration: wire::Declaration,
    dependencies: Vec<Indexed<wire::Declaration>>,
    roles: Vec<RoleClassification>,
    slots: Vec<SlotCompatibility>,
}

/// Exact presentation, provenance, and source-location context for one link.
///
/// These owner-typed records are retained for diagnostics and inspection but
/// remain outside FR-288 and linked semantic equality. The selected declaration
/// closure and models preserve every original QSL locus and display member;
/// the source inventory makes each locus resolvable, and `producer` retains the
/// build provenance that created the admitted package.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PresentationContext {
    producer: wire::Producer,
    sources: Vec<Indexed<wire::Source>>,
    declarations: Vec<Indexed<wire::Declaration>>,
    models: Vec<Indexed<wire::Model>>,
}

impl PresentationContext {
    pub(super) fn from_package(
        package: &v2::wire::Package,
        identity: &DefinitionIdentity,
    ) -> Option<Self> {
        let declaration_indices = reachable_declarations(
            identity.declaration().declaration,
            &package.inherited.declarations,
        )?;
        let declarations = indexed_values(&declaration_indices, &package.inherited.declarations)?;
        let source_indices = declarations
            .iter()
            .map(|declaration| declaration.value.locus.source)
            .collect();
        let model_indices = identity
            .closure()
            .models
            .iter()
            .map(|model| model.index)
            .collect();
        Some(Self {
            producer: package.inherited.producer.clone(),
            sources: indexed_values(&source_indices, &package.inherited.sources)?,
            declarations,
            models: indexed_values(&model_indices, &package.inherited.models)?,
        })
    }

    /// Exact admitted producer and build provenance.
    #[must_use]
    pub const fn producer(&self) -> &wire::Producer {
        &self.producer
    }

    /// Exact source inventory, including paths and original source bytes.
    #[must_use]
    pub fn sources(&self) -> &[Indexed<wire::Source>] {
        &self.sources
    }

    /// Reachable declarations with every original name and locus retained.
    #[must_use]
    pub fn declarations(&self) -> &[Indexed<wire::Declaration>] {
        &self.declarations
    }

    /// Selected models with every original producer-export locus retained.
    #[must_use]
    pub fn models(&self) -> &[Indexed<wire::Model>] {
        &self.models
    }
}

impl SemanticContent {
    pub(super) fn from_package(
        package: &v2::wire::Package,
        declaration: u32,
        binding: &AcceptedBindingDefinition,
    ) -> Option<Self> {
        let declared = package
            .inherited
            .declarations
            .get(usize::try_from(declaration).ok()?)?;
        let mut dependency_indices =
            reachable_declarations(declaration, &package.inherited.declarations)?;
        dependency_indices.remove(&declaration);
        Some(Self {
            declaration: semantic_declaration(declared),
            dependencies: semantic_declarations(
                &dependency_indices,
                &package.inherited.declarations,
            )?,
            roles: binding.roles().to_vec(),
            slots: binding.slots().to_vec(),
        })
    }

    /// Complete typed declaration with presentation and loci excluded.
    #[must_use]
    pub const fn declaration(&self) -> &wire::Declaration {
        &self.declaration
    }

    /// Complete recursively selected declaration dependency graph.
    #[must_use]
    pub fn dependencies(&self) -> &[Indexed<wire::Declaration>] {
        &self.dependencies
    }

    /// Complete accepted role classifications.
    #[must_use]
    pub fn roles(&self) -> &[RoleClassification] {
        &self.roles
    }

    /// Complete accepted slot compatibility facts.
    #[must_use]
    pub fn slots(&self) -> &[SlotCompatibility] {
        &self.slots
    }
}

fn semantic_declarations(
    indices: &BTreeSet<u32>,
    declarations: &[wire::Declaration],
) -> Option<Vec<Indexed<wire::Declaration>>> {
    indices
        .iter()
        .map(|index| {
            Some(Indexed {
                index: *index,
                value: semantic_declaration(declarations.get(usize::try_from(*index).ok()?)?),
            })
        })
        .collect()
}

fn empty_locus() -> wire::Locus {
    wire::Locus {
        source: 0,
        span: wire::Span { start: 0, end: 0 },
    }
}

fn semantic_declaration(declaration: &wire::Declaration) -> wire::Declaration {
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
        value.operator_locus = wire::Nullable(None);
    }
    for temporal in &mut declaration.temporal {
        temporal.locus = empty_locus();
    }
    for binding in &mut declaration.bindings {
        binding.name.clear();
        binding.locus = empty_locus();
    }
    normalize_body(&mut declaration.body);
    declaration
}

fn normalize_body(body: &mut wire::Body) {
    match body {
        wire::Body::Predicate { .. } | wire::Body::State { .. } | wire::Body::Temporal { .. } => {}
        wire::Body::Protocol {
            roles,
            relationships,
            channels,
            compensations,
            controls,
            finish,
            ..
        } => {
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
    }
}

fn normalize_control(operation: &mut wire::ControlOperation) {
    match operation {
        wire::ControlOperation::Choice { cases, .. } => {
            for case in cases {
                case.label.clear();
                case.locus = empty_locus();
            }
        }
        wire::ControlOperation::Parallel { branches, .. } => {
            for branch in branches {
                branch.label.clear();
                branch.locus = empty_locus();
            }
        }
        wire::ControlOperation::Event { related, .. } => {
            for relation in related {
                relation.locus = empty_locus();
            }
        }
        wire::ControlOperation::Sequence { .. }
        | wire::ControlOperation::Repeat { .. }
        | wire::ControlOperation::Await { .. }
        | wire::ControlOperation::Check { .. }
        | wire::ControlOperation::Commit { .. } => {}
    }
}

/// Definition-registry outcome for an exact replay.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DefinitionRegistration {
    /// First registration of this exact key.
    Inserted,
    /// Same key and same complete consistency content already existed.
    Replayed,
}

/// Same FR-288 key was presented with different static consistency content.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DefinitionRegistryRefusal {
    /// Contradicted exact definition identity.
    pub identity: Box<DefinitionIdentity>,
}

/// Finite caller-owned FR-288 consistency registry.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DefinitionRegistry {
    entries: Vec<(DefinitionIdentity, SemanticContent)>,
}

impl DefinitionRegistry {
    pub(super) fn inspect(
        &self,
        identity: &DefinitionIdentity,
        content: &SemanticContent,
    ) -> Result<DefinitionRegistration, DefinitionRegistryRefusal> {
        if let Some((_, known)) = self.entries.iter().find(|(key, _)| key == identity) {
            if known == content {
                return Ok(DefinitionRegistration::Replayed);
            }
            return Err(DefinitionRegistryRefusal {
                identity: Box::new(identity.clone()),
            });
        }
        Ok(DefinitionRegistration::Inserted)
    }

    pub(super) fn commit(
        &mut self,
        identity: DefinitionIdentity,
        content: SemanticContent,
        disposition: DefinitionRegistration,
    ) {
        if disposition == DefinitionRegistration::Inserted {
            self.entries.push((identity, content));
        }
    }

    /// Number of exact definition keys retained.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the registry is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Exact static channel key used by FIFO partitioning.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct StaticChannelKey {
    /// Complete qualified definition identity.
    pub definition: DefinitionIdentity,
    /// Declaration-local channel-table index.
    pub channel: u32,
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use ix_trace_rs::trace;
    use quire_spec_language::protocol_artifact::{handoff, wire as w};

    use super::*;
    use crate::link::{
        admit_binding_definition, BindingDefinitionOffer, ExpectedBindingDefinition,
        RoleClassification, RoleClassificationOffer, RoleInternality, BINDING_DEFINITION_CONTRACT,
    };

    fn package() -> v2::wire::Package {
        let bytes =
            fs::read(Path::new(handoff::PUBLISHED_HANDOFF).join(handoff::PUBLISHED_OFFER_FILE))
                .expect("published v2 package");
        serde_json::from_slice(&bytes).expect("typed v2 wire package")
    }

    fn protocol(package: &v2::wire::Package) -> (u32, &w::Declaration) {
        package
            .inherited
            .declarations
            .iter()
            .enumerate()
            .find_map(|(index, declaration)| {
                matches!(declaration.body, w::Body::Protocol { .. })
                    .then(|| Some((u32::try_from(index).ok()?, declaration)))?
            })
            .expect("one protocol declaration")
    }

    fn artifact() -> BindingArtifactIdentity {
        BindingArtifactIdentity {
            identity: "test:definition-binding".into(),
            revision: w::Revision {
                namespace: "test:binding-revision".into(),
                value: "1".into(),
            },
            digest: w::SelectedDigest {
                domain: "test:binding".into(),
                version: "1".into(),
                algorithm: "sha256".into(),
                value: format!("sha256:{}", "a".repeat(64)),
            },
        }
    }

    fn correspondence() -> w::Correspondence {
        w::Correspondence {
            producer: w::ProducerObject {
                interface: 0,
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
            native: 0,
            relation: 1,
            exports: vec![0],
        }
    }

    fn binding(declaration: u32, protocol: &w::Declaration) -> AcceptedBindingDefinition {
        let role_count = match &protocol.body {
            w::Body::Protocol { roles, .. } => roles.len(),
            _ => 0,
        };
        let roles: Vec<_> = (0..role_count)
            .map(|index| RoleClassification {
                role: u32::try_from(index).expect("role index"),
                internality: if index % 2 == 0 {
                    RoleInternality::Internal
                } else {
                    RoleInternality::External
                },
            })
            .collect();
        let expected = ExpectedBindingDefinition {
            artifact: artifact(),
            declaration,
            roles: roles.clone(),
            slots: Vec::new(),
        };
        admit_binding_definition(
            BindingDefinitionOffer {
                contract: BINDING_DEFINITION_CONTRACT.into(),
                artifact: artifact(),
                declaration,
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
                slots: Vec::new(),
            },
            &expected,
        )
        .expect("exact static binding")
    }

    fn assert_correspondence_change(
        baseline: &DefinitionIdentity,
        axis: &str,
        mutate: impl FnOnce(&mut w::Correspondence),
    ) {
        let mut changed = baseline.clone();
        let correspondence = changed.closure.models[0]
            .value
            .correspondence
            .0
            .as_mut()
            .expect("published correspondence");
        mutate(correspondence);
        assert_ne!(baseline, &changed, "correspondence axis {axis}");
    }

    #[trace("TC-002", "FR-002-AC-1", "FR-002-AC-4")]
    #[test]
    fn tc_002_fr288_retains_each_component_and_selected_closure() {
        let package = package();
        let (declaration, protocol) = protocol(&package);
        let binding = binding(declaration, protocol);
        let identity = DefinitionIdentity::from_package(&package, declaration, &binding)
            .expect("componentwise identity");

        assert_eq!(identity.declaration().declaration, declaration);
        assert_eq!(identity.declaration().source, protocol.locus.source);
        assert_eq!(&identity.owner().requirement, &protocol.requirement);
        assert_eq!(identity.owner().clause, protocol.clause);
        assert_eq!(identity.closure().language, package.inherited.language);
        assert_eq!(identity.closure().profile.definition, protocol.profile);
        assert_eq!(identity.closure().binding_definition, *binding.artifact());
        assert!(identity
            .closure()
            .definitions
            .iter()
            .any(|entry| entry.index == protocol.profile));
        assert!(identity
            .closure()
            .dependencies
            .iter()
            .any(|entry| entry.index == identity.closure().profile.value.artifact));

        let selected_definitions: BTreeSet<_> = identity
            .closure()
            .definitions
            .iter()
            .map(|entry| entry.index)
            .collect();
        let reachable = reachable_declarations(declaration, &package.inherited.declarations)
            .expect("finite declaration closure");
        for index in reachable {
            let selected =
                &package.inherited.declarations[usize::try_from(index).expect("declaration index")];
            assert!(selected_definitions.contains(&selected.profile));
            for value in &selected.values {
                assert!(selected_definitions.contains(&value.profile));
            }
            if let w::Body::Protocol {
                compensations,
                controls,
                ..
            } = &selected.body
            {
                for compensation in compensations {
                    assert!(selected_definitions.contains(&compensation.profile));
                }
                for control in controls {
                    if let w::ControlOperation::Check { profile, .. } = control.operation {
                        assert!(selected_definitions.contains(&profile));
                    }
                }
            }
        }
        assert!(identity
            .closure()
            .temporal_bindings
            .iter()
            .any(|selection| selection.declaration == 5 && selection.definition == 10));
    }

    #[trace("TC-002", "FR-002-AC-3", "FR-002-AC-4")]
    #[test]
    fn tc_002_reachable_declaration_content_is_consistency_content() {
        let package = package();
        let (declaration, protocol) = protocol(&package);
        let binding = binding(declaration, protocol);
        let baseline = SemanticContent::from_package(&package, declaration, &binding)
            .expect("semantic content");
        let dependency = *protocol
            .requires
            .first()
            .expect("published protocol has a predicate dependency");
        assert!(baseline
            .dependencies()
            .iter()
            .any(|entry| entry.index == dependency));

        let mut semantic = package.clone();
        semantic.inherited.declarations[usize::try_from(dependency).expect("declaration index")]
            .clause
            .push_str(":changed-dependency");
        let changed = SemanticContent::from_package(&semantic, declaration, &binding)
            .expect("changed dependency content");
        assert_ne!(baseline, changed);

        let mut presentation = package.clone();
        let dependency = &mut presentation.inherited.declarations
            [usize::try_from(dependency).expect("declaration index")];
        dependency.name.push_str(":display-only");
        dependency.locus.span.start = dependency.locus.span.start.saturating_add(1);
        let changed = SemanticContent::from_package(&presentation, declaration, &binding)
            .expect("dependency presentation content");
        assert_eq!(baseline, changed);
    }

    #[trace("TC-002", "FR-002-AC-4")]
    #[test]
    fn tc_002_each_fr288_component_and_selection_axis_changes_equality() {
        let package = package();
        let (declaration, protocol) = protocol(&package);
        let binding = binding(declaration, protocol);
        let baseline = DefinitionIdentity::from_package(&package, declaration, &binding)
            .expect("componentwise identity");

        let mut changed = baseline.clone();
        changed.source.digest = ByteDigest::of(b"another source document");
        assert_ne!(baseline, changed, "source-document key");

        let mut changed = baseline.clone();
        changed.declaration.declaration = changed
            .declaration
            .declaration
            .checked_add(1)
            .expect("small fixture");
        assert_ne!(baseline, changed, "declaration handle");

        let mut changed = baseline.clone();
        changed.owner.clause.push_str(":another-clause");
        assert_ne!(baseline, changed, "formal clause");

        let mut changed = baseline.clone();
        changed
            .owner
            .requirement
            .identity
            .push_str(":another-owner");
        assert_ne!(baseline, changed, "formal owner");

        let mut changed = baseline.clone();
        changed
            .closure
            .language
            .edition
            .push_str(":another-edition");
        assert_ne!(baseline, changed, "language selection");

        let mut changed = baseline.clone();
        changed
            .closure
            .profile
            .value
            .identity
            .push_str(":another-profile");
        assert_ne!(baseline, changed, "profile selection");

        let mut changed = baseline.clone();
        changed.closure.definitions[0]
            .value
            .identity
            .push_str(":another-definition");
        assert_ne!(baseline, changed, "definition closure");

        let mut changed = baseline.clone();
        changed.closure.dependencies[0]
            .value
            .artifact
            .identity
            .push_str(":another-dependency");
        assert_ne!(baseline, changed, "dependency closure");

        let mut changed = baseline.clone();
        changed.closure.models[0]
            .value
            .profile
            .push_str(":another-model");
        assert_ne!(baseline, changed, "model selection");

        let mut changed = baseline.clone();
        changed.closure.types.swap(0, 1);
        assert_ne!(baseline, changed, "selected type closure");

        let mut changed = baseline.clone();
        changed.closure.temporal_bindings[0].definition += 1;
        assert_ne!(baseline, changed, "temporal profile selection");

        let mut changed = baseline.clone();
        let clock = &mut changed.closure.temporal_bindings[0].clock;
        *clock = match clock {
            v2::wire::ClockConfiguration::TimestampedEvent { .. } => {
                v2::wire::ClockConfiguration::EventPosition {
                    sequence_authority: "another-clock-alternative".into(),
                }
            }
            v2::wire::ClockConfiguration::EventPosition { .. }
            | v2::wire::ClockConfiguration::FixedSample { .. } => {
                v2::wire::ClockConfiguration::TimestampedEvent {
                    timestamp_unit: "another-clock-alternative".into(),
                }
            }
        };
        assert_ne!(baseline, changed, "temporal clock alternative");

        let mut changed = baseline.clone();
        match &mut changed.closure.temporal_bindings[0].clock {
            v2::wire::ClockConfiguration::EventPosition { sequence_authority } => {
                sequence_authority.push_str(":another-parameter");
            }
            v2::wire::ClockConfiguration::FixedSample { unit, .. } => {
                unit.push_str(":another-parameter");
            }
            v2::wire::ClockConfiguration::TimestampedEvent { timestamp_unit } => {
                timestamp_unit.push_str(":another-parameter");
            }
        }
        assert_ne!(baseline, changed, "temporal clock parameter");

        let mut changed = baseline.clone();
        changed
            .closure
            .binding_definition
            .identity
            .push_str(":another-binding");
        assert_ne!(baseline, changed, "static binding selection");
    }

    #[trace("TC-002", "FR-002-AC-4")]
    #[test]
    fn tc_002_each_correspondence_component_changes_fr288() {
        let mut package = package();
        package.inherited.models[0].correspondence = w::Nullable(Some(correspondence()));
        let (declaration, protocol) = protocol(&package);
        let binding = binding(declaration, protocol);
        let baseline = DefinitionIdentity::from_package(&package, declaration, &binding)
            .expect("componentwise identity");

        assert_correspondence_change(&baseline, "interface", |value| {
            value.producer.interface += 1;
        });
        assert_correspondence_change(&baseline, "producer-kind", |value| {
            value.producer.kind.push_str(":other");
        });
        assert_correspondence_change(&baseline, "producer-authority", |value| {
            value.producer.authority.push_str(":other");
        });
        assert_correspondence_change(&baseline, "producer-identity", |value| {
            value.producer.identity.push_str(":other");
        });
        assert_correspondence_change(&baseline, "producer-revision-namespace", |value| {
            value.producer.revision.namespace.push_str(":other");
        });
        assert_correspondence_change(&baseline, "producer-revision-value", |value| {
            value.producer.revision.value.push_str(":other");
        });
        assert_correspondence_change(&baseline, "digest-domain", |value| {
            value.producer.digest.domain.push_str(":other");
        });
        assert_correspondence_change(&baseline, "digest-version", |value| {
            value.producer.digest.version.push_str(":other");
        });
        assert_correspondence_change(&baseline, "digest-algorithm", |value| {
            value.producer.digest.algorithm.push_str(":other");
        });
        assert_correspondence_change(&baseline, "digest-value", |value| {
            value.producer.digest.value.push('0');
        });
        assert_correspondence_change(&baseline, "native", |value| {
            value.native += 1;
        });
        assert_correspondence_change(&baseline, "relation", |value| {
            value.relation += 1;
        });
        assert_correspondence_change(&baseline, "exports", |value| {
            value.exports.push(0);
        });
    }

    #[trace("TC-002", "FR-002-AC-4")]
    #[test]
    fn tc_002_external_paths_spans_and_build_provenance_do_not_change_fr288() {
        let package = package();
        let (declaration, protocol) = protocol(&package);
        let binding = binding(declaration, protocol);
        let baseline = DefinitionIdentity::from_package(&package, declaration, &binding)
            .expect("componentwise identity");
        let baseline_presentation = PresentationContext::from_package(&package, &baseline)
            .expect("exact presentation context");

        let mut presentation = package.clone();
        presentation.inherited.producer.implementation = "another-build".into();
        let source = &mut presentation.inherited.sources
            [usize::try_from(protocol.locus.source).expect("source index")];
        source.path = "another/external/path.quire".into();
        source.native.identity = "another-native-label".into();
        source.native.revision = "another-native-revision".into();
        for export in &mut presentation.inherited.models[0].exports {
            export.locus.span.start = export.locus.span.start.saturating_add(1);
            export.locus.span.end = export.locus.span.end.saturating_add(1);
        }
        let changed = DefinitionIdentity::from_package(&presentation, declaration, &binding)
            .expect("presentation-only identity");
        let changed_presentation = PresentationContext::from_package(&presentation, &changed)
            .expect("changed presentation context");

        assert_eq!(baseline, changed);
        assert_ne!(baseline_presentation, changed_presentation);
        assert_eq!(
            changed_presentation.producer().implementation,
            "another-build"
        );
        assert_eq!(
            changed_presentation
                .sources()
                .iter()
                .find(|source| source.index == protocol.locus.source)
                .expect("selected source")
                .value
                .path,
            "another/external/path.quire"
        );
        assert_eq!(
            changed_presentation.models()[0].value.exports[0]
                .locus
                .span
                .start,
            presentation.inherited.models[0].exports[0].locus.span.start
        );
    }

    #[trace("TC-002", "FR-002-AC-4")]
    #[test]
    fn tc_002_presentation_is_excluded_but_typed_semantics_are_not() {
        let package = package();
        let (declaration, protocol) = protocol(&package);
        let binding = binding(declaration, protocol);
        let baseline = SemanticContent::from_package(&package, declaration, &binding)
            .expect("semantic content");

        let mut presentation = package.clone();
        let changed = &mut presentation.inherited.declarations
            [usize::try_from(declaration).expect("declaration index")];
        changed.name = "Another display name".into();
        changed.locus = w::Locus {
            source: 99,
            span: w::Span {
                start: 900,
                end: 999,
            },
        };
        let presentation_content =
            SemanticContent::from_package(&presentation, declaration, &binding)
                .expect("presentation projection");
        assert_eq!(baseline, presentation_content);

        let mut semantic = package.clone();
        let w::Body::Protocol { finish, .. } = &mut semantic.inherited.declarations
            [usize::try_from(declaration).expect("declaration index")]
        .body
        else {
            panic!("protocol body")
        };
        finish.closure = finish.closure.checked_add(1).expect("small fixture");
        let semantic_content = SemanticContent::from_package(&semantic, declaration, &binding)
            .expect("changed semantic content");
        assert_ne!(baseline, semantic_content);
    }

    #[trace("TC-002", "FR-002-AC-4")]
    #[test]
    fn tc_002_each_typed_declaration_content_family_changes_consistency_content() {
        let package = package();
        let (declaration, protocol) = protocol(&package);
        let binding = binding(declaration, protocol);
        let baseline = SemanticContent::from_package(&package, declaration, &binding)
            .expect("semantic content");
        let declaration_index = usize::try_from(declaration).expect("declaration index");

        let changed = |name: &str, mutate: fn(&mut w::Declaration)| {
            let mut package = package.clone();
            mutate(&mut package.inherited.declarations[declaration_index]);
            let changed = SemanticContent::from_package(&package, declaration, &binding)
                .expect("changed semantic content");
            assert_ne!(baseline, changed, "typed content family: {name}");
        };

        changed("scope", |value| {
            value.scopes[0].parent = w::Nullable(Some(w::Handle {
                declaration: 8,
                index: 1,
            }));
        });
        changed("anchor", |value| {
            value.anchors[0].kind = w::AnchorKind::Current;
        });
        changed("binder", |value| {
            value.binders[0].kind = w::BinderKind::Parameter;
        });
        changed("value", |value| {
            value.values[0].original_expression += 1;
        });
        changed("activation", |value| {
            let w::Body::Protocol { activation, .. } = &mut value.body else {
                panic!("protocol body")
            };
            let w::Activation::Origin { anchor } = activation else {
                panic!("origin activation")
            };
            anchor.index += 1;
        });
        changed("role", |value| {
            let w::Body::Protocol { roles, .. } = &mut value.body else {
                panic!("protocol body")
            };
            roles[0].instance += 1;
        });
        changed("relationship", |value| {
            let w::Body::Protocol {
                roles,
                relationships,
                ..
            } = &mut value.body
            else {
                panic!("protocol body")
            };
            relationships.push(w::Relationship {
                name: "semantic relationship".into(),
                model: roles[0].model.clone(),
                binding: 0,
                locus: value.locus.clone(),
            });
        });
        changed("channel", |value| {
            let w::Body::Protocol { channels, .. } = &mut value.body else {
                panic!("protocol body")
            };
            channels[0].message_type += 1;
        });
        changed("compensation", |value| {
            let w::Body::Protocol { compensations, .. } = &mut value.body else {
                panic!("protocol body")
            };
            compensations[0].attempt_instance += 1;
        });
        changed("temporal requirement", |value| {
            let w::Body::Protocol {
                temporal_requirements,
                ..
            } = &mut value.body
            else {
                panic!("protocol body")
            };
            temporal_requirements.push(6);
        });
        changed("control", |value| {
            let w::Body::Protocol { controls, .. } = &mut value.body else {
                panic!("protocol body")
            };
            controls[0].original_node += 1;
        });
        changed("causal edge", |value| {
            let w::Body::Protocol { causal_edges, .. } = &mut value.body else {
                panic!("protocol body")
            };
            causal_edges[0].from.port = w::Port::Exit;
        });
        changed("guard", |value| {
            let w::Body::Protocol { compensations, .. } = &mut value.body else {
                panic!("protocol body")
            };
            compensations[0].guard.index += 1;
        });
        changed("constraint", |value| {
            let w::Body::Protocol { finish, .. } = &mut value.body else {
                panic!("protocol body")
            };
            finish.constraint.index += 1;
        });
        changed("event", |value| {
            let w::Body::Protocol { controls, .. } = &mut value.body else {
                panic!("protocol body")
            };
            let event = controls.iter_mut().find_map(|control| {
                if let w::ControlOperation::Event { event, .. } = &mut control.operation {
                    Some(event)
                } else {
                    None
                }
            });
            let Some(w::Event::Send { channel }) = event else {
                panic!("send event")
            };
            channel.index += 1;
        });
        changed("run", |value| {
            let w::Body::Protocol { run, .. } = &mut value.body else {
                panic!("protocol body")
            };
            run.index += 1;
        });
        changed("finish", |value| {
            let w::Body::Protocol { finish, .. } = &mut value.body else {
                panic!("protocol body")
            };
            finish.closure += 1;
        });

        let mut await_package = package.clone();
        let await_declaration = &mut await_package.inherited.declarations[declaration_index];
        let w::Body::Protocol {
            compensations,
            controls,
            ..
        } = &mut await_declaration.body
        else {
            panic!("protocol body")
        };
        controls[0].operation = w::ControlOperation::Await {
            after: w::AwaitAnchor::Event {
                node: w::Handle {
                    declaration,
                    index: 1,
                },
            },
            profile: compensations[0].profile,
            clock: compensations[0].clock,
            within: compensations[0].within.clone(),
            event: w::Handle {
                declaration,
                index: 1,
            },
            then_body: w::Handle {
                declaration,
                index: 2,
            },
            timeout: w::Handle {
                declaration,
                index: 3,
            },
        };
        let await_baseline = SemanticContent::from_package(&await_package, declaration, &binding)
            .expect("await content");

        let mut changed_await = await_package.clone();
        let w::Body::Protocol { controls, .. } =
            &mut changed_await.inherited.declarations[declaration_index].body
        else {
            panic!("protocol body")
        };
        let w::ControlOperation::Await { within, .. } = &mut controls[0].operation else {
            panic!("await control")
        };
        within.upper = within.lower.clone();
        let changed_await = SemanticContent::from_package(&changed_await, declaration, &binding)
            .expect("changed await interval");
        assert_ne!(await_baseline, changed_await, "await interval");

        let mut changed_await = await_package;
        let w::Body::Protocol { controls, .. } =
            &mut changed_await.inherited.declarations[declaration_index].body
        else {
            panic!("protocol body")
        };
        let w::ControlOperation::Await { then_body, .. } = &mut controls[0].operation else {
            panic!("await control")
        };
        then_body.index += 1;
        let changed_await = SemanticContent::from_package(&changed_await, declaration, &binding)
            .expect("changed await branch");
        assert_ne!(await_baseline, changed_await, "await branch");
    }

    #[trace("TC-002", "FR-002-AC-3", "FR-002-AC-4")]
    #[test]
    fn tc_002_registry_replays_exactly_and_refuses_contradiction_without_mutation() {
        let package = package();
        let (declaration, protocol) = protocol(&package);
        let binding = binding(declaration, protocol);
        let identity =
            DefinitionIdentity::from_package(&package, declaration, &binding).expect("identity");
        let content =
            SemanticContent::from_package(&package, declaration, &binding).expect("content");
        let mut registry = DefinitionRegistry::default();

        let first = registry.inspect(&identity, &content).expect("new key");
        assert_eq!(first, DefinitionRegistration::Inserted);
        registry.commit(identity.clone(), content.clone(), first);
        let replay = registry.inspect(&identity, &content).expect("exact replay");
        assert_eq!(replay, DefinitionRegistration::Replayed);
        registry.commit(identity.clone(), content, replay);
        assert_eq!(registry.len(), 1);

        let mut changed = package.clone();
        let declaration_value = &mut changed.inherited.declarations
            [usize::try_from(declaration).expect("declaration index")];
        declaration_value.clause.push_str(":conflict");
        let contradictory = SemanticContent::from_package(&changed, declaration, &binding)
            .expect("contradictory content");
        let refusal = registry
            .inspect(&identity, &contradictory)
            .expect_err("same key, different content");
        assert_eq!(*refusal.identity, identity);
        assert_eq!(registry.len(), 1, "inspection cannot mutate the registry");
    }
}
