// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-057, FR-058, FR-059: QSL's `Unsupported` causes and requires-bound
//! outcomes carried as typed non-success dispositions.
//!
//! QSL decides what a protocol construct can do; when it reports that it
//! cannot step a construct (`Unsupported`) or that a request names a domain
//! it has no bound for (`RequiresBound`), this module carries that report
//! unchanged, bound to the construct and to the exact premises of the request.
//! Nothing here approximates, truncates, defaults or completes the analysis:
//! neither disposition is a success, and the only calls a claim's settlement
//! makes into QSL carry the claim itself, never a bound.
//!
//! The premises are generic: the dispatcher of FR-005, FR-029 and FR-030 owns
//! the premise record and supplies it; this module retains it byte for byte.

use qsl_eval::simulation::protocol::Unsupported;
use qsl_eval::simulation::RequiresBound;
use qsl_foundation::bound::{DomainKey, DomainKind};
use qsl_foundation::BackendId;
use qsl_replay::{SettlementBasis, Truth};
use qsl_semantics::check::Capability;
use quire_contract_model::{CheckedNodeId, CheckedPackageV2, CheckedSourceMapEntry};
use quire_semantic_value::catalog::CatalogCode;

use crate::closure::AssessmentExecution;
use crate::declarations::LoopDeclaration;
use crate::link::source_locus;
use crate::refusal::catalog::RetainedBytes;

/// The checked node of a construct and the source-map entries available for
/// it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Construct {
    /// The checked node id.
    pub node: CheckedNodeId,
    /// The source locus, empty when the package records none.
    pub locus: Vec<CheckedSourceMapEntry>,
}

impl Construct {
    /// The construct at `node` of `package`.
    #[must_use]
    pub fn of(package: &CheckedPackageV2, node: &CheckedNodeId) -> Self {
        Self {
            node: node.clone(),
            locus: source_locus(package, node),
        }
    }
}

/// The cause of an `unsupported` disposition, as QSL reports it.
///
/// QSL's `Unsupported` publishes no name accessor, only the variant itself, so
/// the cause is carried as that typed value and compared as that value; its
/// name is the variant's, which is what a debug rendering of the carried
/// value shows.
#[derive(Clone, Debug)]
pub enum UnsupportedCause {
    /// A construct QSL's protocol system does not step, as QSL reported it.
    Construct(Unsupported),
    /// A construct QSL steps but this library's own assessment of an observed
    /// history does not (ADR-010: QSL owns the engines). No QSL `Unsupported`
    /// variant stands behind it.
    Assessment(AssessmentUnsupported),
    /// A claim or discharge no QSL engine serves, with QSL's own catalog code
    /// for it.
    RequestedCapability(CatalogCode),
    /// A cause the caller received from outside and could not type as either
    /// of the above, retained byte for byte and mapped to no known cause
    /// (FR-008).
    Unrecognised(RetainedBytes),
}

/// A construct this library's assessment of an observed history does not
/// assess, although QSL steps it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AssessmentUnsupported {
    /// A `join predicate(..)`: the predicate is QSL's to evaluate.
    JoinPredicate,
    /// An unordered channel whose overflow loses its head or tail: QSL steps
    /// it over a send-ordered queue this library does not reconstruct.
    UnorderedLossSelector,
}

impl PartialEq for UnsupportedCause {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Construct(left), Self::Construct(right)) => left == right,
            (Self::Assessment(left), Self::Assessment(right)) => left == right,
            (Self::RequestedCapability(left), Self::RequestedCapability(right)) => left == right,
            (Self::Unrecognised(left), Self::Unrecognised(right)) => left == right,
            (
                Self::Construct(_)
                | Self::Assessment(_)
                | Self::RequestedCapability(_)
                | Self::Unrecognised(_),
                _,
            ) => false,
        }
    }
}

impl Eq for UnsupportedCause {}

impl UnsupportedCause {
    /// QSL's cause for a request no engine serves:
    /// `unsupported_projection`/`unsupported-requested-capability`, spelled
    /// as QSL spells it wherever it settles such an item. QSL exports no
    /// constant for it.
    #[must_use]
    pub const fn requested_capability() -> Self {
        Self::RequestedCapability(CatalogCode::new(
            "unsupported_projection",
            "unsupported-requested-capability",
        ))
    }

    /// A cause received as bytes this library cannot type, retained unchanged.
    #[must_use]
    pub fn unrecognised(name: &[u8]) -> Self {
        Self::Unrecognised(RetainedBytes::new(name))
    }
}

/// A warning a disposition carries beside its cause.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Warning {
    /// No accepted backend serves the capability kind the claim requires
    /// (QSL FR-290, this library's FR-153): the kind, and the backend the
    /// request named when it named one.
    UnservedCapability {
        /// The required capability kind.
        kind: Capability,
        /// The backend the request named, when it named one.
        backend: Option<BackendId>,
    },
    /// The backend that served a decisive result returned no
    /// `native-run-result/2` protocol-analysis record, so the witness the
    /// claim needs cannot be imported (FR-302): the required capability
    /// kind and the backend.
    NoRunRecord {
        /// The capability kind the claim required.
        kind: Capability,
        /// The backend that served it.
        backend: BackendId,
    },
}

/// An `unsupported` disposition (FR-241 `unsupported`): never false,
/// satisfied, conformant or successful, and the source of no derived result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnsupportedDisposition<P> {
    /// QSL's cause, by name.
    pub cause: UnsupportedCause,
    /// The construct, by checked node and locus; absent when the cause
    /// concerns the whole claim and no one construct.
    pub construct: Option<Construct>,
    /// The exact premises the request carried, unchanged.
    pub premises: P,
    /// The warnings the disposition carries.
    pub warnings: Vec<Warning>,
}

impl<P> UnsupportedDisposition<P> {
    /// The execution state: always `unsupported`.
    #[must_use]
    pub const fn execution(&self) -> AssessmentExecution {
        AssessmentExecution::Unsupported
    }

    /// The truth (FR-242): always `unavailable`.
    #[must_use]
    pub const fn truth(&self) -> Truth {
        Truth::Unavailable
    }

    /// The settlement basis (FR-243): always `unavailable`.
    #[must_use]
    pub const fn basis(&self) -> SettlementBasis {
        SettlementBasis::Unavailable
    }
}

/// A requires-bound disposition: QSL's outcome unchanged, with its unbounded
/// domains. It is non-success on its own axis, folded into no conformance,
/// progress or closure value, and carries no partial result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequiresBoundDisposition<P> {
    /// QSL's outcome, unchanged.
    pub outcome: RequiresBound,
    /// The construct the bound is missing for; absent when the report names
    /// none.
    pub construct: Option<Construct>,
    /// The exact premises the request carried, unchanged.
    pub premises: P,
}

impl<P> RequiresBoundDisposition<P> {
    /// Each unbounded domain with the key and kind QSL names.
    pub fn domains(&self) -> impl Iterator<Item = (&DomainKey, DomainKind)> {
        self.outcome.domains.iter()
    }
}

/// A typed non-success disposition. It has no success variant.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Disposition<P> {
    /// QSL reported an `Unsupported` cause, or no engine serves the request.
    Unsupported(UnsupportedDisposition<P>),
    /// QSL reported unbounded domains.
    RequiresBound(RequiresBoundDisposition<P>),
    /// QSL's backend returned a refusal, by its catalog code.
    Refused(RefusedDisposition<P>),
    /// The selected backend failed for exactly this claim.
    Failed(FailedDisposition<P>),
}

/// A refusal the backend returned, carried by its catalog code. It holds no
/// truth `satisfied` or `violated`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RefusedDisposition<P> {
    /// The backend's code, in its own vocabulary.
    pub code: CatalogCode,
    /// The exact premises the request carried, unchanged.
    pub premises: P,
}

/// A selected backend's failure for one claim, with the diagnostic it
/// reported when it reported one.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FailedDisposition<P> {
    /// The backend's diagnostic code.
    pub diagnostic: Option<CatalogCode>,
    /// The exact premises the request carried, unchanged.
    pub premises: P,
}

/// What QSL reported for one claim.
#[derive(Clone, Debug)]
pub enum QslReport<R> {
    /// QSL's result, carried unchanged by the caller's own result type.
    Result(R),
    /// QSL reported an `Unsupported` cause for a construct.
    Unsupported {
        /// The cause.
        cause: UnsupportedCause,
        /// The construct, when the report names one.
        construct: Option<Construct>,
    },
    /// QSL reported a requires-bound outcome for a construct.
    RequiresBound {
        /// The outcome, unchanged.
        outcome: RequiresBound,
        /// The construct, when the report names one.
        construct: Option<Construct>,
    },
    /// QSL's backend returned a refusal, by its catalog code.
    Refusal(CatalogCode),
    /// The selected backend failed.
    Failed(Option<CatalogCode>),
}

impl<R> QslReport<R> {
    /// The same report with its result, when it has one, passed through `f`.
    pub fn map<S>(self, f: impl FnOnce(R) -> S) -> QslReport<S> {
        match self {
            Self::Result(result) => QslReport::Result(f(result)),
            Self::Unsupported { cause, construct } => QslReport::Unsupported { cause, construct },
            Self::RequiresBound { outcome, construct } => {
                QslReport::RequiresBound { outcome, construct }
            }
            Self::Refusal(code) => QslReport::Refusal(code),
            Self::Failed(diagnostic) => QslReport::Failed(diagnostic),
        }
    }
}

/// The call boundary into QSL's protocol analysis.
///
/// This seam is deliberately minimal. A request carries the claim and nothing
/// else: there is no parameter through which this library could pass a bound
/// it chose. A report is a result, a refusal QSL's backend returned, an
/// `Unsupported` cause or requires-bound outcome with an optional construct,
/// or a failure of the backend; a disposition carries its own warnings.
pub trait QslBoundary<C, R> {
    /// Ask QSL about `claim`.
    fn report(&mut self, claim: &C) -> QslReport<R>;
}

/// One claim's settlement, independent of every other claim's.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClaimSettlement<R, P> {
    /// QSL answered; the result is carried with the request's premises.
    Result {
        /// QSL's result, unchanged.
        result: R,
        /// The premises of the request.
        premises: P,
    },
    /// QSL reported no result for the claim.
    Disposition(Disposition<P>),
}

/// Settle one claim against `boundary`, binding the report to `premises`.
pub fn settle_claim<C, R, P>(
    boundary: &mut (impl QslBoundary<C, R> + ?Sized),
    claim: &C,
    premises: P,
) -> ClaimSettlement<R, P> {
    match boundary.report(claim) {
        QslReport::Result(result) => ClaimSettlement::Result { result, premises },
        QslReport::Unsupported { cause, construct } => {
            ClaimSettlement::Disposition(Disposition::Unsupported(UnsupportedDisposition {
                cause,
                construct,
                premises,
                warnings: Vec::new(),
            }))
        }
        QslReport::Refusal(code) => {
            ClaimSettlement::Disposition(Disposition::Refused(RefusedDisposition {
                code,
                premises,
            }))
        }
        QslReport::Failed(diagnostic) => {
            ClaimSettlement::Disposition(Disposition::Failed(FailedDisposition {
                diagnostic,
                premises,
            }))
        }
        QslReport::RequiresBound { outcome, construct } => {
            ClaimSettlement::Disposition(Disposition::RequiresBound(RequiresBoundDisposition {
                outcome,
                construct,
                premises,
            }))
        }
    }
}

/// Settle each of `claims` against `boundary`: exactly one settlement per
/// claim, in order, none derived from another claim's.
pub fn settle_claims<C, R, P: Clone>(
    boundary: &mut (impl QslBoundary<C, R> + ?Sized),
    claims: &[(C, P)],
) -> Vec<ClaimSettlement<R, P>> {
    claims
        .iter()
        .map(|(claim, premises)| settle_claim(boundary, claim, premises.clone()))
        .collect()
}

// ---------------------------------------------------------------------------
// FR-059: loop invariant and variant stay undischarged
// ---------------------------------------------------------------------------

/// A loop proof's discharge status. QSL serves no engine that discharges a
/// `repeat`'s invariant or variant, so this library produces exactly one.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DischargeStatus {
    /// Declared, and not discharged.
    Undischarged,
}

/// A loop's declared invariant and variant, referenced by checked node and
/// carried as undischarged. No invariant check, variant rank or termination
/// verdict appears here or in any result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoopProof {
    /// The `repeat` control.
    pub control: CheckedNodeId,
    /// The invariant node, when declared.
    pub invariant: Option<CheckedNodeId>,
    /// The variant node, when declared.
    pub variant: Option<CheckedNodeId>,
    /// Always [`DischargeStatus::Undischarged`].
    pub status: DischargeStatus,
}

impl LoopProof {
    /// The proof the loop declares.
    #[must_use]
    pub fn of(declaration: &LoopDeclaration) -> Self {
        Self {
            control: declaration.control.clone(),
            invariant: declaration.invariant.clone(),
            variant: declaration.variant.clone(),
            status: DischargeStatus::Undischarged,
        }
    }
}

/// The disposition of a request to discharge `declaration`'s invariant or
/// variant: `unsupported`, carrying QSL's requested-capability cause, bound to
/// the request's premises. The construct and its locus are the loop's own
/// control's, read from `package`.
#[must_use]
pub fn request_discharge<P>(
    package: &CheckedPackageV2,
    declaration: &LoopDeclaration,
    premises: P,
) -> UnsupportedDisposition<P> {
    let construct = Construct::of(package, &declaration.control);
    UnsupportedDisposition {
        cause: UnsupportedCause::requested_capability(),
        construct: Some(construct),
        premises,
        warnings: Vec::new(),
    }
}
