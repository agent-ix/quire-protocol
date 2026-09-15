// SPDX-License-Identifier: AGPL-3.0-or-later
//! Version-explicit strict compiled-protocol admission.

/// Frozen `quire.compiled-protocol/1` intake.
pub mod v1 {
    use quire_spec_language::protocol_artifact::{self, AdmittedPackage, Expected, Limits, Report};

    /// Admit exact caller-selected version-1 bytes through QSL's strict reader.
    pub fn admit(bytes: &[u8], expected: &Expected<'_>, limits: Limits) -> Report<AdmittedPackage> {
        protocol_artifact::read(bytes, expected, limits)
    }
}

/// `quire.compiled-protocol/2` intake.
pub mod v2 {
    use quire_spec_language::protocol_artifact::{v2, Limits, Report};

    /// Admit exact caller-selected version-2 bytes through QSL's strict reader.
    pub fn admit(
        bytes: &[u8],
        expected: &v2::Expected<'_>,
        limits: Limits,
    ) -> Report<v2::AdmittedPackage> {
        v2::read(bytes, expected, limits)
    }
}
