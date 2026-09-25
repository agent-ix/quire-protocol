// SPDX-License-Identifier: AGPL-3.0-or-later
//! Test-only constructors for [`ValidatedResult`], behind the `test-support`
//! Cargo feature.
//!
//! Production code has no route to a [`ValidatedResult`] other than the
//! Engine-owned transition in [`crate::assessment`]. Downstream crates' test
//! suites still need results with a chosen truth, settlement and execution to
//! exercise their own mapping of a result. This module is that route, and it
//! exists only when a dependent enables `test-support`, which a dependent should
//! do for its `[dev-dependencies]` entry alone. The default build compiles none
//! of it.
//!
//! Each constructor runs the same canonical producer the Engine uses, so every
//! invariant that producer enforces is enforced here, and then strict-reads the
//! produced bytes back against the same input. A value is returned only when
//! both steps accept it; otherwise the typed [`Refusal`] is returned. Nothing
//! here builds a result from unchecked parts.

use super::reader::{read, Expected};
use super::record::{produce, CorrectionInput, Input, Limits, ValidatedResult};
use super::{ErrorCode, Refusal, Result};

/// Produce and strict-read one uncorrected canonical result from `input`.
///
/// `input.correction` must be `None`; a correcting result is built with
/// [`corrected_result`], so that a test states the correction explicitly.
///
/// # Errors
///
/// Returns [`ErrorCode::InvalidSelection`] when `input` already carries a
/// correction, and otherwise any refusal the canonical producer or the strict
/// reader returns for `input` under `limits`.
pub fn validated_result(input: Input<'_>, limits: Limits) -> Result<ValidatedResult> {
    if input.correction.is_some() {
        return Err(Refusal::assembly(
            ErrorCode::InvalidSelection,
            "test-support validated_result takes no correction; use corrected_result",
            super::Usage::default(),
            None,
        ));
    }
    produce_and_read(input, limits)
}

/// Produce and strict-read one canonical result that directly corrects the
/// already-validated predecessor named in `correction`.
///
/// **Test-only.** Correction is not a production capability yet: the
/// owner-authenticated contradiction capability that would supply it is
/// deferred upstream (Plan-001 Task-006), and the Engine derives no correction
/// (see [`crate::assessment::CanonicalRowControls`]). This constructor stands in
/// for that deferred capability so a dependent can test how it reads a
/// correction edge. It is removed or replaced once the capability ships.
///
/// The correction edge is validated by the same lineage rules the canonical
/// producer applies, and the predecessor's bytes are never modified.
///
/// # Errors
///
/// Returns [`ErrorCode::InvalidSelection`] when `input` already carries a
/// correction, and otherwise any refusal the canonical producer or the strict
/// reader returns for the corrected input under `limits`.
pub fn corrected_result<'a>(
    mut input: Input<'a>,
    correction: CorrectionInput<'a>,
    limits: Limits,
) -> Result<ValidatedResult> {
    if input.correction.is_some() {
        return Err(Refusal::assembly(
            ErrorCode::InvalidSelection,
            "test-support corrected_result takes its correction as an argument",
            super::Usage::default(),
            None,
        ));
    }
    input.correction = Some(correction);
    produce_and_read(input, limits)
}

fn produce_and_read(input: Input<'_>, limits: Limits) -> Result<ValidatedResult> {
    let document = produce(input.clone(), limits)?;
    read(
        document.bytes(),
        Expected {
            input,
            enclosing_digest: document.digest(),
        },
        limits,
    )
}
