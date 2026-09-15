// SPDX-License-Identifier: AGPL-3.0-or-later
//! Explicit `quire.compiled-protocol/1` admission.

use quire_spec_language::protocol_artifact::{AdmittedPackage, Expected, Limits, Report};

/// Admit only an independently selected version-1 compiled protocol.
///
/// This path has no access to full-assessment binding, observation, temporal,
/// result, or evidence values.
pub fn admit(bytes: &[u8], expected: &Expected<'_>, limits: Limits) -> Report<AdmittedPackage> {
    quire_spec_language::protocol_artifact::read(bytes, expected, limits)
}
