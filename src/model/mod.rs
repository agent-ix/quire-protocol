// SPDX-License-Identifier: AGPL-3.0-or-later
//! Version-explicit linked-assessment models.
//!
//! The frozen root admission facade deliberately has no access to these
//! capabilities. Full assessment paths consume the versioned models here.

/// Version-2 static and runtime binding capabilities.
/// Frozen `/1` compatibility linking over an admitted legacy package.
pub mod v1;
/// Full-assessment `/2` binding and linking capabilities.
pub mod v2;
