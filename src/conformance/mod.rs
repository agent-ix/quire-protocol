// SPDX-License-Identifier: AGPL-3.0-or-later
//! Finite protocol conformance over strict QObs owner views.
//!
//! [`batch`] owns deterministic assessment of an explicit obligation model and
//! [`incremental`] owns prefix settlement plus exact comparison with batch.
//! Neither path parses source, derives causality from timestamps, repairs an
//! admitted observation, or evaluates a temporal predicate.

pub mod batch;
pub mod incremental;

use crate::refusal::catalog::{CatalogSelection, CodeEntry, CodeModule, Provenance, RefusalCode};
use crate::refusal::{
    AffectedIdentity, OriginatingStage, ProtocolRefusal, RefusalDomain, RetryDecision,
    Unavailability,
};
use crate::ByteDigest;

pub use batch::{
    assess, AdverseKind, AdverseObservation, BoundObservation, GlobalDisposition,
    GlobalPremiseDefect, GlobalPremiseDefectKind, IncompleteCause, Limits, Model,
    ObligationDisposition, ObligationId, ObligationResult, ObservationObligation, Report, Request,
    Usage,
};
pub use incremental::{compare, Incremental};

/// Stable conformance refusal cause.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ErrorCode {
    /// The explicit obligation model is incomplete, duplicated, or cyclic.
    InvalidModel,
    /// An observation binding names no exact obligation.
    InvalidBinding,
    /// Producer-owner axes or global premise sets are cross-wired.
    InvalidPremises,
    /// A configured conformance resource ceiling was exceeded.
    ResourceIncomplete,
    /// A settled incremental result changed without an explicit correction.
    SettlementReopened,
    /// Batch and incremental reports differ for the same exact request.
    PathDivergence,
}

impl ErrorCode {
    /// Stable machine-readable code.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.entry().code
    }

    const fn entry(self) -> &'static CodeEntry {
        match self {
            Self::InvalidModel => &INVALID_MODEL,
            Self::InvalidBinding => &INVALID_BINDING,
            Self::InvalidPremises => &INVALID_PREMISES,
            Self::ResourceIncomplete => &RESOURCE_INCOMPLETE,
            Self::SettlementReopened => &SETTLEMENT_REOPENED,
            Self::PathDivergence => &PATH_DIVERGENCE,
        }
    }
}

/// Typed failure from batch or incremental conformance.
#[derive(Clone, Debug)]
pub struct Refusal {
    code: ErrorCode,
    detail: &'static str,
    outcome_identity: String,
    usage: Usage,
    envelope: Box<ProtocolRefusal>,
}

impl Refusal {
    pub(crate) fn new(
        code: ErrorCode,
        detail: &'static str,
        affected: Option<&ObligationId>,
        usage: Usage,
        discriminator: &[u8],
    ) -> Self {
        let mut preimage = Vec::with_capacity(code.as_str().len() + discriminator.len() + 1);
        preimage.extend_from_slice(code.as_str().as_bytes());
        preimage.push(0);
        preimage.extend_from_slice(discriminator);
        let outcome_identity = format!("sha256:{:x}", ByteDigest::of(&preimage));
        let mut builder = ProtocolRefusal::builder(
            RefusalCode::of(code.entry()),
            CatalogSelection::local(),
            OriginatingStage::ResourceExecution,
        )
        .unavailable_location(Unavailability::NotApplicableToStage);
        if let Some(identity) = affected {
            builder = builder.identity(AffectedIdentity::retained(identity.as_str().as_bytes()));
        }
        Self {
            code,
            detail,
            outcome_identity,
            usage,
            envelope: Box::new(builder.emitting_nothing()),
        }
    }

    /// Stable typed cause.
    #[must_use]
    pub const fn code(&self) -> ErrorCode {
        self.code
    }

    /// Bounded human detail. Callers branch on [`Self::code`].
    #[must_use]
    pub const fn detail(&self) -> &'static str {
        self.detail
    }

    /// Derived identity of this exact negative outcome.
    #[must_use]
    pub fn outcome_identity(&self) -> &str {
        &self.outcome_identity
    }

    /// Work observed before refusal.
    #[must_use]
    pub const fn usage(&self) -> Usage {
        self.usage
    }

    /// Complete fatal-refusal envelope. No partial report accompanies it.
    #[must_use]
    pub fn envelope(&self) -> &ProtocolRefusal {
        self.envelope.as_ref()
    }
}

impl core::fmt::Display for Refusal {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "{}: {}", self.code.as_str(), self.detail)
    }
}

impl std::error::Error for Refusal {}

/// Conformance return type.
pub type Result<T> = std::result::Result<T, Refusal>;

const INVALID_MODEL: CodeEntry = CodeEntry {
    provenance: Provenance::Minted,
    code: "conformance.invalid-model",
    domain: RefusalDomain::Assessment,
    retry: RetryDecision::AfterSelectionChange,
    summary: "the exact obligation model is incomplete, duplicated, or cyclic",
};
const INVALID_BINDING: CodeEntry = CodeEntry {
    provenance: Provenance::Minted,
    code: "conformance.invalid-binding",
    domain: RefusalDomain::Assessment,
    retry: RetryDecision::AfterSelectionChange,
    summary: "an observation binding names no exact obligation",
};
const INVALID_PREMISES: CodeEntry = CodeEntry {
    provenance: Provenance::Minted,
    code: "conformance.invalid-premises",
    domain: RefusalDomain::Assessment,
    retry: RetryDecision::AfterSelectionChange,
    summary: "producer-owner axes or global premise sets are cross-wired",
};
const RESOURCE_INCOMPLETE: CodeEntry = CodeEntry {
    provenance: Provenance::Minted,
    code: "conformance.resource-incomplete",
    domain: RefusalDomain::Assessment,
    retry: RetryDecision::AfterResourceRelease,
    summary: "a conformance resource ceiling was exceeded",
};
const SETTLEMENT_REOPENED: CodeEntry = CodeEntry {
    provenance: Provenance::Minted,
    code: "conformance.settlement-reopened",
    domain: RefusalDomain::Assessment,
    retry: RetryDecision::Never,
    summary: "a settled prefix was changed without an explicit correction",
};
const PATH_DIVERGENCE: CodeEntry = CodeEntry {
    provenance: Provenance::Minted,
    code: "conformance.path-divergence",
    domain: RefusalDomain::Assessment,
    retry: RetryDecision::Never,
    summary: "batch and incremental paths disagree for one exact request",
};

/// Conformance codes registered with the crate-wide refusal catalog.
pub const CODES: CodeModule = CodeModule {
    root: "conformance",
    owner: "C-20/C-21",
    module_path: "quire_protocol::conformance",
    entries: &[
        INVALID_MODEL,
        INVALID_BINDING,
        INVALID_PREMISES,
        RESOURCE_INCOMPLETE,
        SETTLEMENT_REOPENED,
        PATH_DIVERGENCE,
    ],
};
