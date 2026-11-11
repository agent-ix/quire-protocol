// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-150, FR-151, FR-152, FR-153: dispatch of `local-projection` and
//! `realizability` claims to the backend QSL's registry selects.
//!
//! QSL decides what a backend serves. The registry is QSL's own
//! ([`qsl_route::Registry`], FR-290's candidate computation), and a backend
//! joins it at the provider boundary ([`ProviderBoundary::register`]) with the
//! descriptor QSL's registry accepts. This library holds no capability table of
//! its own and the closed built-in table of FR-005 gains no entry.
//!
//! The dispatcher computes no projection, no realizability verdict, no
//! strategy and no witness. It binds the claim to its premises (FR-151), asks
//! the one backend FR-290 names, and carries what comes back unchanged
//! (FR-150, FR-152). While no backend serves the kind it settles the claim
//! `unsupported` (FR-153).

use std::collections::btree_map::Entry;
use std::collections::BTreeMap;

use qsl_foundation::BackendId;
use qsl_package::protocol_view::Delivery;
use qsl_replay::{InconclusiveCause, SeparatingWitnessRecord, SettlementBasis, Truth};
use qsl_route::{BackendDescriptor, Candidate, CandidateOutcome, RegistrationRefusal, Registry};
use qsl_semantics::check::Capability;
use quire_contract_model::CheckedNodeId;
use quire_semantic_value::catalog::CatalogCode;

use crate::declarations::{ChannelDeclaration, Declarations, Ordering, RoleDeclaration};
use crate::disposition::{
    settle_claim, ClaimSettlement, Disposition, QslBoundary, UnsupportedCause,
    UnsupportedDisposition, Warning,
};

/// The two claim kinds this module dispatches.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProtocolClaimKind {
    /// `local-projection`.
    LocalProjection,
    /// `realizability`.
    Realizability,
}

impl ProtocolClaimKind {
    /// QSL's capability kind for the claim kind.
    #[must_use]
    pub const fn capability(self) -> Capability {
        match self {
            Self::LocalProjection => Capability::LocalProjection,
            Self::Realizability => Capability::Realizability,
        }
    }
}

/// The nodes of the claim's admitted fragment whose premises the claim
/// depends on.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Fragment {
    /// The channels of the fragment.
    pub channels: Vec<CheckedNodeId>,
    /// The roles of the fragment.
    pub roles: Vec<CheckedNodeId>,
}

/// One channel's premise as the requester states it (FR-151). The ordering
/// and delivery are the components QSL's read view publishes, so they are
/// typed and must equal the checked package's; the components it does not yet
/// publish (sender, receiver, message type) are `C`, forwarded as stated.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChannelPremise<C> {
    /// The ordering policy.
    pub ordering: Ordering,
    /// The delivery policy.
    pub delivery: Delivery,
    /// The components the read view does not publish.
    pub unread: C,
}

/// The per-claim premises the requester supplied, keyed by the channel or role
/// they are the premise of (FR-151). `C` is a channel's unread components and
/// `I` one role internality, as the requester states them.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Premises<C, I> {
    /// The premise of each channel.
    pub channels: BTreeMap<CheckedNodeId, ChannelPremise<C>>,
    /// The internality of each role.
    pub roles: BTreeMap<CheckedNodeId, I>,
}

impl<C, I> Default for Premises<C, I> {
    fn default() -> Self {
        Self {
            channels: BTreeMap::new(),
            roles: BTreeMap::new(),
        }
    }
}

/// One requested `local-projection` or `realizability` claim.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtocolRequest<A, C, I> {
    /// The claim kind.
    pub kind: ProtocolClaimKind,
    /// The admitted fragment.
    pub fragment: Fragment,
    /// The backend the request names, when it names one.
    pub backend: Option<BackendId>,
    /// The assumptions, as the requester states them.
    pub assumptions: A,
    /// The per-claim premises.
    pub premises: Premises<C, I>,
    /// The source identities the disposition retains (FR-030).
    pub sources: Vec<CheckedNodeId>,
}

/// What a backend is sent and what a disposition retains (FR-030, FR-151):
/// the request exactly as made, with the channel and role declarations the
/// checked package authors for the fragment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Binding<A, C, I> {
    /// The request, unchanged.
    pub request: ProtocolRequest<A, C, I>,
    /// The backend the claim was selected against, once FR-290 selects one.
    pub selected: Option<BackendId>,
    /// The checked package's declaration of each channel of the fragment, in
    /// fragment order.
    pub channels: Vec<ChannelDeclaration>,
    /// The checked package's declaration of each role of the fragment, in
    /// fragment order.
    pub roles: Vec<RoleDeclaration>,
}

/// Why one disposition is refused under FR-030, scoped to that disposition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DispositionRefusal {
    /// The fragment names a channel the checked package does not declare.
    UndeclaredChannel(CheckedNodeId),
    /// The fragment names a role the checked package does not declare.
    UndeclaredRole(CheckedNodeId),
    /// The requester's premises omit the premise of a channel of the fragment.
    AbsentChannelPremise(CheckedNodeId),
    /// The requester's premises omit the internality of a role of the
    /// fragment.
    AbsentRoleInternality(CheckedNodeId),
    /// A decisive result was returned for a request that retains no source
    /// identity (FR-152).
    NoSourceIdentity,
    /// The requester's premise for a channel disagrees with the checked
    /// package's declaration of it.
    SubstitutedChannelPremise(CheckedNodeId),
    /// A decisive `realizability` result carries no trace-position witness
    /// (FR-302).
    DecisiveWithoutWitness,
    /// QSL's registry selected a backend that has no provider.
    NoProvider(BackendId),
}

/// What the dispatcher reads of a backend's result. The result itself is
/// QSL's and is carried unchanged.
pub trait BackendResult {
    /// The result's truth.
    fn truth(&self) -> Truth;
    /// The result's settlement basis.
    fn basis(&self) -> SettlementBasis;
    /// The separating-witness record the result carries, when it carries
    /// one.
    fn witness(&self) -> Option<&SeparatingWitnessRecord>;
    /// QSL's `InconclusiveCause` when the result is its `Inconclusive`
    /// verdict.
    fn inconclusive(&self) -> Option<&InconclusiveCause>;
}

/// The settlement of one requested claim.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProtocolOutcome<R, B> {
    /// The selected backend answered, or the claim settled `unsupported`.
    Settled(ClaimSettlement<R, B>),
    /// FR-290 settled the request itself `invalid-request`
    /// (`ambiguous-backend` or `unknown-backend`) and no backend was called.
    InvalidRequest {
        /// QSL's catalog code, as FR-290 spells it.
        code: CatalogCode,
        /// Every candidate, in candidate order; empty for `unknown-backend`.
        candidates: Vec<Candidate>,
        /// The binding the claim was requested against.
        binding: B,
    },
    /// The disposition is refused under FR-030.
    Refused(DispositionRefusal),
}

impl<R: BackendResult, B> ProtocolOutcome<R, B> {
    /// The truth of the disposition. A refusal, an `unsupported` cause, a
    /// requires-bound outcome, a failure and an `Inconclusive` verdict hold
    /// neither `satisfied` nor `violated`.
    #[must_use]
    pub fn truth(&self) -> Truth {
        match self {
            Self::Settled(ClaimSettlement::Result { result, .. }) => {
                if result.inconclusive().is_some() {
                    Truth::Unavailable
                } else {
                    result.truth()
                }
            }
            Self::Settled(ClaimSettlement::Disposition(_))
            | Self::InvalidRequest { .. }
            | Self::Refused(_) => Truth::Unavailable,
        }
    }
}

/// A requested claim: finite global conformance, whose result the caller
/// computed (FR-004) and this dispatcher carries untouched, or a protocol
/// claim.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClaimRequest<G, A, C, I> {
    /// `global-conformance`, with its result.
    GlobalConformance(G),
    /// `local-projection` or `realizability`.
    Protocol(ProtocolRequest<A, C, I>),
}

/// The disposition of one requested claim, in request order (FR-029).
// One disposition per requested claim, held briefly; boxing would only hide the
// settlement the caller matches on.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClaimDisposition<G, A, C, I, R> {
    /// The global conformance result, unchanged.
    GlobalConformance(G),
    /// The protocol claim's settlement.
    Protocol(ProtocolOutcome<R, Binding<A, C, I>>),
}

/// Why a provider was not registered.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RegistrationError {
    /// QSL's registry refused the descriptor.
    Registry(RegistrationRefusal),
    /// The identity already has a provider.
    ProviderHeld(BackendId),
}

/// The call into one registered backend.
type Provider<'p, A, C, I, R> = Box<dyn QslBoundary<Binding<A, C, I>, R> + 'p>;

/// The provider boundary: QSL's registry, and the call into each backend
/// registered in it. A caller supplies no callback per request.
pub struct ProviderBoundary<'p, A, C, I, R> {
    registry: Registry,
    providers: BTreeMap<BackendId, Provider<'p, A, C, I, R>>,
}

impl<A, C, I, R> Default for ProviderBoundary<'_, A, C, I, R> {
    fn default() -> Self {
        Self {
            registry: Registry::new(),
            providers: BTreeMap::new(),
        }
    }
}

impl<'p, A, C, I, R> ProviderBoundary<'p, A, C, I, R> {
    /// An empty boundary: no backend serves any kind.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// QSL's registry snapshot.
    #[must_use]
    pub const fn registry(&self) -> &Registry {
        &self.registry
    }

    /// Register `provider` under `descriptor`, as QSL's registry admits it.
    ///
    /// # Errors
    ///
    /// [`RegistrationError::Registry`] with QSL's `duplicate-backend` refusal
    /// when the identity conflicts; the identity is then served by no
    /// provider. [`RegistrationError::ProviderHeld`] when the identity
    /// already has a provider; the held provider is kept and `provider` is
    /// dropped.
    pub fn register(
        &mut self,
        descriptor: BackendDescriptor,
        provider: impl QslBoundary<Binding<A, C, I>, R> + 'p,
    ) -> Result<(), RegistrationError> {
        let id = descriptor.id().clone();
        match self.registry.register(descriptor) {
            Ok(()) => match self.providers.entry(id) {
                Entry::Occupied(held) => Err(RegistrationError::ProviderHeld(held.key().clone())),
                Entry::Vacant(slot) => {
                    slot.insert(Box::new(provider));
                    Ok(())
                }
            },
            Err(refusal) => {
                self.providers.remove(&id);
                Err(RegistrationError::Registry(refusal))
            }
        }
    }
}

/// Bind `request` to the checked package's declarations and to the
/// requester's premises (FR-151).
///
/// # Errors
///
/// [`DispositionRefusal`] naming the absent or undeclared component; no
/// default, inferred or sampled premise is supplied.
pub fn bind<A: Clone, C: Clone, I: Clone>(
    declarations: &Declarations,
    request: &ProtocolRequest<A, C, I>,
) -> Result<Binding<A, C, I>, DispositionRefusal> {
    let mut channels = Vec::with_capacity(request.fragment.channels.len());
    for node in &request.fragment.channels {
        let declared = declarations
            .channel(node)
            .ok_or_else(|| DispositionRefusal::UndeclaredChannel(node.clone()))?;
        let stated = request
            .premises
            .channels
            .get(node)
            .ok_or_else(|| DispositionRefusal::AbsentChannelPremise(node.clone()))?;
        if stated.ordering != declared.ordering || stated.delivery != declared.delivery {
            return Err(DispositionRefusal::SubstitutedChannelPremise(node.clone()));
        }
        channels.push(declared.clone());
    }
    let mut roles = Vec::with_capacity(request.fragment.roles.len());
    for node in &request.fragment.roles {
        let declared = declarations
            .role(node)
            .ok_or_else(|| DispositionRefusal::UndeclaredRole(node.clone()))?;
        if !request.premises.roles.contains_key(node) {
            return Err(DispositionRefusal::AbsentRoleInternality(node.clone()));
        }
        roles.push(declared.clone());
    }
    Ok(Binding {
        request: request.clone(),
        selected: None,
        channels,
        roles,
    })
}

/// Settle one claim: bind its premises, take FR-290's candidate set from
/// QSL's registry, send the claim to the one candidate, and carry what
/// returns.
pub fn dispatch<'p, A, C, I, R>(
    boundary: &mut ProviderBoundary<'p, A, C, I, R>,
    declarations: &Declarations,
    request: &ProtocolRequest<A, C, I>,
) -> ProtocolOutcome<R, Binding<A, C, I>>
where
    A: Clone,
    C: Clone,
    I: Clone,
    R: BackendResult,
{
    let binding = match bind(declarations, request) {
        Ok(binding) => binding,
        Err(refusal) => return ProtocolOutcome::Refused(refusal),
    };
    let capability = request.kind.capability();
    let set = match boundary
        .registry
        .candidates(capability, request.backend.as_ref())
    {
        CandidateOutcome::UnknownBackend(_) => {
            return ProtocolOutcome::InvalidRequest {
                code: CatalogCode::new("invalid_capability", "unknown-backend"),
                candidates: Vec::new(),
                binding,
            }
        }
        CandidateOutcome::Candidates(set) => set,
    };
    let [candidate] = set.candidates() else {
        return if set.is_empty() {
            ProtocolOutcome::Settled(ClaimSettlement::Disposition(Disposition::Unsupported(
                UnsupportedDisposition {
                    cause: UnsupportedCause::requested_capability(),
                    construct: None,
                    premises: binding,
                    warnings: vec![Warning::UnservedCapability {
                        kind: set.kind(),
                        backend: set.requested_backend().cloned(),
                    }],
                },
            )))
        } else {
            ProtocolOutcome::InvalidRequest {
                code: CatalogCode::new("invalid_capability", "ambiguous-backend"),
                candidates: set.candidates().to_vec(),
                binding,
            }
        };
    };
    let mut binding = binding;
    binding.selected = Some(candidate.id().clone());
    let Some(provider) = boundary.providers.get_mut(candidate.id()) else {
        return ProtocolOutcome::Refused(DispositionRefusal::NoProvider(candidate.id().clone()));
    };
    let claim = binding.clone();
    let settlement = settle_claim(provider.as_mut(), &claim, binding);
    if let ClaimSettlement::Result { result, premises } = &settlement {
        if let Some(refusal) = carry_refusal(request.kind, result, &premises.request.sources) {
            return ProtocolOutcome::Refused(refusal);
        }
    }
    ProtocolOutcome::Settled(settlement)
}

/// FR-152: a decisive result is carried beside the source identities the
/// disposition retains, and a decisive `realizability` result with its
/// trace-position witness; without them it is refused. The witness is never
/// read, normalised or reordered.
fn carry_refusal<R: BackendResult>(
    kind: ProtocolClaimKind,
    result: &R,
    sources: &[CheckedNodeId],
) -> Option<DispositionRefusal> {
    if !result.basis().is_decisive() {
        return None;
    }
    if sources.is_empty() {
        return Some(DispositionRefusal::NoSourceIdentity);
    }
    let positioned = result
        .witness()
        .is_some_and(|record| record.trace_position.is_some());
    if kind == ProtocolClaimKind::Realizability && !positioned {
        return Some(DispositionRefusal::DecisiveWithoutWitness);
    }
    None
}

/// Settle every requested claim, in request order and independently of one
/// another (FR-029). Global conformance results pass through untouched.
pub fn dispatch_claims<'p, G, A, C, I, R>(
    boundary: &mut ProviderBoundary<'p, A, C, I, R>,
    declarations: &Declarations,
    requests: &[ClaimRequest<G, A, C, I>],
) -> Vec<ClaimDisposition<G, A, C, I, R>>
where
    G: Clone,
    A: Clone,
    C: Clone,
    I: Clone,
    R: BackendResult,
{
    requests
        .iter()
        .map(|request| match request {
            ClaimRequest::GlobalConformance(result) => {
                ClaimDisposition::GlobalConformance(result.clone())
            }
            ClaimRequest::Protocol(request) => {
                ClaimDisposition::Protocol(dispatch(boundary, declarations, request))
            }
        })
        .collect()
}
