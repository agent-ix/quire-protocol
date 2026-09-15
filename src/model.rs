// SPDX-License-Identifier: AGPL-3.0-or-later
//! Version-explicit finite linked protocol models.

/// Frozen version-1 static graph linking.
pub mod v1 {
    use quire_spec_language::protocol_artifact::AdmittedPackage;

    use crate::bounds::inventory::{OperationOutcome, ResourceAccountant};
    use crate::link::{StaticDiagnostics, StaticLinkedProtocol};

    /// Link the selected version-1 declaration without importing runtime values.
    pub fn link_static(
        package: &AdmittedPackage,
        declaration: u32,
    ) -> Result<StaticLinkedProtocol, StaticDiagnostics> {
        crate::link::link_static_v1(package, declaration)
    }

    /// Link the selected version-1 declaration under the shared resource budget.
    pub fn link_static_bounded(
        package: &AdmittedPackage,
        declaration: u32,
        resources: &mut ResourceAccountant<'_>,
    ) -> OperationOutcome<StaticLinkedProtocol, StaticDiagnostics> {
        crate::link::link_static_v1_bounded(package, declaration, resources)
    }
}

/// Version-2 static/runtime binding and exact FR-288 linking.
pub mod v2 {
    use quire_spec_language::protocol_artifact::v2;

    use crate::link::{
        AcceptedBindingDefinition, DefinitionRegistry, LinkDiagnostics, LinkedProtocol,
        RuntimeBindingSet,
    };

    /// Link the complete selected definition and finite runtime binding set.
    pub fn link(
        package: &v2::AdmittedPackage,
        binding: &AcceptedBindingDefinition,
        runtime: RuntimeBindingSet,
        registry: &mut DefinitionRegistry,
    ) -> Result<LinkedProtocol, LinkDiagnostics> {
        crate::link::link_v2(package, binding, runtime, registry)
    }
}
