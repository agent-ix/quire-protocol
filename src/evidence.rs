// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-046: temporal evidence from the checked package's own nodes.
//!
//! The call is QSL's: [`EvidencePackage`] built from a [`BarePackage`]
//! (QSL FR-532) evaluates an [`EvidenceRequest`] and answers an
//! [`EvidenceResult`]. This module reads that result into an
//! [`EvidenceRecord`] and carries every refusal as QSL returned it: a
//! construction refusal ([`PackageMismatch`]) before any call, and a call
//! refusal ([`CallFailure`]) with its cause name unchanged. Nothing is
//! reinterpreted, renamed or merged, and the request and result are
//! in-memory values with no document and no contract identifier.

use qsl_eval::temporal::evidence::{
    ActivationState, EvidencePackage, EvidenceRequest, EvidenceResult, Indeterminate, NonValue,
    PackageMismatch, Settlement, SupportRef, Truth,
};
use qsl_package::BarePackage;
use quire_contract_model::CheckedNodeId;
use quire_eval::CallFailure;
use quire_exact::Meter;

/// What an evidence result says of one temporal obligation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceRecord {
    /// Whether the obligation was activated.
    pub activation: ActivationState,
    /// The truth, absent when the obligation has none.
    pub truth: Option<Truth>,
    /// The settlement, absent when the obligation has none.
    pub settlement: Option<Settlement>,
    /// Why there is no value: inactive, unknown, missing, refused.
    pub non_value: Option<NonValue>,
    /// What the evidence leaves indeterminate.
    pub indeterminate: Option<Indeterminate>,
    /// The observations and atoms that support the decision.
    pub support: Vec<SupportRef>,
}

impl From<&EvidenceResult> for EvidenceRecord {
    fn from(result: &EvidenceResult) -> Self {
        Self {
            activation: result.activation,
            truth: result.truth,
            settlement: result.settlement,
            non_value: result.non_value.clone(),
            indeterminate: result.indeterminate.clone(),
            support: result.support.clone(),
        }
    }
}

/// Why no record was produced.
#[derive(Debug)]
pub enum EvidenceRefusal {
    /// The package did not construct; no call was made and nothing was
    /// charged.
    Construction(PackageMismatch),
    /// QSL's call refusal, unchanged.
    Call(CallFailure),
}

/// The evidence package of `bare`, built from its nodes alone.
///
/// # Errors
///
/// [`EvidenceRefusal::Construction`] with QSL's [`PackageMismatch`].
pub fn package<'p>(bare: &'p BarePackage<'p>) -> Result<EvidencePackage<'p>, EvidenceRefusal> {
    EvidencePackage::from_package(bare).map_err(EvidenceRefusal::Construction)
}

/// Assess the temporal clause `clause` of `package` over `request`.
///
/// # Errors
///
/// [`EvidenceRefusal::Call`] with QSL's [`CallFailure`], charging nothing
/// for an admission refusal.
pub fn assess(
    package: &EvidencePackage<'_>,
    clause: &CheckedNodeId,
    request: &EvidenceRequest,
    meter: &mut Meter,
) -> Result<EvidenceRecord, EvidenceRefusal> {
    package
        .evaluate_temporal_evidence(clause, request, meter)
        .map(|result| EvidenceRecord::from(&result))
        .map_err(EvidenceRefusal::Call)
}
