// SPDX-License-Identifier: AGPL-3.0-or-later
//! Explicit `quire.compiled-protocol/2` admission for full assessment.

use quire_spec_language::protocol_artifact::{v2, Limits, Report};

/// Admit only an independently selected version-2 compiled protocol.
pub fn admit(
    bytes: &[u8],
    expected: &v2::Expected<'_>,
    limits: Limits,
) -> Report<v2::AdmittedPackage> {
    quire_spec_language::protocol_artifact::v2::read(bytes, expected, limits)
}
