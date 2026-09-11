// SPDX-License-Identifier: AGPL-3.0-or-later
//! Five-axis closure and outcome lattice. Owner chunk: **C-15**. Intentionally empty.
//!
//! C-15 lands decision-scope progress, decision-scope closure,
//! surrounding-execution progress, surrounding-execution closure and
//! complete-global-conformance closure as five independent observable axes, and
//! `open | incomplete | unsupported | failed | exhausted` as five independent
//! non-success outcomes. No axis may be computed from or collapsed into another.
