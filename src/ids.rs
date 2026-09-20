// SPDX-License-Identifier: AGPL-3.0-or-later
//! Protocol identity newtypes. Owner chunk: **C-07**.
//!
//! This module is the closed identity vocabulary of the B consumer boundary. It
//! lands in one commit (COLLISION-3) so that no later chunk extends it
//! piecemeal: C-12, C-16, C-18 and C-26 consume these types and never add one.
//! A genuinely new protocol identity is a spec amendment to FR-002-AC-1 or
//! FR-003-AC-3, not a drive-by edit to this file.
//!
//! # Three invariants this module is responsible for
//!
//! **1. Identity is the producer's exact bytes.** Each newtype retains the
//! identity bytes offered by the producer unchanged — no trim, no case fold, no
//! Unicode normalisation, no UTF-8 requirement. B copies; B does not
//! canonicalise. (Canonicalisation of protocol *results* is the C-11 JCS
//! domain, which these identities never reach.)
//!
//! **2. Display text is never identity.** Every identity also carries the
//! producer's human-readable display text. That text takes no part in `PartialEq`,
//! `Eq`, `Hash`, `PartialOrd` or `Ord`: two identities with equal bytes and
//! different display text are equal, and two with equal display text and
//! different bytes are not. No type here implements `FromStr` or `From<String>`,
//! so no display name can ever be promoted into an identity.
//!
//! **3. No two identities are interchangeable.** Each kind is its own `struct`,
//! never a type alias and never a shared generic over a marker, so passing a
//! [`crate::ids::SendId`] where a [`crate::ids::ReceiveId`] is expected is a compile error. The
//! `compile_fail` examples below are the proof, and they run under `cargo test`.
//!
//! # Public API boundary decision: this set is CLOSED, not `#[non_exhaustive]`
//!
//! [`crate::ids::IdentityKind`] is deliberately **not** `#[non_exhaustive]`, and neither is
//! any type in this module. This is a decision, not a derive default:
//!
//! * A `#[non_exhaustive]` enum forces every downstream `match` to carry a
//!   wildcard arm. A wildcard arm silently absorbs a variant added later — which
//!   is precisely the failure mode the owner directive on independently
//!   assertable refusals forbids. Keeping the enum closed means adding a
//!   seventeenth identity *breaks every consumer's match at compile time*, which
//!   is the intended cost of a spec amendment.
//! * The set is not a growth surface. It is the enumeration of FR-002-AC-1's
//!   thirteen linked identities plus FR-003-AC-3's registration, commit and
//!   recovery identities. Its authority is the specification, so it changes only
//!   when the specification changes, and then deliberately.
//! * The newtype `struct`s hold private fields and expose no public
//!   constructor other than [`crate::ids::SendId::new`]-shaped associated functions, so
//!   field-level source compatibility is already controlled without
//!   `#[non_exhaustive]`.
//!
//! # Proof that the newtypes are not interchangeable
//!
//! The positive control first: every constructor and every parameter type below
//! is correct when the matching identity is supplied, so each `compile_fail`
//! case that follows fails for the type mismatch and for nothing else.
//!
//! ```
//! use quire_protocol::ids::*;
//! fn definition(_: DefinitionId) {}
//! fn role_instance(_: RoleInstanceId) {}
//! fn workflow_instance(_: WorkflowInstanceId) {}
//! fn participant(_: ParticipantId) {}
//! fn component(_: ComponentId) {}
//! fn endpoint(_: EndpointId) {}
//! fn channel(_: ChannelId) {}
//! fn message(_: MessageId) {}
//! fn send(_: SendId) {}
//! fn receive(_: ReceiveId) {}
//! fn delivery(_: DeliveryId) {}
//! fn attempt(_: AttemptId) {}
//! fn effect(_: EffectId) {}
//! fn registration(_: RegistrationId) {}
//! fn commit(_: CommitId) {}
//! fn recovery(_: RecoveryId) {}
//!
//! definition(DefinitionId::new(b"d", "d"));
//! role_instance(RoleInstanceId::new(b"d", "d"));
//! workflow_instance(WorkflowInstanceId::new(b"d", "d"));
//! participant(ParticipantId::new(b"d", "d"));
//! component(ComponentId::new(b"d", "d"));
//! endpoint(EndpointId::new(b"d", "d"));
//! channel(ChannelId::new(b"d", "d"));
//! message(MessageId::new(b"d", "d"));
//! send(SendId::new(b"d", "d"));
//! receive(ReceiveId::new(b"d", "d"));
//! delivery(DeliveryId::new(b"d", "d"));
//! attempt(AttemptId::new(b"d", "d"));
//! effect(EffectId::new(b"d", "d"));
//! registration(RegistrationId::new(b"d", "d"));
//! commit(CommitId::new(b"d", "d"));
//! recovery(RecoveryId::new(b"d", "d"));
//! ```
//!
//! Each case below supplies the neighbouring identity to the same parameter and
//! must not compile. The pairs form a closed cycle over all sixteen kinds, so
//! every identity appears once as the expected type and once as the wrong
//! argument.
//!
//! ```compile_fail
//! use quire_protocol::ids::{DefinitionId, RoleInstanceId};
//! fn takes(_: DefinitionId) {}
//! takes(RoleInstanceId::new(b"d", "d"));
//! ```
//!
//! ```compile_fail
//! use quire_protocol::ids::{RoleInstanceId, WorkflowInstanceId};
//! fn takes(_: RoleInstanceId) {}
//! takes(WorkflowInstanceId::new(b"d", "d"));
//! ```
//!
//! ```compile_fail
//! use quire_protocol::ids::{ParticipantId, WorkflowInstanceId};
//! fn takes(_: WorkflowInstanceId) {}
//! takes(ParticipantId::new(b"d", "d"));
//! ```
//!
//! ```compile_fail
//! use quire_protocol::ids::{ComponentId, ParticipantId};
//! fn takes(_: ParticipantId) {}
//! takes(ComponentId::new(b"d", "d"));
//! ```
//!
//! ```compile_fail
//! use quire_protocol::ids::{ComponentId, EndpointId};
//! fn takes(_: ComponentId) {}
//! takes(EndpointId::new(b"d", "d"));
//! ```
//!
//! ```compile_fail
//! use quire_protocol::ids::{ChannelId, EndpointId};
//! fn takes(_: EndpointId) {}
//! takes(ChannelId::new(b"d", "d"));
//! ```
//!
//! ```compile_fail
//! use quire_protocol::ids::{ChannelId, MessageId};
//! fn takes(_: ChannelId) {}
//! takes(MessageId::new(b"d", "d"));
//! ```
//!
//! ```compile_fail
//! use quire_protocol::ids::{MessageId, SendId};
//! fn takes(_: MessageId) {}
//! takes(SendId::new(b"d", "d"));
//! ```
//!
//! ```compile_fail
//! use quire_protocol::ids::{ReceiveId, SendId};
//! fn takes(_: SendId) {}
//! takes(ReceiveId::new(b"d", "d"));
//! ```
//!
//! ```compile_fail
//! use quire_protocol::ids::{DeliveryId, ReceiveId};
//! fn takes(_: ReceiveId) {}
//! takes(DeliveryId::new(b"d", "d"));
//! ```
//!
//! ```compile_fail
//! use quire_protocol::ids::{AttemptId, DeliveryId};
//! fn takes(_: DeliveryId) {}
//! takes(AttemptId::new(b"d", "d"));
//! ```
//!
//! ```compile_fail
//! use quire_protocol::ids::{AttemptId, EffectId};
//! fn takes(_: AttemptId) {}
//! takes(EffectId::new(b"d", "d"));
//! ```
//!
//! ```compile_fail
//! use quire_protocol::ids::{EffectId, RegistrationId};
//! fn takes(_: EffectId) {}
//! takes(RegistrationId::new(b"d", "d"));
//! ```
//!
//! ```compile_fail
//! use quire_protocol::ids::{CommitId, RegistrationId};
//! fn takes(_: RegistrationId) {}
//! takes(CommitId::new(b"d", "d"));
//! ```
//!
//! ```compile_fail
//! use quire_protocol::ids::{CommitId, RecoveryId};
//! fn takes(_: CommitId) {}
//! takes(RecoveryId::new(b"d", "d"));
//! ```
//!
//! ```compile_fail
//! use quire_protocol::ids::{DefinitionId, RecoveryId};
//! fn takes(_: RecoveryId) {}
//! takes(DefinitionId::new(b"d", "d"));
//! ```
//!
//! # Proof that display text cannot become an identity
//!
//! No identity parses from its rendered form, so a refusal, a log line or a
//! diagnostic can never be read back as a protocol identity:
//!
//! ```compile_fail
//! use quire_protocol::ids::SendId;
//! let _: SendId = "send-1".parse().unwrap();
//! ```
//!
//! ```compile_fail
//! use quire_protocol::ids::SendId;
//! let _ = SendId::from(String::from("send-1"));
//! ```
//!
//! ```compile_fail
//! use quire_protocol::ids::ChannelId;
//! let _ = ChannelId::from("orders");
//! ```

use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};

use crate::bounds::inventory::{
    BudgetDimension, ChargeError, OperationOutcome, ResourceAccountant, SourceSite,
};
use crate::bounds::{SiteId, SiteKind, UnitLabel};

pub(crate) const INPUT_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("identity/storage/retain"),
    "ids::*::new",
    BudgetDimension::InputByte,
    SiteKind::Retention,
);
pub(crate) const ALLOCATION_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("identity/storage/allocate"),
    "ids::*::new",
    BudgetDimension::AccountedAllocation,
    SiteKind::Allocation,
);

/// Charge the exact retained identity storage before constructing its buffers.
///
/// This is shared by the public bounded constructors and by bounded owner
/// operations that construct identities internally. Keeping the charge here
/// prevents those paths from independently reimplementing what one identity
/// allocation means.
pub(crate) fn charge_storage(
    identity: &[u8],
    display: &str,
    resources: &mut ResourceAccountant<'_>,
) -> Result<(), ChargeError> {
    let retained =
        identity
            .len()
            .checked_add(display.len())
            .ok_or(ChargeError::CounterOverflow {
                site: INPUT_SOURCE.id(),
                consumed: identity.len(),
                requested: display.len(),
            })?;
    resources.charge_all(&[(INPUT_SOURCE.id(), retained), (ALLOCATION_SOURCE.id(), 1)])
}

/// Faithful `Debug` rendering of identity bytes that are not required to be
/// UTF-8. Debug output is for humans and takes no part in identity.
struct Escaped<'bytes>(&'bytes [u8]);

impl fmt::Debug for Escaped<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("b\"")?;
        for byte in self.0.iter().copied() {
            write!(formatter, "{}", byte.escape_ascii())?;
        }
        formatter.write_str("\"")
    }
}

macro_rules! identities {
    ($( ($name:ident, $kind:ident, $keys:literal) ),* $(,)?) => {
        /// The closed set of linked identities this crate holds.
        ///
        /// Deliberately **not** `#[non_exhaustive]`: see the module-level
        /// decision. A consumer's `match` over this enum is exhaustive with no
        /// catch-all arm, so a seventeenth identity cannot be absorbed
        /// silently — it breaks the build, which is the intended signal that a
        /// specification amendment has occurred.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub enum IdentityKind {
            $(
                #[doc = $keys]
                $kind,
            )*
        }

        impl IdentityKind {
            /// Every kind in the closed set, in specification order.
            pub const ALL: &[IdentityKind] = &[ $( IdentityKind::$kind ),* ];

        }

        $(
            #[doc = $keys]
            ///
            /// Identity is the producer's exact bytes; the display text is
            /// carried alongside and never participates in equality, ordering or
            /// hashing. This type is distinct from every other identity in this
            /// module, so it cannot be passed where another identity is
            /// expected.
            #[derive(Clone)]
            pub struct $name {
                identity: Vec<u8>,
                display: String,
            }

            impl $name {
                /// Retain a producer identity exactly as offered.
                ///
                /// `identity` is copied byte for byte: it is not trimmed,
                /// case-folded, Unicode-normalised or required to be UTF-8.
                /// `display` is human-readable text only and has no authority
                /// to select, repair or establish this identity.
                pub fn new(identity: &[u8], display: &str) -> Self {
                    Self { identity: identity.to_vec(), display: display.to_owned() }
                }

                /// Retain this identity under the shared QProtocol caller budget.
                ///
                /// Exact producer and display bytes, plus the single abstract
                /// identity-storage allocation, are charged transactionally
                /// before either owned buffer is constructed.
                pub fn new_bounded(
                    identity: &[u8],
                    display: &str,
                    resources: &mut ResourceAccountant<'_>,
                ) -> OperationOutcome<Self, core::convert::Infallible> {
                    resources.run_unit(
                        UnitLabel::new(concat!("ids::", stringify!($name), "::new_bounded")),
                        |resources| {
                            if let Err(error) = charge_storage(identity, display, resources) {
                                return OperationOutcome::from_charge(error);
                            }
                            OperationOutcome::Completed(Self::new(identity, display))
                        },
                    )
                }

                /// The producer's exact identity bytes.
                pub fn identity_bytes(&self) -> &[u8] {
                    &self.identity
                }

                /// The producer's display text. Never an identity.
                pub fn display_text(&self) -> &str {
                    &self.display
                }

                /// The kind this identity belongs to.
                pub fn kind(&self) -> IdentityKind {
                    IdentityKind::$kind
                }
            }

            // Equality, ordering and hashing are hand-written over the identity
            // bytes alone. Deriving them would silently make display text part
            // of identity and of every map lookup.
            impl PartialEq for $name {
                fn eq(&self, other: &Self) -> bool {
                    self.identity == other.identity
                }
            }

            impl Eq for $name {}

            impl PartialOrd for $name {
                fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                    Some(self.cmp(other))
                }
            }

            impl Ord for $name {
                fn cmp(&self, other: &Self) -> Ordering {
                    self.identity.cmp(&other.identity)
                }
            }

            impl Hash for $name {
                fn hash<H: Hasher>(&self, state: &mut H) {
                    self.identity.hash(state);
                }
            }

            /// Human-readable rendering only. Changing it changes no identity,
            /// and nothing reads it back.
            impl fmt::Display for $name {
                fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                    formatter.write_str(&self.display)
                }
            }

            impl fmt::Debug for $name {
                fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                    formatter
                        .debug_struct(stringify!($name))
                        .field("identity", &Escaped(&self.identity))
                        .field("display", &self.display)
                        .finish()
                }
            }
        )*
    };
}

identities![
    (
        DefinitionId,
        Definition,
        "Keys one authored protocol definition — the static declaration, including a static role authority, as distinct from any instance of it."
    ),
    (
        RoleInstanceId,
        RoleInstance,
        "Keys one runtime instance of a static role, distinct from the role definition and from the participant that fills it."
    ),
    (
        WorkflowInstanceId,
        WorkflowInstance,
        "Keys one concurrent workflow instance; two instances of the same definition never merge."
    ),
    (
        ParticipantId,
        Participant,
        "Keys the participant bound to a role instance, distinct from the role instance itself."
    ),
    (
        ComponentId,
        Component,
        "Keys one component export that owns endpoints."
    ),
    (
        EndpointId,
        Endpoint,
        "Keys one communication endpoint on a component, distinct from the channel it terminates."
    ),
    (
        ChannelId,
        Channel,
        "Keys one authored channel and its ordering and delivery premises."
    ),
    (
        MessageId,
        Message,
        "Keys one message occurrence carried by a channel, distinct from its send and its receive."
    ),
    (
        SendId,
        Send,
        "Keys one send event; a send is never its paired receive."
    ),
    (
        ReceiveId,
        Receive,
        "Keys one receive event; a receive is never its paired send."
    ),
    (
        DeliveryId,
        Delivery,
        "Keys one delivery of a message. Duplicate deliveries of the same message retain distinct delivery identities."
    ),
    (
        AttemptId,
        Attempt,
        "Keys one operation attempt. Failed, timed-out, duplicate and successful attempts remain distinct."
    ),
    (
        EffectId,
        Effect,
        "Keys one business effect, distinct from the attempt that produced it."
    ),
    (
        RegistrationId,
        Registration,
        "Keys one compensation registration (FR-003-AC-3), distinct from the effect it compensates and from the commit that bounds it."
    ),
    (
        CommitId,
        Commit,
        "Keys one commit (FR-003-AC-3) whose ordering constrains registration and compensation."
    ),
    (
        RecoveryId,
        Recovery,
        "Keys one recovery obligation (FR-003-AC-3), distinct from the registration and from the attempts that pursue it."
    ),
];
