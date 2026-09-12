// SPDX-License-Identifier: AGPL-3.0-or-later
//! Finite linked-protocol graph. Owner chunk: **C-12** (submodule: C-13).
//!
//! FR-002 requires one finite typed representation of an admitted protocol,
//! with every identity kind kept distinct and with neither input order nor
//! display text participating in what is linked. [`LinkedProtocol`] is that
//! representation, and [`link`] builds it.
//!
//! # What this module does not do, and why
//!
//! FR-002's prose also names a refusal list: duplicate role authorities,
//! unresolved or ambiguous references, missing or invalid joins, causal cycles,
//! unbounded or zero-progress iteration, unreachable termination, conflicting
//! channel premises, and ownerless, multiply owned, unlabeled, overlapping or
//! owner-unobservable choices.
//!
//! **Every one of those is already refused by A, before this module runs.** A's
//! `docs/compiled-protocol-v1.md` states the exact structural edge expansion as
//! a table, and `protocol_artifact::validate::control` implements it inside
//! `read`: it derives the required causal-edge set, compares the offered
//! `causal_edges` element by element, enforces strict ascending order on
//! `(owner, kind, from.node, from.port, to.node, to.port)`, checks each
//! `repeat_progress` maximum, and runs an acyclicity check over every edge
//! except `repeat_progress`. The same file covers owner checks, duplicate
//! names, choice visibility, join indices and the repeat structural integer.
//!
//! So this module takes an [`AdmittedPackage`] — a package that has already
//! passed all of it — and links what A admitted. It does not re-derive A's
//! edges. `CLAUDE.md`: "An owner publishes it → import it. Do not mirror it,
//! not even in tests." A second copy would also be the worse kind of duplicate:
//! if B's derivation differed from A's by one edge, every real artifact would
//! refuse, and the disagreement would sit in the half that is not the owner's.
//!
//! What remains, and is genuinely B's, is the identity space. A's records
//! reference things by table position; B's sixteen newtypes (chunk C-07) keep
//! sixteen kinds mutually unassignable and keep display text out of equality.
//! Mapping one onto the other is this module's whole job, and the refusals
//! below are exactly the ones that mapping can raise.
//!
//! # Where a link-time identity comes from
//!
//! A carries **binding requirements**, not instances: A's own contract says
//! B/D/F's admitted request and producer interfaces supply actual instance
//! identities later. There is therefore no runtime instance identity to retain
//! at link time, and inventing one would be the failure `CLAUDE.md` names.
//!
//! What does exist is the producer's own way of referring to a requirement, and
//! that is what is retained: the owning declaration index, the requirement's
//! index in that declaration's binding table, its declared [`BindingKind`] tag,
//! and the exact authority reference the producer attached to it. Those are
//! producer-supplied and exact. They are encoded canonically by
//! [`requirement_identity`] with length-prefixed members, so no two distinct
//! requirements can encode to the same bytes by concatenation.
//!
//! The authored `name` is **not** part of that encoding. It is carried as
//! display text, because FR-002 refuses a name-only identity binding and
//! FR-002-AC-4 requires presentation changes to leave the linked protocol
//! unchanged. Table position *is* part of it, and that is deliberate: A's own
//! `u32` fields select binding requirements by index, so a reordering of that
//! table changes which requirement every channel and role refers to. It is
//! structure, not presentation.
//!
//! These are requirement identities. When the later request interface supplies
//! runtime instance identities, they extend these rather than replacing them,
//! and nothing here has to be rewritten to accept them.
//!
//! # Equality is the linked identity
//!
//! [`LinkedProtocol`] implements `Eq` and `Hash` over its structural content
//! with display text excluded, so FR-002-AC-4 is checked by comparing two
//! linked values. No digest is minted here: a digest would need a domain, and
//! digest domains belong to `FR-015`'s selected vocabulary rather than to this
//! module.
//!
//! # Boundary
//!
//! Owns the linked representation and the identity mapping. The located
//! diagnostics that accumulate across independent definitions are C-13.
//! Compensation registrations, commits and recovery obligations are linked by
//! the compensation chunks (C-16..C-19) and are deliberately absent here.

/// Located link diagnostics. Owner chunk: **C-13**.
///
/// Will own the accumulation of independently established located diagnostics
/// across definitions, so a refusal can report every independent finding rather
/// than only the first.
pub mod diagnostics;

use std::collections::BTreeMap;
use std::fmt;

use quire_spec_language::protocol_artifact::{
    wire::{BindingKind, BindingRequirement, Body, Declaration, Locus},
    AdmittedPackage,
};

use crate::ids::{
    AttemptId, ChannelId, ComponentId, DefinitionId, DeliveryId, EffectId, EndpointId,
    IdentityKind, MessageId, ParticipantId, ReceiveId, RoleInstanceId, SendId, WorkflowInstanceId,
};

// ---------------------------------------------------------------------------
// Refusals
// ---------------------------------------------------------------------------

/// Which member of a channel's premise set a refusal is about.
///
/// Closed, like every other enum B exports: a wildcard arm downstream would
/// absorb a premise added later, and these are exactly the facts a consumer
/// branches on.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ChannelPremise {
    /// The message occurrence the channel carries.
    Message,
    /// The send event.
    Send,
    /// The receive event.
    Receive,
    /// The delivery of the message.
    Delivery,
}

impl ChannelPremise {
    /// Every premise, in the order A declares them on its channel record.
    pub const ALL: [Self; 4] = [Self::Message, Self::Send, Self::Receive, Self::Delivery];

    /// The stable code naming this premise.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Message => "message",
            Self::Send => "send",
            Self::Receive => "receive",
            Self::Delivery => "delivery",
        }
    }

    /// The binding kind A must have declared for this premise.
    pub const fn required_kind(self) -> BindingKind {
        match self {
            Self::Message => BindingKind::Message,
            Self::Send => BindingKind::Send,
            Self::Receive => BindingKind::Receive,
            Self::Delivery => BindingKind::Delivery,
        }
    }
}

/// Why no linked representation was produced.
///
/// Each variant is a fact about B's identity mapping. None of them restates a
/// check A already performs — see the module documentation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LinkRefusal {
    /// The selected declaration index is outside the admitted package.
    NoSuchDeclaration {
        /// The index that was selected.
        selected: u32,
        /// How many declarations the package carries.
        available: usize,
    },
    /// The selected declaration's body is not a protocol, so there is no
    /// protocol to link. Its body kind is named so a caller can see what it
    /// selected instead.
    NotAProtocol {
        /// The selected declaration index.
        declaration: u32,
        /// The body kind actually found.
        found: &'static str,
    },
    /// A binding-requirement index on a protocol record is outside the owning
    /// declaration's binding table.
    NoSuchRequirement {
        /// The record that referred to it.
        referrer: Referrer,
        /// The index it referred to.
        selected: u32,
        /// How many binding requirements the declaration carries.
        available: usize,
    },
    /// A binding requirement exists but declares a different kind from the one
    /// the referring record requires.
    ///
    /// B cannot construct the right identity newtype from the wrong kind, and
    /// choosing one anyway is exactly the confusion the sixteen types exist to
    /// prevent, so this refuses rather than guessing.
    WrongRequirementKind {
        /// The record that referred to it.
        referrer: Referrer,
        /// The kind the referring record requires.
        required: BindingKind,
        /// The kind the requirement declares.
        found: BindingKind,
        /// The original source region of the referring record.
        locus: Locus,
    },
    /// Two distinct protocol records mapped onto one B identity.
    ///
    /// This is B's own invariant: the sixteen kinds are distinct *within* a
    /// kind as well as across kinds, and a collision would silently merge two
    /// runtime things into one linked node.
    CollidingIdentity {
        /// The kind whose space collided.
        kind: IdentityKind,
        /// The identity bytes that two records both produced.
        identity: Vec<u8>,
    },
}

/// The protocol record that referred to a binding requirement.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Referrer {
    /// A role's later role-instance requirement.
    Role {
        /// Index into the protocol's role table.
        index: usize,
    },
    /// One premise of a channel.
    Channel {
        /// Index into the protocol's channel table.
        index: usize,
        /// Which premise.
        premise: ChannelPremise,
    },
    /// The workflow instance the protocol runs as.
    WorkflowInstance,
}

impl fmt::Display for LinkRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoSuchDeclaration {
                selected,
                available,
            } => write!(
                f,
                "declaration {selected} selected from a package carrying {available}"
            ),
            Self::NotAProtocol { declaration, found } => write!(
                f,
                "declaration {declaration} carries a {found} body, not a protocol"
            ),
            Self::NoSuchRequirement {
                referrer,
                selected,
                available,
            } => write!(
                f,
                "{referrer:?} selects binding requirement {selected} from a table of {available}"
            ),
            Self::WrongRequirementKind {
                referrer,
                required,
                found,
                ..
            } => write!(
                f,
                "{referrer:?} requires a {required:?} binding requirement and found {found:?}"
            ),
            Self::CollidingIdentity { kind, .. } => {
                write!(f, "two records produced one {} identity", kind.tag())
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Identity derivation
// ---------------------------------------------------------------------------

/// The exact producer coordinates of one binding requirement, canonically
/// encoded.
///
/// Members are length-prefixed so that no two distinct requirements can encode
/// to the same bytes by concatenation — without the prefixes, a requirement
/// whose authority identity ends in the next member's opening bytes could
/// collide with a different one.
///
/// The authored `name` is deliberately absent: it is display text, and FR-002
/// refuses a name-only identity binding.
#[must_use]
pub fn requirement_identity(
    declaration: u32,
    index: u32,
    requirement: &BindingRequirement,
) -> Vec<u8> {
    let mut bytes = Vec::new();
    push_u32(&mut bytes, declaration);
    push_u32(&mut bytes, index);
    push_bytes(&mut bytes, requirement.kind.as_str().as_bytes());
    push_bytes(&mut bytes, requirement.authority.authority.as_bytes());
    push_bytes(&mut bytes, requirement.authority.identity.as_bytes());
    push_bytes(
        &mut bytes,
        requirement.authority.revision.namespace.as_bytes(),
    );
    push_bytes(&mut bytes, requirement.authority.revision.value.as_bytes());
    push_bytes(
        &mut bytes,
        requirement.authority.digest.to_string().as_bytes(),
    );
    bytes
}

fn push_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

fn push_bytes(bytes: &mut Vec<u8>, value: &[u8]) {
    // A u64 length prefix, so the encoding stays unambiguous for any input this
    // process can hold.
    let length = u64::try_from(value.len()).unwrap_or(u64::MAX);
    bytes.extend_from_slice(&length.to_be_bytes());
    bytes.extend_from_slice(value);
}

// ---------------------------------------------------------------------------
// The linked representation
// ---------------------------------------------------------------------------

/// One linked channel and its four premises, each on its own identity kind.
///
/// The four are separate types, so a send can never be passed where its paired
/// receive is expected, and a delivery is never its message.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct LinkedChannel {
    message: MessageId,
    send: SendId,
    receive: ReceiveId,
    delivery: DeliveryId,
}

impl LinkedChannel {
    /// The message occurrence this channel carries.
    #[must_use]
    pub const fn message(&self) -> &MessageId {
        &self.message
    }
    /// The send event.
    #[must_use]
    pub const fn send(&self) -> &SendId {
        &self.send
    }
    /// The receive event.
    #[must_use]
    pub const fn receive(&self) -> &ReceiveId {
        &self.receive
    }
    /// The delivery of the message.
    #[must_use]
    pub const fn delivery(&self) -> &DeliveryId {
        &self.delivery
    }
}

/// The finite linked protocol.
///
/// Equality and hashing run over the structure below. Every identity inside
/// excludes its display text from equality by construction (chunk C-07), so two
/// packages differing only in authored names link to equal values — which is
/// FR-002-AC-4's presentation half, checked by comparison rather than asserted.
///
/// Maps are `BTreeMap`, so iteration order is the identity order rather than an
/// insertion order or a hash order: a caller that walks a linked protocol sees
/// the same sequence on every run and on every host.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct LinkedProtocol {
    definition: DefinitionId,
    workflow_instance: Option<WorkflowInstanceId>,
    role_instances: Vec<RoleInstanceId>,
    participants: Vec<ParticipantId>,
    components: Vec<ComponentId>,
    endpoints: Vec<EndpointId>,
    attempts: Vec<AttemptId>,
    effects: Vec<EffectId>,
    channels: BTreeMap<ChannelId, LinkedChannel>,
}

impl LinkedProtocol {
    /// The authored definition this protocol was linked from.
    #[must_use]
    pub const fn definition(&self) -> &DefinitionId {
        &self.definition
    }

    /// The workflow instance the protocol runs as, when one is required.
    ///
    /// `None` is a positive statement that the declaration required no workflow
    /// instance, not a lookup that failed.
    #[must_use]
    pub const fn workflow_instance(&self) -> Option<&WorkflowInstanceId> {
        self.workflow_instance.as_ref()
    }

    /// The role instances, in identity order.
    #[must_use]
    pub fn role_instances(&self) -> &[RoleInstanceId] {
        &self.role_instances
    }

    /// The participants, in identity order.
    #[must_use]
    pub fn participants(&self) -> &[ParticipantId] {
        &self.participants
    }

    /// The components, in identity order.
    #[must_use]
    pub fn components(&self) -> &[ComponentId] {
        &self.components
    }

    /// The endpoints, in identity order.
    #[must_use]
    pub fn endpoints(&self) -> &[EndpointId] {
        &self.endpoints
    }

    /// The operation attempts, in identity order.
    #[must_use]
    pub fn attempts(&self) -> &[AttemptId] {
        &self.attempts
    }

    /// The business effects, in identity order.
    #[must_use]
    pub fn effects(&self) -> &[EffectId] {
        &self.effects
    }

    /// The channels and their premises, in channel identity order.
    #[must_use]
    pub const fn channels(&self) -> &BTreeMap<ChannelId, LinkedChannel> {
        &self.channels
    }
}

// ---------------------------------------------------------------------------
// The linker
// ---------------------------------------------------------------------------

/// Link the protocol declared at `declaration` in an admitted package.
///
/// The package has already been admitted by A, which is where the structural
/// refusals live; see the module documentation for what this does and does not
/// check. A refusal here returns no linked representation at all, never a
/// partially built one.
pub fn link(package: &AdmittedPackage, declaration: u32) -> Result<LinkedProtocol, LinkRefusal> {
    link_declarations(&package.package().declarations, declaration)
}

/// The mapping itself, over the declaration table of an already admitted
/// package.
///
/// Private on purpose: its input is a bare declaration slice, which carries no
/// evidence of having been admitted. [`link`] is the only way in, and it takes
/// a type only A's reader can construct.
///
/// A `/2` entry point belongs with the version 2 consumer lane, on the branch
/// that pins a revision publishing `protocol_artifact::v2`. Version 2 inherits
/// this declaration table unchanged, so it will delegate here rather than
/// duplicating the mapping.
fn link_declarations(
    declarations: &[Declaration],
    declaration: u32,
) -> Result<LinkedProtocol, LinkRefusal> {
    let index = usize::try_from(declaration).unwrap_or(usize::MAX);
    let declared = declarations
        .get(index)
        .ok_or(LinkRefusal::NoSuchDeclaration {
            selected: declaration,
            available: declarations.len(),
        })?;

    let Body::Protocol {
        roles, channels, ..
    } = &declared.body
    else {
        return Err(LinkRefusal::NotAProtocol {
            declaration,
            found: body_tag(&declared.body),
        });
    };

    let linker = Linker {
        declaration,
        declared,
    };

    let definition =
        DefinitionId::new(&declaration_identity(declaration, declared), &declared.name);

    let workflow_instance =
        linker
            .sole_requirement(BindingKind::WorkflowInstance)
            .map(|(index, requirement)| {
                WorkflowInstanceId::new(
                    &requirement_identity(declaration, index, requirement),
                    &requirement.name,
                )
            });

    let mut role_instances = Vec::new();
    for (position, role) in roles.iter().enumerate() {
        let referrer = Referrer::Role { index: position };
        let requirement = linker.requirement(
            referrer,
            role.instance,
            BindingKind::RoleInstance,
            &role.locus,
        )?;
        role_instances.push(RoleInstanceId::new(
            &requirement_identity(declaration, role.instance, requirement),
            &role.name,
        ));
    }

    // Participants, components, endpoints, attempts and effects are not named by
    // the protocol's own tables: they are the declaration's binding
    // requirements of those kinds. Reading them from the binding table rather
    // than inferring them from control bodies keeps each one's identity the
    // producer's, and keeps this linker out of A's control-graph semantics.
    let participants = linker.all_of(BindingKind::Participant, ParticipantId::new);
    let components = linker.all_of(BindingKind::Component, ComponentId::new);
    let endpoints = linker.all_of(BindingKind::Endpoint, EndpointId::new);
    let attempts = linker.all_of(BindingKind::Attempt, AttemptId::new);
    let effects = linker.all_of(BindingKind::Effect, EffectId::new);

    let mut linked_channels = BTreeMap::new();
    for (position, channel) in channels.iter().enumerate() {
        let premise = |premise: ChannelPremise, selected: u32| {
            linker.requirement(
                Referrer::Channel {
                    index: position,
                    premise,
                },
                selected,
                premise.required_kind(),
                &channel.locus,
            )
        };
        let message = premise(ChannelPremise::Message, channel.message)?;
        let send = premise(ChannelPremise::Send, channel.send)?;
        let receive = premise(ChannelPremise::Receive, channel.receive)?;
        let delivery = premise(ChannelPremise::Delivery, channel.delivery_instance)?;

        let identity = ChannelId::new(
            &channel_identity(declaration, position, channel.message),
            &channel.name,
        );
        let linked = LinkedChannel {
            message: MessageId::new(
                &requirement_identity(declaration, channel.message, message),
                &message.name,
            ),
            send: SendId::new(
                &requirement_identity(declaration, channel.send, send),
                &send.name,
            ),
            receive: ReceiveId::new(
                &requirement_identity(declaration, channel.receive, receive),
                &receive.name,
            ),
            delivery: DeliveryId::new(
                &requirement_identity(declaration, channel.delivery_instance, delivery),
                &delivery.name,
            ),
        };
        if linked_channels.insert(identity.clone(), linked).is_some() {
            return Err(LinkRefusal::CollidingIdentity {
                kind: IdentityKind::Channel,
                identity: identity.identity_bytes().to_vec(),
            });
        }
    }

    check_distinct(IdentityKind::RoleInstance, &role_instances)?;
    check_distinct(IdentityKind::Participant, &participants)?;
    check_distinct(IdentityKind::Component, &components)?;
    check_distinct(IdentityKind::Endpoint, &endpoints)?;
    check_distinct(IdentityKind::Attempt, &attempts)?;
    check_distinct(IdentityKind::Effect, &effects)?;

    Ok(LinkedProtocol {
        definition,
        workflow_instance,
        role_instances,
        participants,
        components,
        endpoints,
        attempts,
        effects,
        channels: linked_channels,
    })
}

struct Linker<'a> {
    declaration: u32,
    declared: &'a Declaration,
}

impl<'a> Linker<'a> {
    /// The binding requirement at `selected`, once its declared kind matches
    /// what the referring record requires.
    fn requirement(
        &self,
        referrer: Referrer,
        selected: u32,
        required: BindingKind,
        locus: &Locus,
    ) -> Result<&'a BindingRequirement, LinkRefusal> {
        let bindings = &self.declared.bindings;
        let index = usize::try_from(selected).unwrap_or(usize::MAX);
        let requirement = bindings.get(index).ok_or(LinkRefusal::NoSuchRequirement {
            referrer,
            selected,
            available: bindings.len(),
        })?;
        if requirement.kind != required {
            return Err(LinkRefusal::WrongRequirementKind {
                referrer,
                required,
                found: requirement.kind,
                locus: locus.clone(),
            });
        }
        Ok(requirement)
    }

    /// The single requirement of `kind`, when the declaration carries exactly
    /// one.
    ///
    /// More than one is not a refusal here: a declaration may legitimately
    /// require several, and the protocol's own tables name the ones it uses.
    /// Only the workflow instance is read this way, and A's contract gives a
    /// declaration one.
    fn sole_requirement(&self, kind: BindingKind) -> Option<(u32, &'a BindingRequirement)> {
        let mut found = self
            .declared
            .bindings
            .iter()
            .enumerate()
            .filter(|(_, requirement)| requirement.kind == kind);
        match (found.next(), found.next()) {
            (Some((index, requirement)), None) => {
                Some((u32::try_from(index).unwrap_or(u32::MAX), requirement))
            }
            _ => None,
        }
    }

    /// Every requirement of `kind`, built into its own identity newtype.
    fn all_of<T>(&self, kind: BindingKind, build: fn(&[u8], &str) -> T) -> Vec<T> {
        self.declared
            .bindings
            .iter()
            .enumerate()
            .filter(|(_, requirement)| requirement.kind == kind)
            .map(|(index, requirement)| {
                let index = u32::try_from(index).unwrap_or(u32::MAX);
                build(
                    &requirement_identity(self.declaration, index, requirement),
                    &requirement.name,
                )
            })
            .collect()
    }
}

/// The producer coordinates of one declaration.
fn declaration_identity(declaration: u32, declared: &Declaration) -> Vec<u8> {
    let mut bytes = Vec::new();
    push_u32(&mut bytes, declaration);
    push_bytes(&mut bytes, declared.requirement.package.as_bytes());
    push_bytes(&mut bytes, declared.requirement.identity.as_bytes());
    push_bytes(
        &mut bytes,
        declared.requirement.revision.namespace.as_bytes(),
    );
    push_bytes(&mut bytes, declared.requirement.revision.value.as_bytes());
    push_bytes(&mut bytes, declared.clause.as_bytes());
    bytes
}

/// The producer coordinates of one channel.
///
/// A channel's table position selects it — A's records do not refer to channels
/// by name — and its message requirement pins what it carries, so both are
/// retained. The authored channel name is display text.
fn channel_identity(declaration: u32, position: usize, message: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    push_u32(&mut bytes, declaration);
    push_u32(&mut bytes, u32::try_from(position).unwrap_or(u32::MAX));
    push_u32(&mut bytes, message);
    bytes
}

/// Refuse two records that produced one identity within a kind.
fn check_distinct<T: Distinct>(kind: IdentityKind, values: &[T]) -> Result<(), LinkRefusal> {
    for (position, value) in values.iter().enumerate() {
        if let Some(earlier) = values[..position]
            .iter()
            .find(|earlier| earlier.identity_bytes() == value.identity_bytes())
        {
            return Err(LinkRefusal::CollidingIdentity {
                kind,
                identity: earlier.identity_bytes().to_vec(),
            });
        }
    }
    Ok(())
}

/// The one thing `check_distinct` needs from an identity.
///
/// A trait rather than six copies of the same loop. It has six implementations
/// and no seam purpose beyond that, which is why it is private.
trait Distinct {
    fn identity_bytes(&self) -> &[u8];
}

macro_rules! distinct {
    ($($name:ident),* $(,)?) => {
        $(impl Distinct for $name {
            fn identity_bytes(&self) -> &[u8] {
                $name::identity_bytes(self)
            }
        })*
    };
}

distinct!(
    RoleInstanceId,
    ParticipantId,
    ComponentId,
    EndpointId,
    AttemptId,
    EffectId,
);

/// The tag of a declaration body kind, for a refusal that names what it found.
const fn body_tag(body: &Body) -> &'static str {
    match body {
        Body::Predicate { .. } => "predicate",
        Body::State { .. } => "state",
        Body::Temporal { .. } => "temporal",
        Body::Protocol { .. } => "protocol",
    }
}

// ---------------------------------------------------------------------------
// TC-002 (partial, chunk C-12): the identity mapping
// ---------------------------------------------------------------------------

/// These are unit tests rather than a `tests/` lane because the mapping's input
/// is a declaration slice and its public entry takes an `AdmittedPackage`, a
/// type only A's reader can construct. An integration lane cannot reach the
/// refusal paths without a real producer artifact per refusal, so the refusals
/// are driven here and the end-to-end lane stays with the native handoff.
#[cfg(test)]
mod tests {
    use ix_trace_rs::trace;
    use quire_spec_language::protocol_artifact::wire::{
        Activation, ArtifactKind, ArtifactRef, Channel, Execution, ExportRef, Finish, Handle,
        Integer, Interval, Nullable, Ordering, Requirement, Revision, Role, Span, Subject, Wire,
    };
    use quire_spec_language::protocol_artifact::NumberWire;
    use quire_spec_language::ByteDigest;

    use super::*;

    fn handle(index: u32) -> Handle {
        Handle {
            declaration: 0,
            index,
        }
    }

    fn locus() -> Locus {
        Locus {
            source: 0,
            span: Span { start: 0, end: 1 },
        }
    }

    fn artifact(identity: &str) -> ArtifactRef {
        ArtifactRef {
            ref_version: "ix.artifact-ref/3-draft".to_owned(),
            kind: ArtifactKind::Source,
            authority: "ix://agent-ix/example".to_owned(),
            identity: identity.to_owned(),
            revision: Revision {
                namespace: "example".to_owned(),
                value: "1".to_owned(),
            },
            digest: ByteDigest::of(identity.as_bytes()),
            wire: Wire {
                identity: "text/markdown".to_owned(),
                version: "1".to_owned(),
            },
        }
    }

    fn requirement(name: &str, kind: BindingKind) -> BindingRequirement {
        BindingRequirement {
            name: name.to_owned(),
            kind,
            value_type: Nullable(None),
            authority: artifact(name),
            contract: 0,
            model: Nullable(None),
            subject: Subject::Declaration { declaration: 0 },
            anchor: handle(0),
            scope: handle(0),
            relation: Nullable(None),
            requires: Vec::new(),
            locus: locus(),
        }
    }

    fn integer(value: &str) -> Integer {
        Integer(NumberWire::Integer {
            decimal: value.to_owned(),
        })
    }

    /// The binding table every fixture below starts from.
    ///
    /// Index order matters: A's `u32` fields select requirements by index, so
    /// these positions are the structure the identities retain.
    fn bindings() -> Vec<BindingRequirement> {
        vec![
            requirement("the-workflow", BindingKind::WorkflowInstance),
            requirement("the-role-instance", BindingKind::RoleInstance),
            requirement("the-participant", BindingKind::Participant),
            requirement("the-component", BindingKind::Component),
            requirement("the-endpoint", BindingKind::Endpoint),
            requirement("the-message", BindingKind::Message),
            requirement("the-send", BindingKind::Send),
            requirement("the-receive", BindingKind::Receive),
            requirement("the-delivery", BindingKind::Delivery),
            requirement("the-attempt", BindingKind::Attempt),
            requirement("the-effect", BindingKind::Effect),
        ]
    }

    fn channel(name: &str, message: u32) -> Channel {
        Channel {
            name: name.to_owned(),
            from: handle(0),
            to: handle(1),
            message_type: 0,
            ordering: Ordering::Unordered {},
            delivery: Interval {
                lower: integer("0"),
                upper: integer("1"),
            },
            message,
            send: 6,
            receive: 7,
            delivery_instance: 8,
            locus: locus(),
        }
    }

    fn protocol_body(roles: Vec<Role>, channels: Vec<Channel>) -> Body {
        Body::Protocol {
            input: handle(0),
            activation: Activation::Origin { anchor: handle(0) },
            captures: Vec::new(),
            roles,
            relationships: Vec::new(),
            channels,
            compensations: Vec::new(),
            temporal_requirements: Vec::new(),
            controls: Vec::new(),
            causal_edges: Vec::new(),
            run: handle(0),
            finish: Box::new(Finish {
                name: "done".to_owned(),
                binder: handle(0),
                constraint: handle(0),
                closure: 0,
                locus: locus(),
            }),
        }
    }

    fn declaration(body: Body, bindings: Vec<BindingRequirement>) -> Declaration {
        Declaration {
            name: "TheProtocol".to_owned(),
            locus: locus(),
            requirement: Requirement {
                package: "agent-ix/example".to_owned(),
                identity: "the-requirement".to_owned(),
                revision: Revision {
                    namespace: "example".to_owned(),
                    value: "1".to_owned(),
                },
            },
            clause: "the-clause".to_owned(),
            execution: Execution::Initialization {
                name: "start".to_owned(),
            },
            profile: 0,
            requires: Vec::new(),
            scopes: Vec::new(),
            anchors: Vec::new(),
            binders: Vec::new(),
            values: Vec::new(),
            temporal: Vec::new(),
            bindings,
            body,
        }
    }

    fn role(name: &str, instance: u32) -> Role {
        Role {
            name: name.to_owned(),
            model: ExportRef {
                model: 0,
                export: 0,
            },
            instance,
            locus: locus(),
        }
    }

    /// One well-formed protocol declaration.
    fn fixture() -> Vec<Declaration> {
        vec![declaration(
            protocol_body(vec![role("Buyer", 1)], vec![channel("order", 5)]),
            bindings(),
        )]
    }

    #[trace("TC-002", "FR-002-AC-1")]
    #[test]
    fn tc_002_each_identity_kind_is_linked_onto_its_own_type() {
        let linked = link_declarations(&fixture(), 0).expect("the fixture links");

        assert_eq!(linked.role_instances().len(), 1);
        assert_eq!(linked.participants().len(), 1);
        assert_eq!(linked.components().len(), 1);
        assert_eq!(linked.endpoints().len(), 1);
        assert_eq!(linked.attempts().len(), 1);
        assert_eq!(linked.effects().len(), 1);
        assert_eq!(linked.channels().len(), 1);
        assert!(linked.workflow_instance().is_some());

        // Each kind reports its own kind, so nothing was built onto a
        // neighbouring newtype.
        assert_eq!(linked.definition().kind(), IdentityKind::Definition);
        assert_eq!(
            linked.role_instances()[0].kind(),
            IdentityKind::RoleInstance
        );
        assert_eq!(linked.participants()[0].kind(), IdentityKind::Participant);
        assert_eq!(linked.attempts()[0].kind(), IdentityKind::Attempt);
        assert_eq!(linked.effects()[0].kind(), IdentityKind::Effect);

        let (channel, premises) = linked.channels().iter().next().expect("one channel");
        assert_eq!(channel.kind(), IdentityKind::Channel);
        assert_eq!(premises.message().kind(), IdentityKind::Message);
        assert_eq!(premises.send().kind(), IdentityKind::Send);
        assert_eq!(premises.receive().kind(), IdentityKind::Receive);
        assert_eq!(premises.delivery().kind(), IdentityKind::Delivery);

        // A send is not its paired receive, and a delivery is not its message,
        // even though all four came from one channel record.
        assert_ne!(
            premises.send().identity_bytes(),
            premises.receive().identity_bytes()
        );
        assert_ne!(
            premises.message().identity_bytes(),
            premises.delivery().identity_bytes()
        );
    }

    /// FR-002-AC-4, the presentation half: changing every authored name changes
    /// nothing about what was linked.
    #[trace("TC-002", "FR-002-AC-4")]
    #[test]
    fn tc_002_changing_display_text_does_not_change_the_linked_protocol() {
        let linked = link_declarations(&fixture(), 0).expect("the fixture links");

        let mut renamed = fixture();
        renamed[0].name = "ADifferentName".to_owned();
        for binding in &mut renamed[0].bindings {
            binding.name = format!("renamed-{}", binding.name);
        }
        let Body::Protocol {
            roles, channels, ..
        } = &mut renamed[0].body
        else {
            unreachable!("the fixture is a protocol")
        };
        roles[0].name = "Seller".to_owned();
        channels[0].name = "a-different-channel-name".to_owned();

        let renamed = link_declarations(&renamed, 0).expect("the renamed fixture links");
        assert_eq!(
            linked, renamed,
            "authored names are display text and must not reach the linked protocol"
        );
    }

    /// FR-002-AC-4, the semantic half: changing an authored channel premise
    /// changes the linked protocol.
    #[trace("TC-002", "FR-002-AC-4")]
    #[test]
    fn tc_002_changing_an_authored_channel_premise_changes_the_linked_protocol() {
        let linked = link_declarations(&fixture(), 0).expect("the fixture links");

        // A second message requirement, and the channel now carries that one.
        let mut changed = fixture();
        changed[0]
            .bindings
            .push(requirement("another-message", BindingKind::Message));
        let moved = u32::try_from(changed[0].bindings.len() - 1).expect("a small table");
        let Body::Protocol { channels, .. } = &mut changed[0].body else {
            unreachable!("the fixture is a protocol")
        };
        channels[0].message = moved;

        let changed = link_declarations(&changed, 0).expect("the changed fixture links");
        assert_ne!(
            linked, changed,
            "a channel carrying a different message is a different protocol"
        );
    }

    /// The binding table's order is structure, not presentation: A's own `u32`
    /// fields select requirements by index.
    #[trace("TC-002", "FR-002-AC-4")]
    #[test]
    fn tc_002_reordering_the_binding_table_changes_what_records_refer_to() {
        let linked = link_declarations(&fixture(), 0).expect("the fixture links");

        // Swap two requirements of the same kind's neighbours: the roles' and
        // participants' positions move, so every identity built from them moves.
        let mut reordered = fixture();
        reordered[0].bindings.swap(2, 3);

        let reordered = link_declarations(&reordered, 0).expect("the reordered fixture links");
        assert_ne!(
            linked, reordered,
            "a reordered binding table selects different requirements"
        );
    }

    #[trace("TC-002", "FR-002-AC-3")]
    #[test]
    fn tc_002_a_wrong_kind_binding_requirement_is_refused() {
        // The channel's send premise now points at the delivery requirement.
        let mut wrong = fixture();
        let Body::Protocol { channels, .. } = &mut wrong[0].body else {
            unreachable!("the fixture is a protocol")
        };
        channels[0].send = 8;

        let refusal = link_declarations(&wrong, 0).expect_err("a wrong kind refuses");
        assert_eq!(
            refusal,
            LinkRefusal::WrongRequirementKind {
                referrer: Referrer::Channel {
                    index: 0,
                    premise: ChannelPremise::Send,
                },
                required: BindingKind::Send,
                found: BindingKind::Delivery,
                locus: locus(),
            }
        );
    }

    #[trace("TC-002", "FR-002-AC-3")]
    #[test]
    fn tc_002_an_out_of_range_binding_requirement_is_refused() {
        let mut missing = fixture();
        let Body::Protocol { roles, .. } = &mut missing[0].body else {
            unreachable!("the fixture is a protocol")
        };
        roles[0].instance = 99;

        let refusal = link_declarations(&missing, 0).expect_err("an absent requirement refuses");
        assert_eq!(
            refusal,
            LinkRefusal::NoSuchRequirement {
                referrer: Referrer::Role { index: 0 },
                selected: 99,
                available: 11,
            }
        );
    }

    #[trace("TC-002", "FR-002-AC-3")]
    #[test]
    fn tc_002_a_declaration_that_is_not_a_protocol_is_refused() {
        let not_a_protocol = vec![declaration(
            Body::Predicate {
                parameters: Vec::new(),
                result: 0,
                root: handle(0),
            },
            bindings(),
        )];
        assert_eq!(
            link_declarations(&not_a_protocol, 0).expect_err("a predicate is not a protocol"),
            LinkRefusal::NotAProtocol {
                declaration: 0,
                found: "predicate",
            }
        );
    }

    #[trace("TC-002", "FR-002-AC-3")]
    #[test]
    fn tc_002_a_declaration_outside_the_package_is_refused() {
        assert_eq!(
            link_declarations(&fixture(), 7).expect_err("there is one declaration"),
            LinkRefusal::NoSuchDeclaration {
                selected: 7,
                available: 1,
            }
        );
    }

    /// Two channels naming one message requirement produce one channel
    /// identity, which would silently merge them.
    #[trace("TC-002", "FR-002-AC-1")]
    #[test]
    fn tc_002_two_records_producing_one_identity_are_refused() {
        let mut colliding = fixture();
        // Two role records selecting one role-instance requirement: both build
        // the same RoleInstanceId.
        let Body::Protocol { roles, .. } = &mut colliding[0].body else {
            unreachable!("the fixture is a protocol")
        };
        roles.push(role("Seller", 1));

        assert_eq!(
            link_declarations(&colliding, 0).expect_err("one identity for two roles refuses"),
            LinkRefusal::CollidingIdentity {
                kind: IdentityKind::RoleInstance,
                identity: requirement_identity(0, 1, &bindings()[1]),
            }
        );
    }

    /// The encoding's length prefixes are load-bearing.
    ///
    /// Without them, two requirements whose members concatenate to the same
    /// bytes — `"ab" + "c"` against `"a" + "bc"` — would produce one identity
    /// and silently merge two distinct requirements.
    #[trace("TC-002", "FR-002-AC-1")]
    #[test]
    fn tc_002_requirement_identity_is_unambiguous_across_member_boundaries() {
        let mut left = requirement("x", BindingKind::Message);
        left.authority.authority = "ab".to_owned();
        left.authority.identity = "c".to_owned();

        let mut right = requirement("x", BindingKind::Message);
        right.authority.authority = "a".to_owned();
        right.authority.identity = "bc".to_owned();

        assert_ne!(
            requirement_identity(0, 0, &left),
            requirement_identity(0, 0, &right),
            "member boundaries must survive the encoding"
        );

        // The same requirement at two table positions is two identities.
        assert_ne!(
            requirement_identity(0, 0, &left),
            requirement_identity(0, 1, &left)
        );
        // And in two declarations.
        assert_ne!(
            requirement_identity(0, 0, &left),
            requirement_identity(1, 0, &left)
        );
    }

    /// A refusal returns no linked representation, never a partial one.
    #[trace("TC-002", "FR-002-AC-3")]
    #[test]
    fn tc_002_a_refusal_emits_no_linked_representation() {
        let mut wrong = fixture();
        let Body::Protocol { channels, .. } = &mut wrong[0].body else {
            unreachable!("the fixture is a protocol")
        };
        channels[0].receive = 5;

        // `Result` carries one or the other, so there is no partial value to
        // read. This asserts the shape the signature guarantees rather than
        // restating it: the refusal is the whole return.
        let outcome = link_declarations(&wrong, 0);
        assert!(outcome.is_err());
        assert_eq!(outcome.ok(), None);
    }
}
