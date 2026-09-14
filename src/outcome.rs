// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-014/FR-040: closed operation and claim outcome algebras.

use crate::bounds::inventory::ResourceIncomplete;
use crate::refusal::ProtocolRefusal;

/// One public Protocol operation outcome.
///
/// Budget exhaustion is not a refusal, and a fatal refusal cannot carry a
/// success value. Keeping the three states as disjoint variants makes both
/// rules structural rather than conventions at call sites.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OperationOutcome<T> {
    /// The operation completed and returned its declared value.
    Completed(T),
    /// The operation rejected its input with one validated typed refusal.
    Refused(ProtocolRefusal),
    /// The operation made no admissibility claim because a declared resource
    /// bound stopped it.
    ResourceIncomplete(ResourceIncomplete),
}

impl<T> OperationOutcome<T> {
    /// Map a completed value without changing either non-success state.
    #[must_use]
    pub fn map<U>(self, map: impl FnOnce(T) -> U) -> OperationOutcome<U> {
        match self {
            Self::Completed(value) => OperationOutcome::Completed(map(value)),
            Self::Refused(refusal) => OperationOutcome::Refused(refusal),
            Self::ResourceIncomplete(incomplete) => {
                OperationOutcome::ResourceIncomplete(incomplete)
            }
        }
    }
}

/// One admitted claim's independently retained construction outcome.
///
/// The row type is generic because the canonical row is owned by the result
/// subsystem. The algebra itself is cross-cutting: a claim has exactly one row
/// or one claim-local refusal, never both, and a surrounding collection keeps
/// sibling outcomes independent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClaimOutcome<Row> {
    /// One canonical FR-241 claim row.
    Canonical(Row),
    /// No valid row could be constructed for this claim.
    Refused(ProtocolRefusal),
}

impl<Row> ClaimOutcome<Row> {
    /// The canonical row, when this claim produced one.
    #[must_use]
    pub const fn canonical(&self) -> Option<&Row> {
        match self {
            Self::Canonical(row) => Some(row),
            Self::Refused(_) => None,
        }
    }

    /// The claim-local refusal, when no canonical row exists.
    #[must_use]
    pub const fn refusal(&self) -> Option<&ProtocolRefusal> {
        match self {
            Self::Canonical(_) => None,
            Self::Refused(refusal) => Some(refusal),
        }
    }
}
