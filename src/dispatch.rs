// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-150, FR-151, FR-152, FR-153: dispatch of `local-projection` and
//! `realizability` claims to the backend QSL's registry selects.
//!
//! QSL decides what a backend serves. The registry is QSL's own
//! ([`qsl_route::Registry`], FR-290's candidate computation), and a backend
//! joins it at the provider boundary ([`ProviderBoundary::register`](crate::dispatch::ProviderBoundary::register)) with the
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
use qsl_replay::{InconclusiveCause, ItemLabel, SeparatingWitnessRecord, SettlementBasis, Truth};
use qsl_route::{BackendDescriptor, Candidate, CandidateOutcome, RegistrationRefusal, Registry};
use qsl_semantics::check::Capability;
use quire_contract_model::CheckedNodeId;
use quire_semantic_value::catalog::CatalogCode;

use crate::assume_guarantee::{
    AssumeGuaranteeBinding, AssumeGuaranteeRequest, AssumeGuaranteeResult, IdentityMismatch,
    PremiseList,
};
use crate::declarations::{ChannelDeclaration, Declarations, Ordering, RoleDeclaration};
use crate::disposition::{
    settle_claim, ClaimSettlement, Disposition, QslBoundary, UnsupportedCause,
    UnsupportedDisposition, Warning,
};
use crate::refusal::ProtocolCauseKind;

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
/// typed and must equal the checked package's. The sender, receiver and
/// message type are the linked declaration's (QSL publishes them), so the
/// requester states none of them.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChannelPremise {
    /// The ordering policy.
    pub ordering: Ordering,
    /// The delivery policy.
    pub delivery: Delivery,
}

/// The per-claim premises the requester supplied, keyed by the channel or role
/// they are the premise of (FR-151). `I` is one role internality, as the
/// requester states it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Premises<I> {
    /// The premise of each channel.
    pub channels: BTreeMap<CheckedNodeId, ChannelPremise>,
    /// The internality of each role.
    pub roles: BTreeMap<CheckedNodeId, I>,
}

impl<I> Default for Premises<I> {
    fn default() -> Self {
        Self {
            channels: BTreeMap::new(),
            roles: BTreeMap::new(),
        }
    }
}

/// One requested `local-projection` or `realizability` claim.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtocolRequest<A, I> {
    /// The claim kind.
    pub kind: ProtocolClaimKind,
    /// The admitted fragment.
    pub fragment: Fragment,
    /// The backend the request names, when it names one.
    pub backend: Option<BackendId>,
    /// The assumptions, as the requester states them.
    pub assumptions: A,
    /// The per-claim premises.
    pub premises: Premises<I>,
    /// The source identities the disposition retains (FR-030).
    pub sources: Vec<CheckedNodeId>,
}

/// What a backend is sent and what a disposition retains (FR-030, FR-151):
/// the request exactly as made, with the channel and role declarations the
/// checked package authors for the fragment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Binding<A, I> {
    /// The request, unchanged.
    pub request: ProtocolRequest<A, I>,
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
    /// The provider answered with the other claim kind's result.
    WrongAnswerKind,
    /// A `composition` or `refinement` request names no component subject
    /// (FR-200).
    NoSubjects,
    /// A `composition` or `refinement` request names no admitted fragment
    /// (FR-200).
    NoFragment,
    /// A component subject is not a node of the checked package's protocol.
    UndeclaredSubject(CheckedNodeId),
    /// An assumption identity is not a node of the checked package's
    /// protocol.
    UndeclaredAssumption(CheckedNodeId),
    /// A source identity is not a node of the checked package's protocol.
    UndeclaredSource(CheckedNodeId),
    /// A guarantee of a declared cycle is offered as an assumption, in the
    /// request or in the result's retained assumptions (FR-203).
    CycleGuaranteeAsAssumption(CheckedNodeId),
    /// A result states an assessment-execution value its label does not read
    /// into (FR-202).
    ContradictoryExecution(ItemLabel),
    /// A result names an identity the request does not, or omits a field it
    /// must retain (FR-201, FR-204).
    IdentityMismatch(IdentityMismatch),
    /// A result retains a premise list that differs from the request's
    /// (FR-201).
    RetainedPremiseList(PremiseList),
}

impl DispositionRefusal {
    /// The FR-008 cause this refusal is raised under, where the closed
    /// vocabulary names one: the two exact identities that must agree
    /// differ.
    #[must_use]
    pub const fn cause(&self) -> Option<ProtocolCauseKind> {
        match self {
            Self::IdentityMismatch(_)
            | Self::RetainedPremiseList(_)
            | Self::SubstitutedChannelPremise(_) => Some(ProtocolCauseKind::IdentityMismatch),
            _ => None,
        }
    }
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
                match result.truth() {
                    // A decided truth cannot accompany an `Inconclusive`
                    // verdict; the backend's `pending` and `unavailable` are
                    // carried as it states them.
                    Truth::Satisfied | Truth::Violated if result.inconclusive().is_some() => {
                        Truth::Unavailable
                    }
                    truth => truth,
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
pub enum ClaimRequest<G, A, I, T> {
    /// `global-conformance`, with its result.
    GlobalConformance(G),
    /// `local-projection` or `realizability`.
    Protocol(ProtocolRequest<A, I>),
    /// `composition` or `refinement` (FR-200).
    AssumeGuarantee(AssumeGuaranteeRequest<T>),
}

/// The disposition of one requested claim, in request order (FR-029).
// One disposition per requested claim, held briefly; boxing would only hide the
// settlement the caller matches on.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClaimDisposition<G, A, I, T, P, Q> {
    /// The global conformance result, unchanged.
    GlobalConformance(G),
    /// The protocol claim's settlement.
    Protocol(ProtocolOutcome<P, Binding<A, I>>),
    /// The composition or refinement claim's settlement.
    AssumeGuarantee(ProtocolOutcome<Q, AssumeGuaranteeBinding<T>>),
}

/// The claim a registered provider is asked about: one boundary, registry and
/// dispatch path serve every claim kind (FR-200).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Claim<A, I, T> {
    /// A `local-projection` or `realizability` claim.
    Protocol(Binding<A, I>),
    /// A `composition` or `refinement` claim.
    AssumeGuarantee(AssumeGuaranteeBinding<T>),
}

impl<A, I, T> From<Binding<A, I>> for Claim<A, I, T> {
    fn from(binding: Binding<A, I>) -> Self {
        Self::Protocol(binding)
    }
}

impl<A, I, T> From<AssumeGuaranteeBinding<T>> for Claim<A, I, T> {
    fn from(binding: AssumeGuaranteeBinding<T>) -> Self {
        Self::AssumeGuarantee(binding)
    }
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
type Provider<'p, K, R> = Box<dyn QslBoundary<K, R> + 'p>;

/// The provider boundary: QSL's registry, and the call into each backend
/// registered in it. A caller supplies no callback per request. `K` is the
/// claim a provider is asked about: a [`Binding`] for the protocol claims, an
/// [`AssumeGuaranteeBinding`]
/// for composition and refinement.
pub struct ProviderBoundary<'p, K, R> {
    registry: Registry,
    providers: BTreeMap<BackendId, Provider<'p, K, R>>,
}

impl<K, R> Default for ProviderBoundary<'_, K, R> {
    fn default() -> Self {
        Self {
            registry: Registry::new(),
            providers: BTreeMap::new(),
        }
    }
}

impl<'p, K, R> ProviderBoundary<'p, K, R> {
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
        provider: impl QslBoundary<K, R> + 'p,
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

/// FR-290's selection of the one backend that serves `capability`, or the
/// outcome that settles the request without calling a backend: `unsupported`
/// while none serves it, `invalid-request` when the named backend is unknown
/// or the candidates are ambiguous. `binding` is what that outcome retains.
pub(crate) fn select<R, B>(
    registry: &Registry,
    capability: Capability,
    requested: Option<&BackendId>,
    binding: B,
) -> Result<(BackendId, B), Box<ProtocolOutcome<R, B>>> {
    let set = match registry.candidates(capability, requested) {
        CandidateOutcome::UnknownBackend(_) => {
            return Err(Box::new(ProtocolOutcome::InvalidRequest {
                // IR-1513: the owner (quire-contract-codegen) is to publish this code.
                code: CatalogCode::new("invalid_capability", "unknown-backend"),
                candidates: Vec::new(),
                binding,
            }));
        }
        CandidateOutcome::Candidates(set) => set,
    };
    let [candidate] = set.candidates() else {
        return Err(Box::new(if set.is_empty() {
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
                // IR-1513: the owner (quire-contract-codegen) is to publish this code.
                code: CatalogCode::new("invalid_capability", "ambiguous-backend"),
                candidates: set.candidates().to_vec(),
                binding,
            }
        }));
    };
    Ok((candidate.id().clone(), binding))
}

/// Bind `request` to the checked package's declarations and to the
/// requester's premises (FR-151).
///
/// # Errors
///
/// [`DispositionRefusal`] naming the absent or undeclared component; no
/// default, inferred or sampled premise is supplied.
pub fn bind<A: Clone, I: Clone>(
    declarations: &Declarations,
    request: &ProtocolRequest<A, I>,
) -> Result<Binding<A, I>, DispositionRefusal> {
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

/// A boundary that is never asked a kind has no result of that kind.
impl BackendResult for std::convert::Infallible {
    fn truth(&self) -> Truth {
        match *self {}
    }
    fn basis(&self) -> SettlementBasis {
        match *self {}
    }
    fn witness(&self) -> Option<&SeparatingWitnessRecord> {
        match *self {}
    }
    fn inconclusive(&self) -> Option<&InconclusiveCause> {
        match *self {}
    }
}

/// The result a provider returns: its protocol result or its composition or
/// refinement result, each QSL's own and carried unchanged. The two are
/// separate types, so a caller that asks one kind implements the other's
/// reading for nothing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Answer<P, Q> {
    /// The answer to a `local-projection` or `realizability` claim.
    Protocol(P),
    /// The answer to a `composition` or `refinement` claim.
    AssumeGuarantee(Q),
}

/// The steps every claim kind shares once its premises are bound: take
/// FR-290's candidate set from QSL's registry, send the claim to the one
/// candidate, carry what returns, and refuse a result that `refuse` rejects.
pub(crate) fn settle_bound<K, P, Q, R, B: Clone>(
    boundary: &mut ProviderBoundary<'_, K, Answer<P, Q>>,
    binding: B,
    (capability, requested): (Capability, Option<&BackendId>),
    claim_of: impl FnOnce(&B) -> K,
    mark_selected: impl FnOnce(&mut B, BackendId),
    extract: impl FnOnce(Answer<P, Q>) -> Option<R>,
    refuse: impl FnOnce(&R, &B) -> Option<DispositionRefusal>,
) -> ProtocolOutcome<R, B> {
    let (selected, mut binding) = match select(&boundary.registry, capability, requested, binding) {
        Ok(selected) => selected,
        Err(outcome) => return *outcome,
    };
    mark_selected(&mut binding, selected.clone());
    let Some(provider) = boundary.providers.get_mut(&selected) else {
        return ProtocolOutcome::Refused(DispositionRefusal::NoProvider(selected));
    };
    let claim = claim_of(&binding);
    match settle_claim(provider.as_mut(), &claim, binding) {
        ClaimSettlement::Result { result, premises } => {
            let Some(result) = extract(result) else {
                return ProtocolOutcome::Refused(DispositionRefusal::WrongAnswerKind);
            };
            match refuse(&result, &premises) {
                Some(refusal) => ProtocolOutcome::Refused(refusal),
                None => ProtocolOutcome::Settled(ClaimSettlement::Result { result, premises }),
            }
        }
        ClaimSettlement::Disposition(disposition) => {
            ProtocolOutcome::Settled(ClaimSettlement::Disposition(disposition))
        }
    }
}

/// Settle one claim: bind its premises, take FR-290's candidate set from
/// QSL's registry, send the claim to the one candidate, and carry what
/// returns.
pub fn dispatch<K, A, I, P, Q>(
    boundary: &mut ProviderBoundary<'_, K, Answer<P, Q>>,
    declarations: &Declarations,
    request: &ProtocolRequest<A, I>,
) -> ProtocolOutcome<P, Binding<A, I>>
where
    K: From<Binding<A, I>>,
    A: Clone,
    I: Clone,
    P: BackendResult,
{
    let binding = match bind(declarations, request) {
        Ok(binding) => binding,
        Err(refusal) => return ProtocolOutcome::Refused(refusal),
    };
    settle_bound(
        boundary,
        binding,
        (request.kind.capability(), request.backend.as_ref()),
        |binding| K::from(binding.clone()),
        |binding, selected| binding.selected = Some(selected),
        |answer| match answer {
            Answer::Protocol(result) => Some(result),
            Answer::AssumeGuarantee(_) => None,
        },
        |result, binding| carry_refusal(request.kind, result, &binding.request.sources),
    )
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
pub fn dispatch_claims<K, G, A, I, T, P, Q>(
    boundary: &mut ProviderBoundary<'_, K, Answer<P, Q>>,
    declarations: &Declarations,
    components: &[&Declarations],
    requests: &[ClaimRequest<G, A, I, T>],
) -> Vec<ClaimDisposition<G, A, I, T, P, Q>>
where
    K: From<Binding<A, I>> + From<AssumeGuaranteeBinding<T>>,
    G: Clone,
    A: Clone,
    I: Clone,
    T: Clone + PartialEq,
    P: BackendResult,
    Q: AssumeGuaranteeResult<T>,
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
            ClaimRequest::AssumeGuarantee(request) => ClaimDisposition::AssumeGuarantee(
                crate::assume_guarantee::dispatch(boundary, components, request),
            ),
        })
        .collect()
}
