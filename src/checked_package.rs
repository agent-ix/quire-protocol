// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-043: admission of the checked-package handoff.
//!
//! The handoff directory that `quire_spec_language::checked_handoff` writes
//! (QSL FR-517) holds `checked-package.json`, `selection.json`, `identities.json`
//! and `equality.json`. [`admit`] reads the package with Contract IR's
//! [`read_checked_package`], compares the admitted package with the selection
//! sidecar, and reads the two bytes-only files against it. It returns an
//! [`AdmittedHandoff`] only when every step succeeded.
//!
//! This module reads no checked-package bytes of its own and declares no
//! refusal code, format string or axis: every refusal is the owner's value,
//! carried unchanged in [`Refusal`].

use std::fmt;

use qsl_package::{EqualityFileRefusal, IdentityFileRefusal, ReadEqualityRows, ReadIdentityRows};
use quire_contract_model::{
    read_checked_package, CheckedPackageDispatchResult, CheckedPackageEvidence,
    CheckedPackageIncomplete, CheckedPackageReadLimits, CheckedPackageRefusal, CheckedPackageV2,
};
use quire_spec_language::checked_handoff::{Selection, SelectionReadError, SelectionRefusal};

/// The bytes of the four handoff files [`admit`] reads.
#[derive(Clone, Copy, Debug)]
pub struct HandoffFiles<'a> {
    /// `checked-package.json`.
    pub checked_package: &'a [u8],
    /// `selection.json`, the sidecar derived from the source before any bytes
    /// were emitted.
    pub selection: &'a [u8],
    /// `identities.json`.
    pub identities: &'a [u8],
    /// `equality.json`.
    pub equality: &'a [u8],
}

/// A checked package that Contract IR's reader admitted and the handoff's
/// selection accepted, with the two files that join it.
#[derive(Debug)]
pub struct AdmittedHandoff {
    package: Box<CheckedPackageV2>,
    identities: ReadIdentityRows,
    equality: ReadEqualityRows,
}

impl AdmittedHandoff {
    /// The package, the value `read_checked_package` returned.
    #[must_use]
    pub fn package(&self) -> &CheckedPackageV2 {
        &self.package
    }

    /// The selected model identity rows of `identities.json`.
    #[must_use]
    pub const fn identities(&self) -> &ReadIdentityRows {
        &self.identities
    }

    /// The equality rows of `equality.json`.
    #[must_use]
    pub const fn equality(&self) -> &ReadEqualityRows {
        &self.equality
    }
}

/// Why no [`AdmittedHandoff`] exists. Every variant carries its owner's value
/// as that owner returned it.
#[derive(Debug)]
pub enum Refusal {
    /// `read_checked_package` refused the package.
    Reader(Box<CheckedPackageRefusal>),
    /// `read_checked_package` reached a read limit before a conclusion. This
    /// is neither an admitted package nor a refusal.
    Incomplete(CheckedPackageIncomplete),
    /// `selection.json` is not a selection sidecar.
    SelectionFile(SelectionReadError),
    /// The admitted package departs from the selection along one axis.
    Selection(SelectionRefusal),
    /// `identities.json` was refused.
    Identities(IdentityFileRefusal),
    /// `equality.json` was refused.
    Equality(EqualityFileRefusal),
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Reader(refusal) => write!(f, "the reader refused the package: {refusal:?}"),
            Self::Incomplete(incomplete) => {
                write!(f, "the reader reached a limit: {incomplete:?}")
            }
            Self::SelectionFile(error) => write!(f, "{error}"),
            Self::Selection(refusal) => write!(f, "{refusal}"),
            Self::Identities(refusal) => write!(f, "{refusal}"),
            Self::Equality(refusal) => write!(f, "{refusal}"),
        }
    }
}

impl std::error::Error for Refusal {}

/// Admit one handoff.
///
/// The package is read first and only then compared with the selection, so a
/// defect the reader refuses is a [`Refusal::Reader`] and a defect only the
/// selection can see is a [`Refusal::Selection`]. `evidence` is the evidence
/// assembled from the handoff's own files (QSL FR-517) and `limits` the read
/// limits of the caller.
///
/// # Errors
///
/// The first [`Refusal`] in the order package, selection, identities, equality.
pub fn admit(
    files: &HandoffFiles<'_>,
    evidence: &CheckedPackageEvidence,
    limits: CheckedPackageReadLimits,
) -> Result<AdmittedHandoff, Refusal> {
    let package = match read_checked_package(files.checked_package, limits, evidence) {
        CheckedPackageDispatchResult::AdmittedV2(package) => package,
        CheckedPackageDispatchResult::Refused(refusal) => {
            return Err(Refusal::Reader(Box::new(refusal)))
        }
        CheckedPackageDispatchResult::Incomplete(incomplete) => {
            return Err(Refusal::Incomplete(incomplete))
        }
    };
    Selection::read(files.selection)
        .map_err(Refusal::SelectionFile)?
        .check(&package)
        .map_err(Refusal::Selection)?;
    let identities =
        ReadIdentityRows::read(files.identities, &package).map_err(Refusal::Identities)?;
    let equality = ReadEqualityRows::read(files.equality, &package).map_err(Refusal::Equality)?;
    Ok(AdmittedHandoff {
        package,
        identities,
        equality,
    })
}
