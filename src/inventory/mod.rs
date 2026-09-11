// SPDX-License-Identifier: AGPL-3.0-or-later
//! Independently selected `Expected` inventory. Owner chunk: **C-04**.
//!
//! C-04 lands B's own independently selected inventory so that a selection
//! refusal can be attributed to a single axis instead of inheriting A's
//! collapsed `Invalid::Selection`.

/// Per-axis selection discriminator. Owner chunk: **C-04**.
///
/// Will own one B-owned code per axis: contract version, producer identity,
/// source revision, baseline-selection state, baseline digest, dependency
/// closure, profile identity, required feature, resealed bytes.
pub mod axis;

/// `quire_spec_language` `Error` to `ProtocolRefusal` mapping. Owner chunk: **C-05**.
///
/// Will own a total (catch-all-free) mapping carrying A's stage, A's locus or an
/// explicit `Unavailable`, the C-04 axis code, and A's original value retained
/// uninterpreted.
pub mod map;
