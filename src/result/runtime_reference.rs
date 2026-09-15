// SPDX-License-Identifier: AGPL-3.0-or-later
//! Reversible carrier for an authority-qualified opaque runtime subject.
//!
//! Canonical protocol-result fields are strings, whereas QObs runtime subject
//! kind and identity values are intentionally opaque bytes. This module owns
//! only the result-field carrier; [`quire_observation::QualifiedSubject`]
//! remains the owner type at every producer boundary.

use core::fmt;

use quire_observation::QualifiedSubject;
use serde_json::{json, Value};

use crate::digest::jcs::canonicalize;

/// Fixed versioned prefix for the Protocol result-field carrier.
pub const PREFIX: &str = "quire-runtime-v1:";

/// Typed reason why an offered runtime-reference carrier is not reversible.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// The carrier did not use the selected versioned prefix.
    Prefix,
    /// The encoded payload was not even-length lowercase hexadecimal.
    Hex,
    /// The decoded bytes were not one JSON object.
    Json,
    /// The decoded object did not have the complete selected shape.
    Shape,
    /// The decoded JSON was not its exact RFC 8785 canonical encoding.
    NonCanonical,
    /// The decoded authority, kind, or identity differs from the selected subject.
    Mismatch,
    /// The Protocol canonicalizer could not encode the carrier object.
    Canonicalization,
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Prefix => "runtime reference has an unsupported prefix",
            Self::Hex => "runtime reference payload is not lowercase hexadecimal",
            Self::Json => "runtime reference payload is not one JSON object",
            Self::Shape => "runtime reference payload is missing a required member",
            Self::NonCanonical => "runtime reference payload is not canonical",
            Self::Mismatch => "runtime reference payload differs from the selected subject",
            Self::Canonicalization => "runtime reference payload cannot be canonicalized",
        })
    }
}

impl std::error::Error for Error {}

/// Encode one QObs owner subject into the selected string-only result carrier.
pub fn encode_subject(subject: &QualifiedSubject) -> Result<String, Error> {
    let canonical = canonical_subject(subject)?;
    Ok(format!("{PREFIX}{}", hex(canonical.as_bytes())))
}

/// Validate that `encoded` is the exact canonical carrier of `subject`.
///
/// The function deliberately returns no locally reconstructed subject. Callers
/// keep using QObs's owner type; the carrier only proves byte-exact reversible
/// retention in a Protocol result field.
pub fn validate_subject(encoded: &str, subject: &QualifiedSubject) -> Result<(), Error> {
    let payload = encoded.strip_prefix(PREFIX).ok_or(Error::Prefix)?;
    let decoded = unhex(payload)?;
    let text = core::str::from_utf8(&decoded).map_err(|_| Error::Json)?;
    let value: Value = serde_json::from_str(text).map_err(|_| Error::Json)?;
    if !complete_shape(&value) {
        return Err(Error::Shape);
    }
    let object = value
        .as_object()
        .expect("complete_shape accepts only an object");
    for member in ["identityHex", "kindHex"] {
        let bytes = object
            .get(member)
            .and_then(Value::as_str)
            .expect("complete_shape requires each opaque-byte member");
        unhex(bytes)?;
    }
    let canonical = canonicalize(&value).map_err(|_| Error::Canonicalization)?;
    if canonical.as_bytes() != decoded {
        return Err(Error::NonCanonical);
    }
    if canonical != canonical_subject(subject)? {
        return Err(Error::Mismatch);
    }
    Ok(())
}

fn canonical_subject(subject: &QualifiedSubject) -> Result<String, Error> {
    let authority =
        serde_json::to_value(subject.authority()).map_err(|_| Error::Canonicalization)?;
    let value = json!({
        "authority": authority,
        "identityHex": hex(subject.identity().as_bytes()),
        "kindHex": hex(subject.kind().as_bytes()),
    });
    canonicalize(&value).map_err(|_| Error::Canonicalization)
}

fn complete_shape(value: &Value) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };
    if object.len() != 3 {
        return false;
    }
    let Some(authority) = object.get("authority").and_then(Value::as_object) else {
        return false;
    };
    let Some(revision) = authority.get("bundleRevision").and_then(Value::as_object) else {
        return false;
    };
    let Some(digest) = authority.get("digest").and_then(Value::as_object) else {
        return false;
    };
    object
        .get("identityHex")
        .and_then(Value::as_str)
        .is_some_and(|value| !value.is_empty())
        && object
            .get("kindHex")
            .and_then(Value::as_str)
            .is_some_and(|value| !value.is_empty())
        && authority
            .get("bundleIdentity")
            .and_then(Value::as_str)
            .is_some_and(|value| !value.is_empty())
        && revision
            .get("namespace")
            .and_then(Value::as_str)
            .is_some_and(|value| !value.is_empty())
        && revision
            .get("value")
            .and_then(Value::as_str)
            .is_some_and(|value| !value.is_empty())
        && digest
            .get("algorithm")
            .and_then(Value::as_str)
            .is_some_and(|value| !value.is_empty())
        && digest
            .get("domain")
            .and_then(Value::as_str)
            .is_some_and(|value| !value.is_empty())
        && digest
            .get("value")
            .and_then(Value::as_str)
            .is_some_and(|value| !value.is_empty())
        && digest
            .get("version")
            .and_then(Value::as_str)
            .is_some_and(|value| !value.is_empty())
        && object
            .keys()
            .all(|key| matches!(key.as_str(), "authority" | "identityHex" | "kindHex"))
        && authority
            .keys()
            .all(|key| matches!(key.as_str(), "bundleIdentity" | "bundleRevision" | "digest"))
        && revision
            .keys()
            .all(|key| matches!(key.as_str(), "namespace" | "value"))
        && digest
            .keys()
            .all(|key| matches!(key.as_str(), "algorithm" | "domain" | "value" | "version"))
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len().saturating_mul(2));
    for byte in bytes {
        encoded.push(char::from(DIGITS[usize::from(byte >> 4)]));
        encoded.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    encoded
}

fn unhex(value: &str) -> Result<Vec<u8>, Error> {
    let bytes = value.as_bytes();
    let (pairs, remainder) = bytes.as_chunks::<2>();
    if pairs.is_empty() || !remainder.is_empty() {
        return Err(Error::Hex);
    }
    let mut decoded = Vec::with_capacity(pairs.len());
    for [first, second] in pairs {
        let high = nibble(*first).ok_or(Error::Hex)?;
        let low = nibble(*second).ok_or(Error::Hex)?;
        decoded.push((high << 4) | low);
    }
    Ok(decoded)
}

const fn nibble(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        _ => None,
    }
}
