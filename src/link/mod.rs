// SPDX-License-Identifier: AGPL-3.0-or-later
//! Finite linked-protocol graph builder. Owner chunk: **C-12**.
//!
//! C-12 projects A's already-validated `wire::Package` into B-owned newtyped
//! nodes with no input-order or display-text identity. B re-decodes nothing.

/// Located construction-fatal diagnostics. Owner chunk: **C-13**.
///
/// Will own accumulation of independent located diagnostics and the rule that
/// any construction-fatal refusal yields no linked representation at all, with
/// one distinct code per named mutation class.
pub mod diagnostics;
