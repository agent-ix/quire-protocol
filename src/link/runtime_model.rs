// SPDX-License-Identifier: AGPL-3.0-or-later
//! Transactional version-2 protocol linking over exact QSL and QObs values.

use std::collections::BTreeSet;

use quire_spec_language::protocol_artifact::{v2, wire};

use super::{
    binding::{
        validate_runtime, validate_static_definition, AcceptedBindingDefinition,
        BindingDiagnostics, LinkedRuntimeBindings, RuntimeBindingSet,
    },
    channel_premises, declaration_identity,
    definition::{
        DefinitionIdentity, DefinitionRegistration, DefinitionRegistry, DefinitionRegistryRefusal,
        Handled, Indexed, PresentationContext, SemanticContent, StaticChannelKey,
    },
    fifo::FifoPartition,
    link_declarations_v2, ChannelPremises, ControlShape, EdgeShape, StaticDiagnostics,
    StaticLinkedProtocol,
};

/// A complete FIFO expression definition resolved from owner-typed QSL tables.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FifoDefinition {
    binder: Handled<wire::Binder>,
    key: Handled<wire::Value>,
    anchor: Handled<wire::Anchor>,
    values: Vec<Handled<wire::Value>>,
    binders: Vec<Handled<wire::Binder>>,
    anchors: Vec<Handled<wire::Anchor>>,
    scopes: Vec<Handled<wire::Scope>>,
    types: Vec<Indexed<wire::Type>>,
}

impl FifoDefinition {
    /// Exact declaration-qualified FIFO binder definition.
    #[must_use]
    pub const fn binder(&self) -> &Handled<wire::Binder> {
        &self.binder
    }

    /// Exact declaration-qualified key expression definition.
    #[must_use]
    pub const fn key(&self) -> &Handled<wire::Value> {
        &self.key
    }

    /// Exact declaration-qualified FIFO evaluation anchor.
    #[must_use]
    pub const fn anchor(&self) -> &Handled<wire::Anchor> {
        &self.anchor
    }

    /// Complete reachable value-expression graph in handle order.
    #[must_use]
    pub fn values(&self) -> &[Handled<wire::Value>] {
        &self.values
    }

    /// Complete reachable binder definitions in handle order.
    #[must_use]
    pub fn binders(&self) -> &[Handled<wire::Binder>] {
        &self.binders
    }

    /// Complete reachable evaluation anchors in handle order.
    #[must_use]
    pub fn anchors(&self) -> &[Handled<wire::Anchor>] {
        &self.anchors
    }

    /// Complete reachable lexical scopes in handle order.
    #[must_use]
    pub fn scopes(&self) -> &[Handled<wire::Scope>] {
        &self.scopes
    }

    /// Complete selected type closure of the key expression.
    #[must_use]
    pub fn types(&self) -> &[Indexed<wire::Type>] {
        &self.types
    }
}

/// One exact static channel and its five FR-216 premises.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinkedChannelDefinition {
    key: StaticChannelKey,
    premises: ChannelPremises,
    fifo: Option<FifoDefinition>,
}

impl LinkedChannelDefinition {
    /// Complete qualified definition identity owning this channel.
    #[must_use]
    pub const fn definition(&self) -> &DefinitionIdentity {
        &self.key.definition
    }

    /// Declaration-local channel-table index.
    #[must_use]
    pub const fn channel(&self) -> u32 {
        self.key.channel
    }

    /// Exact five-part channel premise set.
    #[must_use]
    pub const fn premises(&self) -> &ChannelPremises {
        &self.premises
    }

    /// Exact FIFO expression definition, or explicit absence for unordered channels.
    #[must_use]
    pub const fn fifo(&self) -> Option<&FifoDefinition> {
        self.fifo.as_ref()
    }

    /// Pair this FIFO channel with the value owner's exact evaluated key value.
    ///
    /// This operation never adds a causal edge. An unordered channel refuses
    /// rather than inventing a FIFO partition.
    pub fn partition<V>(&self, value: V) -> Result<FifoPartition<V>, PartitionRefusal> {
        if self.fifo.is_none() {
            return Err(PartitionRefusal::UnorderedChannel {
                channel: self.key.channel,
            });
        }
        Ok(FifoPartition::new(self.key.clone(), value))
    }
}

/// Why no FIFO runtime partition was constructed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PartitionRefusal {
    /// The exact selected channel declares unordered delivery.
    UnorderedChannel { channel: u32 },
}

/// Finite admitted version-2 control topology.
///
/// This is deliberately distinct from [`StaticLinkedProtocol`]. That frozen
/// version-1 value exposes historical `*InstanceId` values derived from static
/// binding-requirement coordinates. Version 2 keeps static requirements and
/// concrete runtime subjects non-interchangeable: this graph retains only the
/// compiler-validated control structure, while [`LinkedProtocol::runtime`]
/// carries every constructor-guarded runtime identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinkedGraph {
    edges: Vec<EdgeShape>,
    controls: Vec<ControlShape>,
    finish_closure: u32,
}

impl LinkedGraph {
    fn from_static(graph: &StaticLinkedProtocol) -> Self {
        Self {
            edges: graph.edges().to_vec(),
            controls: graph.controls().to_vec(),
            finish_closure: graph.finish_closure(),
        }
    }

    /// Compiler-validated causal edges in the producer's declared order.
    #[must_use]
    pub fn edges(&self) -> &[EdgeShape] {
        &self.edges
    }

    /// Compiler-validated controls in declaration-table order.
    #[must_use]
    pub fn controls(&self) -> &[ControlShape] {
        &self.controls
    }

    /// Static binding-requirement index selected by the finish closure.
    #[must_use]
    pub const fn finish_closure(&self) -> u32 {
        self.finish_closure
    }
}

/// Fully linked version-2 protocol.
#[derive(Clone, Debug)]
pub struct LinkedProtocol {
    identity: DefinitionIdentity,
    content: SemanticContent,
    presentation: PresentationContext,
    static_binding: AcceptedBindingDefinition,
    runtime: LinkedRuntimeBindings,
    graph: LinkedGraph,
    channels: Vec<LinkedChannelDefinition>,
    registration: DefinitionRegistration,
}

impl PartialEq for LinkedProtocol {
    fn eq(&self, other: &Self) -> bool {
        self.identity == other.identity
            && self.content == other.content
            && self.static_binding == other.static_binding
            && self.runtime == other.runtime
            && self.graph == other.graph
            && self.channels == other.channels
            && self.registration == other.registration
    }
}

impl Eq for LinkedProtocol {}

impl LinkedProtocol {
    /// Complete componentwise FR-288 identity.
    #[must_use]
    pub const fn identity(&self) -> &DefinitionIdentity {
        &self.identity
    }

    /// Complete typed static consistency content.
    #[must_use]
    pub const fn content(&self) -> &SemanticContent {
        &self.content
    }

    /// Exact source locations and presentation/build provenance.
    ///
    /// This context is retained for inspection but does not participate in
    /// FR-288 or [`LinkedProtocol`] semantic equality.
    #[must_use]
    pub const fn presentation(&self) -> &PresentationContext {
        &self.presentation
    }

    /// Exact accepted static role and slot compatibility selection.
    #[must_use]
    pub const fn static_binding(&self) -> &AcceptedBindingDefinition {
        &self.static_binding
    }

    /// Exact constructor-guarded runtime subjects and relationships.
    #[must_use]
    pub const fn runtime(&self) -> &LinkedRuntimeBindings {
        &self.runtime
    }

    /// Finite retained causal/control graph.
    #[must_use]
    pub const fn graph(&self) -> &LinkedGraph {
        &self.graph
    }

    /// Exact channels in declaration-table order.
    #[must_use]
    pub fn channels(&self) -> &[LinkedChannelDefinition] {
        &self.channels
    }

    /// Whether this operation inserted the definition or replayed it exactly.
    #[must_use]
    pub const fn registration(&self) -> DefinitionRegistration {
        self.registration
    }
}

/// Exact version-2 linker refusal. Every alternative returns no linked value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LinkDiagnostics {
    /// Existing finite static-graph checks refused.
    Static(StaticDiagnostics),
    /// Static/runtime binding checks refused.
    Binding(BindingDiagnostics),
    /// An admitted graph did not expose a required typed member.
    IncompleteAdmittedGraph {
        /// Exact graph axis that could not be retained.
        axis: &'static str,
        /// Declaration-local channel index when the failure belongs to a channel.
        channel: Option<u32>,
        /// Real source location when the failed member has one.
        locus: Option<wire::Locus>,
    },
    /// One FR-288 key was paired with contradictory static content.
    DefinitionContradiction(DefinitionRegistryRefusal),
}

/// Link an admitted version-2 package transactionally.
///
/// The registry is inspected before the success value is built and committed
/// only after every static, runtime, identity, channel, and FIFO check succeeds.
/// Therefore every refusal leaves both registry and result state unchanged.
pub fn link(
    package: &v2::AdmittedPackage,
    binding: &AcceptedBindingDefinition,
    runtime: RuntimeBindingSet,
    registry: &mut DefinitionRegistry,
) -> Result<LinkedProtocol, LinkDiagnostics> {
    let declaration_index = binding.declaration();
    let static_graph = link_declarations_v2(&package.inherited().declarations, declaration_index)
        .map_err(LinkDiagnostics::Static)?;
    let graph = LinkedGraph::from_static(&static_graph);
    let declaration = package
        .inherited()
        .declarations
        .get(usize::try_from(declaration_index).unwrap_or(usize::MAX))
        .ok_or(LinkDiagnostics::IncompleteAdmittedGraph {
            axis: "declaration",
            channel: None,
            locus: None,
        })?;
    validate_static_definition(declaration, &package.inherited().models, binding)
        .map_err(LinkDiagnostics::Binding)?;

    let scope = declaration_identity(declaration_index, declaration);
    let runtime = validate_runtime(declaration, &scope, binding, runtime)
        .map_err(LinkDiagnostics::Binding)?;
    let identity = DefinitionIdentity::from_package(package.package(), declaration_index, binding)
        .ok_or(LinkDiagnostics::IncompleteAdmittedGraph {
            axis: "definition-identity",
            channel: None,
            locus: Some(declaration.locus.clone()),
        })?;
    let content = SemanticContent::from_package(package.package(), declaration_index, binding)
        .ok_or(LinkDiagnostics::IncompleteAdmittedGraph {
            axis: "semantic-content",
            channel: None,
            locus: Some(declaration.locus.clone()),
        })?;
    let presentation = PresentationContext::from_package(package.package(), &identity).ok_or(
        LinkDiagnostics::IncompleteAdmittedGraph {
            axis: "presentation-context",
            channel: None,
            locus: Some(declaration.locus.clone()),
        },
    )?;
    let channels = build_channels(package.package(), declaration, &identity)?;
    let registration = registry
        .inspect(&identity, &content)
        .map_err(LinkDiagnostics::DefinitionContradiction)?;

    let linked = LinkedProtocol {
        identity: identity.clone(),
        content: content.clone(),
        presentation,
        static_binding: binding.clone(),
        runtime,
        graph,
        channels,
        registration,
    };
    registry.commit(identity, content, registration);
    Ok(linked)
}

fn build_channels(
    package: &v2::wire::Package,
    declaration: &wire::Declaration,
    identity: &DefinitionIdentity,
) -> Result<Vec<LinkedChannelDefinition>, LinkDiagnostics> {
    let wire::Body::Protocol { channels, .. } = &declaration.body else {
        return Err(LinkDiagnostics::IncompleteAdmittedGraph {
            axis: "protocol-body",
            channel: None,
            locus: Some(declaration.locus.clone()),
        });
    };
    channels
        .iter()
        .enumerate()
        .map(|(index, channel)| {
            let channel_index =
                u32::try_from(index).map_err(|_| LinkDiagnostics::IncompleteAdmittedGraph {
                    axis: "channel-index",
                    channel: None,
                    locus: Some(channel.locus.clone()),
                })?;
            let fifo = match &channel.ordering {
                wire::Ordering::Unordered {} => None,
                wire::Ordering::Fifo {
                    binder,
                    key,
                    anchor,
                } => Some(build_fifo(
                    package,
                    binder,
                    key,
                    anchor,
                    channel_index,
                    channel,
                )?),
            };
            Ok(LinkedChannelDefinition {
                key: StaticChannelKey {
                    definition: identity.clone(),
                    channel: channel_index,
                },
                premises: channel_premises(channel),
                fifo,
            })
        })
        .collect()
}

fn build_fifo(
    package: &v2::wire::Package,
    binder: &wire::Handle,
    key: &wire::Handle,
    anchor: &wire::Handle,
    channel_index: u32,
    channel: &wire::Channel,
) -> Result<FifoDefinition, LinkDiagnostics> {
    let binder_value = resolve_table(
        package,
        binder,
        |declaration| &declaration.binders,
        "fifo-binder",
        channel_index,
        channel,
    )?;
    let key_value = resolve_table(
        package,
        key,
        |declaration| &declaration.values,
        "fifo-key",
        channel_index,
        channel,
    )?;
    let anchor_value = resolve_table(
        package,
        anchor,
        |declaration| &declaration.anchors,
        "fifo-anchor",
        channel_index,
        channel,
    )?;
    let dependencies = fifo_dependencies(package, key, binder, anchor).ok_or(
        LinkDiagnostics::IncompleteAdmittedGraph {
            axis: "fifo-expression-closure",
            channel: Some(channel_index),
            locus: Some(channel.locus.clone()),
        },
    )?;
    Ok(FifoDefinition {
        binder: Handled {
            handle: binder.clone(),
            value: semantic_binder(binder_value),
        },
        key: Handled {
            handle: key.clone(),
            value: semantic_value(key_value),
        },
        anchor: Handled {
            handle: anchor.clone(),
            value: semantic_anchor(anchor_value),
        },
        values: dependencies.values,
        binders: dependencies.binders,
        anchors: dependencies.anchors,
        scopes: dependencies.scopes,
        types: dependencies.types,
    })
}

fn empty_locus() -> wire::Locus {
    wire::Locus {
        source: 0,
        span: wire::Span { start: 0, end: 0 },
    }
}

fn semantic_value(value: &wire::Value) -> wire::Value {
    let mut value = value.clone();
    value.locus = empty_locus();
    value.operator_locus = wire::Nullable(None);
    value
}

fn semantic_binder(value: &wire::Binder) -> wire::Binder {
    let mut value = value.clone();
    value.name.clear();
    value.locus = empty_locus();
    value
}

fn semantic_anchor(value: &wire::Anchor) -> wire::Anchor {
    let mut value = value.clone();
    value.locus = empty_locus();
    value
}

fn semantic_scope(value: &wire::Scope) -> wire::Scope {
    let mut value = value.clone();
    value.locus = empty_locus();
    value
}

fn resolve_table<'a, T>(
    package: &'a v2::wire::Package,
    handle: &wire::Handle,
    table: impl FnOnce(&'a wire::Declaration) -> &'a [T],
    axis: &'static str,
    channel_index: u32,
    channel: &wire::Channel,
) -> Result<&'a T, LinkDiagnostics> {
    let declaration = package
        .inherited
        .declarations
        .get(usize::try_from(handle.declaration).unwrap_or(usize::MAX))
        .ok_or_else(|| LinkDiagnostics::IncompleteAdmittedGraph {
            axis,
            channel: Some(channel_index),
            locus: Some(channel.locus.clone()),
        })?;
    table(declaration)
        .get(usize::try_from(handle.index).unwrap_or(usize::MAX))
        .ok_or_else(|| LinkDiagnostics::IncompleteAdmittedGraph {
            axis,
            channel: Some(channel_index),
            locus: Some(channel.locus.clone()),
        })
}

struct FifoDependencies {
    values: Vec<Handled<wire::Value>>,
    binders: Vec<Handled<wire::Binder>>,
    anchors: Vec<Handled<wire::Anchor>>,
    scopes: Vec<Handled<wire::Scope>>,
    types: Vec<Indexed<wire::Type>>,
}

fn fifo_dependencies(
    package: &v2::wire::Package,
    key: &wire::Handle,
    binder: &wire::Handle,
    anchor: &wire::Handle,
) -> Option<FifoDependencies> {
    let mut value_handles = BTreeSet::from([handle_key(key)]);
    let mut pending_values = vec![key.clone()];
    let mut binder_handles = BTreeSet::from([handle_key(binder)]);
    let mut pending_binders = vec![binder.clone()];
    let mut anchor_handles = BTreeSet::from([handle_key(anchor)]);
    let mut scope_handles = BTreeSet::new();

    while !pending_values.is_empty() || !pending_binders.is_empty() {
        while let Some(handle) = pending_values.pop() {
            let value = resolve_wire_table(package, &handle, |declaration| &declaration.values)?;
            scope_handles.insert(handle_key(&value.scope));
            anchor_handles.insert(handle_key(&value.anchor));
            if let wire::Origin::Anchor { anchor } = &value.origin {
                anchor_handles.insert(handle_key(anchor));
            }
            if let wire::Origin::Selected { value } = &value.origin {
                insert_handle(&mut value_handles, &mut pending_values, value);
            }
            collect_value_dependencies(
                &value.operation,
                &mut value_handles,
                &mut pending_values,
                &mut binder_handles,
                &mut pending_binders,
                &mut anchor_handles,
            );
        }
        while let Some(handle) = pending_binders.pop() {
            let value = resolve_wire_table(package, &handle, |declaration| &declaration.binders)?;
            scope_handles.insert(handle_key(&value.scope));
            anchor_handles.insert(handle_key(&value.anchor));
            if let Some(initializer) = &value.initializer.0 {
                insert_handle(&mut value_handles, &mut pending_values, initializer);
            }
        }
    }

    let mut pending_scopes: Vec<_> = scope_handles.iter().copied().map(handle_from_key).collect();
    while let Some(handle) = pending_scopes.pop() {
        let value = resolve_wire_table(package, &handle, |declaration| &declaration.scopes)?;
        if let Some(parent) = &value.parent.0 {
            insert_handle(&mut scope_handles, &mut pending_scopes, parent);
        }
    }

    let mut values = handled_values(package, &value_handles, |declaration| &declaration.values)?;
    for entry in &mut values {
        entry.value = semantic_value(&entry.value);
    }
    let mut binders = handled_values(package, &binder_handles, |declaration| &declaration.binders)?;
    for entry in &mut binders {
        entry.value = semantic_binder(&entry.value);
    }
    let mut anchors = handled_values(package, &anchor_handles, |declaration| &declaration.anchors)?;
    for entry in &mut anchors {
        entry.value = semantic_anchor(&entry.value);
    }
    let mut scopes = handled_values(package, &scope_handles, |declaration| &declaration.scopes)?;
    for entry in &mut scopes {
        entry.value = semantic_scope(&entry.value);
    }
    let mut type_roots = BTreeSet::new();
    type_roots.extend(values.iter().map(|entry| entry.value.value_type));
    type_roots.extend(binders.iter().map(|entry| entry.value.value_type));
    let types = type_closure(package, type_roots)?;

    Some(FifoDependencies {
        values,
        binders,
        anchors,
        scopes,
        types,
    })
}

fn collect_value_dependencies(
    operation: &wire::ValueOperation,
    values: &mut BTreeSet<(u32, u32)>,
    pending_values: &mut Vec<wire::Handle>,
    binders: &mut BTreeSet<(u32, u32)>,
    pending_binders: &mut Vec<wire::Handle>,
    anchors: &mut BTreeSet<(u32, u32)>,
) {
    let mut value = |handle: &wire::Handle| insert_handle(values, pending_values, handle);
    let mut binder = |handle: &wire::Handle| insert_handle(binders, pending_binders, handle);
    match operation {
        wire::ValueOperation::Read { binder: selected } => binder(selected),
        wire::ValueOperation::Group { value: selected }
        | wire::ValueOperation::Field { base: selected, .. }
        | wire::ValueOperation::Unary {
            value: selected, ..
        } => value(selected),
        wire::ValueOperation::Binary { left, right, .. } => {
            value(left);
            value(right);
        }
        wire::ValueOperation::If {
            condition,
            then_value,
            else_value,
        } => {
            value(condition);
            value(then_value);
            value(else_value);
        }
        wire::ValueOperation::Let {
            binder: selected,
            initializer,
            body,
        } => {
            binder(selected);
            value(initializer);
            value(body);
        }
        wire::ValueOperation::Pre {
            value: selected,
            anchor,
        } => {
            value(selected);
            anchors.insert(handle_key(anchor));
        }
        wire::ValueOperation::Call { arguments, .. } => {
            for argument in arguments {
                value(argument);
            }
        }
        wire::ValueOperation::Size { collection, .. } => value(collection),
        wire::ValueOperation::Contains { collection, member } => {
            value(collection);
            value(member);
        }
        wire::ValueOperation::Query {
            binder: selected,
            collection,
            body,
            ..
        } => {
            binder(selected);
            value(collection);
            value(body);
        }
        wire::ValueOperation::Parent { reference, .. } => value(reference),
        wire::ValueOperation::Reaches { start, target, .. } => {
            value(start);
            value(target);
        }
        wire::ValueOperation::Boolean { .. }
        | wire::ValueOperation::Number { .. }
        | wire::ValueOperation::Text { .. }
        | wire::ValueOperation::Enum { .. } => {}
    }
}

fn handle_key(handle: &wire::Handle) -> (u32, u32) {
    (handle.declaration, handle.index)
}

fn handle_from_key((declaration, index): (u32, u32)) -> wire::Handle {
    wire::Handle { declaration, index }
}

fn insert_handle(
    selected: &mut BTreeSet<(u32, u32)>,
    pending: &mut Vec<wire::Handle>,
    handle: &wire::Handle,
) {
    if selected.insert(handle_key(handle)) {
        pending.push(handle.clone());
    }
}

fn handled_values<T: Clone>(
    package: &v2::wire::Package,
    selected: &BTreeSet<(u32, u32)>,
    table: impl Copy + Fn(&wire::Declaration) -> &[T],
) -> Option<Vec<Handled<T>>> {
    selected
        .iter()
        .copied()
        .map(|key| {
            let handle = handle_from_key(key);
            Some(Handled {
                value: resolve_wire_table(package, &handle, table)?.clone(),
                handle,
            })
        })
        .collect()
}

fn resolve_wire_table<'a, T>(
    package: &'a v2::wire::Package,
    handle: &wire::Handle,
    table: impl FnOnce(&'a wire::Declaration) -> &'a [T],
) -> Option<&'a T> {
    let declaration = package
        .inherited
        .declarations
        .get(usize::try_from(handle.declaration).ok()?)?;
    table(declaration).get(usize::try_from(handle.index).ok()?)
}

fn type_closure(
    package: &v2::wire::Package,
    roots: BTreeSet<u32>,
) -> Option<Vec<Indexed<wire::Type>>> {
    let mut selected = roots;
    let mut pending: Vec<_> = selected.iter().copied().collect();
    while let Some(index) = pending.pop() {
        let value = package.inherited.types.get(usize::try_from(index).ok()?)?;
        let nested = match value {
            wire::Type::Option { value } => Some(*value),
            wire::Type::Sequence { element, .. } => Some(*element),
            wire::Type::Boolean {}
            | wire::Type::Scalar { .. }
            | wire::Type::Enum { .. }
            | wire::Type::Record { .. }
            | wire::Type::Object { .. }
            | wire::Type::Reference { .. } => None,
        };
        if let Some(nested) = nested {
            if selected.insert(nested) {
                pending.push(nested);
            }
        }
    }
    selected
        .into_iter()
        .map(|index| {
            Some(Indexed {
                index,
                value: package
                    .inherited
                    .types
                    .get(usize::try_from(index).ok()?)?
                    .clone(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use ix_trace_rs::trace;
    use quire_spec_language::protocol_artifact::{handoff, wire as w};

    use super::*;
    use crate::link::binding::{
        admit_binding_definition, BindingArtifactIdentity, BindingDefinitionOffer,
        ExpectedBindingDefinition, RoleClassification, RoleClassificationOffer, RoleInternality,
        BINDING_DEFINITION_CONTRACT,
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

    fn binding(declaration: u32, protocol: &w::Declaration) -> AcceptedBindingDefinition {
        let role_count = match &protocol.body {
            w::Body::Protocol { roles, .. } => roles.len(),
            _ => 0,
        };
        let artifact = BindingArtifactIdentity {
            identity: "test:fifo-binding".into(),
            revision: w::Revision {
                namespace: "test:binding-revision".into(),
                value: "1".into(),
            },
            digest: w::SelectedDigest {
                domain: "test:binding".into(),
                version: "1".into(),
                algorithm: "sha256".into(),
                value: format!("sha256:{}", "b".repeat(64)),
            },
        };
        let roles: Vec<_> = (0..role_count)
            .map(|index| RoleClassification {
                role: u32::try_from(index).expect("role index"),
                internality: RoleInternality::Internal,
            })
            .collect();
        admit_binding_definition(
            BindingDefinitionOffer {
                contract: BINDING_DEFINITION_CONTRACT.into(),
                artifact: artifact.clone(),
                declaration,
                roles: roles
                    .iter()
                    .map(|role| RoleClassificationOffer {
                        role: role.role,
                        internality: "internal".into(),
                    })
                    .collect(),
                slots: Vec::new(),
            },
            &ExpectedBindingDefinition {
                artifact,
                declaration,
                roles,
                slots: Vec::new(),
            },
        )
        .expect("exact static binding")
    }

    fn fifo_package() -> (v2::wire::Package, u32) {
        let mut package = package();
        let (declaration, _) = protocol(&package);
        let declared = &mut package.inherited.declarations
            [usize::try_from(declaration).expect("declaration index")];
        let w::Body::Protocol { channels, .. } = &mut declared.body else {
            panic!("protocol body")
        };
        channels[0].ordering = w::Ordering::Fifo {
            binder: w::Handle {
                declaration,
                index: 0,
            },
            key: w::Handle {
                declaration,
                index: 0,
            },
            anchor: w::Handle {
                declaration,
                index: 0,
            },
        };
        (package, declaration)
    }

    fn first_fifo(package: &v2::wire::Package, declaration: u32) -> FifoDefinition {
        let protocol = &package.inherited.declarations
            [usize::try_from(declaration).expect("declaration index")];
        let binding = binding(declaration, protocol);
        let identity = DefinitionIdentity::from_package(package, declaration, &binding)
            .expect("definition identity");
        build_channels(package, protocol, &identity).expect("exact channels")[0]
            .fifo()
            .expect("FIFO definition")
            .clone()
    }

    #[trace("TC-002", "FR-002-AC-4")]
    #[test]
    fn tc_002_fifo_retains_expression_closure_and_exact_partition_identity() {
        let (mut package, declaration) = fifo_package();
        let fifo_binder = w::Handle {
            declaration,
            index: 0,
        };
        let fifo_key = w::Handle {
            declaration,
            index: 0,
        };
        let fifo_anchor = w::Handle {
            declaration,
            index: 0,
        };
        let declaration_value = &mut package.inherited.declarations
            [usize::try_from(declaration).expect("declaration index")];
        let w::Body::Protocol { channels, .. } = &mut declaration_value.body else {
            panic!("protocol body")
        };
        channels.push(channels[0].clone());

        let protocol = &package.inherited.declarations
            [usize::try_from(declaration).expect("declaration index")];
        let binding = binding(declaration, protocol);
        let identity = DefinitionIdentity::from_package(&package, declaration, &binding)
            .expect("definition identity");
        let channels = build_channels(&package, protocol, &identity).expect("exact channels");
        let fifo = channels[0].fifo().expect("FIFO definition");

        assert_eq!(fifo.binder().handle, fifo_binder);
        assert_eq!(fifo.key().handle, fifo_key);
        assert_eq!(fifo.anchor().handle, fifo_anchor);
        assert!(fifo.values().iter().any(|entry| entry.handle == fifo_key));
        assert!(fifo.binders().iter().any(|entry| {
            entry.handle
                == w::Handle {
                    declaration,
                    index: 1,
                }
        }));
        assert!(fifo.anchors().iter().any(|entry| {
            entry.handle
                == w::Handle {
                    declaration,
                    index: 2,
                }
        }));
        assert!(fifo.scopes().iter().any(|entry| {
            entry.handle
                == w::Handle {
                    declaration,
                    index: 0,
                }
        }));
        assert!(fifo.types().iter().any(|entry| {
            entry.index == fifo.key().value.value_type
                && entry.value
                    == package.inherited.types
                        [usize::try_from(fifo.key().value.value_type).expect("type index")]
        }));
        assert_eq!(
            channels[0].premises().delivery(),
            &match &protocol.body {
                w::Body::Protocol { channels, .. } => channels[0].delivery.clone(),
                _ => unreachable!("protocol body"),
            }
        );

        let first = channels[0]
            .partition(String::from("partition-a"))
            .expect("FIFO");
        let replay = channels[0]
            .partition(String::from("partition-a"))
            .expect("FIFO");
        let changed_value = channels[0]
            .partition(String::from("partition-b"))
            .expect("FIFO");
        let changed_channel = channels[1]
            .partition(String::from("partition-a"))
            .expect("FIFO");
        assert_eq!(first, replay);
        assert_ne!(first, changed_value);
        assert_ne!(first, changed_channel);
        assert_eq!(first.channel(), 0);
        assert_eq!(changed_channel.channel(), 1);
    }

    #[trace("TC-002", "FR-002-AC-4")]
    #[test]
    fn tc_002_fifo_semantic_dependencies_change_but_presentation_does_not() {
        let (package, declaration) = fifo_package();
        let baseline = first_fifo(&package, declaration);
        let declaration_index = usize::try_from(declaration).expect("declaration index");

        let mut changed = package.clone();
        changed.inherited.declarations[declaration_index].binders[0].kind = w::BinderKind::Capture;
        assert_ne!(baseline, first_fifo(&changed, declaration), "FIFO binder");

        let mut changed = package.clone();
        changed.inherited.declarations[declaration_index].values[0].original_expression += 1;
        assert_ne!(
            baseline,
            first_fifo(&changed, declaration),
            "FIFO key expression"
        );

        let mut changed = package.clone();
        let key_type =
            usize::try_from(changed.inherited.declarations[declaration_index].values[0].value_type)
                .expect("type index");
        changed.inherited.types[key_type] = w::Type::Boolean {};
        assert_ne!(
            baseline,
            first_fifo(&changed, declaration),
            "FIFO selected type"
        );

        let mut changed = package.clone();
        changed.inherited.declarations[declaration_index].binders[1].kind = w::BinderKind::Capture;
        assert_ne!(
            baseline,
            first_fifo(&changed, declaration),
            "FIFO reachable semantic dependency"
        );

        let mut presentation = package.clone();
        let declaration = &mut presentation.inherited.declarations[declaration_index];
        declaration.binders[0].name.push_str(":display");
        declaration.binders[0].locus.span.start += 1;
        declaration.values[0].locus.span.start += 1;
        declaration.values[0].operator_locus = w::Nullable(None);
        declaration.anchors[0].locus.span.start += 1;
        declaration.scopes[0].locus.span.start += 1;
        assert_eq!(
            baseline,
            first_fifo(
                &presentation,
                u32::try_from(declaration_index).expect("small index")
            )
        );
    }

    #[trace("TC-002", "FR-002-AC-4")]
    #[test]
    fn tc_002_unordered_channel_refuses_a_fifo_partition() {
        let package = package();
        let (declaration, protocol) = protocol(&package);
        let binding = binding(declaration, protocol);
        let identity = DefinitionIdentity::from_package(&package, declaration, &binding)
            .expect("definition identity");
        let channels = build_channels(&package, protocol, &identity).expect("exact channels");

        assert_eq!(channels[0].fifo(), None);
        assert_eq!(
            channels[0].partition("not-fifo"),
            Err(PartitionRefusal::UnorderedChannel { channel: 0 })
        );
    }
}
