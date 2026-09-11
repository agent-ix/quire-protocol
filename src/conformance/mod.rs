// SPDX-License-Identifier: AGPL-3.0-or-later
//! Conformance-evaluation facade. Owner chunk: **C-00** (facade only).

/// Batch evaluator and violation classes. Owner chunk: **C-20**.
///
/// Will own unexpected, missing, causally misordered, duplicate-effect,
/// cross-workflow and compensation violations as six distinct codes, each
/// naming the exact affected obligation.
pub mod batch;

/// Settled-incremental evaluator. Owner chunk: **C-21**.
///
/// Will own the deterministic, injected-scheduler equivalence between batch
/// replay and settled incremental assessment, with no wall-clock premise.
pub mod incremental;
