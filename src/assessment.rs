// SPDX-License-Identifier: AGPL-3.0-or-later
//! Shared, producer-owned facts consumed by assessment and result construction.
//!
//! This module is deliberately neutral: conformance assesses these inputs and
//! the result owner serializes them, but neither subsystem owns or re-derives
//! the QObs facts or the declared global-closure premise set.

use quire_observation::authority::{closure as observation_closure, progress};

use crate::closure::GlobalConformanceClosure;

/// Required global-closure premise kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum PremiseKind {
    /// Exact execution-closure premise.
    Execution,
    /// Exact branch-closure premise.
    Branch,
    /// Exact workflow-closure premise.
    Workflow,
}

impl PremiseKind {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Execution => "execution",
            Self::Branch => "branch",
            Self::Workflow => "workflow",
        }
    }
}

/// One exact required global-closure premise.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct GlobalPremise {
    /// Premise axis.
    pub kind: PremiseKind,
    /// Exact premise identity.
    pub identity: String,
    /// Exact scope identity.
    pub scope_identity: String,
    /// Exact producer authority identity.
    pub authority_identity: String,
}

/// Global closure state plus independently selected required and carried sets.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GlobalConformanceInput {
    /// Independently supplied FR-247 state.
    pub state: GlobalConformanceClosure,
    /// Complete required premise population in authored order.
    pub required_premises: Vec<GlobalPremise>,
    /// Exact carried premise population in authored order.
    pub premises: Vec<GlobalPremise>,
}

/// Four independent QObs owner views retained without re-derivation.
#[derive(Clone, Copy, Debug)]
pub struct AxisInputs<'a> {
    /// Decision-scope progress authority view.
    pub decision_scope_progress: &'a progress::View,
    /// Decision-scope closure authority view.
    pub decision_scope_closure: &'a observation_closure::View,
    /// Surrounding-execution progress authority view.
    pub surrounding_execution_progress: &'a progress::View,
    /// Surrounding-execution closure authority view.
    pub surrounding_execution_closure: &'a observation_closure::View,
}
