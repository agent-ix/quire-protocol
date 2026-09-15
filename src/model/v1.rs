// SPDX-License-Identifier: AGPL-3.0-or-later
//! Static linking for the explicit `quire.compiled-protocol/1` compatibility lane.
//!
//! This module deliberately accepts only A's admitted `/1` package and a
//! declaration coordinate. It cannot consume the `/2` static/runtime binding
//! capabilities used by a full assessment.

use quire_spec_language::protocol_artifact::AdmittedPackage;

use crate::link::{diagnostics::Diagnostics, LinkedProtocol};

/// Links one protocol declaration from an already admitted `/1` package.
///
/// The returned representation is the frozen static compatibility graph. New
/// assessment integrations use [`crate::model::v2::link`] after constructing a
/// version-2 assessment context instead.
pub fn link_static(
    package: &AdmittedPackage,
    declaration: u32,
) -> Result<LinkedProtocol, Diagnostics> {
    crate::link::link(package, declaration)
}
