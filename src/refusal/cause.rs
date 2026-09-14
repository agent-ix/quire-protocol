// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-038/FR-039: closed local causes and exact retained source interiors.

use core::fmt;

use super::catalog::RetainedBytes;
use super::{CauseDetail, SourceLocation};

/// Stable member identity for the closed `quire.protocol/refusal/1` cause set.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ProtocolCauseKind {
    /// A required definition is absent.
    MissingDefinition,
    /// Two definitions claim one identity with conflicting content.
    ConflictingDefinition,
    /// An observation or value is attached to the wrong anchor.
    WrongAnchor,
    /// A required observation is absent.
    MissingObservation,
    /// Definedness was required but not established.
    UnprovedDefinedness,
    /// A defined expression or obligation evaluated adversely.
    LogicalViolation,
    /// Two exact identities that must agree differ.
    IdentityMismatch,
    /// Required evidence is absent or incomplete.
    IncompleteEvidence,
    /// The requested capability is not supported by the selected contract.
    UnsupportedCapability,
    /// Projection would discard a required finding.
    ProjectionFindingLoss,
}

impl ProtocolCauseKind {
    /// Every local cause member, in contract order.
    pub const ALL: [Self; 10] = [
        Self::MissingDefinition,
        Self::ConflictingDefinition,
        Self::WrongAnchor,
        Self::MissingObservation,
        Self::UnprovedDefinedness,
        Self::LogicalViolation,
        Self::IdentityMismatch,
        Self::IncompleteEvidence,
        Self::UnsupportedCapability,
        Self::ProjectionFindingLoss,
    ];

    /// Stable spelling under `quire.protocol/refusal/1`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MissingDefinition => "missing-definition",
            Self::ConflictingDefinition => "conflicting-definition",
            Self::WrongAnchor => "wrong-anchor",
            Self::MissingObservation => "missing-observation",
            Self::UnprovedDefinedness => "unproved-definedness",
            Self::LogicalViolation => "logical-violation",
            Self::IdentityMismatch => "identity-mismatch",
            Self::IncompleteEvidence => "incomplete-evidence",
            Self::UnsupportedCapability => "unsupported-capability",
            Self::ProjectionFindingLoss => "projection-finding-loss",
        }
    }
}

/// One Protocol-owned cause with a member-specific typed payload.
///
/// There is no generic `(code, text)` constructor. Each member states which
/// exact identities its owning operation must retain.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum ProtocolCause {
    /// A required definition is absent.
    MissingDefinition { definition: RetainedBytes },
    /// Two definitions claim one identity with conflicting content.
    ConflictingDefinition {
        definition: RetainedBytes,
        conflicting_definition: RetainedBytes,
    },
    /// An observation or runtime value names the wrong anchor.
    WrongAnchor {
        expected: RetainedBytes,
        observed: RetainedBytes,
    },
    /// A required observation is absent.
    MissingObservation { observation: RetainedBytes },
    /// Definedness was required but not established.
    UnprovedDefinedness { expression: RetainedBytes },
    /// A defined obligation evaluated adversely.
    LogicalViolation { obligation: RetainedBytes },
    /// Two exact identities that must agree differ.
    IdentityMismatch {
        expected: RetainedBytes,
        observed: RetainedBytes,
    },
    /// Required evidence is absent or incomplete.
    IncompleteEvidence { evidence: RetainedBytes },
    /// The selected contract does not support a requested capability.
    UnsupportedCapability { capability: RetainedBytes },
    /// Projection would omit a required finding.
    ProjectionFindingLoss { finding: RetainedBytes },
}

impl ProtocolCause {
    /// Closed member identity independent of payload and rendering.
    #[must_use]
    pub const fn kind(&self) -> ProtocolCauseKind {
        match self {
            Self::MissingDefinition { .. } => ProtocolCauseKind::MissingDefinition,
            Self::ConflictingDefinition { .. } => ProtocolCauseKind::ConflictingDefinition,
            Self::WrongAnchor { .. } => ProtocolCauseKind::WrongAnchor,
            Self::MissingObservation { .. } => ProtocolCauseKind::MissingObservation,
            Self::UnprovedDefinedness { .. } => ProtocolCauseKind::UnprovedDefinedness,
            Self::LogicalViolation { .. } => ProtocolCauseKind::LogicalViolation,
            Self::IdentityMismatch { .. } => ProtocolCauseKind::IdentityMismatch,
            Self::IncompleteEvidence { .. } => ProtocolCauseKind::IncompleteEvidence,
            Self::UnsupportedCapability { .. } => ProtocolCauseKind::UnsupportedCapability,
            Self::ProjectionFindingLoss { .. } => ProtocolCauseKind::ProjectionFindingLoss,
        }
    }
}

/// Exact selected source contract for a retained source-owned cause.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct SourceContract {
    identity: RetainedBytes,
    revision: RetainedBytes,
}

impl SourceContract {
    /// Retain a non-empty source contract identity and revision byte-exactly.
    ///
    /// # Errors
    ///
    /// Returns which required field was empty. No default or nearest version
    /// is selected.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "Task-013 and Tasks 002-006 construct source capabilities inside Protocol"
        )
    )]
    pub(crate) fn new(identity: &[u8], revision: &[u8]) -> Result<Self, EmptySourceContractField> {
        if identity.is_empty() {
            return Err(EmptySourceContractField::Identity);
        }
        if revision.is_empty() {
            return Err(EmptySourceContractField::Revision);
        }
        Ok(Self {
            identity: RetainedBytes::new(identity),
            revision: RetainedBytes::new(revision),
        })
    }

    /// Exact source contract identity bytes.
    #[must_use]
    pub const fn identity(&self) -> &RetainedBytes {
        &self.identity
    }

    /// Exact source contract revision bytes.
    #[must_use]
    pub const fn revision(&self) -> &RetainedBytes {
        &self.revision
    }
}

/// Required source-contract field that was empty.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub(crate) enum EmptySourceContractField {
    /// Contract identity.
    Identity,
    /// Contract revision.
    Revision,
}

impl fmt::Display for EmptySourceContractField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Identity => formatter.write_str("source contract identity is empty"),
            Self::Revision => formatter.write_str("source contract revision is empty"),
        }
    }
}

impl std::error::Error for EmptySourceContractField {}

/// A source-owned field, either retained exactly or explicitly unavailable.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum SourceField<T> {
    /// The selected source contract supplied this value.
    Available(T),
    /// The selected source contract supplies no value for this field.
    Unavailable,
}

/// One exact source-owned cause under its selected contract.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct SourceCause {
    contract: SourceContract,
    code: RetainedBytes,
    stage: SourceField<RetainedBytes>,
    locus: SourceField<SourceLocation>,
    payload: CauseDetail,
}

impl SourceCause {
    /// Retain a source cause only when the offered contract exactly equals the
    /// independently selected contract.
    ///
    /// # Errors
    ///
    /// Returns both contracts when identity or revision differs. The code is
    /// not interpreted before this comparison.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "Tasks 002-006 retain known source causes through exact owner adapters"
        )
    )]
    pub(crate) fn retain(
        selected: &SourceContract,
        offered: SourceContract,
        code: &[u8],
        stage: SourceField<RetainedBytes>,
        locus: SourceField<SourceLocation>,
        payload: CauseDetail,
    ) -> Result<Self, SourceContractMismatch> {
        if selected != &offered {
            return Err(SourceContractMismatch {
                selected: selected.clone(),
                offered,
            });
        }
        Ok(Self {
            contract: offered,
            code: RetainedBytes::new(code),
            stage,
            locus,
            payload,
        })
    }

    /// Exact selected source contract.
    #[must_use]
    pub const fn contract(&self) -> &SourceContract {
        &self.contract
    }

    /// Stable source-owned code bytes, uninterpreted by Protocol.
    #[must_use]
    pub const fn code(&self) -> &RetainedBytes {
        &self.code
    }

    /// Source-owned stage when supplied.
    #[must_use]
    pub const fn stage(&self) -> &SourceField<RetainedBytes> {
        &self.stage
    }

    /// Source-owned locus when supplied.
    #[must_use]
    pub const fn locus(&self) -> &SourceField<SourceLocation> {
        &self.locus
    }

    /// Opaque source-owned payload or its explicit absence state.
    #[must_use]
    pub const fn payload(&self) -> &CauseDetail {
        &self.payload
    }
}

/// The offered source contract differs from the independently selected one.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub(crate) struct SourceContractMismatch {
    /// Independently selected contract.
    pub(crate) selected: SourceContract,
    /// Contract carried by the offered source response.
    pub(crate) offered: SourceContract,
}

impl fmt::Display for SourceContractMismatch {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("offered source contract differs from the selected contract")
    }
}

impl std::error::Error for SourceContractMismatch {}

/// Unknown source contract, revision, or code retained without interpretation.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct UnsupportedSourceContract {
    contract: SourceContract,
    code: RetainedBytes,
    stage: SourceField<RetainedBytes>,
    locus: SourceField<SourceLocation>,
    payload: CauseDetail,
}

impl UnsupportedSourceContract {
    /// Retain the original unknown value byte-exactly.
    #[must_use]
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "Tasks 002-006 retain unknown source causes without interpretation"
        )
    )]
    pub(crate) fn retain(
        contract: SourceContract,
        code: &[u8],
        stage: SourceField<RetainedBytes>,
        locus: SourceField<SourceLocation>,
        payload: CauseDetail,
    ) -> Self {
        Self {
            contract,
            code: RetainedBytes::new(code),
            stage,
            locus,
            payload,
        }
    }

    /// Original unknown contract.
    #[must_use]
    pub const fn contract(&self) -> &SourceContract {
        &self.contract
    }

    /// Original unknown stable-code bytes.
    #[must_use]
    pub const fn code(&self) -> &RetainedBytes {
        &self.code
    }

    /// Original source stage when supplied.
    #[must_use]
    pub const fn stage(&self) -> &SourceField<RetainedBytes> {
        &self.stage
    }

    /// Original source locus when supplied.
    #[must_use]
    pub const fn locus(&self) -> &SourceField<SourceLocation> {
        &self.locus
    }

    /// Original opaque payload or explicit absence state.
    #[must_use]
    pub const fn payload(&self) -> &CauseDetail {
        &self.payload
    }
}

/// One refusal cause. The three namespaces cannot be substituted by type.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum RefusalCause {
    /// Protocol-owned `quire.protocol/refusal/1` cause.
    Protocol(ProtocolCause),
    /// Exact source-owned cause under a selected known contract.
    Source(SourceCause),
    /// Unknown source value retained under the local unsupported disposition.
    UnsupportedSourceContract(UnsupportedSourceContract),
}

impl From<ProtocolCause> for RefusalCause {
    fn from(cause: ProtocolCause) -> Self {
        Self::Protocol(cause)
    }
}

impl From<SourceCause> for RefusalCause {
    fn from(cause: SourceCause) -> Self {
        Self::Source(cause)
    }
}

impl From<UnsupportedSourceContract> for RefusalCause {
    fn from(cause: UnsupportedSourceContract) -> Self {
        Self::UnsupportedSourceContract(cause)
    }
}
