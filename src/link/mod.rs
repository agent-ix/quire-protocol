// SPDX-License-Identifier: AGPL-3.0-or-later
//! Finite linked-protocol graph. Owner chunk: **C-12** (submodule: C-13).
//!
//! FR-002 requires one finite typed representation of an admitted protocol,
//! with every identity kind kept distinct and with neither input order nor
//! display text participating in what is linked. [`LinkedProtocol`] is that
//! representation, and [`link`] builds it.
//!
//! # What A refuses, what this module refuses, and what nobody refuses yet
//!
//! FR-002 names a refusal list: duplicate role authorities, unresolved or
//! ambiguous references, missing or invalid joins, causal cycles, unbounded or
//! zero-progress iteration, unreachable termination, conflicting channel
//! premises, and ownerless, multiply owned, unlabeled, overlapping or
//! owner-unobservable choices.
//!
//! An earlier version of this module claimed A refused **all** of it. That was
//! false, and it had been written into `spec/tests.md` as coverage evidence. The
//! division below was checked against A's source at the pinned revision.
//!
//! **A refuses, and this module does not repeat:** the causal edge set — A
//! derives it from the authored controls per the table in A's
//! `docs/compiled-protocol-v1.md`, compares the offered `causal_edges` element
//! by element, enforces strict ascending order on the full edge key, checks each
//! `repeat_progress` maximum and runs an acyclicity check — plus joins,
//! reference resolution, owner checks, duplicate names and choice visibility
//! resolution. `CLAUDE.md`: "An owner publishes it → import it. Do not mirror
//! it, not even in tests." A second copy would also be the worse kind of
//! duplicate: one edge of divergence and every real artifact refuses, with the
//! disagreement in the half that is not the owner's.
//!
//! **This module refuses, because A does not:**
//!
//! - **Zero-progress iteration.** A's `nonnegative` rejects only a negative, so
//!   `maximum = 0` is admitted, and A's contract documents N=0 as merely making
//!   the progress edge unreachable. A refuses `maximum_attempts <= 0` for
//!   compensation in the next function along, so the check exists there and not
//!   here. [`LinkRefusal::ZeroProgressIteration`].
//! - **Duplicate role authorities.** A dedupes role *names*, which are display
//!   text. Nothing dedupes `role.model`, the export that says which authority
//!   the role speaks for. [`LinkRefusal::DuplicateRoleAuthority`].
//!
//! **Nobody refuses yet, and this is recorded rather than implied:**
//!
//! - **Unreachable termination.** There is no reachability or liveness analysis
//!   in A, and none here. Exit reachability is a consequence of the edge
//!   expansion, never checked.
//! - **Overlapping choices.** A type-checks each case guard and stops. Deciding
//!   whether two authored guards can both hold is a decision procedure over
//!   arbitrary predicates, not a structural check, and neither repository has
//!   one.
//! - **Owner-unobservable choices.** A resolves the `visible` handles and does
//!   nothing further; values carry no role attribution to relate a guard's
//!   operands to that set.
//!
//! Those three are FR-002-AC-3 obligations that no code discharges today. They
//! are named here, and in the matrix, so the gap is visible rather than covered
//! by a sentence claiming A has them.
//!
//! What remains, and is genuinely this module's, is the identity space. A's
//! records reference things by table position; B's sixteen newtypes (chunk C-07)
//! keep sixteen kinds mutually unassignable and keep display text out of
//! equality. Mapping one onto the other is this module's main job.
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

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use quire_spec_language::protocol_artifact::{
    wire::{
        BindingKind, BindingRequirement, Body, CausalEdge, Control, ControlOperation, Declaration,
        Integer, Locus,
    },
    AdmittedPackage, NumberWire,
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
/// Two groups. [`LinkRefusal::ZeroProgressIteration`] and
/// [`LinkRefusal::DuplicateRoleAuthority`] are FR-002 obligations A leaves open.
/// The rest are facts about B's own identity mapping.
///
/// [`LinkRefusal::NoSuchRequirement`] and [`LinkRefusal::WrongRequirementKind`]
/// do overlap A's `fn binding`, which range-checks and kind-checks the same
/// indices. They are kept because the mapping cannot proceed without the answer
/// — B must know a requirement's kind to build the right newtype from it — and
/// a refusal is the honest result of asking. They are defence in depth over an
/// already admitted package, not a second implementation of a rule A owns.
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
    /// A repeat whose authored maximum is zero.
    ///
    /// FR-002 requires "unbounded or zero-progress iteration" refused. A admits
    /// it: `Repeat.maximum` goes through A's `nonnegative`, which rejects only a
    /// negative, and A's own contract documents N=0 as merely making the
    /// progress edge unreachable. A refuses `maximum_attempts <= 0` for
    /// compensation in the very next function, so the check was written
    /// deliberately in one place and not this one.
    ZeroProgressIteration {
        /// The repeat control's index in the declaration's control table.
        control: u32,
        /// The original source region of that control.
        locus: Locus,
    },
    /// Two roles naming one authority.
    ///
    /// A dedupes role *names*, which are display text, and nothing dedupes
    /// `role.model` — the export that actually says which authority the role
    /// speaks for. Two roles over one authority with distinct role-instance
    /// bindings are admitted by A.
    DuplicateRoleAuthority {
        /// The role-table index of the later of the two.
        role: usize,
        /// The model export both name.
        model: (u32, u32),
        /// The original source region of that role.
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
            Self::ZeroProgressIteration { control, .. } => write!(
                f,
                "repeat control {control} authors a maximum of zero, which can make no progress"
            ),
            Self::DuplicateRoleAuthority { role, model, .. } => write!(
                f,
                "role {role} names model export {model:?}, which another role already names"
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
    scope: &[u8],
    declaration: u32,
    index: u32,
    requirement: &BindingRequirement,
) -> Vec<u8> {
    let mut bytes = Vec::new();
    push_bytes(&mut bytes, scope);
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

/// One of A's validated causal edges, projected into a hashable record.
///
/// A projection, not a re-derivation: A derives this set from the authored
/// controls and refuses an offer whose `causal_edges` disagree with it, so what
/// is retained here is A's answer.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct EdgeShape {
    /// A's edge kind, as A spells it.
    pub kind: &'static str,
    /// The expanded control that owns the edge.
    pub owner: u32,
    /// The source control and its port.
    pub from: (u32, &'static str),
    /// The target control and its port.
    pub to: (u32, &'static str),
    /// The authored maximum, on a repeat-progress edge.
    pub maximum: Option<u32>,
}

/// Project one of A's edges.
fn edge_shape(edge: &CausalEdge) -> EdgeShape {
    EdgeShape {
        kind: edge.kind.as_str(),
        owner: edge.owner.index,
        from: (edge.from.node.index, edge.from.port.as_str()),
        to: (edge.to.node.index, edge.to.port.as_str()),
        maximum: edge.maximum.0.as_ref().map(numeric_fingerprint),
    }
}

/// One control's structure, with every authored name and label removed.
///
/// `LinkedProtocol` used to carry identities and channels only, so two packages
/// differing solely in a repeat's `maximum` (1 against 9999), or in a parallel
/// `join`, or in `Finish.closure`, linked to **equal** values — while the matrix
/// claimed FR-002-AC-4 was executing. AC-4 requires that changing an authored
/// branch, join, iteration maximum, channel premise or visibility premise
/// changes the linked protocol identity, and only the channel-premise clause
/// actually held.
///
/// These are the facts A has already validated, retained rather than re-derived:
/// A's own `causal_edges` carry the branch, join and repeat-progress structure,
/// and the fields below carry what the edge set does not — the repeat maximum,
/// the choice's owner and visibility, and the finish closure. Nothing here is a
/// name or a label, so a presentation change still leaves the value equal.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ControlShape {
    /// The control's operation kind, as A spells it.
    kind: &'static str,
    /// Structural indices the operation carries, in declared order.
    indices: Vec<u32>,
}

impl ControlShape {
    /// The operation kind.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        self.kind
    }

    /// The structural indices, in declared order.
    #[must_use]
    pub fn indices(&self) -> &[u32] {
        &self.indices
    }
}

/// The structural shape of one control, with names and labels excluded.
fn control_shape(control: &Control) -> ControlShape {
    let mut indices = Vec::new();
    let kind = match &control.operation {
        ControlOperation::Sequence { children } => {
            indices.extend(children.iter().map(|child| child.index));
            "sequence"
        }
        ControlOperation::Choice {
            owner,
            visible,
            cases,
        } => {
            indices.push(owner.index);
            indices.extend(visible.iter().map(|handle| handle.index));
            indices.extend(cases.iter().map(|case| case.body.index));
            "choice"
        }
        ControlOperation::Parallel { branches, join } => {
            indices.extend(branches.iter().map(|branch| branch.body.index));
            indices.extend(join.iter().copied());
            "parallel"
        }
        ControlOperation::Repeat {
            owner,
            visible,
            maximum,
            guard,
            body,
            exhausted,
        } => {
            indices.push(owner.index);
            indices.extend(visible.iter().map(|handle| handle.index));
            indices.push(guard.index);
            indices.push(body.index);
            indices.push(exhausted.index);
            // The authored maximum is the iteration bound AC-4 names. It is
            // retained as its offered decimal text so a differently spelled but
            // equal value is still a different authored protocol, which is A's
            // own rule for exact numbers.
            indices.push(numeric_fingerprint(maximum));
            "repeat"
        }
        ControlOperation::Await {
            profile,
            clock,
            event,
            then_body,
            timeout,
            ..
        } => {
            indices.push(*profile);
            indices.push(*clock);
            indices.push(event.index);
            indices.push(then_body.index);
            indices.push(timeout.index);
            "await"
        }
        ControlOperation::Event { binder, .. } => {
            indices.push(binder.index);
            "event"
        }
        ControlOperation::Check { profile, value } => {
            indices.push(*profile);
            indices.push(value.index);
            "check"
        }
        ControlOperation::Commit {
            owner,
            binder,
            constraint,
            instance,
        } => {
            indices.push(owner.index);
            indices.push(binder.index);
            indices.push(constraint.index);
            indices.push(*instance);
            "commit"
        }
    };
    ControlShape { kind, indices }
}

/// Whether an offered integer is exactly zero.
///
/// Reads the offered decimal spelling rather than interpreting the number: A
/// has already validated it as an integer, and B owns no numeric domain. A
/// leading sign and leading zeros are handled because `"0"`, `"+0"`, `"-0"` and
/// `"000"` are all the same authored bound.
fn is_zero(value: &Integer) -> bool {
    match &value.0 {
        NumberWire::Integer { decimal } => {
            let digits = decimal.trim_start_matches(['+', '-']);
            !digits.is_empty() && digits.bytes().all(|byte| byte == b'0')
        }
        NumberWire::Rational { numerator, .. } => {
            let digits = numerator.trim_start_matches(['+', '-']);
            !digits.is_empty() && digits.bytes().all(|byte| byte == b'0')
        }
    }
}

/// A stable fingerprint of an offered exact number's spelling.
///
/// Not a value: B does not interpret A's numerals, and A has already validated
/// this one. It only has to differ when the spelling differs.
fn numeric_fingerprint(value: &Integer) -> u32 {
    let text = match &value.0 {
        NumberWire::Integer { decimal } => decimal.clone(),
        NumberWire::Rational {
            numerator,
            denominator,
        } => format!("{numerator}/{denominator}"),
    };
    text.bytes().fold(2_166_136_261_u32, |hash, byte| {
        (hash ^ u32::from(byte)).wrapping_mul(16_777_619)
    })
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
    /// A's own validated causal edges, retained rather than re-derived.
    ///
    /// A derives this set from the authored controls and refuses an offer whose
    /// `causal_edges` disagree, so retaining it carries the branch, join and
    /// repeat-progress structure into the linked identity without this module
    /// implementing any of A's rules.
    edges: Vec<EdgeShape>,
    /// One shape per control, names and labels excluded.
    controls: Vec<ControlShape>,
    /// The finish closure's binding-requirement index.
    finish_closure: u32,
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

    /// The role instances, in the protocol's own role-table order.
    ///
    /// Not identity order: a role's identity is keyed on the binding requirement
    /// it selects, and roles are returned in the order the protocol declares
    /// them. That order is structure — A's records select roles by table
    /// position — so a reordered role table is a different protocol, and this
    /// vector's order says so.
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

    /// A's validated causal edges, in A's own order.
    #[must_use]
    pub fn edges(&self) -> &[EdgeShape] {
        &self.edges
    }

    /// One structural shape per control, in control-table order.
    #[must_use]
    pub fn controls(&self) -> &[ControlShape] {
        &self.controls
    }

    /// The binding-requirement index the finish closure selects.
    #[must_use]
    pub const fn finish_closure(&self) -> u32 {
        self.finish_closure
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
        roles,
        channels,
        controls,
        causal_edges,
        finish,
        ..
    } = &declared.body
    else {
        return Err(LinkRefusal::NotAProtocol {
            declaration,
            found: body_tag(&declared.body),
        });
    };

    // Every identity below is scoped by the owning declaration's own producer
    // coordinates, so two packages cannot mint the same one.
    let scope = declaration_identity(declaration, declared);
    let linker = Linker {
        scope: scope.clone(),
        declaration,
        declared,
    };

    let definition = DefinitionId::new(&scope, &declared.name);

    let workflow_instance =
        linker
            .sole_requirement(BindingKind::WorkflowInstance)
            .map(|(index, requirement)| {
                WorkflowInstanceId::new(
                    &requirement_identity(&scope, declaration, index, requirement),
                    &requirement.name,
                )
            });

    // Refusals A does not make. See the module documentation: A's validator
    // covers the edge set, joins, cycles and reference resolution, and these are
    // the items on FR-002's list that it leaves open.
    for (position, control) in controls.iter().enumerate() {
        if let ControlOperation::Repeat { maximum, .. } = &control.operation {
            if is_zero(maximum) {
                return Err(LinkRefusal::ZeroProgressIteration {
                    control: u32::try_from(position).unwrap_or(u32::MAX),
                    locus: control.locus.clone(),
                });
            }
        }
    }

    let mut seen_authorities: Vec<(u32, u32)> = Vec::new();
    for (position, role) in roles.iter().enumerate() {
        let model = (role.model.model, role.model.export);
        if seen_authorities.contains(&model) {
            return Err(LinkRefusal::DuplicateRoleAuthority {
                role: position,
                model,
                locus: role.locus.clone(),
            });
        }
        seen_authorities.push(model);
    }

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
            &requirement_identity(&scope, declaration, role.instance, requirement),
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
            &channel_identity(&scope, declaration, position, channel.message),
            &channel.name,
        );
        let linked = LinkedChannel {
            message: MessageId::new(
                &requirement_identity(&scope, declaration, channel.message, message),
                &message.name,
            ),
            send: SendId::new(
                &requirement_identity(&scope, declaration, channel.send, send),
                &send.name,
            ),
            receive: ReceiveId::new(
                &requirement_identity(&scope, declaration, channel.receive, receive),
                &receive.name,
            ),
            delivery: DeliveryId::new(
                &requirement_identity(&scope, declaration, channel.delivery_instance, delivery),
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

    // Only roles can collide.
    //
    // `all_of` enumerates the binding table, so a participant's, component's,
    // endpoint's, attempt's or effect's identity carries its own table index and
    // is distinct by construction — those five checks could never fire, and
    // deleting them left every test green. They were also the expensive ones: a
    // nested byte-slice scan, measured at 462ms for 16 000 entries in release
    // and clean quadratic, so an artifact well inside A's `entries` ceiling of
    // 100 000 burned seconds of CPU in a check that could not return an error.
    //
    // Roles are different: two roles may select one role-instance requirement,
    // so this one is live. It is a sorted scan rather than a nested one.
    check_distinct_roles(&role_instances)?;

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
        edges: causal_edges.iter().map(edge_shape).collect(),
        controls: controls.iter().map(control_shape).collect(),
        finish_closure: finish.closure,
    })
}

struct Linker<'a> {
    scope: Vec<u8>,
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
                    &requirement_identity(&self.scope, self.declaration, index, requirement),
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
///
/// `scope` is the owning declaration's identity. Without it this was twelve
/// bytes of table positions with no package in them, so two declarations from
/// two different producers, differing in package, identity and clause, produced
/// byte-identical channel, role-instance and participant identities — and a
/// consumer keying a map by one of those silently merged two unrelated parties.
/// That is the failure `CollidingIdentity` exists to prevent, reached from
/// outside a single package instead of inside one.
fn channel_identity(scope: &[u8], declaration: u32, position: usize, message: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    push_bytes(&mut bytes, scope);
    push_u32(&mut bytes, declaration);
    push_u32(&mut bytes, u32::try_from(position).unwrap_or(u32::MAX));
    push_u32(&mut bytes, message);
    bytes
}

/// Refuse two roles that produced one role-instance identity.
///
/// Sorted rather than nested: the previous version compared every value against
/// every earlier one, which is quadratic, and it did that for six collections of
/// which five could not collide.
fn check_distinct_roles(values: &[RoleInstanceId]) -> Result<(), LinkRefusal> {
    let mut seen: BTreeSet<&[u8]> = BTreeSet::new();
    for value in values {
        if !seen.insert(value.identity_bytes()) {
            return Err(LinkRefusal::CollidingIdentity {
                kind: IdentityKind::RoleInstance,
                identity: value.identity_bytes().to_vec(),
            });
        }
    }
    Ok(())
}

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

/// # None of these carries an `FR-002-AC-1` tag
///
/// AC-1 requires concurrent workflow instances, role instances, participants,
/// duplicate deliveries and attempts to retain **distinct exact identities**.
/// Those are runtime instance identities, and they do not exist at link time —
/// A carries binding requirements, and the owner has ruled AC-1 unresolved until
/// instance identities exist.
///
/// What the tests below establish is that the sixteen kinds are separate types
/// and that the requirement encoding is unambiguous. Both are real, and neither
/// is AC-1. Tagging them `FR-002-AC-1` would count that criterion as backed by
/// tests that cannot fail when it is violated, which is the same defect
/// `spec/tests.md` already records for the version 2 lane. They carry
/// `Task-002`, matching `tests/tc_002_identities.rs`.
///
/// These are unit tests rather than a `tests/` lane because the mapping's input
/// is a declaration slice and its public entry takes an `AdmittedPackage`, a
/// type only A's reader can construct. An integration lane cannot reach the
/// refusal paths without a real producer artifact per refusal, so the refusals
/// are driven here and the end-to-end lane stays with the native handoff.
#[cfg(test)]
mod tests {
    use ix_trace_rs::trace;
    use quire_spec_language::protocol_artifact::wire::{
        Activation, ArtifactKind, ArtifactRef, Branch, Channel, Control, ControlOperation,
        Execution, ExportRef, Finish, Handle, Integer, Interval, Nullable, Ordering, Requirement,
        Revision, Role, Span, Subject, Wire,
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

    /// A repeat control with the given authored maximum.
    fn repeat_control(maximum: &str) -> Control {
        Control {
            name: "loop".to_owned(),
            original_node: 0,
            locus: locus(),
            operation: ControlOperation::Repeat {
                owner: handle(0),
                visible: Vec::new(),
                maximum: integer(maximum),
                guard: handle(0),
                body: handle(0),
                exhausted: handle(0),
            },
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

    #[trace("TC-002", "Task-002")]
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

        // `kind()` returns a per-type compile-time constant, so asserting it
        // here restates what the type system already guarantees: building a
        // `SendId` where a `ReceiveId` is declared is a compile error at the
        // constructor, not a test failure. What can actually go wrong is the
        // four channel premises being built from the same binding index, so
        // that is what is asserted.
        let (_channel, premises) = linked.channels().iter().next().expect("one channel");

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
    #[trace("TC-002", "Task-002")]
    #[test]
    fn tc_002_two_records_producing_one_identity_are_refused() {
        let mut colliding = fixture();
        // Two role records selecting one role-instance requirement: both build
        // the same RoleInstanceId.
        let Body::Protocol { roles, .. } = &mut colliding[0].body else {
            unreachable!("the fixture is a protocol")
        };
        // A distinct authority, so this reaches the identity collision rather
        // than the duplicate-authority refusal.
        let mut second = role("Seller", 1);
        second.model = ExportRef {
            model: 0,
            export: 1,
        };
        roles.push(second);

        assert_eq!(
            link_declarations(&colliding, 0).expect_err("one identity for two roles refuses"),
            LinkRefusal::CollidingIdentity {
                kind: IdentityKind::RoleInstance,
                identity: requirement_identity(
                    &declaration_identity(0, &fixture()[0]),
                    0,
                    1,
                    &bindings()[1]
                ),
            }
        );
    }

    /// The encoding's length prefixes are load-bearing.
    ///
    /// Without them, two requirements whose members concatenate to the same
    /// bytes — `"ab" + "c"` against `"a" + "bc"` — would produce one identity
    /// and silently merge two distinct requirements.
    #[trace("TC-002", "Task-002")]
    #[test]
    fn tc_002_requirement_identity_is_unambiguous_across_member_boundaries() {
        let mut left = requirement("x", BindingKind::Message);
        left.authority.authority = "ab".to_owned();
        left.authority.identity = "c".to_owned();

        let mut right = requirement("x", BindingKind::Message);
        right.authority.authority = "a".to_owned();
        right.authority.identity = "bc".to_owned();

        assert_ne!(
            requirement_identity(b"scope", 0, 0, &left),
            requirement_identity(b"scope", 0, 0, &right),
            "member boundaries must survive the encoding"
        );

        // The same requirement at two table positions is two identities.
        assert_ne!(
            requirement_identity(b"scope", 0, 0, &left),
            requirement_identity(b"scope", 0, 1, &left)
        );
        // And in two declarations.
        assert_ne!(
            requirement_identity(b"scope", 0, 0, &left),
            requirement_identity(b"scope", 1, 0, &left)
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

        // `outcome.ok() == None` is true whenever `is_err()` is, so asserting
        // both proved one thing twice. What is actually worth asserting is that
        // the refusal names the premise that was wrong rather than some
        // neighbouring one, and that a working fixture does link.
        match link_declarations(&wrong, 0) {
            Err(LinkRefusal::WrongRequirementKind { referrer, .. }) => assert_eq!(
                referrer,
                Referrer::Channel {
                    index: 0,
                    premise: ChannelPremise::Receive,
                }
            ),
            other => panic!("expected the receive premise to be named, got {other:?}"),
        }
        assert!(link_declarations(&fixture(), 0).is_ok());
    }

    /// A repeat that authors a maximum of zero can make no progress.
    ///
    /// A admits it: `nonnegative` rejects only a negative, and A's contract
    /// documents N=0 as making the progress edge unreachable. FR-002 requires it
    /// refused, so this module refuses it.
    #[trace("TC-002", "FR-002-AC-3")]
    #[test]
    fn tc_002_a_zero_maximum_repeat_is_refused() {
        for spelling in ["0", "-0", "+0", "000"] {
            let mut zero = fixture();
            let Body::Protocol { controls, .. } = &mut zero[0].body else {
                unreachable!("the fixture is a protocol")
            };
            controls.push(repeat_control(spelling));
            assert_eq!(
                link_declarations(&zero, 0).expect_err("zero progress refuses"),
                LinkRefusal::ZeroProgressIteration {
                    control: 0,
                    locus: locus(),
                },
                "{spelling} is zero however it is spelled"
            );
        }

        // A bounded repeat links.
        let mut bounded = fixture();
        let Body::Protocol { controls, .. } = &mut bounded[0].body else {
            unreachable!("the fixture is a protocol")
        };
        controls.push(repeat_control("1"));
        assert!(
            link_declarations(&bounded, 0).is_ok(),
            "one iteration is progress"
        );
    }

    /// Two roles may not name one authority.
    ///
    /// A dedupes role names, which are display text; nothing dedupes
    /// `role.model`, which is the authority the role speaks for.
    #[trace("TC-002", "FR-002-AC-3")]
    #[test]
    fn tc_002_two_roles_naming_one_authority_are_refused() {
        let mut duplicate = fixture();
        let Body::Protocol { roles, .. } = &mut duplicate[0].body else {
            unreachable!("the fixture is a protocol")
        };
        // A different name and a different binding, the same authority.
        roles.push(role("Seller", 1));

        assert_eq!(
            link_declarations(&duplicate, 0).expect_err("one authority, two roles"),
            LinkRefusal::DuplicateRoleAuthority {
                role: 1,
                model: (0, 0),
                locus: locus(),
            }
        );
    }

    /// FR-002-AC-4's semantic half, for the four mutations that used to pass.
    ///
    /// `LinkedProtocol` carried identities and channels only, so two packages
    /// differing solely in a repeat maximum, a parallel join, a choice's
    /// visibility or the finish closure linked to equal values — while the
    /// matrix claimed AC-4 was executing.
    #[trace("TC-002", "FR-002-AC-4")]
    #[test]
    fn tc_002_changing_an_authored_control_changes_the_linked_protocol() {
        let with_repeat = |maximum: &str| {
            let mut package = fixture();
            let Body::Protocol { controls, .. } = &mut package[0].body else {
                unreachable!("the fixture is a protocol")
            };
            controls.push(repeat_control(maximum));
            link_declarations(&package, 0).expect("a bounded repeat links")
        };
        assert_ne!(
            with_repeat("1"),
            with_repeat("9999"),
            "an authored iteration maximum is part of the linked protocol"
        );

        let with_join = |join: Vec<u32>| {
            let mut package = fixture();
            let Body::Protocol { controls, .. } = &mut package[0].body else {
                unreachable!("the fixture is a protocol")
            };
            controls.push(Control {
                name: "fork".to_owned(),
                original_node: 0,
                locus: locus(),
                operation: ControlOperation::Parallel {
                    branches: vec![Branch {
                        label: "left".to_owned(),
                        body: handle(0),
                        locus: locus(),
                    }],
                    join,
                },
            });
            link_declarations(&package, 0).expect("a parallel links")
        };
        assert_ne!(
            with_join(vec![0]),
            with_join(vec![0, 1, 2]),
            "an authored join is part of the linked protocol"
        );

        let with_closure = |closure: u32| {
            let mut package = fixture();
            let Body::Protocol { finish, .. } = &mut package[0].body else {
                unreachable!("the fixture is a protocol")
            };
            finish.closure = closure;
            link_declarations(&package, 0).expect("the fixture links")
        };
        assert_ne!(
            with_closure(0),
            with_closure(1),
            "the finish closure is part of the linked protocol"
        );
    }

    /// Two packages from two producers do not share identities.
    ///
    /// Every identity but the definition's used to be table positions only —
    /// twelve bytes with no package in them — so two declarations differing in
    /// package, identity and clause produced byte-identical role-instance,
    /// participant and channel identities. A consumer collecting from both into
    /// one map merged two unrelated parties.
    #[trace("TC-002", "FR-002-AC-1")]
    #[test]
    fn tc_002_two_packages_do_not_share_identities() {
        let first = link_declarations(&fixture(), 0).expect("the fixture links");

        let mut other = fixture();
        other[0].requirement.package = "agent-ix/other".to_owned();
        other[0].requirement.identity = "another-requirement".to_owned();
        other[0].clause = "another-clause".to_owned();
        let second = link_declarations(&other, 0).expect("the other fixture links");

        assert_ne!(first.definition(), second.definition());
        assert_ne!(
            first.role_instances()[0].identity_bytes(),
            second.role_instances()[0].identity_bytes(),
            "a role instance must not be shared across packages"
        );
        assert_ne!(
            first.participants()[0].identity_bytes(),
            second.participants()[0].identity_bytes()
        );
        assert_ne!(
            first.channels().keys().next().expect("one channel"),
            second.channels().keys().next().expect("one channel"),
            "a channel must not be shared across packages"
        );
        assert_ne!(first, second);
    }
}
