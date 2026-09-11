// SPDX-License-Identifier: AGPL-3.0-or-later
//! Compensation facade. Owner chunk: **C-00** (facade only).
//!
//! Four chunks share this directory on disjoint files (COLLISION-8): C-16 and
//! C-17 are parallel-safe; C-18 and C-19 are strictly serial after them.

/// Registration, retry and commit algebra. Owner chunk: **C-16**.
///
/// Will own the registration laws over C-07 identities: one paired successful
/// effect registers exactly one compensation after the effect, replay is
/// idempotent, and the four ordering/bound violations stay distinct.
pub mod algebra;

/// Assessor purity seam. Owner chunk: **C-17**.
///
/// Will own the action-capable Rust double the assessor takes by trait, proving
/// zero network, storage, telemetry, business-operation and
/// compensation-operation calls across every assessment boundary case.
pub mod purity;

/// Trigger, capture and anchor activation. Owner chunk: **C-18**.
///
/// Will own `triggered | inactive | activation-unknown` over the linked graph,
/// six identity-substitution refusals and seven missing-authority refusals.
pub mod activate;

/// Full and partial recovery relations. Owner chunk: **C-19**.
///
/// Will own the relation under which an achieved partial state satisfies an
/// authored partial recovery while failing full restoration.
pub mod recovery;
