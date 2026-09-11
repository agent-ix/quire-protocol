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

/// The body lines of the first block opened by `header` and closed by a `}` in
/// column zero.
fn block_body<'a>(source: &'a str, header: &str) -> Vec<&'a str> {
    let mut lines = source.lines().skip_while(|line| line.trim() != header);
    assert!(lines.next().is_some(), "block header not found: {header}");
    lines.take_while(|line| *line != "}").collect()
}

/// Names introduced by `pub fn` in a block body.
fn public_fn_names<'a>(body: &[&'a str]) -> Vec<&'a str> {
    body.iter()
        .filter_map(|line| line.trim().strip_prefix("pub fn "))
        .filter_map(|rest| rest.split('(').next())
        .collect()
}

/// Names introduced by `pub const` in a block body.
fn public_const_names<'a>(body: &[&'a str]) -> Vec<&'a str> {
    body.iter()
        .filter_map(|line| line.trim().strip_prefix("pub const "))
        .filter_map(|rest| rest.split(':').next())
        .map(str::trim)
        .collect()
}

/// Names introduced by `pub <name>:` fields in a block body.
fn public_field_names<'a>(body: &[&'a str]) -> Vec<&'a str> {
    body.iter()
        .filter_map(|line| line.trim().strip_prefix("pub "))
        .filter(|rest| {
            rest.contains(':') && !rest.starts_with("fn ") && !rest.starts_with("const ")
        })
        .filter_map(|rest| rest.split(':').next())
        .collect()
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
    let fields = public_field_names(&block_body(CLAIMS_SOURCE, "pub struct ClaimDispositions {"));
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

    assert_eq!(
        ConformanceDisposition::NOT_REQUESTED,
        ConformanceDisposition::NotRequested
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
    let fields = public_field_names(&block_body(CLAIMS_SOURCE, "pub struct ClaimDispositions {"));
    let impl_body = block_body(CLAIMS_SOURCE, "impl ClaimDispositions {");
    let methods = public_fn_names(&impl_body);
    let constants = public_const_names(&impl_body);

    assert_eq!(fields, AUDITED_FIELDS);
    let audited_methods: Vec<&str> = AUDITED_METHODS.iter().map(|(name, _)| *name).collect();
    assert_eq!(
        methods, audited_methods,
        "the public method surface of ClaimDispositions changed; classify the new \
         method as SingleClaim or AllClaims, or remove it"
    );
    assert_eq!(constants, AUDITED_CONSTANTS);

    for name in fields
        .iter()
        .chain(methods.iter())
        .chain(constants.iter())
        .copied()
    {
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

    // Exactly one inherent impl block, so the audited surface is the whole
    // surface, and no trait shortcut re-introduces one.
    assert_eq!(
        CLAIMS_SOURCE.matches("impl ClaimDispositions").count(),
        1,
        "ClaimDispositions must have exactly one inherent impl block"
    );
    for forbidden in [
        "impl Default for ClaimDispositions",
        "impl From<ClaimDispositions>",
        "impl core::fmt::Display for ClaimDispositions",
        "impl std::fmt::Display for ClaimDispositions",
        "impl core::ops::Deref for ClaimDispositions",
        "impl std::ops::Deref for ClaimDispositions",
    ] {
        assert!(
            !CLAIMS_SOURCE.contains(forbidden),
            "`{forbidden}` would offer a verdict-shaped shortcut"
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
