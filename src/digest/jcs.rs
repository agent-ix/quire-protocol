// SPDX-License-Identifier: AGPL-3.0-or-later
//! The consumer's single RFC 8785 (JCS) result-domain digest layer.
//!
//! This domain covers **protocol results only**. Nothing here accepts a
//! producer-owned reference: [`crate::digest::producer::ProducerDigestRef`]
//! exposes no value this module will consume, and this module exposes no value
//! that can be copied back into a producer slot without passing the
//! cross-domain guard.
//!
//! The canonicalisation is deliberately narrow. RFC 8785 number serialisation
//! is the ECMAScript `Number::toString` algorithm, which this chunk does not
//! implement; a JSON number is therefore **refused** rather than canonicalised
//! under an approximation. A silently wrong canonical form would produce a
//! wrong digest that still looks well formed, which is the exact failure this
//! crate exists to refuse.

use std::fmt;

use serde_json::{Map, Value};

use crate::digest::producer::DigestDomain;
use crate::ByteDigest;

/// A value that cannot be canonicalised under this chunk's RFC 8785 subset.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JcsError {
    /// A JSON number was offered; ECMAScript number serialisation is not
    /// implemented here and is never approximated.
    UnsupportedNumber(String),
    /// The canonical byte length cannot be represented by `usize`.
    LengthOverflow,
}

impl fmt::Display for JcsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedNumber(value) => {
                write!(formatter, "RFC 8785 number serialisation is not implemented in this crate; refused {value}")
            }
            Self::LengthOverflow => {
                formatter.write_str("RFC 8785 canonical byte length overflowed")
            }
        }
    }
}

impl std::error::Error for JcsError {}

/// Serialise a value in RFC 8785 canonical form.
///
/// Object members are emitted in ascending order of their UTF-16 code-unit
/// sequences, with no insignificant whitespace, and strings carry the minimal
/// escaping RFC 8785 mandates.
pub fn canonicalize(value: &Value) -> Result<String, JcsError> {
    let mut out = String::new();
    write_value(value, &mut out)?;
    Ok(out)
}

/// Compute the exact canonical UTF-8 byte length without allocating the output.
///
/// Member order does not affect encoded length, so this pass needs no sorting
/// buffer. A producer can therefore precheck its output-byte ceiling before
/// the canonical output allocation begins.
pub fn canonical_len(value: &Value) -> Result<usize, JcsError> {
    match value {
        Value::Null | Value::Bool(true) => Ok(4),
        Value::Bool(false) => Ok(5),
        Value::Number(number) => Err(JcsError::UnsupportedNumber(number.to_string())),
        Value::String(text) => string_len(text),
        Value::Array(items) => {
            let mut length = 2usize;
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    length = checked_add(length, 1)?;
                }
                length = checked_add(length, canonical_len(item)?)?;
            }
            Ok(length)
        }
        Value::Object(members) => {
            let mut length = 2usize;
            for (index, (key, member)) in members.iter().enumerate() {
                if index > 0 {
                    length = checked_add(length, 1)?;
                }
                length = checked_add(length, string_len(key)?)?;
                length = checked_add(length, 1)?;
                length = checked_add(length, canonical_len(member)?)?;
            }
            Ok(length)
        }
    }
}

fn string_len(text: &str) -> Result<usize, JcsError> {
    let mut length = 2usize;
    for character in text.chars() {
        let encoded = match character {
            '"' | '\\' | '\u{8}' | '\u{c}' | '\n' | '\r' | '\t' => 2,
            control if (control as u32) < 0x20 => 6,
            other => other.len_utf8(),
        };
        length = checked_add(length, encoded)?;
    }
    Ok(length)
}

fn checked_add(left: usize, right: usize) -> Result<usize, JcsError> {
    left.checked_add(right).ok_or(JcsError::LengthOverflow)
}

fn write_value(value: &Value, out: &mut String) -> Result<(), JcsError> {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(true) => out.push_str("true"),
        Value::Bool(false) => out.push_str("false"),
        Value::Number(number) => return Err(JcsError::UnsupportedNumber(number.to_string())),
        Value::String(text) => write_string(text, out),
        Value::Array(items) => {
            out.push('[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                write_value(item, out)?;
            }
            out.push(']');
        }
        Value::Object(members) => write_object(members, out)?,
    }
    Ok(())
}

fn write_object(members: &Map<String, Value>, out: &mut String) -> Result<(), JcsError> {
    let mut ordered: Vec<(&String, &Value)> = members.iter().collect();
    ordered.sort_by_key(|(key, _)| utf16_units(key));
    out.push('{');
    for (index, (key, member)) in ordered.into_iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        write_string(key, out);
        out.push(':');
        write_value(member, out)?;
    }
    out.push('}');
    Ok(())
}

fn utf16_units(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

fn write_string(text: &str, out: &mut String) {
    out.push('"');
    for character in text.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            control if (control as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", control as u32));
            }
            other => out.push(other),
        }
    }
    out.push('"');
}

/// A digest in the consumer's protocol-result domain.
///
/// Constructing one always hashes a canonical form this crate produced. There
/// is no constructor that adopts an offered text without the cross-domain guard
/// in [`crate::inventory`] deciding the value belongs to this domain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResultDigest(ByteDigest);

impl ResultDigest {
    /// The one domain a `ResultDigest` ever belongs to.
    pub const DOMAIN: DigestDomain = DigestDomain::VerificationJcs;

    /// Digest a protocol result over its RFC 8785 canonical form.
    pub fn of_result(value: &Value) -> Result<Self, JcsError> {
        Ok(Self(ByteDigest::of(canonicalize(value)?.as_bytes())))
    }

    /// The canonical text of this result-domain value.
    ///
    /// Prefixed `sha256-jcs:`, which the published vocabulary requires for this
    /// domain and which a raw `sha256` value may not substitute for. The
    /// producer's own `Display` renders `sha256:`, so this does not delegate to
    /// it: rendering both domains identically is what let a cross-domain value
    /// pass the algorithm-label test as a merely divergent one.
    pub fn text(&self) -> String {
        self.to_string()
    }
}

impl fmt::Display for ResultDigest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(DigestDomain::VerificationJcs.algorithm_label())?;
        formatter.write_str(":")?;
        fmt::LowerHex::fmt(&self.0, formatter)
    }
}
