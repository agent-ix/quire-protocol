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

/// Discriminating `quire_spec_language::Error` retention. Owner chunk: **C-05**.
///
/// Retains A's complete typed error and stable code, and adds C-04's selection
/// axis where A deliberately collapses several selection failures. Converting
/// that value into `ProtocolRefusal` remains gated on the selected diagnostic
/// catalog and typed-cause contract owned upstream; this module does not invent
/// either one.
pub mod map;
