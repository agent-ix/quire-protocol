// SPDX-License-Identifier: AGPL-3.0-or-later
//! Strict bounded hostile-byte reader for canonical protocol results.

use serde_json::Value;
use std::sync::OnceLock;

use crate::bounds::inventory::{
    BudgetDimension, ChargeError, OperationOutcome, ResourceAccountant, SourceSite,
};
use crate::bounds::{SiteId, SiteKind, UnitLabel};
use crate::digest::jcs::canonicalize;
use crate::refusal::ProtocolStage;

use super::record::{produce, Input, Limits, Usage, ValidatedResult, CONTRACT, SCHEMA_BYTES};
use super::{ErrorCode, Refusal, Result};

pub(crate) const INPUT_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("result/input/decode"),
    "result::reader::read",
    BudgetDimension::InputByte,
    SiteKind::Iteration,
);
pub(crate) const STRING_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("result/string/retain"),
    "result::reader::read",
    BudgetDimension::InputByte,
    SiteKind::Retention,
);
pub(crate) const OPERATION_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("result/field/visit"),
    "result::reader::read",
    BudgetDimension::Operation,
    SiteKind::Traversal,
);
pub(crate) const DEPTH_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("result/container/depth"),
    "result::reader::read",
    BudgetDimension::Operation,
    SiteKind::Traversal,
);
pub(crate) const ALLOCATION_SOURCE: SourceSite = SourceSite::new(
    SiteId::new("result/document/read-allocate"),
    "result::reader::read",
    BudgetDimension::AccountedAllocation,
    SiteKind::Allocation,
);

static RESULT_SCHEMA_VALIDATOR: OnceLock<Option<jsonschema::JSONSchema>> = OnceLock::new();

/// Independent selections required to accept result bytes.
#[derive(Clone, Debug)]
pub struct Expected<'a> {
    /// Complete semantic input from independently validated owner views.
    pub input: Input<'a>,
}

#[derive(Clone, Copy)]
pub(crate) struct StructuralSources {
    input: SourceSite,
    string: SourceSite,
    depth: SourceSite,
    operation: SourceSite,
}

impl StructuralSources {
    pub(crate) const fn new(
        input: SourceSite,
        string: SourceSite,
        depth: SourceSite,
        operation: SourceSite,
    ) -> Self {
        Self {
            input,
            string,
            depth,
            operation,
        }
    }
}

const RESULT_SOURCES: StructuralSources =
    StructuralSources::new(INPUT_SOURCE, STRING_SOURCE, DEPTH_SOURCE, OPERATION_SOURCE);

/// Strict-read canonical result bytes against independent selections.
pub fn read_bounded(
    bytes: &[u8],
    expected: Expected<'_>,
    limits: Limits,
    resources: &mut ResourceAccountant<'_>,
) -> OperationOutcome<ValidatedResult, Refusal> {
    resources.run_unit(
        UnitLabel::new("result::reader::read_bounded"),
        |resources| read_bounded_unit(bytes, expected, limits, resources),
    )
}

fn read_bounded_unit(
    bytes: &[u8],
    expected: Expected<'_>,
    limits: Limits,
    resources: &mut ResourceAccountant<'_>,
) -> OperationOutcome<ValidatedResult, Refusal> {
    let effective = limits.effective();
    if let Err(error) = charge_structural(bytes, effective, resources, RESULT_SOURCES) {
        return OperationOutcome::from_charge(error);
    }
    if let Err(error) = resources.charge(ALLOCATION_SOURCE.id(), 1) {
        return OperationOutcome::from_charge(error);
    }
    OperationOutcome::from_domain(read(bytes, expected, effective))
}

pub(crate) fn charge_structural(
    bytes: &[u8],
    limits: Limits,
    resources: &mut ResourceAccountant<'_>,
    sources: StructuralSources,
) -> core::result::Result<StructuralUsage, ChargeError> {
    resources.charge_with_limit(sources.input.id(), bytes.len(), limits.max_input_bytes)?;
    let observed = structural_usage(bytes);
    resources.charge_with_limit(
        sources.string.id(),
        observed.string_bytes,
        limits.max_string_bytes,
    )?;
    resources.charge_with_limit(sources.depth.id(), observed.depth, limits.max_depth)?;
    resources.charge_with_limit(
        sources.operation.id(),
        observed.fields,
        limits.max_visited_fields,
    )?;
    Ok(observed)
}

#[derive(Clone, Copy)]
enum ScanFrame {
    Array { expects_value: bool },
    Object { expects_key: bool },
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct StructuralUsage {
    pub(crate) fields: usize,
    pub(crate) depth: usize,
    pub(crate) string_bytes: usize,
}

/// Count the object-member and array-element visits made by `visited_fields`
/// without constructing a JSON value or retaining any offered string.
pub(crate) fn structural_usage(bytes: &[u8]) -> StructuralUsage {
    const STACK_CAPACITY: usize = Limits::owner_max().max_depth;

    let mut stack = [ScanFrame::Array {
        expects_value: false,
    }; STACK_CAPACITY];
    let mut stack_len = 0usize;
    let mut fields = 0usize;
    let mut max_depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    let mut in_primitive = false;
    let mut string_bytes = 0usize;
    let mut max_string_bytes = 0usize;

    for byte in bytes.iter().copied() {
        if in_string {
            if escaped {
                escaped = false;
                string_bytes += 1;
            } else if byte == b'\\' {
                escaped = true;
                string_bytes += 1;
            } else if byte == b'"' {
                in_string = false;
                max_string_bytes = max_string_bytes.max(string_bytes);
            } else {
                string_bytes += 1;
            }
            continue;
        }
        if in_primitive {
            if matches!(byte, b',' | b']' | b'}' | b' ' | b'\n' | b'\r' | b'\t') {
                in_primitive = false;
            } else {
                continue;
            }
        }
        match byte {
            b'"' => {
                count_parent_value(&mut stack[..stack_len], &mut fields);
                in_string = true;
                string_bytes = 0;
            }
            b'{' => {
                count_array_value(&mut stack[..stack_len], &mut fields);
                if stack_len == stack.len() {
                    return StructuralUsage {
                        fields,
                        depth: stack_len + 1,
                        string_bytes: max_string_bytes,
                    };
                }
                stack[stack_len] = ScanFrame::Object { expects_key: true };
                stack_len += 1;
                max_depth = max_depth.max(stack_len);
            }
            b'[' => {
                count_array_value(&mut stack[..stack_len], &mut fields);
                if stack_len == stack.len() {
                    return StructuralUsage {
                        fields,
                        depth: stack_len + 1,
                        string_bytes: max_string_bytes,
                    };
                }
                stack[stack_len] = ScanFrame::Array {
                    expects_value: true,
                };
                stack_len += 1;
                max_depth = max_depth.max(stack_len);
            }
            b',' => match stack[..stack_len].last_mut() {
                Some(ScanFrame::Array { expects_value }) => *expects_value = true,
                Some(ScanFrame::Object { expects_key }) => *expects_key = true,
                None => {}
            },
            b']' | b'}' => {
                stack_len = stack_len.saturating_sub(1);
            }
            b':' | b' ' | b'\n' | b'\r' | b'\t' => {}
            _ => {
                count_array_value(&mut stack[..stack_len], &mut fields);
                in_primitive = true;
            }
        }
    }
    StructuralUsage {
        fields,
        depth: max_depth,
        string_bytes: max_string_bytes,
    }
}

fn count_parent_value(stack: &mut [ScanFrame], fields: &mut usize) {
    match stack.last_mut() {
        Some(ScanFrame::Array { expects_value }) if *expects_value => {
            *fields += 1;
            *expects_value = false;
        }
        Some(ScanFrame::Object { expects_key }) if *expects_key => {
            *fields += 1;
            *expects_key = false;
        }
        Some(ScanFrame::Array { .. }) | Some(ScanFrame::Object { .. }) | None => {}
    }
}

fn count_array_value(stack: &mut [ScanFrame], fields: &mut usize) {
    if let Some(ScanFrame::Array { expects_value }) = stack.last_mut() {
        if *expects_value {
            *fields += 1;
            *expects_value = false;
        }
    }
}

/// Strict-read canonical result bytes against independent selections.
pub fn read(bytes: &[u8], expected: Expected<'_>, limits: Limits) -> Result<ValidatedResult> {
    let effective = limits.effective();
    let observed = preflight(bytes, effective, true, ProtocolStage::ResultRead)?;
    let value: Value = serde_json::from_slice(bytes).map_err(|_| {
        Refusal::read(
            ErrorCode::InvalidJson,
            "result is not one valid closed JSON value",
            observed,
            None,
        )
    })?;
    let object = value.as_object().ok_or_else(|| {
        Refusal::read(
            ErrorCode::InvalidJson,
            "result root is not an object",
            observed,
            None,
        )
    })?;
    if object.get("contractVersion").and_then(Value::as_str) != Some(CONTRACT)
        || object.get("kind").and_then(Value::as_str) != Some("ProtocolResult")
    {
        return Err(Refusal::read(
            ErrorCode::ContractMismatch,
            "result contractVersion or kind differs from the selected contract",
            observed,
            None,
        ));
    }
    if !schema_accepts(&value, SCHEMA_BYTES, &RESULT_SCHEMA_VALIDATOR) {
        return Err(Refusal::read(
            ErrorCode::InvalidJson,
            "result is not a closed instance of the selected schema",
            observed,
            None,
        ));
    }
    let canonical = canonicalize(&value).map_err(|_| {
        Refusal::read(
            ErrorCode::NonCanonical,
            "result contains a JSON number outside the selected JCS subset",
            observed,
            None,
        )
    })?;
    if canonical.as_bytes() != bytes {
        return Err(Refusal::read(
            ErrorCode::NonCanonical,
            "result has duplicate, reordered, trailing, or noncanonical bytes",
            observed,
            None,
        ));
    }
    let mut expected_document = produce(expected.input, limits)?;
    let expected_key = serde_json::to_value(expected_document.result_key()).map_err(|_| {
        Refusal::read(
            ErrorCode::IdentityMismatch,
            "selected FR-286 result key cannot be represented",
            observed,
            None,
        )
    })?;
    if object.get("resultKey") != Some(&expected_key) {
        return Err(Refusal::read(
            ErrorCode::IdentityMismatch,
            "resultKey differs from the independently selected FR-286 key",
            observed,
            None,
        ));
    }
    if expected_document.bytes() != bytes {
        return Err(Refusal::read(
            ErrorCode::InvalidSelection,
            "result fields differ from independently selected owner facts",
            observed,
            None,
        ));
    }
    expected_document.usage = observed;
    Ok(ValidatedResult {
        document: expected_document,
    })
}

pub(crate) fn preflight_output(bytes: &[u8], limits: Limits, semantic: Usage) -> Result<Usage> {
    let stage = ProtocolStage::ResultAssembly;
    let mut usage = preflight(bytes, limits, false, stage)?;
    usage.claims = semantic.claims;
    usage.premises = semantic.premises;
    usage.support_records = semantic.support_records;
    usage.limitations = semantic.limitations;
    usage.lineage_nodes = semantic.lineage_nodes;
    check_semantic(usage, limits, stage)?;
    Ok(usage)
}

pub(crate) fn preflight_external(bytes: &[u8], limits: Limits) -> Result<Usage> {
    preflight(bytes, limits, true, ProtocolStage::ResultRead)
}

pub(crate) fn schema_accepts(
    instance: &Value,
    schema_bytes: &[u8],
    cache: &'static OnceLock<Option<jsonschema::JSONSchema>>,
) -> bool {
    cache
        .get_or_init(|| {
            let schema = serde_json::from_slice(schema_bytes).ok()?;
            jsonschema::JSONSchema::options()
                .with_draft(jsonschema::Draft::Draft202012)
                .compile(&schema)
                .ok()
        })
        .as_ref()
        .is_some_and(|validator| validator.is_valid(instance))
}

fn preflight(bytes: &[u8], limits: Limits, input: bool, stage: ProtocolStage) -> Result<Usage> {
    let byte_limit = if input {
        limits.max_input_bytes
    } else {
        limits.max_output_bytes
    };
    if bytes.len() > byte_limit {
        return Err(Refusal::at_stage(
            ErrorCode::ResourceIncomplete,
            "wire bytes exceed the effective ceiling",
            Usage {
                wire_bytes: bytes.len(),
                ..Usage::default()
            },
            None,
            stage,
        ));
    }
    if core::str::from_utf8(bytes).is_err() {
        return Err(Refusal::at_stage(
            ErrorCode::InvalidJson,
            "result bytes are not UTF-8",
            Usage {
                wire_bytes: bytes.len(),
                ..Usage::default()
            },
            None,
            stage,
        ));
    }
    let mut depth = 0usize;
    let mut max_depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    let mut string_bytes = 0usize;
    let mut max_string = 0usize;
    for byte in bytes.iter().copied() {
        if in_string {
            if escaped {
                escaped = false;
                // At most one increment occurs per byte in `bytes`, so this
                // cannot exceed the slice's representable `usize` length.
                string_bytes += 1;
            } else if byte == b'\\' {
                escaped = true;
                string_bytes += 1;
            } else if byte == b'"' {
                in_string = false;
                max_string = max_string.max(string_bytes);
            } else {
                string_bytes += 1;
            }
            continue;
        }
        match byte {
            b'"' => {
                in_string = true;
                string_bytes = 0;
            }
            b'{' | b'[' => {
                // Container openings are a subset of the input byte
                // population, whose length already fits `usize`.
                depth += 1;
                max_depth = max_depth.max(depth);
            }
            b'}' | b']' => {
                depth = depth.saturating_sub(1);
            }
            _ => {}
        }
    }
    let usage = Usage {
        wire_bytes: bytes.len(),
        depth: max_depth,
        string_bytes: max_string,
        ..Usage::default()
    };
    if max_depth > limits.max_depth || max_string > limits.max_string_bytes {
        return Err(Refusal::at_stage(
            ErrorCode::ResourceIncomplete,
            "JSON depth or string size exceeds the effective ceiling",
            usage,
            None,
            stage,
        ));
    }
    let value: Value = serde_json::from_slice(bytes).map_err(|_| {
        Refusal::at_stage(
            ErrorCode::InvalidJson,
            "wire bytes are not valid JSON",
            usage,
            None,
            stage,
        )
    })?;
    let visited = visited_fields(&value, limits.max_visited_fields, stage)?;
    Ok(Usage {
        visited_fields: visited,
        ..usage
    })
}

fn visited_fields(root: &Value, limit: usize, stage: ProtocolStage) -> Result<usize> {
    let mut stack = vec![root];
    let mut visited = 0usize;
    while let Some(value) = stack.pop() {
        match value {
            Value::Array(items) => {
                let Some(next) = visited.checked_add(items.len()) else {
                    return Err(Refusal::at_stage(
                        ErrorCode::ResourceIncomplete,
                        "visited-field work cannot be represented",
                        Usage {
                            visited_fields: usize::MAX,
                            ..Usage::default()
                        },
                        None,
                        stage,
                    ));
                };
                visited = next;
                stack.extend(items);
            }
            Value::Object(members) => {
                let Some(next) = visited.checked_add(members.len()) else {
                    return Err(Refusal::at_stage(
                        ErrorCode::ResourceIncomplete,
                        "visited-field work cannot be represented",
                        Usage {
                            visited_fields: usize::MAX,
                            ..Usage::default()
                        },
                        None,
                        stage,
                    ));
                };
                visited = next;
                stack.extend(members.values());
            }
            Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
        }
        if visited > limit {
            return Err(Refusal::at_stage(
                ErrorCode::ResourceIncomplete,
                "visited-field work exceeds the effective ceiling",
                Usage {
                    visited_fields: visited,
                    ..Usage::default()
                },
                None,
                stage,
            ));
        }
    }
    Ok(visited)
}

fn check_semantic(usage: Usage, limits: Limits, stage: ProtocolStage) -> Result<()> {
    if usage.claims > limits.max_claims
        || usage.premises > limits.max_premises
        || usage.support_records > limits.max_support_records
        || usage.limitations > limits.max_limitations
        || usage.lineage_nodes > limits.max_lineage_nodes
    {
        return Err(Refusal::at_stage(
            ErrorCode::ResourceIncomplete,
            "semantic work exceeds an effective ceiling",
            usage,
            None,
            stage,
        ));
    }
    Ok(())
}
