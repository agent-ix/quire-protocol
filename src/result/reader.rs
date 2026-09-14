// SPDX-License-Identifier: AGPL-3.0-or-later
//! Strict bounded hostile-byte reader for canonical protocol results.

use serde_json::{Map, Value};
use std::sync::OnceLock;

use crate::digest::jcs::{canonicalize, ResultDigest};

use super::record::{produce, Input, Limits, Usage, ValidatedResult, CONTRACT, SCHEMA_BYTES};
use super::{ErrorCode, Refusal, Result};

static RESULT_SCHEMA_VALIDATOR: OnceLock<Option<jsonschema::JSONSchema>> = OnceLock::new();

/// Independent selections required to accept result bytes.
#[derive(Clone, Debug)]
pub struct Expected<'a> {
    /// Complete semantic input from independently validated owner views.
    pub input: Input<'a>,
    /// Enclosing non-self-referential digest carried outside the result bytes.
    pub enclosing_digest: &'a str,
}

/// Strict-read canonical result bytes against independent selections.
pub fn read(bytes: &[u8], expected: Expected<'_>, limits: Limits) -> Result<ValidatedResult> {
    let effective = limits.effective();
    let observed = preflight(bytes, effective, true)?;
    let value: Value = serde_json::from_slice(bytes).map_err(|_| {
        Refusal::new(
            ErrorCode::InvalidJson,
            "result is not one valid closed JSON value",
            observed,
            None,
        )
    })?;
    let object = value.as_object().ok_or_else(|| {
        Refusal::new(
            ErrorCode::InvalidJson,
            "result root is not an object",
            observed,
            None,
        )
    })?;
    if object.get("contractVersion").and_then(Value::as_str) != Some(CONTRACT)
        || object.get("kind").and_then(Value::as_str) != Some("ProtocolResult")
    {
        return Err(Refusal::new(
            ErrorCode::ContractMismatch,
            "result contractVersion or kind differs from the selected contract",
            observed,
            None,
        ));
    }
    if !schema_accepts(&value, SCHEMA_BYTES, &RESULT_SCHEMA_VALIDATOR) {
        return Err(Refusal::new(
            ErrorCode::InvalidJson,
            "result is not a closed instance of the selected schema",
            observed,
            None,
        ));
    }
    let canonical = canonicalize(&value).map_err(|_| {
        Refusal::new(
            ErrorCode::NonCanonical,
            "result contains a JSON number outside the selected JCS subset",
            observed,
            None,
        )
    })?;
    if canonical.as_bytes() != bytes {
        return Err(Refusal::new(
            ErrorCode::NonCanonical,
            "result has duplicate, reordered, trailing, or noncanonical bytes",
            observed,
            None,
        ));
    }
    let carried_id = object
        .get("resultId")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            Refusal::new(
                ErrorCode::IdentityMismatch,
                "resultId is missing or not a string",
                observed,
                None,
            )
        })?;
    let mut without_id = object.clone();
    without_id.remove("resultId");
    let preimage = Value::Object(Map::from_iter([
        (
            "identityVersion".to_owned(),
            Value::String("quire.protocol.result-identity/v1-draft.1".to_owned()),
        ),
        ("result".to_owned(), Value::Object(without_id)),
    ]));
    let derived_id = ResultDigest::of_result(&preimage)
        .map_err(|_| {
            Refusal::new(
                ErrorCode::IdentityMismatch,
                "result identity preimage cannot be canonicalized",
                observed,
                Some(carried_id),
            )
        })?
        .text();
    if carried_id != derived_id {
        return Err(Refusal::new(
            ErrorCode::IdentityMismatch,
            "resultId differs from the FR-286 preimage",
            observed,
            Some(carried_id),
        ));
    }
    let digest = ResultDigest::of_result(&value)
        .map_err(|_| {
            Refusal::new(
                ErrorCode::DigestMismatch,
                "result digest preimage cannot be canonicalized",
                observed,
                Some(carried_id),
            )
        })?
        .text();
    if digest != expected.enclosing_digest {
        return Err(Refusal::new(
            ErrorCode::DigestMismatch,
            "enclosing result digest does not match canonical bytes",
            observed,
            Some(carried_id),
        ));
    }
    let mut expected_document = produce(expected.input, limits)?;
    if expected_document.result_id() != carried_id {
        return Err(Refusal::new(
            ErrorCode::IdentityMismatch,
            "result identity differs from independently selected facts",
            observed,
            Some(carried_id),
        ));
    }
    if expected_document.bytes() != bytes {
        return Err(Refusal::new(
            ErrorCode::InvalidSelection,
            "result fields differ from independently selected owner facts",
            observed,
            Some(carried_id),
        ));
    }
    expected_document.usage = observed;
    Ok(ValidatedResult {
        document: expected_document,
    })
}

pub(crate) fn preflight_output(bytes: &[u8], limits: Limits, semantic: Usage) -> Result<Usage> {
    let mut usage = preflight(bytes, limits, false)?;
    usage.claims = semantic.claims;
    usage.premises = semantic.premises;
    usage.support_records = semantic.support_records;
    usage.limitations = semantic.limitations;
    usage.lineage_nodes = semantic.lineage_nodes;
    check_semantic(usage, limits)?;
    Ok(usage)
}

pub(crate) fn preflight_external(bytes: &[u8], limits: Limits) -> Result<Usage> {
    preflight(bytes, limits, true)
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

fn preflight(bytes: &[u8], limits: Limits, input: bool) -> Result<Usage> {
    let byte_limit = if input {
        limits.max_input_bytes
    } else {
        limits.max_output_bytes
    };
    if bytes.len() > byte_limit {
        return Err(Refusal::new(
            ErrorCode::ResourceIncomplete,
            "wire bytes exceed the effective ceiling",
            Usage {
                wire_bytes: bytes.len(),
                ..Usage::default()
            },
            None,
        ));
    }
    if core::str::from_utf8(bytes).is_err() {
        return Err(Refusal::new(
            ErrorCode::InvalidJson,
            "result bytes are not UTF-8",
            Usage {
                wire_bytes: bytes.len(),
                ..Usage::default()
            },
            None,
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
                string_bytes = string_bytes.saturating_add(1);
            } else if byte == b'\\' {
                escaped = true;
                string_bytes = string_bytes.saturating_add(1);
            } else if byte == b'"' {
                in_string = false;
                max_string = max_string.max(string_bytes);
            } else {
                string_bytes = string_bytes.saturating_add(1);
            }
            continue;
        }
        match byte {
            b'"' => {
                in_string = true;
                string_bytes = 0;
            }
            b'{' | b'[' => {
                depth = depth.saturating_add(1);
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
        return Err(Refusal::new(
            ErrorCode::ResourceIncomplete,
            "JSON depth or string size exceeds the effective ceiling",
            usage,
            None,
        ));
    }
    let value: Value = serde_json::from_slice(bytes).map_err(|_| {
        Refusal::new(
            ErrorCode::InvalidJson,
            "wire bytes are not valid JSON",
            usage,
            None,
        )
    })?;
    let visited = visited_fields(&value, limits.max_visited_fields)?;
    Ok(Usage {
        visited_fields: visited,
        ..usage
    })
}

fn visited_fields(root: &Value, limit: usize) -> Result<usize> {
    let mut stack = vec![root];
    let mut visited = 0usize;
    while let Some(value) = stack.pop() {
        match value {
            Value::Array(items) => {
                visited = visited.saturating_add(items.len());
                stack.extend(items);
            }
            Value::Object(members) => {
                visited = visited.saturating_add(members.len());
                stack.extend(members.values());
            }
            Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
        }
        if visited > limit {
            return Err(Refusal::new(
                ErrorCode::ResourceIncomplete,
                "visited-field work exceeds the effective ceiling",
                Usage {
                    visited_fields: visited,
                    ..Usage::default()
                },
                None,
            ));
        }
    }
    Ok(visited)
}

fn check_semantic(usage: Usage, limits: Limits) -> Result<()> {
    if usage.claims > limits.max_claims
        || usage.premises > limits.max_premises
        || usage.support_records > limits.max_support_records
        || usage.limitations > limits.max_limitations
        || usage.lineage_nodes > limits.max_lineage_nodes
    {
        return Err(Refusal::new(
            ErrorCode::ResourceIncomplete,
            "semantic work exceeds an effective ceiling",
            usage,
            None,
        ));
    }
    Ok(())
}
