// SPDX-License-Identifier: AGPL-3.0-or-later
//! Exact QSL `/2` state-view fixtures for Protocol integration tests.

use qsl_foundation::ByteDigest;
use quire_spec_language::protocol_artifact::{v2, wire as w, ExactInteger, ProtocolNumber};
use quire_spec_language::state::{
    AssessmentAuthority, AuthorityAdapter, AuthorityEvidence, BinderInput, CanonicalDigest,
    ContextualSlot, FieldInput, FieldValue, InputSlot, MissingInput, ObjectInput, ObjectKey,
    ObservationDigest, ObservationIdentity, ObservationKey, PopulationInput, StateView,
    StaticAuthority, Value as StateValue, ValueKind as StateValueKind,
    OBSERVATION_CONTRACT_REVISION,
};

fn canonical(value: char) -> CanonicalDigest {
    CanonicalDigest {
        algorithm: "sha256".into(),
        domain: "filament-canonical-json-1".into(),
        value: format!("sha256:{}", value.to_string().repeat(64)),
    }
}

fn observation_digest(value: char) -> ObservationDigest {
    ObservationDigest(format!("sha256:{}", value.to_string().repeat(64)))
}

fn state_authority(
    package: &v2::AdmittedPackage,
    owner: u32,
    requirement_index: u32,
) -> AuthorityEvidence {
    let inherited = package.inherited();
    let requirement = &inherited.declarations[owner as usize].bindings[requirement_index as usize];
    let selection = requirement
        .model
        .0
        .as_ref()
        .expect("model-bound requirement");
    let producer = inherited.dependencies
        [inherited.models[selection.model as usize].artifact as usize]
        .artifact
        .clone();
    let static_selection = StaticAuthority {
        document_identity: "document:selected".into(),
        document_digest: canonical('1'),
        model_identity: "model:selected".into(),
        model_digest: canonical('2'),
        profile_identity: "profile:selected".into(),
        profile_digest: canonical('3'),
        configuration_identity: "configuration:selected".into(),
        configuration_digest: canonical('4'),
    };
    let assessment_selection = AssessmentAuthority {
        population_identity: "population:selected".into(),
        membership_digest: observation_digest('5'),
        membership_complete: true,
        snapshot_identity: "snapshot:selected".into(),
        snapshot_digest: observation_digest('6'),
        window_identity: None,
        window_digest: None,
        closure_identity: "closure:selected".into(),
        closure_digest: observation_digest('7'),
    };
    let mut adapter_artifact = requirement.authority.clone();
    adapter_artifact.kind = w::ArtifactKind::Binding;
    adapter_artifact.identity = "explicit-d-f-compatibility".into();
    adapter_artifact.digest = ByteDigest::of(b"explicit compatibility mapping fixture");
    adapter_artifact.wire.identity = "quire.state.authority-adapter".into();
    adapter_artifact.wire.version = "1".into();
    let compiled = package.artifact().expect("strict package artifact").clone();
    let observation = requirement.authority.clone();
    let adapter = AuthorityAdapter {
        artifact: adapter_artifact,
        compiled: compiled.clone(),
        requirement: observation.clone(),
        producer: producer.clone(),
        observation: observation.clone(),
        observation_contract_revision: OBSERVATION_CONTRACT_REVISION.into(),
        static_selection: static_selection.clone(),
        assessment_selection: assessment_selection.clone(),
    };
    AuthorityEvidence {
        observation_contract_revision: OBSERVATION_CONTRACT_REVISION.into(),
        producer,
        observation,
        compiled,
        adapter: Some(adapter),
        static_selection,
        assessment_selection,
    }
}

fn binder_requirement(package: &v2::AdmittedPackage, binder: &w::Handle) -> u32 {
    let declaration = &package.inherited().declarations[binder.declaration as usize];
    declaration.anchors[declaration.binders[binder.index as usize].anchor.index as usize]
        .binding
        .0
        .expect("runtime-bound compensation binder")
}

fn field_export(
    package: &v2::AdmittedPackage,
    model: u32,
    record: &str,
    field: &str,
) -> w::ExportRef {
    let export = package.inherited().models[model as usize]
        .exports
        .iter()
        .position(|export| export.kind == w::ExportKind::Field && export.path == [record, field])
        .expect("selected model field");
    w::ExportRef {
        model,
        export: u32::try_from(export).expect("bounded fixture export"),
    }
}

fn object_population(
    package: &v2::AdmittedPackage,
    binders: &[(w::Handle, &str, Vec<FieldInput>)],
) -> (Vec<BinderInput>, PopulationInput) {
    let inherited = package.inherited();
    let first = &binders[0].0;
    let declaration = &inherited.declarations[first.declaration as usize];
    let first_binding = &declaration.binders[first.index as usize];
    let w::Type::Object {
        export: object_type,
    } = &inherited.types[first_binding.value_type as usize]
    else {
        panic!("object binder type")
    };
    let population_index = declaration
        .bindings
        .iter()
        .position(|requirement| {
            requirement.kind == w::BindingKind::Population
                && requirement.anchor == first_binding.anchor
                && requirement
                    .model
                    .0
                    .as_ref()
                    .is_some_and(|model| model.model == object_type.model)
        })
        .and_then(|index| u32::try_from(index).ok())
        .expect("object population requirement");
    let closure_index = declaration
        .bindings
        .iter()
        .position(|requirement| {
            requirement.kind == w::BindingKind::Closure
                && requirement.requires == [population_index]
        })
        .and_then(|index| u32::try_from(index).ok())
        .expect("object population closure");
    let universe = declaration.bindings[population_index as usize]
        .model
        .0
        .clone()
        .expect("population export");
    let entries = binders
        .iter()
        .map(|(binder, identifier, fields)| {
            let binding = &declaration.binders[binder.index as usize];
            assert_eq!(binding.anchor, first_binding.anchor);
            let key = ObjectKey {
                observation: ObservationKey {
                    anchor: binding.anchor.clone(),
                    snapshot: ObservationIdentity("snapshot:selected".into()),
                    window: None,
                    record: ObservationIdentity(format!("record:{identifier}")),
                },
                model: object_type.model,
                universe: universe.clone(),
                object_type: object_type.clone(),
                identifier: (*identifier).into(),
            };
            let object_name = inherited.models[object_type.model as usize].exports
                [object_type.export as usize]
                .path
                .first()
                .expect("object name");
            let complete_fields = inherited.models[object_type.model as usize]
                .exports
                .iter()
                .enumerate()
                .filter(|(_, export)| {
                    export.kind == w::ExportKind::Field
                        && export.path.len() == 2
                        && export.path.first() == Some(object_name)
                })
                .map(|(index, _)| w::ExportRef {
                    model: object_type.model,
                    export: u32::try_from(index).expect("bounded field export"),
                })
                .map(|field| {
                    fields
                        .iter()
                        .find(|offered| offered.field == field)
                        .cloned()
                        .unwrap_or_else(|| FieldInput {
                            field: field.clone(),
                            value: FieldValue::Contextual(ContextualSlot::Unavailable(
                                MissingInput::Field {
                                    object: key.clone(),
                                    field,
                                },
                            )),
                        })
                })
                .collect();
            (
                BinderInput {
                    binder: binder.clone(),
                    requirement: Some(binder_requirement(package, binder)),
                    authority: Some(state_authority(
                        package,
                        binder.declaration,
                        binder_requirement(package, binder),
                    )),
                    value: InputSlot::Available(StateValue::new(
                        binding.value_type,
                        StateValueKind::Object(key.clone()),
                    )),
                },
                ObjectInput {
                    key,
                    fields: complete_fields,
                },
            )
        })
        .collect::<Vec<_>>();
    (
        entries.iter().map(|(binder, _)| binder.clone()).collect(),
        PopulationInput {
            requirement: w::Handle {
                declaration: first.declaration,
                index: population_index,
            },
            closure_requirement: w::Handle {
                declaration: first.declaration,
                index: closure_index,
            },
            authority: state_authority(package, first.declaration, population_index),
            membership: Ok(()),
            closure: Ok(()),
            objects: entries.into_iter().map(|(_, object)| object).collect(),
        },
    )
}

/// Supplies one exact plain-record binder used by guard and retry expressions.
pub fn plain_binder(package: &v2::AdmittedPackage, binder: w::Handle, ready: bool) -> BinderInput {
    let declaration = &package.inherited().declarations[binder.declaration as usize];
    let binding = &declaration.binders[binder.index as usize];
    let w::Type::Record { export } = &package.inherited().types[binding.value_type as usize] else {
        panic!("plain binder record type")
    };
    let boolean_type = package
        .inherited()
        .types
        .iter()
        .position(|value| matches!(value, w::Type::Boolean {}))
        .and_then(|index| u32::try_from(index).ok())
        .expect("Boolean wire type");
    let record_name = package.inherited().models[export.model as usize].exports
        [export.export as usize]
        .path
        .first()
        .expect("record name")
        .clone();
    let ready_field = field_export(package, export.model, &record_name, "ready");
    let requirement = binder_requirement(package, &binder);
    BinderInput {
        binder,
        requirement: Some(requirement),
        authority: Some(state_authority(
            package,
            binding.anchor.declaration,
            requirement,
        )),
        value: InputSlot::Available(StateValue::new(
            binding.value_type,
            StateValueKind::Record(vec![FieldInput {
                field: ready_field,
                value: FieldValue::Compiled(InputSlot::Available(StateValue::new(
                    boolean_type,
                    StateValueKind::Boolean(ready),
                ))),
            }]),
        )),
    }
}

/// Supplies all exact state needed by the published Full retry relation.
pub fn retry_view(
    package: &v2::AdmittedPackage,
    forward: w::Handle,
    trigger: w::Handle,
    earlier: w::Handle,
    later: w::Handle,
    earlier_tally: i64,
    later_tally: i64,
) -> StateView {
    let declaration = &package.inherited().declarations[earlier.declaration as usize];
    let total = field_export(package, 0, "Workflow", "total");
    let tally = w::ExportRef {
        model: 0,
        export: 6,
    };
    let total_type = declaration
        .values
        .iter()
        .find_map(|value| match &value.operation {
            w::ValueOperation::Field { field, .. } if field == &total => Some(value.value_type),
            _ => None,
        })
        .expect("published total type");
    let tally_type = declaration
        .values
        .iter()
        .find_map(|value| match &value.operation {
            w::ValueOperation::Field { field, .. } if field == &tally => Some(value.value_type),
            _ => None,
        })
        .expect("published tally type");
    let tally_field = |value| FieldInput {
        field: tally.clone(),
        value: FieldValue::Compiled(InputSlot::Available(StateValue::new(
            tally_type,
            StateValueKind::Number(ProtocolNumber::Integer(ExactInteger::new(value))),
        ))),
    };
    let total_field = FieldInput {
        field: total,
        value: FieldValue::Compiled(InputSlot::Available(StateValue::new(
            total_type,
            StateValueKind::Number(ProtocolNumber::Integer(ExactInteger::new(1))),
        ))),
    };
    let (mut binders, forward_population) =
        object_population(package, &[(forward, "forward", vec![total_field])]);
    let (retry_binders, retry_population) = object_population(
        package,
        &[
            (earlier, "earlier", vec![tally_field(earlier_tally)]),
            (later, "later", vec![tally_field(later_tally)]),
        ],
    );
    binders.extend(retry_binders);
    binders.push(plain_binder(package, trigger, false));
    StateView {
        binders,
        populations: vec![forward_population, retry_population],
    }
}

/// Supplies all exact state needed by the published Full recovery relation.
pub fn published_recovery_view(
    package: &v2::AdmittedPackage,
    forward: w::Handle,
    trigger: w::Handle,
    recovery: w::Handle,
    target: i64,
    restored: i64,
) -> StateView {
    let declaration = &package.inherited().declarations[recovery.declaration as usize];
    let total = field_export(package, 0, "Workflow", "total");
    let receipts = w::ExportRef {
        model: 0,
        export: 5,
    };
    let total_type = declaration
        .values
        .iter()
        .find_map(|value| match &value.operation {
            w::ValueOperation::Field { field, .. } if field == &total => Some(value.value_type),
            _ => None,
        })
        .expect("published total type");
    let receipts_type = declaration
        .values
        .iter()
        .find_map(|value| match &value.operation {
            w::ValueOperation::Field { field, .. } if field == &receipts => Some(value.value_type),
            _ => None,
        })
        .expect("published receipts type");
    let w::Type::Sequence { element, .. } = package.inherited().types[receipts_type as usize]
    else {
        panic!("published receipts sequence")
    };
    let receipts_field = FieldInput {
        field: receipts,
        value: FieldValue::Compiled(InputSlot::Available(StateValue::new(
            receipts_type,
            StateValueKind::Sequence(vec![InputSlot::Available(StateValue::new(
                element,
                StateValueKind::Number(ProtocolNumber::Integer(ExactInteger::new(restored))),
            ))]),
        ))),
    };
    let total_field = FieldInput {
        field: total,
        value: FieldValue::Compiled(InputSlot::Available(StateValue::new(
            total_type,
            StateValueKind::Number(ProtocolNumber::Integer(ExactInteger::new(target))),
        ))),
    };
    let (mut binders, forward_population) =
        object_population(package, &[(forward, "forward", vec![total_field])]);
    let (recovery_binders, recovery_population) =
        object_population(package, &[(recovery, "recovery", vec![receipts_field])]);
    binders.extend(recovery_binders);
    binders.push(plain_binder(package, trigger, false));
    StateView {
        binders,
        populations: vec![forward_population, recovery_population],
    }
}
