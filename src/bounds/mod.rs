// SPDX-License-Identifier: AGPL-3.0-or-later
//! Count, allocation and deadline guards. Owner chunk: **C-29**.
//!
//! C-29 lands prechecked counts and accounted allocations as a mechanism
//! separate from monotonic deadline observation, sharing no refusal code. B
//! reconciles against A's `Limits`, `Usage`, `Dimension` and `Exhaustion`; it
//! does not re-implement A's accounting.

/// Resource-site inventory and mutant-kill assignment. Owner chunk: **C-30**.
///
/// Will own the bidirectional reconciliation between caller-controlled
/// allocation, retention, queue, iteration and traversal sites and their
/// inventory entries.
pub mod inventory;
