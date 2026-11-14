// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-150, FR-152, FR-153, FR-302: the provider that runs QSL's own
//! `protocol-analysis` engine behind the dispatch boundary.
//!
//! The engine, its descriptor and its result are QSL's (ADR-036). This module
//! adds no decision procedure: it builds the engine's `analyze` item from the
//! request the caller supplies, runs it on QSL's linked engines, and returns
//! the engine's record unchanged, so the dispatcher carries the witness as the
//! engine wrote it. A report QSL gives no result for stays the typed
//! `unsupported` path of FR-153.

use qsl_analyze::analyze::{
    analyze_items, AnalyzeRecord, Disposed, DispositionKind, Engines, Settlement,
};
use qsl_analyze::protocol_analysis::manifest::descriptor;
use qsl_analyze::protocol_analysis::{
    projection_item, realizability_item, ProjectionRequest, ProtocolAnalysisArtifact,
    RealizabilityRequest,
};
use qsl_foundation::RequestIndex;
use qsl_replay::protocol_analysis::{
    AnalysisOutcome, ProtocolAnalysisSettlement, RealizabilityCounterexample,
};
use qsl_replay::{
    InconclusiveCause, SeparatingWitnessRecord, SettlementBasis, TerminalValue, Truth,
};
use qsl_route::{BackendDescriptor, ProviderOrigin, RegistrationRefusal};
use quire_contract_model::CheckedPackageV2;
use quire_exact::Cancel;

use crate::dispatch::{Answer, BackendResult, Binding, Claim};
use crate::disposition::{QslBoundary, QslReport, UnsupportedCause};

/// The request the caller states for one claim, in the engine's own request
/// type. The key members and the run budget are the caller's to state; QSL
/// refuses an absent member (FR-572).
pub enum EngineRequest<'p> {
    /// A `local-projection` request.
    Projection(Box<ProjectionRequest>),
    /// A `realizability` request.
    Realizability(Box<RealizabilityRequest<'p>>),
}

/// QSL's engine descriptor, to register at the
/// [`ProviderBoundary`](crate::dispatch::ProviderBoundary) beside
/// [`QslEngine`].
///
/// # Errors
///
/// QSL's registry refusal of the engine's own manifest.
pub fn engine_descriptor(origin: ProviderOrigin) -> Result<BackendDescriptor, RegistrationRefusal> {
    descriptor(origin)
}

/// The result of a run of QSL's protocol analysis engine: QSL's `analyze`
/// record, unchanged, and the FR-573 settlement read from its artifact.
#[derive(Debug)]
pub struct EngineResult {
    record: Box<AnalyzeRecord>,
    settlement: ProtocolAnalysisSettlement,
}

impl EngineResult {
    fn of(record: AnalyzeRecord) -> Option<Self> {
        let settlement = record.protocol_analysis.as_deref()?.settlement();
        Some(Self {
            record: Box::new(record),
            settlement,
        })
    }

    /// QSL's record, as the engine returned it.
    #[must_use]
    pub fn record(&self) -> &AnalyzeRecord {
        &self.record
    }

    /// QSL's result artifact: the key and the outcome.
    #[must_use]
    pub fn artifact(&self) -> Option<&ProtocolAnalysisArtifact> {
        self.record.protocol_analysis.as_deref()
    }

    /// QSL's FR-573 settlement of the result.
    #[must_use]
    pub const fn settlement(&self) -> &ProtocolAnalysisSettlement {
        &self.settlement
    }
}

impl BackendResult for EngineResult {
    fn truth(&self) -> Truth {
        self.settlement.truth
    }

    fn basis(&self) -> SettlementBasis {
        self.settlement.basis
    }

    fn witness(&self) -> Option<&SeparatingWitnessRecord> {
        match self.artifact()? {
            ProtocolAnalysisArtifact::Projection(result) => match &result.outcome {
                AnalysisOutcome::Refuted(found) => Some(&found.separating_witness),
                _ => None,
            },
            ProtocolAnalysisArtifact::Realizability(result) => match &result.outcome {
                AnalysisOutcome::Refuted(
                    RealizabilityCounterexample::IndistinguishableHistories {
                        separating_witness,
                        ..
                    }
                    | RealizabilityCounterexample::EnvironmentDefeat {
                        separating_witness, ..
                    },
                ) => Some(separating_witness),
                _ => None,
            },
        }
    }

    fn inconclusive(&self) -> Option<&InconclusiveCause> {
        match &self.settlement.value {
            TerminalValue::Inconclusive(cause) => Some(cause),
            _ => None,
        }
    }
}

/// QSL's protocol analysis engine as a provider. `supply` states the engine's
/// request for each bound claim; the engine decides it.
pub struct QslEngine<'p, S> {
    engines: Engines,
    cancel: Cancel,
    package: &'p CheckedPackageV2,
    supply: S,
}

impl<'p, S> QslEngine<'p, S> {
    /// The provider over `package`, running QSL's linked engines.
    ///
    /// # Errors
    ///
    /// QSL's registry refusal of an engine's own manifest.
    pub fn new(package: &'p CheckedPackageV2, supply: S) -> Result<Self, RegistrationRefusal> {
        Ok(Self {
            engines: Engines::qsl()?,
            cancel: Cancel::new(),
            package,
            supply,
        })
    }
}

/// A report QSL gives no result for.
fn without_result<R>(record: &AnalyzeRecord) -> QslReport<R> {
    match &record.settlement {
        Settlement::Disposed(Disposed {
            kind: DispositionKind::Unsupported,
            ..
        }) => QslReport::Unsupported {
            cause: UnsupportedCause::requested_capability(),
            construct: None,
        },
        Settlement::Disposed(Disposed { diagnostic, .. }) => {
            QslReport::Failed(diagnostic.as_ref().map(|found| found.code))
        }
        Settlement::Terminal(terminal) => match terminal.value() {
            TerminalValue::Declined { .. } => terminal
                .diagnostic()
                .map_or(QslReport::Failed(None), QslReport::Refusal),
            _ => QslReport::Failed(terminal.diagnostic()),
        },
    }
}

impl<'p, A, C, I, T, Q, S> QslBoundary<Claim<A, C, I, T>, Answer<EngineResult, Q>>
    for QslEngine<'p, S>
where
    S: FnMut(&Binding<A, C, I>) -> EngineRequest<'p>,
{
    fn report(&mut self, claim: &Claim<A, C, I, T>) -> QslReport<Answer<EngineResult, Q>> {
        let Claim::Protocol(binding) = claim else {
            // Composition and refinement are not this engine's kinds.
            return QslReport::Unsupported {
                cause: UnsupportedCause::requested_capability(),
                construct: None,
            };
        };
        let index = RequestIndex::new(0);
        let item = match (self.supply)(binding) {
            EngineRequest::Projection(request) => projection_item(index, self.package, *request),
            EngineRequest::Realizability(request) => {
                realizability_item(index, self.package, *request)
            }
        };
        let mut records = analyze_items(&self.engines, vec![item], &self.cancel);
        let Some(record) = records.pop() else {
            return QslReport::Failed(None);
        };
        if record.protocol_analysis.is_none() {
            return without_result(&record);
        }
        match EngineResult::of(record) {
            Some(result) => QslReport::Result(Answer::Protocol(result)),
            None => QslReport::Failed(None),
        }
    }
}
