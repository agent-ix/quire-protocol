// SPDX-License-Identifier: AGPL-3.0-or-later
//! Claim disposition product type and registry. Owner chunk: **C-22**.
//!
//! C-22 lands conformance, monitorability, projection, realizability, refinement
//! and composition as six separate dispositions held simultaneously in one
//! record, each independently `supported | unsupported | failed | established`,
//! with no aggregate verdict field a caller could mistake for a result.

/// Fragment, backend and premise retention. Owner chunk: **C-23**.
///
/// Will own per-disposition retention of the exact admitted fragment, backend,
/// assumptions and communication/visibility/environment premises, such that
/// removing or substituting any one refuses only that disposition.
pub mod premises;

/// Circularity, backend matching and the no-fallback rule. Owner chunk: **C-24**.
///
/// Will own pre-dispatch rejection of circular assumptions and backend
/// identity/version mismatches as distinct codes, and the rule that a selected
/// backend failure invokes no fallback.
pub mod dispatch;
