// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-005 (partial, chunk C-22): the six-disposition claim record.
//!
//! Covers the FR-005-AC-1 *shape* obligation — six separate dispositions held
//! simultaneously, each independently observable and independently assertable,
//! with no aggregate verdict. It does not cover FR-005-AC-1's end-to-end
//! scenario (a monitorable-but-unprojectable protocol carrying a real finite
//! conformance result), nor AC-2, AC-3, AC-4 or AC-5; those need premise
//! retention (C-23), dispatch (C-24) and a conformance evaluator (C-20/C-25).
//!
//! # Why exhaustive enumeration and not random sampling
//!
//! The record space is finite and small: 5 states over 6 claims = 5^6 = 15 625
//! records. Enumerating it is a proof over the whole domain; sampling it would
//! only be evidence. `Cargo.toml` is COLLISION-7 in the chunk plan — exactly one
//! chunk per wave may add a dependency — so no `proptest` harness may be
//! introduced here, and the repository has none today.
//!
//! # How "no aggregate verdict" is proved rather than asserted
//!
//! Three tests together, none of which is a restatement of the claim:
//!
//! * `tc_005_no_aggregate_verdict_field` reads this repository's own
//!   `src/claims/mod.rs` and enumerates the public surface of
//!   `ClaimDispositions` — its fields, its constants and its inherent methods.
//!   It requires that surface to equal the audited table below exactly, so a new
//!   public accessor fails the test until it is classified, and it requires no
//!   public name to use verdict vocabulary.
//! * `tc_005_single_claim_observations_read_one_field` proves every accessor
//!   audited as single-claim is a function of that claim's disposition alone:
//!   holding the claim's state fixed and ranging the other five over all 15 625
//!   records leaves the answer constant. A roll-up smuggled through a `Claim`
//!   parameter would vary and fail here.
//! * `tc_005_all_claim_observations_are_injective` proves every accessor audited
//!   as all-claims returns 15 625 distinct values over the 15 625 records. An
//!   accessor that collapsed the six into one value could not be injective, so
//!   adding one fails this test.
//!
//! The audit table is the bridge: the source read makes it complete, and the two
//! behavioural tests make each entry mean something. Stable Rust has no
//! reflection over inherent methods, so the source read is the coverage
//! mechanism; that limit is stated here rather than hidden.

use std::collections::HashSet;

use ix_trace_rs::trace;
use quire_protocol::claims::{
    Claim, ClaimDisposition, ClaimDispositions, ClaimRefusal, CompositionDisposition,
    ConformanceDisposition, MonitorabilityDisposition, ProjectionDisposition,
    RealizabilityDisposition, RefinementDisposition,
};

/// The audited module's own source, read for the public-surface audit.
const CLAIMS_SOURCE: &str = include_str!("../src/claims/mod.rs");

/// This test file's own source, read so that every audited accessor can be
/// required to be *exercised* by the behavioural test its classification names.
///
/// Stable Rust has no reflection over inherent methods, so classifying a method
/// in [`AUDITED_METHODS`] cannot by itself make a behavioural test call it. This
/// second source read closes that gap: an accessor classified `SingleClaim` must
/// appear inside [`single_claim_observation`], and one classified `AllClaims`
/// must appear inside `tc_005_all_claim_observations_are_injective`. Adding a
/// verdict therefore requires adding it to the table *and* wiring it into the
/// test that would then reject it.
const TEST_SOURCE: &str = include_str!("tc_005_claim_dispositions.rs");

/// Every file of the `claims` module.
///
/// The audit reads all of them. It used to read only `mod.rs`, and `premises.rs`
/// and `dispatch.rs` are declared by `mod.rs`, so a second `impl ClaimDispositions`
/// in either carried a verdict one file away from the audit — and the test went
/// on asserting there was exactly one inherent impl block.
const MODULE_SOURCES: &[(&str, &str)] = &[
    ("src/claims/mod.rs", CLAIMS_SOURCE),
    (
        "src/claims/premises.rs",
        include_str!("../src/claims/premises.rs"),
    ),
    (
        "src/claims/dispatch.rs",
        include_str!("../src/claims/dispatch.rs"),
    ),
];

/// Number of states every disposition declares.
const STATES: usize = 5;

/// Number of claims the record holds.
const CLAIMS: usize = 6;

// ---------------------------------------------------------------------------
// The audited public surface of `ClaimDispositions`
// ---------------------------------------------------------------------------

/// How a public accessor is allowed to read the record.
///
/// There is no third category. An accessor is either scoped to one claim, in
/// which case it must read that claim's field alone, or it ranges over all six,
/// in which case it must be injective. A verdict is neither.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Surface {
    /// Takes a [`Claim`] and reads only that claim's disposition.
    SingleClaim,
    /// Ranges over all six dispositions and preserves all six.
    AllClaims,
}

/// Every public inherent method of `ClaimDispositions`, with its classification.
const AUDITED_METHODS: &[(&str, Surface)] = &[
    ("state_code", Surface::SingleClaim),
    ("establishes", Surface::SingleClaim),
    ("refusal", Surface::SingleClaim),
    ("refusal_per_claim", Surface::AllClaims),
    ("state_codes", Surface::AllClaims),
];

/// Every public associated constant of `ClaimDispositions`.
///
/// A constant is a value, not an observation of a record, so it cannot be a
/// verdict over the six. It is audited so that the surface stays enumerated.
const AUDITED_CONSTANTS: &[&str] = &["NOTHING_REQUESTED"];

/// The six public fields, in declared order.
const AUDITED_FIELDS: &[&str] = &[
    "conformance",
    "monitorability",
    "projection",
    "realizability",
    "refinement",
    "composition",
];

/// Name segments that mark a value as a roll-up of the six.
///
/// Scoped to `ClaimDispositions`' own public surface, which is the surface a
/// caller could mistake for a verdict. Registry names elsewhere in the module
/// (`Claim::ALL`, `ClaimDisposition::VALUES`) are enumerations of the domain,
/// not observations of a record, and are out of scope.
const VERDICT_VOCABULARY: &[&str] = &[
    "verdict",
    "overall",
    "aggregate",
    "summary",
    "summarise",
    "summarize",
    "rollup",
    "score",
    "pass",
    "passed",
    "passes",
    "ok",
    "success",
    "successful",
    "succeeded",
    "status",
    "valid",
    "result",
    "total",
    "any",
    "all",
    "every",
    "complete",
    "final",
    "is",
    "outcome",
    "good",
    "clean",
];

// ---------------------------------------------------------------------------
// Enumeration helpers
// ---------------------------------------------------------------------------

/// Every record in the 5^6 space, in a stable order.
fn all_records() -> Vec<ClaimDispositions> {
    let mut records = Vec::with_capacity(STATES.pow(CLAIMS as u32));
    let mut digits = [0usize; CLAIMS];
    loop {
        records.push(record_from(digits));
        let mut position = 0;
        loop {
            if position == CLAIMS {
                return records;
            }
            digits[position] += 1;
            if digits[position] < STATES {
                break;
            }
            digits[position] = 0;
            position += 1;
        }
    }
}

/// Build the record whose claim `i` sits at state index `digits[i]`.
fn record_from(digits: [usize; CLAIMS]) -> ClaimDispositions {
    let mut record = ClaimDispositions::NOTHING_REQUESTED;
    for claim in Claim::ALL {
        record = with_state(record, claim, digits[claim.index()]);
    }
    record
}

/// Replace exactly one claim's state, leaving the other five untouched.
fn with_state(mut record: ClaimDispositions, claim: Claim, state: usize) -> ClaimDispositions {
    match claim {
        Claim::Conformance => record.conformance = ConformanceDisposition::VALUES[state],
        Claim::Monitorability => {
            record.monitorability = MonitorabilityDisposition::VALUES[state];
        }
        Claim::Projection => record.projection = ProjectionDisposition::VALUES[state],
        Claim::Realizability => record.realizability = RealizabilityDisposition::VALUES[state],
        Claim::Refinement => record.refinement = RefinementDisposition::VALUES[state],
        Claim::Composition => record.composition = CompositionDisposition::VALUES[state],
    }
    record
}

/// The stable code of one claim's state index, read from that claim's own type.
fn state_code_of(claim: Claim, state: usize) -> &'static str {
    match claim {
        Claim::Conformance => ConformanceDisposition::VALUES[state].code(),
        Claim::Monitorability => MonitorabilityDisposition::VALUES[state].code(),
        Claim::Projection => ProjectionDisposition::VALUES[state].code(),
        Claim::Realizability => RealizabilityDisposition::VALUES[state].code(),
        Claim::Refinement => RefinementDisposition::VALUES[state].code(),
        Claim::Composition => CompositionDisposition::VALUES[state].code(),
    }
}

/// Whether one claim's state index establishes that claim.
fn state_establishes(claim: Claim, state: usize) -> bool {
    match claim {
        Claim::Conformance => ConformanceDisposition::VALUES[state].establishes(),
        Claim::Monitorability => MonitorabilityDisposition::VALUES[state].establishes(),
        Claim::Projection => ProjectionDisposition::VALUES[state].establishes(),
        Claim::Realizability => RealizabilityDisposition::VALUES[state].establishes(),
        Claim::Refinement => RefinementDisposition::VALUES[state].establishes(),
        Claim::Composition => CompositionDisposition::VALUES[state].establishes(),
    }
}

/// The claim a refusal belongs to, matched exhaustively with no `_` arm.
///
/// A seventh [`ClaimRefusal`] variant fails to compile here rather than being
/// absorbed by a wildcard.
fn refusal_claim(refusal: ClaimRefusal) -> Claim {
    match refusal {
        ClaimRefusal::Conformance(_) => Claim::Conformance,
        ClaimRefusal::Monitorability(_) => Claim::Monitorability,
        ClaimRefusal::Projection(_) => Claim::Projection,
        ClaimRefusal::Realizability(_) => Claim::Realizability,
        ClaimRefusal::Refinement(_) => Claim::Refinement,
        ClaimRefusal::Composition(_) => Claim::Composition,
    }
}

/// Every single-claim observation the audited surface offers, as one value.
fn single_claim_observation(
    record: ClaimDispositions,
    claim: Claim,
) -> (&'static str, bool, Option<ClaimRefusal>) {
    (
        record.state_code(claim),
        record.establishes(claim),
        record.refusal(claim),
    )
}

// ---------------------------------------------------------------------------
// Source-text audit helpers
// ---------------------------------------------------------------------------

/// One Rust source file with comments, string literals and char literals
/// replaced by spaces of the same length.
///
/// Every parser below reads this, never the raw text. Without it the audit is
/// blinded by ordinary source: a `"` inside a char literal shifts quote
/// pairing, and a `}` inside a string literal ends a block early. Both were
/// demonstrated against the previous version — a literal containing a
/// column-zero `}` placed in the last audited method hid every method after it.
///
/// Lengths are preserved so that byte offsets still line up with the original.
fn stripped(source: &str) -> String {
    let characters: Vec<char> = source.chars().collect();
    let mut out = String::with_capacity(source.len());
    let mut index = 0;
    let mut block_depth = 0usize;

    let blank = |text: &str, out: &mut String| {
        for character in text.chars() {
            out.push(if character == '\n' { '\n' } else { ' ' });
        }
    };

    while index < characters.len() {
        let character = characters[index];
        if block_depth > 0 {
            if character == '/' && characters.get(index + 1) == Some(&'*') {
                block_depth += 1;
                blank("  ", &mut out);
                index += 2;
            } else if character == '*' && characters.get(index + 1) == Some(&'/') {
                block_depth -= 1;
                blank("  ", &mut out);
                index += 2;
            } else {
                blank(&character.to_string(), &mut out);
                index += 1;
            }
            continue;
        }
        match character {
            '/' if characters.get(index + 1) == Some(&'/') => {
                while index < characters.len() && characters[index] != '\n' {
                    blank(&characters[index].to_string(), &mut out);
                    index += 1;
                }
            }
            '/' if characters.get(index + 1) == Some(&'*') => {
                block_depth = 1;
                blank("  ", &mut out);
                index += 2;
            }
            '\'' => {
                // A char literal, a lifetime or a label. Only a char literal
                // can hold a quote or a brace.
                let closes = if characters.get(index + 1) == Some(&'\\') {
                    characters.get(index + 3) == Some(&'\'')
                } else {
                    characters.get(index + 2) == Some(&'\'')
                };
                if closes {
                    let width = if characters.get(index + 1) == Some(&'\\') {
                        4
                    } else {
                        3
                    };
                    for _ in 0..width {
                        out.push(' ');
                    }
                    index += width;
                } else {
                    out.push(character);
                    index += 1;
                }
            }
            '"' => {
                out.push(' ');
                index += 1;
                while index < characters.len() {
                    match characters[index] {
                        '\\' => {
                            blank("  ", &mut out);
                            index += 2;
                        }
                        '"' => {
                            out.push(' ');
                            index += 1;
                            break;
                        }
                        other => {
                            blank(&other.to_string(), &mut out);
                            index += 1;
                        }
                    }
                }
            }
            other => {
                out.push(other);
                index += 1;
            }
        }
    }
    out
}

/// Every `impl` header in a stripped source, with its brace-matched body.
///
/// The header is everything between `impl` and its opening brace, with runs of
/// whitespace collapsed, so a header rustfmt wrapped across lines reads the
/// same as a single-line one. That wrapping was a live bypass: a trait name
/// long enough to make rustfmt split `impl Trait` from `for ClaimDispositions`
/// beat a check that required both on one line — and that split is rustfmt's
/// own canonical output, not a contrivance.
fn impl_blocks(source: &str) -> Vec<(String, String)> {
    let characters: Vec<char> = source.chars().collect();
    let mut blocks = Vec::new();
    let mut index = 0;

    while let Some(offset) = source[index..].find("impl") {
        let at = index + offset;
        let before_is_boundary = at == 0
            || !characters
                .get(at - 1)
                .is_some_and(|c| c.is_alphanumeric() || *c == '_');
        let after_is_boundary = !characters
            .get(at + 4)
            .is_some_and(|c| c.is_alphanumeric() || *c == '_');
        index = at + 4;
        if !before_is_boundary || !after_is_boundary {
            continue;
        }
        let Some(open_offset) = source[at..].find('{') else {
            continue;
        };
        let open = at + open_offset;
        let header: String = source[at + 4..open]
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");

        let mut depth = 0usize;
        let mut close = open;
        for (position, character) in source[open..].char_indices() {
            match character {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        close = open + position;
                        break;
                    }
                }
                _ => {}
            }
        }
        blocks.push((header, source[open + 1..close].to_owned()));
        index = close.max(index);
    }
    blocks
}

/// Names introduced by a `pub` function item, whatever modifiers precede `fn`.
///
/// `pub const fn`, `pub unsafe fn` and `pub extern "C" fn` are all public
/// functions. The previous parser matched the literal prefix `pub fn `, so
/// `pub const fn established_everywhere` was invisible to it — and the constant
/// parser then mis-captured it and reported a "constant" named
/// `fn established_everywhere(self) -> bool {`, which points a maintainer at
/// the wrong defect.
fn public_fn_names(body: &str) -> Vec<String> {
    let mut names = Vec::new();
    for line in body.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("pub ") else {
            continue;
        };
        let mut words = rest.split_whitespace();
        let mut saw_fn = false;
        for word in words.by_ref() {
            if word == "fn" {
                saw_fn = true;
                break;
            }
            // Only modifiers may sit between `pub` and `fn`.
            if !matches!(word, "const" | "unsafe" | "async" | "extern") && !word.starts_with('"') {
                break;
            }
        }
        if !saw_fn {
            continue;
        }
        if let Some(name) = words.next().and_then(|word| word.split('(').next()) {
            if !name.is_empty() {
                names.push(name.split('<').next().unwrap_or(name).to_owned());
            }
        }
    }
    names
}

/// Names introduced by `pub const` in a block body, excluding `pub const fn`.
fn public_const_names(body: &str) -> Vec<String> {
    body.lines()
        .filter_map(|line| line.trim().strip_prefix("pub const "))
        .filter(|rest| !rest.starts_with("fn "))
        .filter_map(|rest| rest.split(':').next())
        .map(|name| name.trim().to_owned())
        .collect()
}

/// Names introduced by `pub <name>:` fields in a block body.
fn public_field_names(body: &str) -> Vec<String> {
    body.lines()
        .map(str::trim)
        .filter_map(|line| line.strip_prefix("pub "))
        .filter(|rest| {
            rest.contains(':') && !rest.starts_with("fn ") && !rest.starts_with("const ")
        })
        .filter_map(|rest| rest.split(':').next())
        .map(str::to_owned)
        .collect()
}

/// Every macro invocation name in a stripped body.
///
/// A macro invocation inside the audited impl block is refused outright. It is
/// the worst bypass found: a `macro_rules!` expanding to a `pub fn` verdict,
/// placed in the very block the audit claims to enumerate, was invisible to
/// every line parser and passed fmt and clippy. Expanding macros is a job for a
/// parser this test does not have, so the audit refuses the construct rather
/// than pretending to read through it.
fn macro_invocations(body: &str) -> Vec<String> {
    let characters: Vec<char> = body.chars().collect();
    let mut found = Vec::new();
    for (index, character) in characters.iter().enumerate() {
        if *character != '!' {
            continue;
        }
        // `!=` is an operator, and a leading `!` is negation.
        if characters.get(index + 1) == Some(&'=') {
            continue;
        }
        let mut start = index;
        while start > 0 && identifier_char(characters[start - 1]) {
            start -= 1;
        }
        if start == index {
            continue;
        }
        found.push(characters[start..index].iter().collect());
    }
    found
}

/// The brace-matched body of the `ClaimDispositions` struct declaration.
fn record_struct_body(source: &str) -> String {
    let header = "pub struct ClaimDispositions {";
    let start = source
        .find(header)
        .expect("the record's struct declaration");
    let open = start + header.len() - 1;
    let mut depth = 0usize;
    for (position, character) in source[open..].char_indices() {
        match character {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return source[open + 1..open + position].to_owned();
                }
            }
            _ => {}
        }
    }
    panic!("unterminated struct body")
}

/// Whether `character` may appear in a Rust identifier.
fn identifier_char(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}

/// The source region from `marker` up to the next column-zero `}`.
fn region<'a>(source: &'a str, marker: &str) -> &'a str {
    let start = source
        .find(marker)
        .unwrap_or_else(|| panic!("region marker not found: {marker}"));
    let rest = &source[start..];
    let end = rest.find("\n}\n").unwrap_or(rest.len());
    &rest[..end]
}

/// The source region a surface classification obliges an accessor to appear in.
///
/// The markers are assembled at run time rather than written as whole literals,
/// because this function's own source is part of [`TEST_SOURCE`]: a literal
/// `fn name(` here would be the first match and the region would be this
/// function instead of the intended one.
fn obliged_region(surface: Surface) -> &'static str {
    let name = match surface {
        Surface::SingleClaim => "single_claim_observation",
        Surface::AllClaims => "tc_005_all_claim_observations_are_injective",
    };
    region(TEST_SOURCE, &format!("fn {name}("))
}

/// Whether a snake- or screaming-snake-case name uses verdict vocabulary.
fn uses_verdict_vocabulary(name: &str) -> bool {
    name.to_ascii_lowercase()
        .split('_')
        .any(|segment| VERDICT_VOCABULARY.contains(&segment))
}

// ---------------------------------------------------------------------------
// FR-005-AC-1 shape: six separate dispositions in one record
// ---------------------------------------------------------------------------

/// The record holds exactly six named fields of six unrelated types, each one
/// the disposition of its own claim.
#[trace("TC-005", "FR-005-AC-1")]
#[test]
fn tc_005_six_dispositions_are_separate_fields() {
    let fields = public_field_names(&record_struct_body(&stripped(CLAIMS_SOURCE)));
    assert_eq!(
        fields, AUDITED_FIELDS,
        "the record must hold exactly the six FR-005 claims, in declared order"
    );

    // Each field's type names its own claim, and the six claims are distinct.
    assert_eq!(ConformanceDisposition::CLAIM, Claim::Conformance);
    assert_eq!(MonitorabilityDisposition::CLAIM, Claim::Monitorability);
    assert_eq!(ProjectionDisposition::CLAIM, Claim::Projection);
    assert_eq!(RealizabilityDisposition::CLAIM, Claim::Realizability);
    assert_eq!(RefinementDisposition::CLAIM, Claim::Refinement);
    assert_eq!(CompositionDisposition::CLAIM, Claim::Composition);

    let claims: HashSet<Claim> = Claim::ALL.into_iter().collect();
    assert_eq!(claims.len(), CLAIMS, "the six claims must be distinct");
    assert_eq!(Claim::ALL.len(), AUDITED_FIELDS.len());

    // Every claim is held simultaneously: one record carries all six at once,
    // and each field answers for its own claim.
    let mut record = ClaimDispositions::NOTHING_REQUESTED;
    for claim in Claim::ALL {
        record = with_state(record, claim, establishing_state(claim));
    }
    for claim in Claim::ALL {
        assert!(
            record.establishes(claim),
            "{} must be established in the all-established record",
            claim.code()
        );
    }
}

/// The declared value sets are the right size and carry no duplicates.
///
/// Stable Rust cannot assert at compile time that `VALUES` lists every variant;
/// this is the runtime half, and the exhaustive matches in `src/claims/mod.rs`
/// are the compile-time half.
#[trace("TC-005", "FR-005-AC-1")]
#[test]
fn tc_005_declared_value_sets_are_exhaustive() {
    for claim in Claim::ALL {
        let codes: HashSet<&'static str> = (0..STATES)
            .map(|state| state_code_of(claim, state))
            .collect();
        assert_eq!(
            codes.len(),
            STATES,
            "{} must declare {STATES} distinct states",
            claim.code()
        );

        // Every state code is prefixed by its own claim's code.
        //
        // `declare_disposition!` takes the claim and the code prefix as two
        // independent arguments, so the claim's name is written twice. The
        // literal table in `tc_005_not_requested_establishes_nothing_and_is_its_own_state`
        // already ties the two together, but only for the `NotRequested` state.
        // This generalises that one row to all five states of all six claims, so
        // the binding is a property of the macro rather than of one hand-written
        // table that a later state could be added without.
        for state in 0..STATES {
            let code = state_code_of(claim, state);
            assert_eq!(
                code.split('/').next(),
                Some(claim.code()),
                "state code {code} is not prefixed by its own claim's code {}",
                claim.code()
            );
        }

        let establishing: Vec<usize> = (0..STATES)
            .filter(|state| state_establishes(claim, *state))
            .collect();
        assert_eq!(
            establishing.len(),
            1,
            "exactly one state of {} may establish it",
            claim.code()
        );
    }

    // Every declared position agrees with the state's own `index`, so
    // `VALUES` is an ordered registry and not an unrelated list.
    for (position, state) in ConformanceDisposition::VALUES.iter().enumerate() {
        assert_eq!(state.index(), position);
    }
    for (position, state) in MonitorabilityDisposition::VALUES.iter().enumerate() {
        assert_eq!(state.index(), position);
    }
    for (position, state) in ProjectionDisposition::VALUES.iter().enumerate() {
        assert_eq!(state.index(), position);
    }
    for (position, state) in RealizabilityDisposition::VALUES.iter().enumerate() {
        assert_eq!(state.index(), position);
    }
    for (position, state) in RefinementDisposition::VALUES.iter().enumerate() {
        assert_eq!(state.index(), position);
    }
    for (position, state) in CompositionDisposition::VALUES.iter().enumerate() {
        assert_eq!(state.index(), position);
    }
    for (position, claim) in Claim::ALL.iter().enumerate() {
        assert_eq!(claim.index(), position);
    }

    assert_eq!(ConformanceDisposition::VALUES.len(), STATES);
    assert_eq!(MonitorabilityDisposition::VALUES.len(), STATES);
    assert_eq!(ProjectionDisposition::VALUES.len(), STATES);
    assert_eq!(RealizabilityDisposition::VALUES.len(), STATES);
    assert_eq!(RefinementDisposition::VALUES.len(), STATES);
    assert_eq!(CompositionDisposition::VALUES.len(), STATES);
}

/// The state index that establishes a claim.
fn establishing_state(claim: Claim) -> usize {
    let found: Vec<usize> = (0..STATES)
        .filter(|state| state_establishes(claim, *state))
        .collect();
    assert_eq!(found.len(), 1);
    found[0]
}

// ---------------------------------------------------------------------------
// Independent observability
// ---------------------------------------------------------------------------

/// Changing one disposition changes that claim's observation and leaves the
/// other five bit-identical, over the whole 5^6 record space.
#[trace("TC-005", "FR-005-AC-1")]
#[test]
fn tc_005_each_disposition_is_separately_observable() {
    let records = all_records();
    assert_eq!(records.len(), STATES.pow(CLAIMS as u32));

    for record in &records {
        for claim in Claim::ALL {
            for state in 0..STATES {
                let mutated = with_state(*record, claim, state);

                assert_eq!(
                    single_claim_observation(mutated, claim),
                    (
                        state_code_of(claim, state),
                        state_establishes(claim, state),
                        mutated.refusal(claim)
                    ),
                    "{} must observe the state it was set to",
                    claim.code()
                );

                for other in Claim::ALL {
                    if other == claim {
                        continue;
                    }
                    assert_eq!(
                        single_claim_observation(mutated, other),
                        single_claim_observation(*record, other),
                        "setting {} changed the observation of {}",
                        claim.code(),
                        other.code()
                    );
                }
            }
        }
    }
}

/// Every one of the twenty-four non-establishing (claim, state) pairs lands on
/// its own typed `ClaimRefusal` variant carrying its own state, and on a code no
/// other pair can produce.
#[trace("TC-005", "FR-005-AC-1")]
#[test]
fn tc_005_each_refusal_is_independently_assertable() {
    let mut seen_codes: HashSet<&'static str> = HashSet::new();

    for claim in Claim::ALL {
        for state in 0..STATES {
            // Start from the record in which every other claim is established,
            // so a refusal cannot be produced by a neighbouring claim.
            let mut record = ClaimDispositions::NOTHING_REQUESTED;
            for other in Claim::ALL {
                record = with_state(record, other, establishing_state(other));
            }
            let record = with_state(record, claim, state);

            let refusal = record.refusal(claim);
            if state_establishes(claim, state) {
                assert_eq!(refusal, None, "{} established but refused", claim.code());
                continue;
            }

            let refusal = refusal.expect("a non-establishing state must refuse");
            assert_eq!(
                refusal_claim(refusal),
                claim,
                "refusal landed on the wrong claim's variant"
            );
            assert_eq!(refusal.claim(), claim);
            assert_eq!(refusal.code(), state_code_of(claim, state));
            assert!(
                seen_codes.insert(refusal.code()),
                "refusal code {} is shared with another claim or state",
                refusal.code()
            );

            // No other claim refuses in this record.
            for other in Claim::ALL {
                if other == claim {
                    continue;
                }
                assert_eq!(
                    record.refusal(other),
                    None,
                    "{} refused while only {} was perturbed",
                    other.code(),
                    claim.code()
                );
            }
        }
    }

    assert_eq!(
        seen_codes.len(),
        CLAIMS * (STATES - 1),
        "every non-establishing (claim, state) pair needs its own code"
    );
}

/// `Unsupported` and `Failed` are distinct facts with distinct codes, per claim
/// and across claims — never a shared "not established".
#[trace("TC-005", "FR-005-AC-1")]
#[test]
fn tc_005_unsupported_and_failed_are_distinct() {
    let pairs: [(ClaimRefusal, ClaimRefusal); CLAIMS] = [
        (
            ClaimRefusal::Conformance(ConformanceDisposition::Unsupported),
            ClaimRefusal::Conformance(ConformanceDisposition::Failed),
        ),
        (
            ClaimRefusal::Monitorability(MonitorabilityDisposition::Unsupported),
            ClaimRefusal::Monitorability(MonitorabilityDisposition::Failed),
        ),
        (
            ClaimRefusal::Projection(ProjectionDisposition::Unsupported),
            ClaimRefusal::Projection(ProjectionDisposition::Failed),
        ),
        (
            ClaimRefusal::Realizability(RealizabilityDisposition::Unsupported),
            ClaimRefusal::Realizability(RealizabilityDisposition::Failed),
        ),
        (
            ClaimRefusal::Refinement(RefinementDisposition::Unsupported),
            ClaimRefusal::Refinement(RefinementDisposition::Failed),
        ),
        (
            ClaimRefusal::Composition(CompositionDisposition::Unsupported),
            ClaimRefusal::Composition(CompositionDisposition::Failed),
        ),
    ];

    let mut codes: HashSet<&'static str> = HashSet::new();
    for (unsupported, failed) in pairs {
        assert_ne!(unsupported, failed);
        assert_ne!(unsupported.code(), failed.code());
        assert_eq!(unsupported.claim(), failed.claim());
        assert!(codes.insert(unsupported.code()));
        assert!(codes.insert(failed.code()));
    }
    assert_eq!(codes.len(), 2 * CLAIMS);
}

/// `NotRequested` is this module's "not required" trap: a genuine observation
/// that establishes nothing, and is neither `Unsupported` nor `Failed`.
#[trace("TC-005", "FR-005-AC-1")]
#[test]
fn tc_005_not_requested_establishes_nothing_and_is_its_own_state() {
    for claim in Claim::ALL {
        let record = ClaimDispositions::NOTHING_REQUESTED;
        assert!(
            !record.establishes(claim),
            "a claim nobody requested was never established: {}",
            claim.code()
        );
        let refusal = record
            .refusal(claim)
            .expect("not-requested must be observable as its own refusal");
        assert_eq!(refusal.claim(), claim);
        assert!(refusal.code().ends_with("/not-requested"));
        assert_ne!(refusal.code(), format!("{}/unsupported", claim.code()));
        assert_ne!(refusal.code(), format!("{}/failed", claim.code()));
    }

    // Every disposition's `NOT_REQUESTED` is that disposition's own
    // not-requested state, and every one of them is in the record above.
    assert_eq!(
        ConformanceDisposition::NOT_REQUESTED,
        ConformanceDisposition::NotRequested
    );
    assert_eq!(
        MonitorabilityDisposition::NOT_REQUESTED,
        MonitorabilityDisposition::NotRequested
    );
    assert_eq!(
        ProjectionDisposition::NOT_REQUESTED,
        ProjectionDisposition::NotRequested
    );
    assert_eq!(
        RealizabilityDisposition::NOT_REQUESTED,
        RealizabilityDisposition::NotRequested
    );
    assert_eq!(
        RefinementDisposition::NOT_REQUESTED,
        RefinementDisposition::NotRequested
    );
    assert_eq!(
        CompositionDisposition::NOT_REQUESTED,
        CompositionDisposition::NotRequested
    );
    assert_eq!(
        ClaimDispositions::NOTHING_REQUESTED,
        ClaimDispositions {
            conformance: ConformanceDisposition::NOT_REQUESTED,
            monitorability: MonitorabilityDisposition::NOT_REQUESTED,
            projection: ProjectionDisposition::NOT_REQUESTED,
            realizability: RealizabilityDisposition::NOT_REQUESTED,
            refinement: RefinementDisposition::NOT_REQUESTED,
            composition: CompositionDisposition::NOT_REQUESTED,
        }
    );
    assert_eq!(
        ClaimDispositions::NOTHING_REQUESTED.state_codes(),
        [
            "conformance/not-requested",
            "monitorability/not-requested",
            "projection/not-requested",
            "realizability/not-requested",
            "refinement/not-requested",
            "composition/not-requested",
        ]
    );
}

// ---------------------------------------------------------------------------
// No aggregate verdict
// ---------------------------------------------------------------------------

/// The public surface of `ClaimDispositions` is exactly the audited table, and
/// no public name on it uses verdict vocabulary.
///
/// This is the completeness half of the no-aggregate proof: a new public field,
/// constant or method fails here until it is classified as single-claim or
/// all-claims, and the two behavioural tests then hold it to that
/// classification.
#[trace("TC-005", "FR-005-AC-1")]
#[test]
fn tc_005_no_aggregate_verdict_field() {
    // The whole module, not one file. `premises.rs` and `dispatch.rs` are
    // declared by `mod.rs` and an inherent impl is legal in either, so reading
    // only `mod.rs` left a verdict one file away from the audit.
    let module: Vec<(&str, String)> = MODULE_SOURCES
        .iter()
        .map(|(name, source)| (*name, stripped(source)))
        .collect();

    // Exactly one `impl` in the whole module names the record, and it is the
    // inherent block.
    let mut naming_the_record = Vec::new();
    for (file, source) in &module {
        for (header, body) in impl_blocks(source) {
            if header.contains("ClaimDispositions") {
                naming_the_record.push((*file, header, body));
            }
        }
    }
    let headers: Vec<&str> = naming_the_record
        .iter()
        .map(|(_, header, _)| header.as_str())
        .collect();
    assert_eq!(
        headers,
        ["ClaimDispositions"],
        "exactly one impl in the claims module may name the record, and it is \
         the audited inherent block; found in {:?}",
        naming_the_record
            .iter()
            .map(|(file, header, _)| (*file, header.as_str()))
            .collect::<Vec<_>>()
    );
    let impl_body = &naming_the_record[0].2;

    // No macro invocation inside it. A macro expanding to a `pub fn` verdict,
    // in the very block this test enumerates, is invisible to every parser
    // here; expansion is a job for a real parser, so the construct is refused
    // rather than read through.
    assert_eq!(
        macro_invocations(impl_body),
        Vec::<String>::new(),
        "no macro may be invoked inside the audited impl block"
    );

    // No public free function anywhere in the module may take or return the
    // record: that is a verdict with the receiver moved to a parameter.
    for (file, source) in &module {
        let mut module_level = source.clone();
        for (_, body) in impl_blocks(source) {
            module_level = module_level.replace(&body, "");
        }
        for name in public_fn_names(&module_level) {
            let signature_line = source
                .lines()
                .find(|line| line.contains(&format!("fn {name}")))
                .unwrap_or_default();
            assert!(
                !signature_line.contains("ClaimDispositions"),
                "{file} declares a public free function `{name}` over the record; \
                 a verdict is not made acceptable by moving the receiver into a \
                 parameter"
            );
        }
    }

    // No inherent impl on a disposition type may take the record either.
    for (file, source) in &module {
        for (header, body) in impl_blocks(source) {
            if header.contains("ClaimDispositions") || header.contains(" for ") {
                continue;
            }
            for name in public_fn_names(&body) {
                let signature_line = body
                    .lines()
                    .find(|line| line.contains(&format!("fn {name}")))
                    .unwrap_or_default();
                assert!(
                    !signature_line.contains("ClaimDispositions"),
                    "{file}: `impl {header}` declares `{name}` over the record"
                );
            }
        }
    }

    let fields = public_field_names(&record_struct_body(&module[0].1));
    let methods = public_fn_names(impl_body);
    let constants = public_const_names(impl_body);

    assert_eq!(fields, AUDITED_FIELDS);
    let audited_methods: Vec<String> = AUDITED_METHODS
        .iter()
        .map(|(name, _)| (*name).to_owned())
        .collect();
    assert_eq!(
        methods, audited_methods,
        "the public method surface of ClaimDispositions changed; classify the new \
         method as SingleClaim or AllClaims, or remove it"
    );
    assert_eq!(constants, AUDITED_CONSTANTS);

    for name in fields.iter().chain(methods.iter()).chain(constants.iter()) {
        assert!(
            !uses_verdict_vocabulary(name),
            "public name `{name}` on ClaimDispositions reads as a roll-up of the six"
        );
    }

    // Every audited accessor is actually exercised by the behavioural test its
    // classification names. Classification without wiring is not a proof.
    for (name, surface) in AUDITED_METHODS {
        assert!(
            obliged_region(*surface).contains(&format!(".{name}(")),
            "`{name}` is classified {surface:?} but is never called by the test that \
             enforces that classification"
        );
    }

    // `refusal_per_claim` is a fixed-size array, so there is no `is_empty()`
    // one-call verdict.
    assert!(
        CLAIMS_SOURCE.contains("pub fn refusal_per_claim(self) -> [Option<ClaimRefusal>; 6]"),
        "refusal_per_claim must stay a fixed-size array, never a Vec"
    );
}

/// Every accessor audited as single-claim is a function of that claim's
/// disposition alone.
///
/// For each claim and each of its states, the observation is computed over all
/// 15 625 records and required to be constant: the other five dispositions
/// cannot influence it, so no roll-up can be smuggled through the `Claim`
/// parameter.
#[trace("TC-005", "FR-005-AC-1")]
#[test]
fn tc_005_single_claim_observations_read_one_field() {
    assert!(AUDITED_METHODS
        .iter()
        .any(|(_, surface)| *surface == Surface::SingleClaim));

    let records = all_records();
    for claim in Claim::ALL {
        for state in 0..STATES {
            let observations: HashSet<(&'static str, bool, Option<ClaimRefusal>)> = records
                .iter()
                .map(|record| single_claim_observation(with_state(*record, claim, state), claim))
                .collect();
            assert_eq!(
                observations.len(),
                1,
                "a single-claim observation of {} varied with another claim's state",
                claim.code()
            );
        }
    }
}

/// Every accessor audited as all-claims is injective over the record space.
///
/// 15 625 records must yield 15 625 distinct values. A verdict — any value that
/// collapses the six — cannot be injective, so this test fails the moment one is
/// added and classified here.
#[trace("TC-005", "FR-005-AC-1")]
#[test]
fn tc_005_all_claim_observations_are_injective() {
    assert!(AUDITED_METHODS
        .iter()
        .any(|(_, surface)| *surface == Surface::AllClaims));

    let records = all_records();
    let total = records.len();

    let refusal_images: HashSet<[Option<ClaimRefusal>; CLAIMS]> =
        records.iter().map(|r| r.refusal_per_claim()).collect();
    assert_eq!(
        refusal_images.len(),
        total,
        "refusal_per_claim collapsed distinct records onto one value"
    );

    let code_images: HashSet<[&'static str; CLAIMS]> =
        records.iter().map(|r| r.state_codes()).collect();
    assert_eq!(
        code_images.len(),
        total,
        "state_codes collapsed distinct records onto one value"
    );

    // The record itself distinguishes all 15 625 states, so nothing is lost
    // before an accessor is reached.
    let records_set: HashSet<ClaimDispositions> = records.iter().copied().collect();
    assert_eq!(records_set.len(), total);
}
