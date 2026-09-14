// SPDX-License-Identifier: AGPL-3.0-or-later
//! Canonical protocol-result owner and downstream mapping views.
//!
//! [`record`] owns typed construction and immutable result bytes, [`reader`]
//! owns the hostile-byte boundary, [`global`] validates global premise sets,
//! [`lineage`] validates support and correction edges, and [`contract_ir`]
//! owns the selected read-only mapping. No module evaluates a predicate or
//! parses Quire source.

pub mod contract_ir;
pub mod global;
pub mod lineage;
pub mod reader;
pub mod record;

use crate::refusal::catalog::{CatalogSelection, CodeEntry, CodeModule, Provenance, RefusalCode};
use crate::refusal::{
    AffectedIdentity, ProtocolRefusal, ProtocolStage, RefusalAxis, RefusalDomain, RetryDecision,
    Unavailability,
};

pub use crate::assessment::{AxisInputs, GlobalConformanceInput, GlobalPremise, PremiseKind};
pub use reader::{read, read_bounded, Expected};
pub use record::{
    produce, produce_bounded, Activation, Adequacy, Assumption, AssumptionState, AxisView, Claim,
    ClaimKind, ClaimStrength, CompletenessView, CorrectionInput, CorrectionRelation,
    CorrectionView, DecisionSupport, DependenceRelation, Document, Input, Limitation, Limits,
    Participation, ProtocolAdequacy, ProvenanceRecord, Settlement, SettlementBasis, SupportRef,
    Truth, Usage, ValidatedResult, CONTRACT, SCHEMA_BYTES, SCHEMA_SHA256,
};

/// Stable result-owner or mapping refusal cause.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ErrorCode {
    /// A caller selection is absent, malformed, duplicated, or inconsistent.
    InvalidSelection,
    /// A configured owner ceiling was exceeded.
    ResourceIncomplete,
    /// Input is not one complete closed JSON document.
    InvalidJson,
    /// Input is valid JSON but is not the one canonical byte encoding.
    NonCanonical,
    /// A document selects an unknown contract or schema.
    ContractMismatch,
    /// A derived result or mapping identity does not match.
    IdentityMismatch,
    /// An enclosing result digest does not match the canonical bytes.
    DigestMismatch,
    /// An execution/truth/settlement/activation combination is forbidden.
    InvalidCombination,
    /// Independent progress, closure, completeness, or authority facts crossed.
    AxisMismatch,
    /// The global-conformance premise population is not exact.
    GlobalPremiseMismatch,
    /// A support or correction lineage edge is missing, cyclic, or not direct.
    LineageInvalid,
    /// A mapped document differs from the view re-derived from its source.
    MappingMismatch,
}

impl ErrorCode {
    /// Stable machine-readable code.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.entry().code
    }

    const fn entry(self) -> &'static CodeEntry {
        match self {
            Self::InvalidSelection => &RESULT_INVALID_SELECTION,
            Self::ResourceIncomplete => &RESULT_RESOURCE_INCOMPLETE,
            Self::InvalidJson => &RESULT_INVALID_JSON,
            Self::NonCanonical => &RESULT_NONCANONICAL,
            Self::ContractMismatch => &RESULT_CONTRACT_MISMATCH,
            Self::IdentityMismatch => &RESULT_IDENTITY_MISMATCH,
            Self::DigestMismatch => &RESULT_DIGEST_MISMATCH,
            Self::InvalidCombination => &RESULT_INVALID_COMBINATION,
            Self::AxisMismatch => &RESULT_AXIS_MISMATCH,
            Self::GlobalPremiseMismatch => &RESULT_GLOBAL_PREMISE_MISMATCH,
            Self::LineageInvalid => &RESULT_LINEAGE_INVALID,
            Self::MappingMismatch => &contract_ir::MAPPING_MISMATCH,
        }
    }
}

/// Typed refusal returned by every result-owner and mapping operation.
#[derive(Clone, Debug)]
pub struct Refusal {
    code: ErrorCode,
    detail: &'static str,
    usage: Usage,
    envelope: ProtocolRefusal,
}

impl Refusal {
    pub(crate) fn assembly(
        code: ErrorCode,
        detail: &'static str,
        usage: Usage,
        identity: Option<&str>,
    ) -> Self {
        Self::at_stage(code, detail, usage, identity, ProtocolStage::ResultAssembly)
    }

    pub(crate) fn read(
        code: ErrorCode,
        detail: &'static str,
        usage: Usage,
        identity: Option<&str>,
    ) -> Self {
        Self::at_stage(code, detail, usage, identity, ProtocolStage::ResultRead)
    }

    pub(crate) fn at_stage(
        code: ErrorCode,
        detail: &'static str,
        usage: Usage,
        identity: Option<&str>,
        stage: ProtocolStage,
    ) -> Self {
        let refusal_code = RefusalCode::of(code.entry());
        let mut builder = ProtocolRefusal::legacy_builder(
            refusal_code,
            CatalogSelection::local(),
            stage,
            RefusalAxis::from_registered_code(refusal_code),
        )
        .expect("result codes belong to a result stage")
        .unavailable_location(Unavailability::NotApplicableToStage);
        if let Some(identity) = identity {
            builder = builder.identity(AffectedIdentity::retained(identity.as_bytes()));
        }
        Self {
            code,
            detail,
            usage,
            envelope: builder.emitting_nothing(),
        }
    }

    /// Stable typed cause.
    #[must_use]
    pub const fn code(&self) -> ErrorCode {
        self.code
    }

    /// Bounded diagnostic detail; callers branch on [`Self::code`].
    #[must_use]
    pub const fn detail(&self) -> &'static str {
        self.detail
    }

    /// Work observed before refusal. No partial document or view is exposed.
    #[must_use]
    pub const fn usage(&self) -> Usage {
        self.usage
    }

    /// Complete FR-036/FR-040 refusal envelope.
    #[must_use]
    pub fn envelope(&self) -> &ProtocolRefusal {
        &self.envelope
    }
}

impl core::fmt::Display for Refusal {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "{}: {}", self.code.as_str(), self.detail)
    }
}

impl std::error::Error for Refusal {}

/// Result-owner return type.
pub type Result<T> = std::result::Result<T, Refusal>;

macro_rules! code_entry {
    ($name:ident, $code:literal, $domain:expr, $retry:expr, $summary:literal) => {
        const $name: CodeEntry = CodeEntry {
            provenance: Provenance::Minted,
            code: $code,
            domain: $domain,
            retry: $retry,
            summary: $summary,
        };
    };
}

code_entry!(
    RESULT_INVALID_SELECTION,
    "result.invalid-selection",
    RefusalDomain::Result,
    RetryDecision::AfterSelectionChange,
    "a result input selection is absent, duplicated, or inconsistent"
);
code_entry!(
    RESULT_RESOURCE_INCOMPLETE,
    "result.resource-incomplete",
    RefusalDomain::Result,
    RetryDecision::AfterResourceRelease,
    "a result-owner resource ceiling was exceeded"
);
code_entry!(
    RESULT_INVALID_JSON,
    "result.invalid-json",
    RefusalDomain::Result,
    RetryDecision::Never,
    "result bytes are not one closed JSON document"
);
code_entry!(
    RESULT_NONCANONICAL,
    "result.noncanonical",
    RefusalDomain::Result,
    RetryDecision::Never,
    "result bytes are not the selected RFC 8785 encoding"
);
code_entry!(
    RESULT_CONTRACT_MISMATCH,
    "result.contract-mismatch",
    RefusalDomain::Result,
    RetryDecision::AfterSelectionChange,
    "the result contract or schema selection differs"
);
code_entry!(
    RESULT_IDENTITY_MISMATCH,
    "result.identity-mismatch",
    RefusalDomain::Result,
    RetryDecision::Never,
    "the derived result identity differs from the carried identity"
);
code_entry!(
    RESULT_DIGEST_MISMATCH,
    "result.digest-mismatch",
    RefusalDomain::Result,
    RetryDecision::Never,
    "the enclosing digest differs from the canonical result bytes"
);
code_entry!(
    RESULT_INVALID_COMBINATION,
    "result.invalid-combination",
    RefusalDomain::Result,
    RetryDecision::Never,
    "the retained result axes form a forbidden combination"
);
code_entry!(
    RESULT_AXIS_MISMATCH,
    "result.axis-mismatch",
    RefusalDomain::Result,
    RetryDecision::AfterSelectionChange,
    "independent owner axes are cross-wired or substituted"
);
code_entry!(
    RESULT_GLOBAL_PREMISE_MISMATCH,
    "result.global-premise-mismatch",
    RefusalDomain::Result,
    RetryDecision::AfterSelectionChange,
    "the global closure premise population is not exact"
);
code_entry!(
    RESULT_LINEAGE_INVALID,
    "result.lineage-invalid",
    RefusalDomain::Result,
    RetryDecision::AfterSelectionChange,
    "a support or correction lineage edge is invalid"
);
/// Result-owner codes registered with the crate-wide refusal catalog.
pub const RESULT_CODES: CodeModule = CodeModule {
    root: "result",
    owner: "C-26/C-27",
    module_path: "quire_protocol::result",
    entries: &[
        RESULT_INVALID_SELECTION,
        RESULT_RESOURCE_INCOMPLETE,
        RESULT_INVALID_JSON,
        RESULT_NONCANONICAL,
        RESULT_CONTRACT_MISMATCH,
        RESULT_IDENTITY_MISMATCH,
        RESULT_DIGEST_MISMATCH,
        RESULT_INVALID_COMBINATION,
        RESULT_AXIS_MISMATCH,
        RESULT_GLOBAL_PREMISE_MISMATCH,
        RESULT_LINEAGE_INVALID,
    ],
};
