// SPDX-License-Identifier: AGPL-3.0-or-later
//! Canonical protocol result record. Owner chunk: **C-26**.
//!
//! C-26 lands truth, adequacy, the five closure axes, settlement, support, claim
//! dispositions, limitations and provenance as orthogonal typed fields.
//! COLLISION-9: C-27 and C-28 extend this record strictly serially.

/// Global-closure premise requirements. Owner chunk: **C-27**.
///
/// Will own the rule that global claims carry a complete-global-conformance
/// closure plus exact execution, branch and workflow premise identities, while
/// non-global claims carry `not-required` with no global premises.
pub mod global;

/// Support lineage validation. Owner chunk: **C-28**.
///
/// Will own distinct refusals for missing predecessors, cycles, self-reference
/// and invented independence, and the immutability of supersession: a later
/// contradiction emits a new linked result and never reseals the earlier one.
pub mod lineage;
