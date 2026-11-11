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
use quire_contract_model::{CheckedNodeId, CheckedPackageV2, CheckedSourceMapEntry};

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

/// QSL's cause name for a request that no engine serves.
pub const REQUESTED_CAPABILITY: &str = "unsupported-requested-capability";

/// The cause of an `unsupported` disposition, by QSL's own name.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UnsupportedCause {
    /// A construct QSL's protocol system does not step, as QSL reported it.
    Construct(Unsupported),
    /// A claim or discharge no QSL engine serves: [`REQUESTED_CAPABILITY`].
    RequestedCapability,
    /// A cause name this library does not recognise, retained byte for byte
    /// and mapped to no known cause (FR-008).
    Unrecognised(RetainedBytes),
}

/// The payload-less QSL `Unsupported` variants, by their QSL names.
const NAMED: [(&str, Unsupported); 12] = [
    ("WeakMemory", Unsupported::WeakMemory),
    ("AccessFootprint", Unsupported::AccessFootprint),
    ("SymbolicCapacity", Unsupported::SymbolicCapacity),
    ("UnorderedLossSelector", Unsupported::UnorderedLossSelector),
    ("UndecidedInitialGuard", Unsupported::UndecidedInitialGuard),
    ("Capture", Unsupported::Capture),
    (
        "ActivationOnEachTemplate",
        Unsupported::ActivationOnEachTemplate,
    ),
    ("JoinPredicate", Unsupported::JoinPredicate),
    ("UnruledOutstanding", Unsupported::UnruledOutstanding),
    ("OverNotAnObject", Unsupported::OverNotAnObject),
    ("UnboundAttemptBinder", Unsupported::UnboundAttemptBinder),
    ("NodelessBinder", Unsupported::NodelessBinder),
];

impl UnsupportedCause {
    /// The cause named `name` when it is one QSL defines, otherwise the name
    /// retained unchanged as [`Self::Unrecognised`].
    #[must_use]
    pub fn from_name(name: &[u8]) -> Self {
        if name == REQUESTED_CAPABILITY.as_bytes() {
            return Self::RequestedCapability;
        }
        if name == b"UnenumerableRecord" {
            return Self::Construct(Unsupported::UnenumerableRecord);
        }
        NAMED
            .iter()
            .find(|(known, _)| known.as_bytes() == name)
            .map_or_else(
                || Self::Unrecognised(RetainedBytes::new(name)),
                |(_, cause)| Self::Construct(*cause),
            )
    }

    /// The cause name, unchanged: the QSL variant name, QSL's requested
    /// capability name, or the bytes that were offered.
    #[must_use]
    pub fn name(&self) -> &[u8] {
        match self {
            Self::Construct(cause) => match cause {
                Unsupported::Control(_) => b"Control",
                Unsupported::WeakMemory => b"WeakMemory",
                Unsupported::AccessFootprint => b"AccessFootprint",
                Unsupported::SymbolicCapacity => b"SymbolicCapacity",
                Unsupported::UnorderedLossSelector => b"UnorderedLossSelector",
                Unsupported::UndecidedInitialGuard => b"UndecidedInitialGuard",
                Unsupported::Capture => b"Capture",
                Unsupported::ActivationOnEachTemplate => b"ActivationOnEachTemplate",
                Unsupported::JoinPredicate => b"JoinPredicate",
                Unsupported::UnruledOutstanding => b"UnruledOutstanding",
                Unsupported::OverNotAnObject => b"OverNotAnObject",
                Unsupported::UnboundAttemptBinder => b"UnboundAttemptBinder",
                Unsupported::NodelessBinder => b"NodelessBinder",
                Unsupported::UnenumerableRecord => b"UnenumerableRecord",
            },
            Self::RequestedCapability => REQUESTED_CAPABILITY.as_bytes(),
            Self::Unrecognised(name) => name.as_bytes(),
        }
    }
}

/// An `unsupported` disposition (FR-241 `unsupported`): never false,
/// satisfied, conformant or successful, and the source of no derived result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnsupportedDisposition<P> {
    /// QSL's cause, by name.
    pub cause: UnsupportedCause,
    /// The construct, by checked node and locus.
    pub construct: Construct,
    /// The exact premises the request carried, unchanged.
    pub premises: P,
}

impl<P> UnsupportedDisposition<P> {
    /// The execution state: always `unsupported`.
    #[must_use]
    pub const fn execution(&self) -> AssessmentExecution {
        AssessmentExecution::Unsupported
    }
}

/// A requires-bound disposition: QSL's outcome unchanged, with its unbounded
/// domains. It is non-success on its own axis, folded into no conformance,
/// progress or closure value, and carries no partial result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequiresBoundDisposition<P> {
    /// QSL's outcome, unchanged.
    pub outcome: RequiresBound,
    /// The construct the bound is missing for.
    pub construct: Construct,
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
        /// The construct.
        construct: Construct,
    },
    /// QSL reported a requires-bound outcome for a construct.
    RequiresBound {
        /// The outcome, unchanged.
        outcome: RequiresBound,
        /// The construct.
        construct: Construct,
    },
}

/// The call boundary into QSL's protocol analysis. A request carries the
/// claim and nothing else: there is no parameter through which this library
/// could pass a bound it chose.
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

/// Settle each of `claims` against `boundary`: exactly one settlement per
/// claim, in order, none derived from another claim's.
pub fn settle_claims<C, R, P: Clone>(
    boundary: &mut impl QslBoundary<C, R>,
    claims: &[(C, P)],
) -> Vec<ClaimSettlement<R, P>> {
    claims
        .iter()
        .map(|(claim, premises)| match boundary.report(claim) {
            QslReport::Result(result) => ClaimSettlement::Result {
                result,
                premises: premises.clone(),
            },
            QslReport::Unsupported { cause, construct } => {
                ClaimSettlement::Disposition(Disposition::Unsupported(UnsupportedDisposition {
                    cause,
                    construct,
                    premises: premises.clone(),
                }))
            }
            QslReport::RequiresBound { outcome, construct } => {
                ClaimSettlement::Disposition(Disposition::RequiresBound(RequiresBoundDisposition {
                    outcome,
                    construct,
                    premises: premises.clone(),
                }))
            }
        })
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
/// the request's premises.
#[must_use]
pub fn request_discharge<P>(
    declaration: &LoopDeclaration,
    construct: Construct,
    premises: P,
) -> UnsupportedDisposition<P> {
    debug_assert_eq!(declaration.control, construct.node);
    UnsupportedDisposition {
        cause: UnsupportedCause::RequestedCapability,
        construct,
        premises,
    }
}
