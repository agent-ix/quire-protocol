// SPDX-License-Identifier: AGPL-3.0-or-later
//! Per-axis selection discriminator.
//!
//! A's reader collapses every selection divergence into the single shared code
//! `Invalid::Selection`. Three genuinely different refusal axes therefore carry
//! one code, so bypassing one guard falls through to a later failure that a
//! test cannot tell apart. B holds its own independently selected inventory, so
//! on a selection refusal it can name the axis that actually diverged.

use std::fmt;

/// The single selected component that diverged, named by this crate's own
/// inventory after a refusal.
///
/// Every variant is produced by exactly one comparison, so no two of them can
/// share a value.
///
/// This is a per-component, post-refusal discriminator. It is not vocabulary
/// the producer or core publishes: the version 1 producer collapses all of this
/// into one `Invalid::Selection`, and this enum is a local refinement of that
/// single code, authorised by `FR-013` — "It does not type the axis" — with
/// `FR-013-AC-8` stating that adding or removing a member changes no obligation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum DivergedSelection {
    /// The offered compiled-protocol contract identity is not the selected one.
    ContractIdentity,
    /// The offered contract revision is not the selected revision.
    ContractRevision,
    /// The offered contract digest differs from the selected value.
    ContractDigest,
    /// The offered contract reference diverges outside identity/revision/digest.
    ContractReference,
    /// The offered producer implementation is not the selected implementation.
    ProducerIdentity,
    /// The offered producer revision is not the selected revision.
    ProducerRevision,
    /// The offered producer binary reference is not the selected binary.
    ProducerBinary,
    /// The offered baseline identity is not the selected semantic baseline.
    BaselineIdentity,
    /// The offered baseline is a different selection state of the same baseline
    /// identity — a superseded, draft or otherwise unselected revision.
    BaselineSelectionState,
    /// The offered baseline digest differs from the selected value.
    BaselineDigest,
    /// The offered baseline reference diverges outside identity/revision/digest.
    BaselineReference,
    /// A selection refusal B's inventory could not attribute to one axis.
    ///
    /// This is an explicit, nameable outcome, not a silent fallback: it is
    /// produced only when the offer is unreadable as JSON or omits a top-level
    /// selection member, and a dedicated test asserts no landed mutation
    /// reaches it.
    Undetermined,
}

impl DivergedSelection {
    /// Stable axis name, for diagnostics only; it is never parsed back.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ContractIdentity => "contract-identity",
            Self::ContractRevision => "contract-revision",
            Self::ContractDigest => "contract-digest",
            Self::ContractReference => "contract-reference",
            Self::ProducerIdentity => "producer-identity",
            Self::ProducerRevision => "producer-revision",
            Self::ProducerBinary => "producer-binary",
            Self::BaselineIdentity => "baseline-identity",
            Self::BaselineSelectionState => "baseline-selection-state",
            Self::BaselineDigest => "baseline-digest",
            Self::BaselineReference => "baseline-reference",
            Self::Undetermined => "undetermined",
        }
    }

    /// Every axis this discriminator can name, for coverage assertions.
    pub const ALL: [Self; 12] = [
        Self::ContractIdentity,
        Self::ContractRevision,
        Self::ContractDigest,
        Self::ContractReference,
        Self::ProducerIdentity,
        Self::ProducerRevision,
        Self::ProducerBinary,
        Self::BaselineIdentity,
        Self::BaselineSelectionState,
        Self::BaselineDigest,
        Self::BaselineReference,
        Self::Undetermined,
    ];
}

impl fmt::Display for DivergedSelection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}
