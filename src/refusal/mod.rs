// SPDX-License-Identifier: AGPL-3.0-or-later
//! Refusal envelope facade. Owner chunk: **C-01**.
//!
//! C-01 lands the B-owned `ProtocolRefusal` envelope here: refusal domain
//! (`admission | representation | assessment | result | projection`), the
//! source-owned stable code, the affected identity, an explicitly present or
//! explicitly absent location, the originating stage and its prerequisites, an
//! ordered cause chain, and an explicit partial-output state.
//!
//! Per COLLISION-5 this file owns only the envelope and the domain enum. Later
//! chunks add their own code enums in their own files below; they never edit
//! this module.

/// Diagnostic-catalog selection and `unsupported-source-contract`. Owner chunk: **C-02**.
///
/// Will own validation of a refusal code against the exact selected versioned
/// catalog, retaining catalog identity, revision, digest and the original
/// uninterpreted value rather than interpreting through a nearest known version.
pub mod catalog;

/// Typed cause taxonomy, declared cause ordering, partial-output policy. Owner chunk: **C-03**.
///
/// Will own the five structurally distinct cause pairs, declared (not sorted)
/// cause order, and the rule that a fatal refusal emits no success value.
pub mod cause;
