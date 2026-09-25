// SPDX-License-Identifier: AGPL-3.0-or-later
//! Finite linked-protocol graph. Owner chunk: **C-12** (submodule: C-13).
//!
//! FR-002 requires one finite typed representation of an admitted protocol,
//! with every identity kind kept distinct and with neither input order nor
//! display text participating in what is linked. [`crate::link::LinkedProtocol`] is that
//! representation, and [`crate::link::link`] builds it.
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
//!   here. [`crate::link::LinkRefusal::ZeroProgressIteration`].
//! - **Duplicate role authorities.** A dedupes role *names*, which are display
//!   text. Nothing dedupes `role.model`, the export that says which authority
//!   the role speaks for. [`crate::link::LinkRefusal::DuplicateRoleAuthority`].
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
//! index in that declaration's binding table, its declared
//! [`quire_spec_language::protocol_artifact::wire::BindingKind`] tag,
//! and the exact authority reference the producer attached to it. Those are
//! producer-supplied and exact. They are encoded canonically by
//! [`crate::link::requirement_identity`] with length-prefixed members, so no two distinct
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
//! [`crate::link::LinkedProtocol`] implements `Eq` and `Hash` over its structural content
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
/// Owns the accumulation of independently established located diagnostics, so
/// a refusal reports every independent finding rather than only the first.
pub mod diagnostics;

/// The `FR-002-AC-2` interleaving proof. Owner chunk: **C-14**.
///
/// Test-only: AC-2 is a property of the retained causal graph rather than a
/// refusal, so nothing in the shipped surface consumes it. See the module
/// documentation.
#[cfg(test)]
mod interleaving;

use diagnostics::{Accumulator, Diagnostics};

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use quire_spec_language::protocol_artifact::{
    wire::{
        BindingKind, BindingRequirement, Body, CausalEdge, Channel, Control, ControlOperation,
        Declaration, Integer, Locus, Ordering,
    },
    AdmittedPackage, NumberWire,
};

use crate::bounds::inventory::{
    BudgetDimension, ChargeError, OperationOutcome, ResourceAccountant, SourceSite,
};
use crate::bounds::{SiteId, SiteKind, UnitLabel};
use crate::ids::{
    AttemptId, ChannelId, ComponentId, DefinitionId, DeliveryId, EffectId, EndpointId,
    IdentityKind, MessageId, ParticipantId, ReceiveId, RoleInstanceId, SendId, WorkflowInstanceId,
};

pub(crate) const DEFINITION_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("link/definition/select"),
    "link::link",
    BudgetDimension::Definition,
    SiteKind::Iteration,
);
pub(crate) const NODE_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("link/node/retain"),
    "link::link",
    BudgetDimension::Node,
    SiteKind::Retention,
);
pub(crate) const EDGE_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("link/edge/traverse"),
    "link::link",
    BudgetDimension::Edge,
    SiteKind::Traversal,
);
pub(crate) const ROLE_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("link/role/retain"),
    "link::link",
    BudgetDimension::Role,
    SiteKind::Retention,
);
pub(crate) const MESSAGE_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("link/message/retain"),
    "link::link",
    BudgetDimension::Message,
    SiteKind::Retention,
);
pub(crate) const BRANCH_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("link/interleaving/branch/queue"),
    "link::interleaving",
    BudgetDimension::Branch,
    SiteKind::Queue,
);
pub(crate) const ITERATION_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("link/repeat/iterate"),
    "link::control_shape",
    BudgetDimension::Iteration,
    SiteKind::Iteration,
);
pub(crate) const ACTIVE_INSTANCE_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("link/active-instance/retain"),
    "link::link",
    BudgetDimension::ActiveInstance,
    SiteKind::Retention,
);
pub(crate) const GRAPH_ALLOCATION_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("link/graph/allocate"),
    "link::link",
    BudgetDimension::AccountedAllocation,
    SiteKind::Allocation,
);

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
        /// The original source region of the referring record.
        ///
        /// The index is an authored field of that record, and `FR-002-AC-3`
        /// requires an unresolved reference refused at its source location. The
        /// site that range-checks it already holds the referrer's `locus` — it
        /// passes the same one to [`LinkRefusal::WrongRequirementKind`] on the
        /// very next line — so an earlier revision that dropped it here was
        /// discarding a region it had, not reporting one it lacked.
        locus: Locus,
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
        /// The original source region of the later of the two records.
        ///
        /// The later one, because that is the record an author would delete or
        /// change to resolve the collision; the earlier one is not at fault by
        /// itself. Both records are authored and both carry a region, so this
        /// finding is located even though what it refuses — one identity from
        /// two — is a fact about the pair rather than about either.
        locus: Locus,
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

impl LinkRefusal {
    /// The original source region this finding is about, when it has one.
    ///
    /// Two variants return `None`, and neither is about the submitted artifact.
    /// [`Self::NoSuchDeclaration`] and [`Self::NotAProtocol`] both refuse the
    /// **caller's own `declaration` argument** to [`link`] — a selection made at
    /// the call site, not a record in the package — so there is no authored text
    /// to point at. Manufacturing a zero span, or borrowing the package's, would
    /// put a caret over source that is not what was refused. See [`diagnostics`]
    /// for why that is an absence rather than a placeholder.
    ///
    /// Every finding that *is* about the artifact carries a region, which is
    /// what `FR-002`'s "independent **located** diagnostics established from the
    /// submitted artifact" asks for. Two earlier revisions of this chunk fell
    /// short of that and defended the shortfall in prose: the first left
    /// [`Self::NoSuchRequirement`] unlocated although the referrer's region was
    /// in hand at the site that checks it, and the second left
    /// [`Self::CollidingIdentity`] unlocated although both colliding records are
    /// authored.
    #[must_use]
    pub fn locus(&self) -> Option<&Locus> {
        match self {
            Self::NoSuchRequirement { locus, .. }
            | Self::WrongRequirementKind { locus, .. }
            | Self::ZeroProgressIteration { locus, .. }
            | Self::DuplicateRoleAuthority { locus, .. }
            | Self::CollidingIdentity { locus, .. } => Some(locus),
            Self::NoSuchDeclaration { .. } | Self::NotAProtocol { .. } => None,
        }
    }
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
                ..
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
                // Debug, not a minted label: `IdentityKind::tag()` published
                // sixteen lowercase strings, eight of which re-spelled
                // FR-210's protocol-subject vocabulary, and was removed for
                // that reason. The variant name is Rust's own rendering of
                // this crate's discriminator, which is what the neighbouring
                // arms already print for `Referrer` and the requirement kinds.
                write!(f, "two records produced one {kind:?} identity")
            }
        }
    }
}

impl std::error::Error for LinkRefusal {}

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

/// One linked channel: its four bound occurrences and its authored premise set.
///
/// The four occurrences are separate types, so a send can never be passed where
/// its paired receive is expected, and a delivery is never its message. They are
/// *references* — which binding requirement each premise resolved to — and they
/// are not the premises themselves; two channels differing in sender, receiver,
/// message type, ordering or delivery cardinality resolve the same four
/// requirements. [`ChannelPremises`] is what makes those axes identity-bearing,
/// and an earlier revision of this doc counted it as a fifth premise rather than
/// as the answer to a different question.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct LinkedChannel {
    message: MessageId,
    send: SendId,
    receive: ReceiveId,
    delivery: DeliveryId,
    premises: ChannelPremises,
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

/// The authored premises of one channel, with its display name excluded.
///
/// `FR-216` fixes the premise set's key as six components — sender, receiver,
/// message types, ordering policy, delivery cardinality and duplication premise
/// — and `FR-054-AC-4` makes every one of them significant: "Changing any
/// channel premise changes the conformance request identity and its result
/// provenance." `FR-002-AC-4` carries the same obligation into the linked
/// representation.
///
/// An earlier revision of [`LinkedChannel`] carried the four *binding
/// requirements* and nothing else, so a channel whose sender, receiver, message
/// type, ordering policy or delivery cardinality was changed linked to an
/// **equal** value while `spec/tests.md` recorded FR-002-AC-4 as executing.
/// That is the same shape [`ControlShape`] was added to close, found the same
/// way: by mutating one axis at a time and comparing.
///
/// # Five of the six, and the sixth is an open question upstream
///
/// This struct carries five: `from` is the sender, `to` the receiver,
/// `message_type` the message type, `ordering` the ordering policy, and
/// `delivery` the cardinality. A's `Channel` at the pinned revision publishes
/// no field for the **duplication premise**, and it is not derived here —
/// reading "duplication can occur" off an interval bound would be this crate
/// interpreting A's numerals, and `FR-216` Absence forbids reading an absent
/// premise as a default.
///
/// That is a contradiction between a requirement and an artifact, and
/// `CLAUDE.md` says a contradiction is a stop rather than a paragraph. It is
/// asked, not absorbed: **`agent-ix/quire-spec-language` issue 85** puts the
/// question to the producer, and `FR-216`'s own Open gap 1 records the
/// specification half of the same disagreement. Until one of them answers,
/// `FR-002-AC-4` and `FR-054-AC-4` are **partially backed** in `spec/tests.md`
/// — five components proven identity-bearing, one with no authored field to
/// mutate — and this doc comment claims nothing further.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ChannelPremises {
    sender: u32,
    receiver: u32,
    message_type: u32,
    ordering: &'static str,
    ordering_scope: Vec<u32>,
    delivery: (u32, u32),
}

impl ChannelPremises {
    // No accessors.
    //
    // What `FR-002-AC-4` and `FR-054-AC-4` require of this type is that
    // changing a component changes the linked protocol, and that is carried by
    // `PartialEq` and `Hash` over the fields. A reader that wants a component
    // back has no obligation asking for it: nothing in this crate consumes one,
    // and `Debug` already renders the set for a diagnostic.
    //
    // An earlier revision shipped six `pub` accessors with no caller anywhere
    // in `src/` or `tests/`. That is the same API debt `link::interleaving`
    // declines to take on in its own module documentation, and the argument
    // does not stop applying because the item is an accessor rather than an
    // enumerator. When a consumer needs a component, it arrives with that
    // consumer.
}

/// Project one channel's authored premises.
///
/// The two tags are A's own `Ordering` spellings at the pinned revision, quoted
/// as [`body_tag`] and [`control_shape`] quote theirs: A's `tagged!` records
/// publish no accessor for them, and the alternative — collapsing the two into
/// one untagged shape — would make an unordered channel and a FIFO one
/// comparable on their scopes alone.
fn channel_premises(channel: &Channel) -> ChannelPremises {
    let (ordering, ordering_scope) = match &channel.ordering {
        Ordering::Unordered {} => ("unordered", Vec::new()),
        Ordering::Fifo {
            binder,
            key,
            anchor,
        } => ("fifo", vec![binder.index, key.index, anchor.index]),
    };
    ChannelPremises {
        sender: channel.from.index,
        receiver: channel.to.index,
        message_type: channel.message_type,
        ordering,
        ordering_scope,
        delivery: (
            numeric_fingerprint(&channel.delivery.lower),
            numeric_fingerprint(&channel.delivery.upper),
        ),
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

    /// Whether this control's sub-bodies are alternatives rather than
    /// concurrent branches.
    ///
    /// A derives the same `branch` and `join` edge kinds for a `Parallel` and
    /// for a `Choice`, so the causal edge set alone cannot tell concurrency
    /// from alternation. The operation kind can, and this is the one place that
    /// reads it: the spellings are the ones `control_shape` writes, a few
    /// lines below.
    ///
    /// - `choice` — exactly one case runs (`FR-053`).
    /// - `repeat` — the body arm and the exhausted arm are alternatives, and
    ///   the body arm repeats rather than running once.
    /// - `await` — the `then` arm and the `timeout` arm are alternatives.
    ///
    /// A `sequence` and a `parallel` admit no alternative, and a leaf control
    /// has no sub-body to choose between.
    #[must_use]
    pub fn admits_alternatives(&self) -> bool {
        matches!(self.kind, "choice" | "repeat" | "await")
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct LinkWork {
    definitions: usize,
    nodes: usize,
    edges: usize,
    roles: usize,
    messages: usize,
    branches: usize,
    iterations: usize,
    active_instances: usize,
}

trait IdentityBuilder {
    type Error;

    fn build<T>(
        &mut self,
        identity: &[u8],
        display: &str,
        constructor: fn(&[u8], &str) -> T,
    ) -> Result<T, Self::Error>;
}

struct UnboundedIdentityBuilder;

impl IdentityBuilder for UnboundedIdentityBuilder {
    type Error = core::convert::Infallible;

    fn build<T>(
        &mut self,
        identity: &[u8],
        display: &str,
        constructor: fn(&[u8], &str) -> T,
    ) -> Result<T, Self::Error> {
        Ok(constructor(identity, display))
    }
}

struct BoundedIdentityBuilder<'resources, 'inventory> {
    resources: &'resources mut ResourceAccountant<'inventory>,
}

impl IdentityBuilder for BoundedIdentityBuilder<'_, '_> {
    type Error = ChargeError;

    fn build<T>(
        &mut self,
        identity: &[u8],
        display: &str,
        constructor: fn(&[u8], &str) -> T,
    ) -> Result<T, Self::Error> {
        crate::ids::charge_storage(identity, display, self.resources)?;
        Ok(constructor(identity, display))
    }
}

enum LinkBuildError<E> {
    Domain(Diagnostics),
    Identity(E),
}

fn link_work(
    declarations: &[Declaration],
    declaration: u32,
) -> Result<Option<LinkWork>, ChargeError> {
    let index = usize::try_from(declaration).unwrap_or(usize::MAX);
    let Some(declared) = declarations.get(index) else {
        return Ok(None);
    };
    let Body::Protocol {
        roles,
        channels,
        controls,
        causal_edges,
        ..
    } = &declared.body
    else {
        return Ok(None);
    };

    let mut branches = 0usize;
    let mut iterations = 0usize;
    for control in controls {
        let increment = match &control.operation {
            ControlOperation::Choice { cases, .. } => cases.len(),
            ControlOperation::Parallel { branches, .. } => branches.len(),
            ControlOperation::Repeat { .. } | ControlOperation::Await { .. } => 2,
            ControlOperation::Sequence { .. }
            | ControlOperation::Event { .. }
            | ControlOperation::Check { .. }
            | ControlOperation::Commit { .. } => 0,
        };
        branches = branches
            .checked_add(increment)
            .ok_or(ChargeError::CounterOverflow {
                site: BRANCH_SOURCE.id(),
                consumed: branches,
                requested: increment,
            })?;
        if matches!(control.operation, ControlOperation::Repeat { .. }) {
            iterations = iterations
                .checked_add(1)
                .ok_or(ChargeError::CounterOverflow {
                    site: ITERATION_SOURCE.id(),
                    consumed: iterations,
                    requested: 1,
                })?;
        }
    }
    let workflow_instances = declared
        .bindings
        .iter()
        .filter(|requirement| requirement.kind == BindingKind::WorkflowInstance)
        .count();
    let active_instances = roles
        .len()
        .checked_add(usize::from(workflow_instances == 1))
        .ok_or(ChargeError::CounterOverflow {
            site: ACTIVE_INSTANCE_SOURCE.id(),
            consumed: roles.len(),
            requested: usize::from(workflow_instances == 1),
        })?;

    Ok(Some(LinkWork {
        definitions: declarations.len(),
        nodes: controls.len(),
        edges: causal_edges.len(),
        roles: roles.len(),
        messages: channels.len(),
        branches,
        iterations,
        active_instances,
    }))
}

fn link_declarations_bounded(
    declarations: &[Declaration],
    declaration: u32,
    resources: &mut ResourceAccountant<'_>,
) -> OperationOutcome<LinkedProtocol, Diagnostics> {
    let work = match link_work(declarations, declaration) {
        Ok(Some(work)) => work,
        Ok(None) => {
            return OperationOutcome::from_domain(link_declarations(declarations, declaration));
        }
        Err(error) => return OperationOutcome::from_charge(error),
    };
    let charges = [
        (GRAPH_ALLOCATION_SOURCE.id(), 1),
        (DEFINITION_SOURCE.id(), work.definitions),
        (NODE_SOURCE.id(), work.nodes),
        (EDGE_SOURCE.id(), work.edges),
        (ROLE_SOURCE.id(), work.roles),
        (MESSAGE_SOURCE.id(), work.messages),
        (BRANCH_SOURCE.id(), work.branches),
        (ITERATION_SOURCE.id(), work.iterations),
        (ACTIVE_INSTANCE_SOURCE.id(), work.active_instances),
    ];
    if let Err(error) = resources.charge_all(&charges) {
        return OperationOutcome::from_charge(error);
    }
    let mut identities = BoundedIdentityBuilder { resources };
    match link_declarations_with(declarations, declaration, &mut identities) {
        Ok(linked) => OperationOutcome::Completed(linked),
        Err(LinkBuildError::Domain(error)) => OperationOutcome::DomainError(error),
        Err(LinkBuildError::Identity(error)) => OperationOutcome::from_charge(error),
    }
}

/// Link the protocol declared at `declaration` in an admitted package.
///
/// The package has already been admitted by A, which is where the structural
/// refusals live; see the module documentation for what this does and does not
/// check.
///
/// A refusal returns no linked representation at all, never a partially built
/// one, and returns **every** independent finding rather than the first. The
/// two are the same obligation read from both ends: the traversal continues
/// past a failure precisely so that it can report the rest, and the result type
/// is what stops a partially built graph escaping alongside them. See
/// [`diagnostics`] for which checks are independent and which are skipped
/// because their input was never established.
pub fn link(package: &AdmittedPackage, declaration: u32) -> Result<LinkedProtocol, Diagnostics> {
    link_declarations(&package.package().declarations, declaration)
}

/// Link one admitted protocol under the shared QProtocol caller budget.
///
/// Every structural population the linker will retain, traverse, queue, or
/// iterate is charged transactionally before identity construction or graph
/// mapping begins.
pub fn link_bounded(
    package: &AdmittedPackage,
    declaration: u32,
    resources: &mut ResourceAccountant<'_>,
) -> OperationOutcome<LinkedProtocol, Diagnostics> {
    resources.run_unit(UnitLabel::new("link::link_bounded"), |resources| {
        link_declarations_bounded(&package.package().declarations, declaration, resources)
    })
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
) -> Result<LinkedProtocol, Diagnostics> {
    let mut identities = UnboundedIdentityBuilder;
    match link_declarations_with(declarations, declaration, &mut identities) {
        Ok(linked) => Ok(linked),
        Err(LinkBuildError::Domain(error)) => Err(error),
        Err(LinkBuildError::Identity(never)) => match never {},
    }
}

fn link_declarations_with<B: IdentityBuilder>(
    declarations: &[Declaration],
    declaration: u32,
    identities: &mut B,
) -> Result<LinkedProtocol, LinkBuildError<B::Error>> {
    // The two gating checks. Neither accumulates with anything, because a
    // declaration that is absent or is not a protocol leaves no tables to walk:
    // every check below reads one of them.
    let index = usize::try_from(declaration).unwrap_or(usize::MAX);
    let declared = declarations.get(index).ok_or_else(|| {
        LinkBuildError::Domain(Diagnostics::of(LinkRefusal::NoSuchDeclaration {
            selected: declaration,
            available: declarations.len(),
        }))
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
        return Err(LinkBuildError::Domain(Diagnostics::of(
            LinkRefusal::NotAProtocol {
                declaration,
                found: body_tag(&declared.body),
            },
        )));
    };

    let mut found = Accumulator::default();

    // Every identity below is scoped by the owning declaration's own producer
    // coordinates, so two packages cannot mint the same one.
    let scope = declaration_identity(declaration, declared);
    let linker = Linker {
        scope: scope.clone(),
        declaration,
        declared,
    };

    let definition = identities
        .build(&scope, &declared.name, DefinitionId::new)
        .map_err(LinkBuildError::Identity)?;

    let workflow_instance = match linker.sole_requirement(BindingKind::WorkflowInstance) {
        Some((index, requirement)) => Some(
            identities
                .build(
                    &requirement_identity(&scope, declaration, index, requirement),
                    &requirement.name,
                    WorkflowInstanceId::new,
                )
                .map_err(LinkBuildError::Identity)?,
        ),
        None => None,
    };

    // Refusals A does not make. See the module documentation: A's validator
    // covers the edge set, joins, cycles and reference resolution, and these are
    // the items on FR-002's list that it leaves open.
    //
    // Every control is checked, not just up to the first zero: two repeats that
    // can make no progress are two authoring mistakes, and reporting one of
    // them sends the author back for a second link to be told about the other.
    for (position, control) in controls.iter().enumerate() {
        if let ControlOperation::Repeat { maximum, .. } = &control.operation {
            if is_zero(maximum) {
                found.record(LinkRefusal::ZeroProgressIteration {
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
            found.record(LinkRefusal::DuplicateRoleAuthority {
                role: position,
                model,
                locus: role.locus.clone(),
            });
        }
        // Pushed whether or not it duplicated one already seen. A third role
        // over the same authority is then reported against the first, not
        // against the second, so N roles over one authority give N-1 findings
        // rather than one.
        seen_authorities.push(model);
    }

    // Each role's binding requirement is resolved independently of every other
    // role's. A role whose requirement is absent or of the wrong kind
    // contributes no identity — building one from the wrong kind is the
    // confusion the sixteen newtypes exist to prevent — so it is recorded and
    // skipped, and the next role is still resolved.
    let mut role_instances: Vec<(RoleInstanceId, Locus)> = Vec::new();
    for (position, role) in roles.iter().enumerate() {
        let referrer = Referrer::Role { index: position };
        let resolved = found.keep(linker.requirement(
            referrer,
            role.instance,
            BindingKind::RoleInstance,
            &role.locus,
        ));
        if let Some(requirement) = resolved {
            let identity = identities
                .build(
                    &requirement_identity(&scope, declaration, role.instance, requirement),
                    &role.name,
                    RoleInstanceId::new,
                )
                .map_err(LinkBuildError::Identity)?;
            role_instances.push((identity, role.locus.clone()));
        }
    }

    // Participants, components, endpoints, attempts and effects are not named by
    // the protocol's own tables: they are the declaration's binding
    // requirements of those kinds. Reading them from the binding table rather
    // than inferring them from control bodies keeps each one's identity the
    // producer's, and keeps this linker out of A's control-graph semantics.
    let participants = linker
        .all_of(BindingKind::Participant, ParticipantId::new, identities)
        .map_err(LinkBuildError::Identity)?;
    let components = linker
        .all_of(BindingKind::Component, ComponentId::new, identities)
        .map_err(LinkBuildError::Identity)?;
    let endpoints = linker
        .all_of(BindingKind::Endpoint, EndpointId::new, identities)
        .map_err(LinkBuildError::Identity)?;
    let attempts = linker
        .all_of(BindingKind::Attempt, AttemptId::new, identities)
        .map_err(LinkBuildError::Identity)?;
    let effects = linker
        .all_of(BindingKind::Effect, EffectId::new, identities)
        .map_err(LinkBuildError::Identity)?;

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
        // All four premises, and in this order, before any of them is used. A
        // channel with a bad message and a bad receive is two findings: the
        // earlier `?` chain reported the message and left the receive to be
        // discovered on a later run, which is the behaviour this chunk exists
        // to replace.
        let message = found.keep(premise(ChannelPremise::Message, channel.message));
        let send = found.keep(premise(ChannelPremise::Send, channel.send));
        let receive = found.keep(premise(ChannelPremise::Receive, channel.receive));
        let delivery = found.keep(premise(ChannelPremise::Delivery, channel.delivery_instance));

        // A channel's linked form is derived from premises that resolved, so
        // it is a dependent check: a channel missing one contributes nothing
        // rather than a record built from three quarters of its premises. The
        // compiler enforces that, not a convention — the four are `Option`s and
        // there is no `LinkedChannel` to build without all four.
        let (Some(message), Some(send), Some(receive), Some(delivery)) =
            (message, send, receive, delivery)
        else {
            continue;
        };

        let identity = identities
            .build(
                &channel_identity(&scope, declaration, position, channel.message),
                &channel.name,
                ChannelId::new,
            )
            .map_err(LinkBuildError::Identity)?;
        let linked = LinkedChannel {
            message: identities
                .build(
                    &requirement_identity(&scope, declaration, channel.message, message),
                    &message.name,
                    MessageId::new,
                )
                .map_err(LinkBuildError::Identity)?,
            send: identities
                .build(
                    &requirement_identity(&scope, declaration, channel.send, send),
                    &send.name,
                    SendId::new,
                )
                .map_err(LinkBuildError::Identity)?,
            receive: identities
                .build(
                    &requirement_identity(&scope, declaration, channel.receive, receive),
                    &receive.name,
                    ReceiveId::new,
                )
                .map_err(LinkBuildError::Identity)?,
            delivery: identities
                .build(
                    &requirement_identity(&scope, declaration, channel.delivery_instance, delivery),
                    &delivery.name,
                    DeliveryId::new,
                )
                .map_err(LinkBuildError::Identity)?,
            premises: channel_premises(channel),
        };
        // The key is checked, and the finding is not dead code.
        //
        // `channel_identity` mixes `position` into the preimage, so two
        // channels at two distinct positions normally cannot produce one
        // `ChannelId` — which is why the five scans below were deleted as
        // unreachable. This one is not quite in that class: `channel_identity`
        // saturates the position with `u32::try_from(..).unwrap_or(u32::MAX)`,
        // so every position at or past `u32::MAX` is written as `u32::MAX` and
        // two of them with one message do collide.
        //
        // The reason to keep it is the cost of the alternative. Dropping the
        // check does not drop a refusal; it turns one into a *silent
        // overwrite*, because `BTreeMap::insert` replaces. The linker would
        // return `Ok` with a channel missing from the representation and no
        // diagnostic anywhere. One `is_some()` on a map insert buys that back,
        // and it is O(1) rather than the quadratic scans that were deleted.
        match linked_channels.entry(identity) {
            std::collections::btree_map::Entry::Occupied(existing) => {
                found.record(LinkRefusal::CollidingIdentity {
                    kind: IdentityKind::Channel,
                    identity: existing.key().identity_bytes().to_vec(),
                    locus: channel.locus.clone(),
                });
            }
            std::collections::btree_map::Entry::Vacant(vacant) => {
                vacant.insert(linked);
            }
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
    //
    // It runs over the roles that resolved. A collision among those is a real
    // collision, so a finding here is never spurious; the absence of one proves
    // nothing when a role was skipped above, which is the incompleteness
    // `diagnostics` states rather than hides.
    check_distinct_roles(&role_instances, &mut found);

    if let Some(diagnostics) = found.into_diagnostics() {
        return Err(LinkBuildError::Domain(diagnostics));
    }

    // The loci were carried only so a collision could name the later record's
    // region; the linked representation stores the identities alone.
    let role_instances: Vec<RoleInstanceId> = role_instances
        .into_iter()
        .map(|(identity, _locus)| identity)
        .collect();

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
        let requirement = bindings
            .get(index)
            .ok_or_else(|| LinkRefusal::NoSuchRequirement {
                referrer,
                selected,
                available: bindings.len(),
                locus: locus.clone(),
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
    fn all_of<T, B: IdentityBuilder>(
        &self,
        kind: BindingKind,
        build: fn(&[u8], &str) -> T,
        identities: &mut B,
    ) -> Result<Vec<T>, B::Error> {
        self.declared
            .bindings
            .iter()
            .enumerate()
            .filter(|(_, requirement)| requirement.kind == kind)
            .map(|(index, requirement)| {
                let index = u32::try_from(index).unwrap_or(u32::MAX);
                identities.build(
                    &requirement_identity(&self.scope, self.declaration, index, requirement),
                    &requirement.name,
                    build,
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
fn check_distinct_roles(values: &[(RoleInstanceId, Locus)], found: &mut Accumulator) {
    let mut seen: BTreeSet<&[u8]> = BTreeSet::new();
    for (value, locus) in values {
        if !seen.insert(value.identity_bytes()) {
            found.record(LinkRefusal::CollidingIdentity {
                kind: IdentityKind::RoleInstance,
                identity: value.identity_bytes().to_vec(),
                locus: locus.clone(),
            });
        }
    }
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
    use qsl_foundation::ByteDigest;
    use quire_spec_language::protocol_artifact::wire::{
        Activation, ArtifactKind, ArtifactRef, Branch, Channel, ChoiceCase, Control,
        ControlOperation, EdgeKind, Endpoint, Execution, ExportRef, Finish, Handle, Integer,
        Interval, Nullable, Ordering, Port, Requirement, Revision, Role, Span, Subject, Wire,
    };
    use quire_spec_language::protocol_artifact::NumberWire;

    use crate::bounds::inventory::{
        production_inventory, Budget, BudgetDimension, OperationOutcome, ResourceAccountant,
    };

    use super::interleaving::{interleavings, precedence, Endpoint as Step};
    use super::*;

    fn handle(index: u32) -> Handle {
        Handle {
            declaration: 0,
            index,
        }
    }

    /// The region of a record that is not in one of the stamped tables.
    ///
    /// The declaration itself and its finish clause: one of each per fixture,
    /// so there is nothing to tell apart.
    fn locus() -> Locus {
        locus_at(0, 0)
    }

    /// A's source table index.
    const BINDINGS: u32 = 1;
    /// A's source table index.
    const ROLES: u32 = 2;
    /// A's source table index.
    const CHANNELS: u32 = 3;
    /// A's source table index.
    const CONTROLS: u32 = 4;

    /// One distinct source region per record.
    ///
    /// Every fixture record used to carry the same `source 0, 0..1`, which made
    /// every `locus` in every expectation compare equal to every other. A
    /// refusal that named the wrong role, control or channel still matched, so
    /// `FR-002-AC-3`'s "refused **at its source location**" was asserted by
    /// tests that could not see the location. These coordinates are arbitrary —
    /// A's `source` is an index into a source table this crate does not resolve
    /// — but they are *distinct*, which is the only property the assertions
    /// need.
    fn locus_at(source: u32, index: u32) -> Locus {
        Locus {
            source,
            span: Span {
                start: index,
                end: index + 1,
            },
        }
    }

    /// Give every record in a fixture its own region, by table and position.
    ///
    /// Applied after a fixture is built and after any mutation that adds or
    /// moves a record, so a record's region always follows its position rather
    /// than being fixed when the helper that made it ran.
    fn stamp(declarations: &mut [Declaration]) {
        for declaration in declarations.iter_mut() {
            for (index, requirement) in declaration.bindings.iter_mut().enumerate() {
                requirement.locus = locus_at(BINDINGS, index as u32);
            }
            let Body::Protocol {
                roles,
                channels,
                controls,
                ..
            } = &mut declaration.body
            else {
                continue;
            };
            for (index, role) in roles.iter_mut().enumerate() {
                role.locus = locus_at(ROLES, index as u32);
            }
            for (index, channel) in channels.iter_mut().enumerate() {
                channel.locus = locus_at(CHANNELS, index as u32);
            }
            for (index, control) in controls.iter_mut().enumerate() {
                control.locus = locus_at(CONTROLS, index as u32);
            }
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
        let mut declarations = vec![declaration(
            protocol_body(vec![role("Buyer", 1)], vec![channel("order", 5)]),
            bindings(),
        )];
        stamp(&mut declarations);
        declarations
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
            Diagnostics::of(LinkRefusal::WrongRequirementKind {
                referrer: Referrer::Channel {
                    index: 0,
                    premise: ChannelPremise::Send,
                },
                required: BindingKind::Send,
                found: BindingKind::Delivery,
                locus: locus_at(CHANNELS, 0),
            })
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
            Diagnostics::of(LinkRefusal::NoSuchRequirement {
                referrer: Referrer::Role { index: 0 },
                selected: 99,
                available: bindings().len(),
                locus: locus_at(ROLES, 0),
            })
        );
        // And it is located: the index is authored, so FR-002-AC-3's "at its
        // source location" applies to it as much as to the wrong-kind case.
        assert_eq!(refusal.located().count(), 1);
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
            Diagnostics::of(LinkRefusal::NotAProtocol {
                declaration: 0,
                found: "predicate",
            })
        );
    }

    #[trace("TC-002", "FR-002-AC-3")]
    #[test]
    fn tc_002_a_declaration_outside_the_package_is_refused() {
        assert_eq!(
            link_declarations(&fixture(), 7).expect_err("there is one declaration"),
            Diagnostics::of(LinkRefusal::NoSuchDeclaration {
                selected: 7,
                available: 1,
            })
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
            Diagnostics::of(LinkRefusal::CollidingIdentity {
                kind: IdentityKind::RoleInstance,
                identity: requirement_identity(
                    &declaration_identity(0, &fixture()[0]),
                    0,
                    1,
                    &bindings()[1]
                ),
                locus: locus(),
            })
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
            Err(diagnostics) => match diagnostics.first() {
                LinkRefusal::WrongRequirementKind { referrer, .. } => assert_eq!(
                    *referrer,
                    Referrer::Channel {
                        index: 0,
                        premise: ChannelPremise::Receive,
                    }
                ),
                other => panic!("expected the receive premise to be named, got {other:?}"),
            },
            Ok(linked) => panic!("a wrong premise kind must refuse, got {linked:?}"),
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
                Diagnostics::of(LinkRefusal::ZeroProgressIteration {
                    control: 0,
                    locus: locus(),
                }),
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
    ///
    /// This fixture is two findings, not one, and it always was. Its second
    /// role selects role-instance requirement 1, which is the one the first
    /// role selects, so the two roles also produce one role-instance identity.
    /// The previous revision of this test asserted a sole
    /// `DuplicateRoleAuthority` and carried a comment saying the second role
    /// had "a different binding" — it does not, and the first-refusal-wins
    /// linker returned before reaching the collision, so the comment was never
    /// contradicted by a run. Accumulating the findings is what made the second
    /// one visible.
    #[trace("TC-002", "FR-002-AC-3")]
    #[test]
    fn tc_002_two_roles_naming_one_authority_are_refused() {
        let mut duplicate = fixture();
        let Body::Protocol { roles, .. } = &mut duplicate[0].body else {
            unreachable!("the fixture is a protocol")
        };
        // A different name, the same authority, and the same role-instance
        // binding as `fixture`'s Buyer.
        roles.push(role("Seller", 1));

        let diagnostics = link_declarations(&duplicate, 0).expect_err("one authority, two roles");
        assert_eq!(
            diagnostics.iter().collect::<Vec<_>>(),
            [
                &LinkRefusal::DuplicateRoleAuthority {
                    role: 1,
                    model: (0, 0),
                    locus: locus(),
                },
                &LinkRefusal::CollidingIdentity {
                    kind: IdentityKind::RoleInstance,
                    identity: requirement_identity(
                        &declaration_identity(0, &fixture()[0]),
                        0,
                        1,
                        &bindings()[1]
                    ),
                    locus: locus(),
                },
            ]
        );
    }

    // -----------------------------------------------------------------------
    // C-13: accumulating located diagnostics
    // -----------------------------------------------------------------------

    /// Every zero-progress repeat is reported, not just the first.
    ///
    /// The mutation that proves this: make `link_declarations` return after
    /// recording the first `ZeroProgressIteration` and this fails at three
    /// findings against one, while every other test in this module stays green.
    #[trace("TC-002", "Task-002")]
    #[test]
    fn tc_002_every_zero_progress_repeat_is_reported() {
        let mut zeroes = fixture();
        let Body::Protocol { controls, .. } = &mut zeroes[0].body else {
            unreachable!("the fixture is a protocol")
        };
        controls.push(repeat_control("0"));
        controls.push(repeat_control("1"));
        controls.push(repeat_control("-0"));
        controls.push(repeat_control("000"));

        let diagnostics =
            link_declarations(&zeroes, 0).expect_err("three repeats make no progress");
        let controls: Vec<u32> = diagnostics
            .iter()
            .map(|finding| match finding {
                LinkRefusal::ZeroProgressIteration { control, .. } => *control,
                other => panic!("expected only zero-progress findings, got {other:?}"),
            })
            .collect();

        // Control 1 authors a maximum of one and is absent. The order is the
        // control table's, which is what "declared, not sorted" means.
        assert_eq!(controls, [0, 2, 3]);
    }

    /// All four channel premises are resolved before any is used, so a channel
    /// with two bad premises reports two findings.
    ///
    /// Under the previous `?` chain the receive was never reached: the message
    /// refusal returned first, and an author fixing it learned about the
    /// receive only on the next run.
    #[trace("TC-002", "Task-002")]
    #[test]
    fn tc_002_every_failing_channel_premise_is_reported() {
        let mut wrong = fixture();
        let Body::Protocol { channels, .. } = &mut wrong[0].body else {
            unreachable!("the fixture is a protocol")
        };
        // Binding 9 is the attempt and binding 3 the component: neither is a
        // message, and neither is a receive.
        channels[0].message = 9;
        channels[0].receive = 3;

        let diagnostics = link_declarations(&wrong, 0).expect_err("two premises are wrong");
        let premises: Vec<ChannelPremise> = diagnostics
            .iter()
            .map(|finding| match finding {
                LinkRefusal::WrongRequirementKind {
                    referrer: Referrer::Channel { premise, .. },
                    ..
                } => *premise,
                other => panic!("expected channel-premise findings, got {other:?}"),
            })
            .collect();
        assert_eq!(premises, [ChannelPremise::Message, ChannelPremise::Receive]);
    }

    // A channel that loses a premise contributes nothing to the linked
    // representation, and there is deliberately no test for it: the four
    // premises are `Option`s, so there is no `LinkedChannel` to build without
    // all four and the compiler refuses the alternative. An earlier revision of
    // this chunk did assert it, against the now-deleted channel-identity
    // collision check; that check was unreachable, so the assertion held no
    // matter what the skip did.

    /// A gating refusal carries exactly one finding.
    ///
    /// There is no declaration to walk, so there is nothing to accumulate with,
    /// and reporting anything alongside it would be reporting a check that
    /// never ran.
    #[trace("TC-002", "Task-002")]
    #[test]
    fn tc_002_a_gating_refusal_accumulates_with_nothing() {
        for diagnostics in [
            link_declarations(&fixture(), 7).expect_err("there is one declaration"),
            link_declarations(
                &[declaration(
                    Body::Predicate {
                        parameters: Vec::new(),
                        result: 0,
                        root: handle(0),
                    },
                    bindings(),
                )],
                0,
            )
            .expect_err("a predicate is not a protocol"),
        ] {
            assert_eq!(diagnostics.count(), 1, "got {diagnostics:?}");
        }
    }

    /// Every finding about the submitted artifact carries its source region,
    /// and the two that refuse the caller's own argument carry none.
    ///
    /// `FR-002` asks for "independent located diagnostics established from the
    /// submitted artifact". Two earlier revisions of this chunk left a finding
    /// about artifact content unlocated — first an out-of-range binding index,
    /// then an identity collision — and defended each absence in prose. This
    /// test is what closes that off: it fails if any artifact finding loses its
    /// region, and it fails if a region is invented for a refusal of the
    /// `declaration` argument, which is not artifact content at all.
    #[trace("TC-002", "Task-002")]
    #[test]
    fn tc_002_only_findings_over_authored_text_carry_a_region() {
        let mut mixed = fixture();
        let Body::Protocol {
            roles, controls, ..
        } = &mut mixed[0].body
        else {
            unreachable!("the fixture is a protocol")
        };
        controls.push(repeat_control("0"));
        roles.push(role("Seller", 1));

        // Three findings over artifact content: a zero-progress repeat, a
        // duplicate authority, and the role-instance collision the second role
        // also causes. All three are authored, so all three are located. The
        // facts are read from `located` and `locus`, never from the rendered
        // text — FR-037 forbids deriving a refusal fact from prose.
        let diagnostics = link_declarations(&mixed, 0).expect_err("three findings");
        assert_eq!(diagnostics.count(), 3, "got {diagnostics:?}");
        let located: Vec<&Locus> = diagnostics.located().map(|(_, locus)| locus).collect();
        assert_eq!(located, [&locus(), &locus(), &locus()]);

        // Rendering is presentation, and is asserted only as a line count: one
        // line per finding, and no fourth line invented.
        assert_eq!(diagnostics.to_string().lines().count(), 3);

        // The two refusals of the caller's `declaration` argument are the only
        // unlocated findings, and they stay unlocated: the argument is a
        // selection made at the call site, with no authored text behind it.
        for diagnostics in [
            link_declarations(&fixture(), 7).expect_err("there is one declaration"),
            link_declarations(
                &[declaration(
                    Body::Predicate {
                        parameters: Vec::new(),
                        result: 0,
                        root: handle(0),
                    },
                    bindings(),
                )],
                0,
            )
            .expect_err("a predicate is not a protocol"),
        ] {
            assert_eq!(diagnostics.located().count(), 0, "got {diagnostics:?}");
        }
    }

    /// Many findings still produce no linked representation.
    ///
    /// The two halves of `Task-002`'s requirement are one type: the accumulated
    /// findings are the error, so there is no shape in which a caller receives
    /// both a graph and a reason it should not have one.
    #[trace("TC-002", "FR-002-AC-3")]
    #[test]
    fn tc_002_accumulated_findings_still_emit_no_linked_representation() {
        let mut broken = fixture();
        let Body::Protocol {
            roles,
            channels,
            controls,
            ..
        } = &mut broken[0].body
        else {
            unreachable!("the fixture is a protocol")
        };
        controls.push(repeat_control("0"));
        roles.push(role("Seller", 1));
        channels[0].send = 8;

        let outcome = link_declarations(&broken, 0);
        assert!(outcome.is_err(), "got {outcome:?}");
        let diagnostics = outcome.expect_err("four findings");
        // Exact, not a floor: a floor passes a regression that drops one.
        assert_eq!(diagnostics.count(), 4, "got {diagnostics:?}");
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
    // `Task-002`, not `FR-002-AC-1`. Cross-package distinctness of *link-time*
    // identities is what this establishes; AC-1 is about concurrent workflow
    // instances, role instances, participants, duplicate deliveries and
    // attempts retaining distinct identities at run time, which do not exist
    // here. The module header 860 lines above already said none of these tests
    // carries an AC-1 tag, and `spec/tests.md` repeated it — while this one
    // did, and `quire coverage` counted FR-002 backed partly on it.
    #[trace("TC-002", "Task-002")]
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

    // -----------------------------------------------------------------------
    // C-14: FR-002-AC-2, permitted interleavings
    // -----------------------------------------------------------------------

    /// A leaf control. A derives one `sequence` edge from `enter` to `exit` for
    /// it, which is what makes a branch body an ordered step rather than two
    /// unrelated endpoints.
    fn check_control() -> Control {
        Control {
            name: "step".to_owned(),
            original_node: 0,
            locus: locus(),
            operation: ControlOperation::Check {
                profile: 0,
                value: handle(0),
            },
        }
    }

    fn sequence_control(children: Vec<u32>) -> Control {
        Control {
            name: "then".to_owned(),
            original_node: 0,
            locus: locus(),
            operation: ControlOperation::Sequence {
                children: children.into_iter().map(handle).collect(),
            },
        }
    }

    fn branch(label: &str, body: u32) -> Branch {
        Branch {
            label: label.to_owned(),
            body: handle(body),
            locus: locus(),
        }
    }

    fn endpoint(node: u32, port: Port) -> Endpoint {
        Endpoint {
            node: handle(node),
            port,
        }
    }

    /// One of A's derived edges, written as A derives it.
    fn edge(kind: EdgeKind, owner: u32, from: (u32, Port), to: (u32, Port)) -> CausalEdge {
        CausalEdge {
            kind,
            owner: handle(owner),
            from: endpoint(from.0, from.1),
            to: endpoint(to.0, to.1),
            maximum: Nullable(None),
        }
    }

    /// A parallel of two branches, each an authored sequence of two steps.
    ///
    /// The control table and the edge set are A's, not a model of A's: the
    /// edges below are exactly what A's `edges` derives for this table —
    /// `branch` from the parallel's `enter` to each branch body's `enter`,
    /// `join` from each body's `exit` to the parallel's `exit`, a `sequence`
    /// chain through each body's children, and one `sequence` edge across each
    /// leaf — in A's own sort order. B retains them; it derives none of them.
    fn parallel_fixture() -> Vec<Declaration> {
        let mut declarations = fixture();
        let Body::Protocol {
            controls,
            causal_edges,
            ..
        } = &mut declarations[0].body
        else {
            unreachable!("the fixture is a protocol")
        };

        controls.push(Control {
            name: "split".to_owned(),
            original_node: 0,
            locus: locus(),
            operation: ControlOperation::Parallel {
                branches: vec![branch("left", 1), branch("right", 2)],
                join: vec![0, 1],
            },
        });
        controls.push(sequence_control(vec![3, 4]));
        controls.push(sequence_control(vec![5, 6]));
        for _ in 0..4 {
            controls.push(check_control());
        }

        use EdgeKind::{Branch as BranchEdge, Join, Sequence};
        use Port::{Enter, Exit};
        *causal_edges = vec![
            edge(BranchEdge, 0, (0, Enter), (1, Enter)),
            edge(BranchEdge, 0, (0, Enter), (2, Enter)),
            edge(Join, 0, (1, Exit), (0, Exit)),
            edge(Join, 0, (2, Exit), (0, Exit)),
            edge(Sequence, 1, (1, Enter), (3, Enter)),
            edge(Sequence, 1, (3, Exit), (4, Enter)),
            edge(Sequence, 1, (4, Exit), (1, Exit)),
            edge(Sequence, 2, (2, Enter), (5, Enter)),
            edge(Sequence, 2, (5, Exit), (6, Enter)),
            edge(Sequence, 2, (6, Exit), (2, Exit)),
            edge(Sequence, 3, (3, Enter), (3, Exit)),
            edge(Sequence, 4, (4, Enter), (4, Exit)),
            edge(Sequence, 5, (5, Enter), (5, Exit)),
            edge(Sequence, 6, (6, Enter), (6, Exit)),
        ];

        stamp(&mut declarations);
        declarations
    }

    /// The endpoints of one branch, in the order that branch authored them.
    fn left_branch() -> [Step; 6] {
        [
            (1, "enter"),
            (3, "enter"),
            (3, "exit"),
            (4, "enter"),
            (4, "exit"),
            (1, "exit"),
        ]
    }

    fn right_branch() -> [Step; 6] {
        [
            (2, "enter"),
            (5, "enter"),
            (5, "exit"),
            (6, "enter"),
            (6, "exit"),
            (2, "exit"),
        ]
    }

    /// Where one endpoint sits in one interleaving.
    fn position(order: &[Step], endpoint: Step) -> usize {
        order
            .iter()
            .position(|placed| *placed == endpoint)
            .unwrap_or_else(|| panic!("{endpoint:?} is not in {order:?}"))
    }

    /// Both branches may interleave, and neither loses its own order.
    ///
    /// `FR-002-AC-2` in its own terms, and upstream `FR-052-AC-1`: "Split
    /// shipments S1 and S2 may interleave while each branch preserves its own
    /// order". The count is the check that neither half slipped. Two branches
    /// of six endpoints, with the parallel's own `enter` first and `exit` last,
    /// merge in exactly `C(12, 6) = 924` ways: fewer means a permitted order
    /// was lost, more means an order was admitted that breaks a branch chain.
    /// The number is arithmetic over the fixture, not a figure read off a run.
    #[trace("TC-002", "FR-002-AC-2")]
    #[test]
    fn tc_002_bounded_parallel_branches_admit_every_interleaving_that_keeps_each_branch() {
        let linked = link_declarations(&parallel_fixture(), 0).expect("the fixture links");
        let orders = interleavings(linked.controls(), linked.edges())
            .expect("the fixture is a parallel and two sequences");

        assert_eq!(orders.len(), 924, "C(12, 6) merges of two six-step chains");

        let relation = precedence(linked.edges());
        for order in &orders {
            assert_eq!(order.len(), 14, "every endpoint is placed once: {order:?}");
            for (from, to) in &relation {
                assert!(
                    position(order, *from) < position(order, *to),
                    "{from:?} -> {to:?} is not preserved by {order:?}"
                );
            }
        }

        // Both extremes and one strict alternation are admitted, so "may
        // interleave" is checked by exhibiting the interleavings rather than by
        // counting alone.
        let left = left_branch();
        let right = right_branch();
        for expected in [
            [&left[..], &right[..]].concat(),
            [&right[..], &left[..]].concat(),
            left.iter()
                .zip(right.iter())
                .flat_map(|(l, r)| [*l, *r])
                .collect(),
        ] {
            let whole: Vec<Step> = [&[(0, "enter")][..], &expected, &[(0, "exit")][..]].concat();
            assert!(orders.contains(&whole), "{whole:?} is not admitted");
        }
    }

    /// Every permitted interleaving satisfies the **declared** join.
    ///
    /// The rule is upstream's: `FR-228` names the admitted completion rule the
    /// "all-branch join", so the parallel's `exit` may not be reached until
    /// every branch the declaration names in `join` has completed.
    ///
    /// The word to earn is *declared*. An earlier revision read the join from
    /// A's retained `Join` edges and asserted the order respected them, which
    /// is a restatement of the test above: the same edges are already in
    /// `precedence`, so a topological sort was being asked whether it had
    /// sorted topologically. A fixture declaring a different join rule with the
    /// same edges would have passed it unchanged.
    ///
    /// So this reads `ControlOperation::Parallel`'s own `join` field out of the
    /// authored declaration and checks two things the enumerator cannot supply:
    /// that A's `Join` edges are exactly the declared branches and no others,
    /// and that every admitted order puts each declared branch's completion
    /// before the parallel's. The first is the link between the authored field
    /// and the retained graph; without it the second is vacuous.
    #[trace("TC-002", "FR-002-AC-2")]
    #[test]
    fn tc_002_every_interleaving_satisfies_the_declared_join() {
        let fixture = parallel_fixture();
        let Body::Protocol {
            controls,
            causal_edges,
            ..
        } = &fixture[0].body
        else {
            unreachable!("the fixture is a protocol")
        };
        let ControlOperation::Parallel { branches, join } = &controls[0].operation else {
            unreachable!("control 0 is the parallel")
        };

        // The declared join names branch positions; these are the bodies it
        // requires to have completed.
        let declared: BTreeSet<u32> = join
            .iter()
            .map(|position| branches[*position as usize].body.index)
            .collect();
        assert_eq!(
            declared,
            BTreeSet::from([1, 2]),
            "the fixture declares the all-branch join"
        );

        // A's retained join edges are exactly those bodies' exits reaching the
        // parallel's exit — no fewer, so the rule is not weakened, and no more,
        // so it is not a different rule wearing the same name.
        let joining: BTreeSet<u32> = causal_edges
            .iter()
            .filter(|edge| edge.kind == EdgeKind::Join && edge.to.node.index == 0)
            .map(|edge| edge.from.node.index)
            .collect();
        assert_eq!(
            joining, declared,
            "A's join edges are not the declared join"
        );

        let linked = link_declarations(&fixture, 0).expect("the fixture links");
        for order in interleavings(linked.controls(), linked.edges())
            .expect("the fixture is a parallel and two sequences")
        {
            let exit = position(&order, (0, "exit"));
            for body in &declared {
                assert!(
                    position(&order, (*body, "exit")) < exit,
                    "branch {body} completes after the parallel in {order:?}"
                );
            }
        }
    }

    /// An edge set spanning a choice, a repeat or an await is refused, not
    /// enumerated.
    ///
    /// A derives `branch` and `join` edges identically for a `Parallel` and a
    /// `Choice`, so a linear extension over a choice's edges reports an order
    /// taking *both* cases — a false answer, not a missing one. The guard is
    /// the control kind, which is the only thing that distinguishes them.
    ///
    /// This is the mutation-proof for the guard: delete `admits_alternatives`'s
    /// `"choice"` arm and this test fails while every other test stays green,
    /// because no other fixture carries an exclusive control.
    #[trace("TC-002", "FR-002-AC-2")]
    #[test]
    fn tc_002_an_exclusive_control_is_refused_rather_than_interleaved() {
        let linked = link_declarations(&sweep_fixture(), 0).expect("the fixture links");

        let refused = interleavings(linked.controls(), linked.edges())
            .expect_err("the sweep fixture carries a choice and a repeat");
        // Control 7 is the choice, and it is the first exclusive control in
        // table order.
        assert_eq!(refused.control, 7);
        assert_eq!(refused.kind, "choice");

        // Refused before enumerating, which also matters: the sweep fixture is
        // thirteen controls wide and its linear extensions are a factorial the
        // module doc declines to bound. `linear_extensions` is the function
        // with that cost, and it is not what the guard calls.
        assert!(
            linked
                .controls()
                .iter()
                .any(ControlShape::admits_alternatives),
            "the fixture is the one carrying an exclusive control"
        );
    }

    /// The join is read from the retained edges, not assumed from the shape.
    ///
    /// Drop one branch's `join` edge and an order appears in which that branch
    /// completes after the parallel does. The assertion above would pass
    /// vacuously if the permitted set were computed from anything but A's
    /// edges; this is the mutation that says it is not.
    #[trace("TC-002", "FR-002-AC-2")]
    #[test]
    fn tc_002_a_dropped_join_edge_admits_a_branch_completing_late() {
        let mut missing = parallel_fixture();
        let Body::Protocol { causal_edges, .. } = &mut missing[0].body else {
            unreachable!("the fixture is a protocol")
        };
        causal_edges.retain(|edge| edge.kind != EdgeKind::Join || edge.from.node.index != 2);

        let linked = link_declarations(&missing, 0).expect("the fixture links");
        let orders = interleavings(linked.controls(), linked.edges())
            .expect("the fixture is a parallel and two sequences");

        assert!(
            orders
                .iter()
                .any(|order| { position(order, (2, "exit")) > position(order, (0, "exit")) }),
            "dropping the join edge left the join enforced by something else"
        );
    }

    /// The permitted set is a property of the relation, not of the table order.
    ///
    /// `FR-002-AC-4` refuses an edge added by reordering equal-time input. A
    /// sorts `causal_edges` on the full edge key and refuses an offer whose
    /// order differs, so a reordered table cannot reach this crate from an
    /// admitted artifact at all; what this checks is that nothing downstream of
    /// the retained sequence depends on it. `LinkedProtocol` still compares the
    /// edges as the sequence A supplied, which is A's canonical one.
    #[trace("TC-002", "FR-002-AC-2")]
    #[test]
    fn tc_002_reordering_the_edge_table_admits_the_same_interleavings() {
        let linked = link_declarations(&parallel_fixture(), 0).expect("the fixture links");

        let mut reversed = parallel_fixture();
        let Body::Protocol { causal_edges, .. } = &mut reversed[0].body else {
            unreachable!("the fixture is a protocol")
        };
        causal_edges.reverse();
        let reversed = link_declarations(&reversed, 0).expect("the fixture links");

        assert_eq!(
            interleavings(linked.controls(), linked.edges()),
            interleavings(reversed.controls(), reversed.edges())
        );
    }

    // -----------------------------------------------------------------------
    // C-14: the FR-050..FR-054 mutation sweep
    // -----------------------------------------------------------------------

    /// A role whose authority is chosen rather than fixed.
    fn role_with_model(name: &str, instance: u32, model: (u32, u32)) -> Role {
        Role {
            name: name.to_owned(),
            model: ExportRef {
                model: model.0,
                export: model.1,
            },
            instance,
            locus: locus(),
        }
    }

    fn choice_case(label: &str, guard: u32, body: u32) -> ChoiceCase {
        ChoiceCase {
            label: label.to_owned(),
            guard: handle(guard),
            body: handle(body),
            locus: locus(),
        }
    }

    /// A channel carrying a FIFO premise with an authored key scope.
    fn fifo_channel(name: &str, message: u32) -> Channel {
        let mut channel = channel(name, message);
        channel.ordering = Ordering::Fifo {
            binder: handle(0),
            key: handle(1),
            anchor: handle(2),
        };
        channel
    }

    /// The fixture every mutation below is one edit away from.
    ///
    /// `parallel_fixture` plus the records the remaining axes need: a choice
    /// with an owner, a visible set and two labelled cases, a bounded repeat
    /// with an authored maximum, and a second channel carrying a FIFO premise.
    /// Each added control carries the edges A derives for it, so the fixture
    /// stays one A could have emitted.
    fn sweep_fixture() -> Vec<Declaration> {
        let mut declarations = parallel_fixture();
        let Body::Protocol {
            channels,
            controls,
            causal_edges,
            ..
        } = &mut declarations[0].body
        else {
            unreachable!("the fixture is a protocol")
        };

        channels.push(fifo_channel("receipt", 5));

        // 7: the choice, over leaf bodies 8 and 9.
        controls.push(Control {
            name: "decide".to_owned(),
            original_node: 0,
            locus: locus(),
            operation: ControlOperation::Choice {
                owner: handle(0),
                visible: vec![handle(0)],
                cases: vec![choice_case("ship", 0, 8), choice_case("cancel", 1, 9)],
            },
        });
        controls.push(check_control());
        controls.push(check_control());

        // 10: the bounded repeat, over body 11 and exhausted branch 12.
        controls.push(Control {
            name: "retry".to_owned(),
            original_node: 0,
            locus: locus(),
            operation: ControlOperation::Repeat {
                owner: handle(0),
                visible: vec![handle(0)],
                maximum: integer("3"),
                guard: handle(11),
                body: handle(11),
                exhausted: handle(12),
            },
        });
        controls.push(check_control());
        controls.push(check_control());

        use EdgeKind::{Branch as BranchEdge, Join, RepeatProgress, Sequence};
        use Port::{Enter, Exit};
        causal_edges.extend([
            edge(BranchEdge, 7, (7, Enter), (8, Enter)),
            edge(BranchEdge, 7, (7, Enter), (9, Enter)),
            edge(Join, 7, (8, Exit), (7, Exit)),
            edge(Join, 7, (9, Exit), (7, Exit)),
            edge(Sequence, 8, (8, Enter), (8, Exit)),
            edge(Sequence, 9, (9, Enter), (9, Exit)),
            edge(BranchEdge, 10, (10, Enter), (10, Exit)),
            edge(BranchEdge, 10, (10, Enter), (11, Enter)),
            edge(BranchEdge, 10, (10, Enter), (12, Enter)),
            CausalEdge {
                maximum: Nullable(Some(integer("3"))),
                ..edge(RepeatProgress, 10, (11, Exit), (10, Enter))
            },
            edge(Join, 10, (12, Exit), (10, Exit)),
            edge(Sequence, 11, (11, Enter), (11, Exit)),
            edge(Sequence, 12, (12, Enter), (12, Exit)),
        ]);

        stamp(&mut declarations);
        declarations
    }

    fn budget_fixture() -> Vec<Declaration> {
        let mut declarations = sweep_fixture();
        let declared = &mut declarations[0];
        let second_role = u32::try_from(declared.bindings.len()).expect("small fixture");
        declared.bindings.push(requirement(
            "the-second-role-instance",
            BindingKind::RoleInstance,
        ));
        let Body::Protocol {
            roles, controls, ..
        } = &mut declared.body
        else {
            unreachable!("the fixture is a protocol")
        };
        roles.push(Role {
            name: "Seller".to_owned(),
            model: ExportRef {
                model: 0,
                export: 1,
            },
            instance: second_role,
            locus: locus(),
        });
        let repeat = controls
            .iter()
            .find(|control| matches!(control.operation, ControlOperation::Repeat { .. }))
            .expect("sweep fixture has a repeat")
            .clone();
        controls.push(repeat);
        stamp(&mut declarations);
        declarations.push(declarations[0].clone());
        declarations
    }

    #[trace("TC-002", "TC-009", "NFR-002-AC-1", "FR-016-AC-2", "FR-016-AC-4")]
    #[test]
    fn tc_009_link_charges_every_structural_population_before_mapping() {
        let declarations = budget_fixture();
        let work = link_work(&declarations, 0)
            .expect("budget accounting does not overflow")
            .expect("budget fixture has a selected protocol");
        let dimensions = [
            (BudgetDimension::Definition, work.definitions),
            (BudgetDimension::Node, work.nodes),
            (BudgetDimension::Edge, work.edges),
            (BudgetDimension::Role, work.roles),
            (BudgetDimension::Message, work.messages),
            (BudgetDimension::Branch, work.branches),
            (BudgetDimension::Iteration, work.iterations),
            (BudgetDimension::ActiveInstance, work.active_instances),
        ];
        let inventory = production_inventory().expect("production inventory");
        for (dimension, required) in dimensions {
            assert!(required > 1, "{dimension} needs a positive one-short case");
            let short = Budget::nothing_granted()
                .grant(dimension, required - 1)
                .expect("positive one-short grant");
            let mut short_resources = ResourceAccountant::new(short, &inventory);
            assert!(matches!(
                link_declarations_bounded(&declarations, 0, &mut short_resources),
                OperationOutcome::ResourceIncomplete(value)
                    if value.dimension() == dimension && value.requested() == required
            ));
            assert_eq!(
                short_resources.usage().consumed(dimension),
                0,
                "{dimension} preflight is transactional"
            );

            let exact = Budget::nothing_granted()
                .grant(dimension, required)
                .expect("positive exact grant");
            let mut exact_resources = ResourceAccountant::new(exact, &inventory);
            assert!(matches!(
                link_declarations_bounded(&declarations, 0, &mut exact_resources),
                OperationOutcome::Completed(_)
            ));
            assert_eq!(exact_resources.usage().consumed(dimension), required);
        }
    }

    #[trace("TC-002", "TC-009", "NFR-002-AC-1", "NFR-002-AC-3", "FR-016-AC-2")]
    #[test]
    fn tc_009_link_charges_each_identity_allocation_on_the_real_mapping_path() {
        let declarations = budget_fixture();
        let inventory = production_inventory().expect("production inventory");

        let one_allocation = Budget::nothing_granted()
            .grant(BudgetDimension::AccountedAllocation, 1)
            .expect("one preconsumed allocation");
        let mut graph_resources = ResourceAccountant::new(one_allocation, &inventory);
        assert!(matches!(
            DefinitionId::new_bounded(b"already-used", "already used", &mut graph_resources),
            OperationOutcome::Completed(_)
        ));
        assert!(matches!(
            link_declarations_bounded(&declarations, 0, &mut graph_resources),
            OperationOutcome::ResourceIncomplete(value)
                if value.dimension() == BudgetDimension::AccountedAllocation
                    && value.site() == GRAPH_ALLOCATION_SOURCE.id()
                    && value.consumed() == 1
                    && value.requested() == 1
        ));

        let mut measured = ResourceAccountant::new(Budget::nothing_granted(), &inventory);
        assert!(matches!(
            link_declarations_bounded(&declarations, 0, &mut measured),
            OperationOutcome::Completed(_)
        ));
        let required = measured
            .usage()
            .consumed(BudgetDimension::AccountedAllocation);
        assert!(
            required > 1,
            "the fixture must construct several identities"
        );

        let short = Budget::nothing_granted()
            .grant(BudgetDimension::AccountedAllocation, required - 1)
            .expect("positive one-short allocation grant");
        let mut short_resources = ResourceAccountant::new(short, &inventory);
        assert!(matches!(
            link_declarations_bounded(&declarations, 0, &mut short_resources),
            OperationOutcome::ResourceIncomplete(value)
                if value.dimension() == BudgetDimension::AccountedAllocation
                    && value.site() == crate::ids::ALLOCATION_SOURCE.id()
                    && value.consumed() == required - 1
                    && value.requested() == 1
        ));

        let exact = Budget::nothing_granted()
            .grant(BudgetDimension::AccountedAllocation, required)
            .expect("positive exact allocation grant");
        let mut exact_resources = ResourceAccountant::new(exact, &inventory);
        assert!(matches!(
            link_declarations_bounded(&declarations, 0, &mut exact_resources),
            OperationOutcome::Completed(_)
        ));
        assert_eq!(
            exact_resources
                .usage()
                .consumed(BudgetDimension::AccountedAllocation),
            required
        );

        let retained = measured.usage().consumed(BudgetDimension::InputByte);
        assert!(retained > 1, "the fixture must retain identity bytes");
        let short = Budget::nothing_granted()
            .grant(BudgetDimension::InputByte, retained - 1)
            .expect("positive one-short retained-byte grant");
        let mut short_resources = ResourceAccountant::new(short, &inventory);
        assert!(matches!(
            link_declarations_bounded(&declarations, 0, &mut short_resources),
            OperationOutcome::ResourceIncomplete(value)
                if value.dimension() == BudgetDimension::InputByte
                    && value.site() == crate::ids::INPUT_SOURCE.id()
        ));

        let exact = Budget::nothing_granted()
            .grant(BudgetDimension::InputByte, retained)
            .expect("positive exact retained-byte grant");
        let mut exact_resources = ResourceAccountant::new(exact, &inventory);
        assert!(matches!(
            link_declarations_bounded(&declarations, 0, &mut exact_resources),
            OperationOutcome::Completed(_)
        ));
        assert_eq!(
            exact_resources.usage().consumed(BudgetDimension::InputByte),
            retained
        );
    }

    /// The protocol body of a fixture under mutation.
    fn body(
        declarations: &mut [Declaration],
    ) -> (
        &mut Vec<Role>,
        &mut Vec<Channel>,
        &mut Vec<Control>,
        &mut Vec<CausalEdge>,
        &mut Finish,
    ) {
        let Body::Protocol {
            roles,
            channels,
            controls,
            causal_edges,
            finish,
            ..
        } = &mut declarations[0].body
        else {
            unreachable!("the fixture is a protocol")
        };
        (roles, channels, controls, causal_edges, finish)
    }

    /// What stops one mutation, and how that is observed here.
    enum Guard {
        /// This crate refuses it. The first finding must be this one, and a
        /// refusal carries no linked representation by the shape of the result.
        Refused(LinkRefusal),
        /// Admitted here, and the linked protocol differs. This is the
        /// `FR-002-AC-4` half that makes a semantic change identity-bearing.
        Distinct,
        /// Admitted here, and the linked protocol compares equal. The other
        /// `FR-002-AC-4` half: presentation is not identity.
        Presentation,
        /// A refuses it inside `read`, so an admitted package cannot carry it.
        /// These fixtures are built as wire records and never pass through
        /// `read`, so what is checked here is the weaker true statement: the
        /// mutation does not link to an equal protocol. The named check is A's.
        Upstream(&'static str),
        /// Nobody refuses it. The mutation links, and the criterion stays open.
        ///
        /// Asserting the absence is deliberate. When a guard lands, this row
        /// fails and has to be moved to [`Guard::Refused`], which is how the
        /// hole gets closed in the matrix at the same time as in the code.
        Unguarded(&'static str),
    }

    /// One mutation axis: what is changed, and what stops it.
    struct Axis {
        /// The criterion the axis belongs to.
        criterion: &'static str,
        /// What the mutation changes, in the requirement's own words.
        change: &'static str,
        /// The mutation, applied to a fresh `sweep_fixture`.
        apply: fn(&mut Vec<Declaration>),
        /// What stops it.
        guard: Guard,
    }

    /// Every `FR-050`..`FR-054` mutation axis this crate can express.
    ///
    /// # What is not here, and why
    ///
    /// - **An ownerless or multiply owned choice.** `FR-053` requires both
    ///   refused. Neither is representable: A's `ControlOperation::Choice`
    ///   carries exactly one `owner: Handle`, so there is no artifact with
    ///   none and none with two. The type is the guard, and a row asserting a
    ///   refusal that can never be offered would be theatre.
    /// - **A duplication premise.** `FR-216` fixes six premise components and
    ///   A's `Channel` publishes five; see [`ChannelPremises`]. An axis with no
    ///   authored field has no mutation.
    /// - **Runtime instance identities.** `FR-050-AC-3`'s two concurrent
    ///   workflow instances are not link-time subjects; the owner has ruled
    ///   `FR-002-AC-1` unresolved until instance identities exist.
    fn sweep() -> Vec<Axis> {
        vec![
            // --- FR-050: identities ---------------------------------------
            Axis {
                criterion: "FR-050",
                change: "a second role over one authority",
                apply: |declarations| {
                    declarations[0]
                        .bindings
                        .push(requirement("another-instance", BindingKind::RoleInstance));
                    let (roles, ..) = body(declarations);
                    roles.push(role_with_model("Seller", 11, (0, 0)));
                },
                guard: Guard::Refused(LinkRefusal::DuplicateRoleAuthority {
                    role: 1,
                    model: (0, 0),
                    locus: locus_at(ROLES, 1),
                }),
            },
            Axis {
                criterion: "FR-050",
                change: "two roles selecting one role-instance requirement",
                apply: |declarations| {
                    let (roles, ..) = body(declarations);
                    roles.push(role_with_model("Seller", 1, (1, 0)));
                },
                guard: Guard::Refused(LinkRefusal::CollidingIdentity {
                    kind: IdentityKind::RoleInstance,
                    // The identity the second role collides with, read off a
                    // successful link of the same fixture rather than
                    // recomputed here. An earlier revision called
                    // `requirement_identity` inline with hand-passed
                    // arguments, so a wrong declaration, index or requirement
                    // moved both sides together and the row still passed.
                    identity: link_declarations(&sweep_fixture(), 0)
                        .expect("the sweep fixture links")
                        .role_instances()[0]
                        .identity_bytes()
                        .to_vec(),
                    locus: locus_at(ROLES, 1),
                }),
            },
            Axis {
                criterion: "FR-050-AC-2",
                change: "a component identity substituted for a role instance",
                apply: |declarations| {
                    let (roles, ..) = body(declarations);
                    roles[0].instance = 3;
                },
                guard: Guard::Refused(LinkRefusal::WrongRequirementKind {
                    referrer: Referrer::Role { index: 0 },
                    required: BindingKind::RoleInstance,
                    found: BindingKind::Component,
                    locus: locus_at(ROLES, 0),
                }),
            },
            Axis {
                criterion: "FR-050-AC-4",
                change: "an unresolved role-instance reference",
                apply: |declarations| {
                    let (roles, ..) = body(declarations);
                    roles[0].instance = 99;
                },
                guard: Guard::Refused(LinkRefusal::NoSuchRequirement {
                    referrer: Referrer::Role { index: 0 },
                    selected: 99,
                    available: bindings().len(),
                    locus: locus_at(ROLES, 0),
                }),
            },
            Axis {
                criterion: "FR-050",
                change: "a role's authored name",
                apply: |declarations| {
                    let (roles, ..) = body(declarations);
                    roles[0].name = "Purchaser".to_owned();
                },
                guard: Guard::Presentation,
            },
            // --- FR-051: correspondence -----------------------------------
            Axis {
                criterion: "FR-051-AC-4",
                change: "a delivery requirement substituted for a message premise",
                apply: |declarations| {
                    let (_, channels, ..) = body(declarations);
                    channels[0].message = 8;
                },
                guard: Guard::Refused(LinkRefusal::WrongRequirementKind {
                    referrer: Referrer::Channel {
                        index: 0,
                        premise: ChannelPremise::Message,
                    },
                    required: BindingKind::Message,
                    found: BindingKind::Delivery,
                    locus: locus_at(CHANNELS, 0),
                }),
            },
            // --- FR-052: control and termination --------------------------
            Axis {
                criterion: "FR-052-AC-3",
                change: "a zero-progress iteration maximum",
                apply: |declarations| {
                    let (_, _, controls, ..) = body(declarations);
                    let ControlOperation::Repeat { maximum, .. } = &mut controls[10].operation
                    else {
                        unreachable!("control 10 is the repeat")
                    };
                    *maximum = integer("0");
                },
                guard: Guard::Refused(LinkRefusal::ZeroProgressIteration {
                    control: 10,
                    locus: locus_at(CONTROLS, 10),
                }),
            },
            Axis {
                criterion: "FR-052-AC-4",
                change: "an authored iteration maximum",
                apply: |declarations| {
                    let (_, _, controls, ..) = body(declarations);
                    let ControlOperation::Repeat { maximum, .. } = &mut controls[10].operation
                    else {
                        unreachable!("control 10 is the repeat")
                    };
                    *maximum = integer("4");
                },
                guard: Guard::Distinct,
            },
            Axis {
                criterion: "FR-052-AC-4",
                change: "a parallel's join rule",
                apply: |declarations| {
                    let (_, _, controls, ..) = body(declarations);
                    let ControlOperation::Parallel { join, .. } = &mut controls[0].operation else {
                        unreachable!("control 0 is the parallel")
                    };
                    join.pop();
                },
                guard: Guard::Distinct,
            },
            Axis {
                criterion: "FR-052-AC-4",
                change: "which body a parallel branch runs",
                apply: |declarations| {
                    let (_, _, controls, ..) = body(declarations);
                    let ControlOperation::Parallel { branches, .. } = &mut controls[0].operation
                    else {
                        unreachable!("control 0 is the parallel")
                    };
                    branches.swap(0, 1);
                },
                guard: Guard::Distinct,
            },
            Axis {
                criterion: "FR-052",
                change: "a control's authored name",
                apply: |declarations| {
                    let (_, _, controls, ..) = body(declarations);
                    controls[0].name = "fork".to_owned();
                },
                guard: Guard::Presentation,
            },
            Axis {
                criterion: "FR-052",
                change: "the termination's closure binding",
                apply: |declarations| {
                    let (.., finish) = body(declarations);
                    finish.closure = 1;
                },
                guard: Guard::Distinct,
            },
            Axis {
                criterion: "FR-052",
                change: "the termination's authored name",
                apply: |declarations| {
                    let (.., finish) = body(declarations);
                    finish.name = "finished".to_owned();
                },
                guard: Guard::Presentation,
            },
            Axis {
                criterion: "FR-052-AC-3",
                change: "a causal cycle",
                apply: |declarations| {
                    let (_, _, _, causal_edges, _) = body(declarations);
                    causal_edges.push(edge(
                        EdgeKind::Sequence,
                        0,
                        (0, Port::Exit),
                        (0, Port::Enter),
                    ));
                },
                guard: Guard::Upstream("A's acyclicity check over the offered edge set"),
            },
            Axis {
                criterion: "FR-052-AC-3",
                change: "a missing join edge",
                apply: |declarations| {
                    let (_, _, _, causal_edges, _) = body(declarations);
                    causal_edges
                        .retain(|edge| edge.kind != EdgeKind::Join || edge.from.node.index != 2);
                },
                guard: Guard::Upstream("A's element-by-element comparison of the derived edge set"),
            },
            Axis {
                criterion: "FR-052",
                change: "a control no causal edge reaches",
                apply: |declarations| {
                    let (_, _, controls, ..) = body(declarations);
                    controls.push(check_control());
                },
                guard: Guard::Unguarded(
                    "unreachable termination: no reachability analysis exists in either repository",
                ),
            },
            // --- FR-053: choice and visibility ----------------------------
            Axis {
                criterion: "FR-053",
                change: "a choice's owning role",
                apply: |declarations| {
                    let (_, _, controls, ..) = body(declarations);
                    let ControlOperation::Choice { owner, .. } = &mut controls[7].operation else {
                        unreachable!("control 7 is the choice")
                    };
                    *owner = handle(1);
                },
                guard: Guard::Distinct,
            },
            Axis {
                criterion: "FR-002-AC-4",
                change: "a choice's visibility premise",
                apply: |declarations| {
                    let (_, _, controls, ..) = body(declarations);
                    let ControlOperation::Choice { visible, .. } = &mut controls[7].operation
                    else {
                        unreachable!("control 7 is the choice")
                    };
                    visible.push(handle(1));
                },
                guard: Guard::Distinct,
            },
            Axis {
                criterion: "FR-053",
                change: "which body a choice case selects",
                apply: |declarations| {
                    let (_, _, controls, ..) = body(declarations);
                    let ControlOperation::Choice { cases, .. } = &mut controls[7].operation else {
                        unreachable!("control 7 is the choice")
                    };
                    cases[0].body = handle(9);
                },
                guard: Guard::Distinct,
            },
            Axis {
                criterion: "FR-053",
                change: "a choice case's label",
                apply: |declarations| {
                    let (_, _, controls, ..) = body(declarations);
                    let ControlOperation::Choice { cases, .. } = &mut controls[7].operation else {
                        unreachable!("control 7 is the choice")
                    };
                    cases[0].label = "dispatch".to_owned();
                },
                guard: Guard::Presentation,
            },
            Axis {
                criterion: "FR-053-AC-3",
                change: "two choice cases under one guard",
                apply: |declarations| {
                    let (_, _, controls, ..) = body(declarations);
                    let ControlOperation::Choice { cases, .. } = &mut controls[7].operation else {
                        unreachable!("control 7 is the choice")
                    };
                    cases[1].guard = cases[0].guard.clone();
                },
                guard: Guard::Unguarded(
                    "overlapping choices: guard disjointness is a decision procedure over \
                     arbitrary predicates, which neither repository has",
                ),
            },
            Axis {
                criterion: "FR-053-AC-2",
                change: "a choice whose owner is outside its visible set",
                apply: |declarations| {
                    let (_, _, controls, ..) = body(declarations);
                    let ControlOperation::Choice { visible, .. } = &mut controls[7].operation
                    else {
                        unreachable!("control 7 is the choice")
                    };
                    *visible = vec![handle(1)];
                },
                guard: Guard::Unguarded(
                    "owner-unobservable choices: values carry no role attribution to relate a \
                     guard's operands to the visible set",
                ),
            },
            // --- FR-054: channel premises ---------------------------------
            Axis {
                criterion: "FR-054-AC-3",
                change: "an unresolved send premise",
                apply: |declarations| {
                    let (_, channels, ..) = body(declarations);
                    channels[0].send = 99;
                },
                guard: Guard::Refused(LinkRefusal::NoSuchRequirement {
                    referrer: Referrer::Channel {
                        index: 0,
                        premise: ChannelPremise::Send,
                    },
                    selected: 99,
                    available: bindings().len(),
                    locus: locus_at(CHANNELS, 0),
                }),
            },
            Axis {
                criterion: "FR-054-AC-4",
                change: "a channel's sender",
                apply: |declarations| {
                    let (_, channels, ..) = body(declarations);
                    channels[0].from = handle(2);
                },
                guard: Guard::Distinct,
            },
            Axis {
                criterion: "FR-054-AC-4",
                change: "a channel's receiver",
                apply: |declarations| {
                    let (_, channels, ..) = body(declarations);
                    channels[0].to = handle(2);
                },
                guard: Guard::Distinct,
            },
            Axis {
                criterion: "FR-054-AC-4",
                change: "a channel's message type",
                apply: |declarations| {
                    let (_, channels, ..) = body(declarations);
                    channels[0].message_type = 1;
                },
                guard: Guard::Distinct,
            },
            Axis {
                criterion: "FR-054-AC-4",
                change: "a channel's ordering policy",
                apply: |declarations| {
                    let (_, channels, ..) = body(declarations);
                    channels[0].ordering = Ordering::Fifo {
                        binder: handle(0),
                        key: handle(1),
                        anchor: handle(2),
                    };
                },
                guard: Guard::Distinct,
            },
            Axis {
                criterion: "FR-054-AC-2",
                change: "a FIFO premise's key scope",
                apply: |declarations| {
                    let (_, channels, ..) = body(declarations);
                    let Ordering::Fifo { key, .. } = &mut channels[1].ordering else {
                        unreachable!("channel 1 carries the FIFO premise")
                    };
                    *key = handle(3);
                },
                guard: Guard::Distinct,
            },
            Axis {
                criterion: "FR-054-AC-4",
                change: "a channel's delivery cardinality",
                apply: |declarations| {
                    let (_, channels, ..) = body(declarations);
                    channels[0].delivery.upper = integer("2");
                },
                guard: Guard::Distinct,
            },
            Axis {
                criterion: "FR-054",
                change: "a channel's authored name",
                apply: |declarations| {
                    let (_, channels, ..) = body(declarations);
                    channels[0].name = "purchase-order".to_owned();
                },
                guard: Guard::Presentation,
            },
        ]
    }

    /// Every mutation reaches its guard, one axis at a time.
    ///
    /// `FR-002-AC-3` requires each mutation refused at its source location with
    /// no linked representation, and `FR-002-AC-4` requires a semantic change
    /// to change the linked identity while a presentation change does not. One
    /// table discharges both, because the interesting question for an axis is
    /// *which* of the two it lands on — and five of these axes landed on
    /// neither before this chunk: a channel's sender, receiver, message type,
    /// ordering policy and delivery cardinality all linked to an equal protocol
    /// while the matrix recorded AC-4 as executing. See [`ChannelPremises`].
    /// Link a mutated fixture, re-stamping so every record keeps its own region.
    fn sweep_outcome(apply: fn(&mut Vec<Declaration>)) -> Result<LinkedProtocol, Diagnostics> {
        let mut mutated = sweep_fixture();
        apply(&mut mutated);
        // A mutation that adds or moves a record changes which position each
        // record holds, and a region follows the position.
        stamp(&mut mutated);
        link_declarations(&mutated, 0)
    }

    /// Every axis this crate refuses is refused, with the right finding at the
    /// right place.
    ///
    /// `FR-002-AC-3` asks for a refusal "at its source location", so the
    /// expected `LinkRefusal` is compared whole — the finding's kind, its
    /// fields **and** its `locus`. `stamp` gives each record its own region,
    /// so a refusal that names the wrong control, role or channel fails here
    /// rather than comparing equal to the right one.
    #[trace("TC-002", "FR-002-AC-3")]
    #[test]
    fn tc_002_every_refused_axis_is_refused_at_its_source() {
        let mut seen = 0;
        for Axis {
            criterion,
            change,
            apply,
            guard,
        } in sweep()
        {
            let Guard::Refused(expected) = guard else {
                continue;
            };
            seen += 1;
            let diagnostics = sweep_outcome(apply)
                .err()
                .unwrap_or_else(|| panic!("{criterion}: {change} was not refused"));
            assert_eq!(diagnostics.first(), &expected, "{criterion}: {change}");
        }
        assert_eq!(seen, 7, "the sweep's refusal axes");
    }

    /// Every semantic axis changes the linked protocol, and every presentation
    /// axis leaves it equal.
    ///
    /// This is the `FR-002-AC-4` half of the sweep, and it carries its own tag:
    /// an earlier revision ran both halves under one test tagged
    /// `FR-002-AC-3`, so `quire coverage` saw eighteen identity axes backing a
    /// refusal criterion and nothing backing the identity one.
    #[trace("TC-002", "FR-002-AC-4")]
    #[test]
    fn tc_002_every_semantic_axis_changes_the_linked_protocol() {
        let base = link_declarations(&sweep_fixture(), 0).expect("the sweep fixture links");
        let mut seen = 0;
        for Axis {
            criterion,
            change,
            apply,
            guard,
        } in sweep()
        {
            let expect_change = match guard {
                Guard::Distinct => true,
                Guard::Presentation => false,
                _ => continue,
            };
            seen += 1;
            let linked = sweep_outcome(apply)
                .unwrap_or_else(|error| panic!("{criterion}: {change} refused: {error}"));
            if expect_change {
                assert_ne!(
                    linked, base,
                    "{criterion}: {change} left the identity equal"
                );
            } else {
                assert_eq!(linked, base, "{criterion}: {change} changed the identity");
            }
        }
        assert_eq!(seen, 18, "the sweep's identity axes");
    }

    /// The axes this crate does not guard, recorded as still unguarded.
    ///
    /// Two kinds, and neither discharges an acceptance criterion — the tag is
    /// `Task-002`, not `FR-002-AC-3`, because every assertion here says a
    /// mutation is **accepted**:
    ///
    /// - `Guard::Upstream` — A refuses it inside `read`, before this crate sees
    ///   it. What is checked is that B does not *also* refuse it, which is what
    ///   would happen if someone re-implemented A's acyclicity or join check
    ///   here. An earlier revision asserted the linked protocol differed
    ///   instead; both axes add or remove a `CausalEdge`, and the edge set is
    ///   part of the identity, so that assertion could not fail and said
    ///   nothing about A.
    /// - `Guard::Unguarded` — nobody refuses it, in either repository. The
    ///   assertion is the hole itself, so that the day someone closes it, this
    ///   test fails and says which row to move.
    #[trace("TC-002", "Task-002")]
    #[test]
    fn tc_002_the_unguarded_axes_are_still_unguarded() {
        let mut seen = 0;
        for Axis {
            criterion,
            change,
            apply,
            guard,
        } in sweep()
        {
            let note = match guard {
                Guard::Upstream(check) => {
                    format!(
                        "upstream it is refused by {check}, and this crate must not re-check it"
                    )
                }
                Guard::Unguarded(hole) => {
                    format!("{hole} — move this axis to Guard::Refused and update spec/tests.md")
                }
                _ => continue,
            };
            seen += 1;
            let outcome = sweep_outcome(apply);
            assert!(
                outcome.is_ok(),
                "{criterion}: {change} is refused here. {note}"
            );
        }
        assert_eq!(seen, 5, "the sweep's unguarded axes");
    }
}
