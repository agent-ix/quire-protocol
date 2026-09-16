// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-008 (partial, chunk C-31): the reproduction record.
//!
//! This file covers one slice of TC-008: that the deterministic seed, toolchain
//! identity and corpus identity needed to reproduce a local run are recorded as
//! typed fields, and that a missing one refuses rather than defaults. The
//! equivalent-presentation permutation properties of NFR-001-AC-1 and the
//! full one-axis mutation matrix of NFR-001-AC-2 are not covered here.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use ix_trace_rs::trace;
use quire_protocol::bounds::inventory::{
    production_inventory, Budget, BudgetDimension, OperationOutcome, ResourceAccountant,
};
use quire_protocol::repro::{
    CorpusIdentity, ReproductionInputKind, ReproductionInputs, ReproductionRecord,
    ReproductionRefusal, Seed, ToolchainIdentity,
};

const TOOLCHAIN_BYTES: &[u8] = b"rustc-1.98.0-x86_64-unknown-linux-gnu";
const CORPUS_BYTES: &[u8] = b"corpus:9f2c1ad4";

fn toolchain() -> ToolchainIdentity {
    ToolchainIdentity::new(TOOLCHAIN_BYTES, "rustc 1.98.0 (x86_64-unknown-linux-gnu)")
}

fn corpus() -> CorpusIdentity {
    CorpusIdentity::new(CORPUS_BYTES, "protocol corpus 9f2c1ad4")
}

fn complete_inputs() -> ReproductionInputs {
    ReproductionInputs::nothing_offered()
        .with_seed(Seed::new(0x5eed_0000_0000_0001))
        .with_toolchain(toolchain())
        .with_corpus(corpus())
}

fn complete_record() -> ReproductionRecord {
    complete_inputs()
        .record()
        .expect("every reproduction input was supplied and non-empty")
}

fn hash_of(record: &ReproductionRecord) -> u64 {
    let mut hasher = DefaultHasher::new();
    record.hash(&mut hasher);
    hasher.finish()
}

/// A deterministic replay driven solely by the recorded reproduction inputs.
///
/// The generator lives in the test, not in `src`: this crate owns no
/// canonicalizer and no serialiser. It exists to demonstrate that two runs
/// described by equal records replay to identical bytes, and that a run
/// described by a different record does not.
fn replay_bytes(record: &ReproductionRecord) -> Vec<u8> {
    let mut state = record.seed().value() | 1;
    for byte in record
        .toolchain()
        .bytes()
        .iter()
        .chain(record.corpus().bytes())
    {
        state = state.rotate_left(7) ^ u64::from(*byte);
    }
    let mut out = Vec::with_capacity(64);
    for _ in 0..8 {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        out.extend_from_slice(&state.to_be_bytes());
    }
    out
}

/// Assert that `refusal` is the only refusal this input could produce: every
/// other member of the closed refusal set must be excluded by name.
fn only_refusal(refusal: ReproductionRefusal, expected: ReproductionRefusal) {
    assert_eq!(refusal, expected, "expected exactly {}", expected.code());
    assert_eq!(refusal.code(), expected.code());
    for other in ReproductionRefusal::ALL {
        if other == expected {
            continue;
        }
        assert_ne!(
            refusal,
            other,
            "{} must not be reachable from this input",
            other.code()
        );
        assert_ne!(
            refusal.code(),
            other.code(),
            "{} shares a code with {}",
            expected.code(),
            other.code()
        );
    }
}

/// Tracing: TC-008, NFR-001-AC-2.
#[test]
#[trace("TC-008", "NFR-001-AC-2")]
fn tc_008_seed_toolchain_corpus_are_typed_fields() {
    let record = complete_record();

    // Three separately readable typed fields, each returning its own type.
    let seed: Seed = record.seed();
    let toolchain: &ToolchainIdentity = record.toolchain();
    let corpus: &CorpusIdentity = record.corpus();

    assert_eq!(seed.value(), 0x5eed_0000_0000_0001);
    assert_eq!(toolchain.bytes(), TOOLCHAIN_BYTES);
    assert_eq!(corpus.bytes(), CORPUS_BYTES);

    // Identity bytes are retained unchanged — no trim, case fold or
    // normalisation on the way in.
    let padded = ToolchainIdentity::new(b"  rustc-1.98.0  ", "rustc 1.98.0");
    assert_eq!(padded.bytes(), b"  rustc-1.98.0  ");
    assert_ne!(padded.bytes(), TOOLCHAIN_BYTES);

    // The kind set the record requires is exactly three, and is closed.
    assert_eq!(ReproductionInputKind::ALL.len(), 3);
    assert_eq!(
        ReproductionInputKind::ALL
            .map(ReproductionInputKind::label)
            .to_vec(),
        vec!["seed", "toolchain", "corpus"]
    );
}

/// Tracing: TC-008, NFR-001-AC-2.
#[test]
#[trace("TC-008", "NFR-001-AC-2")]
fn tc_008_absent_seed_refuses_with_its_own_code() {
    let refusal = ReproductionInputs::nothing_offered()
        .with_toolchain(toolchain())
        .with_corpus(corpus())
        .record()
        .expect_err("a run with no recorded seed is not reproducible");

    only_refusal(refusal, ReproductionRefusal::SeedAbsent);
    assert_eq!(refusal.input_kind(), ReproductionInputKind::Seed);
    assert!(refusal.is_absence());
}

/// Tracing: TC-008, NFR-001-AC-2.
#[test]
#[trace("TC-008", "NFR-001-AC-2")]
fn tc_008_absent_toolchain_identity_refuses_with_its_own_code() {
    let refusal = ReproductionInputs::nothing_offered()
        .with_seed(Seed::new(7))
        .with_corpus(corpus())
        .record()
        .expect_err("a run with no recorded toolchain identity is not reproducible");

    only_refusal(refusal, ReproductionRefusal::ToolchainIdentityAbsent);
    assert_eq!(refusal.input_kind(), ReproductionInputKind::Toolchain);
    assert!(refusal.is_absence());
}

/// Tracing: TC-008, NFR-001-AC-2.
#[test]
#[trace("TC-008", "NFR-001-AC-2")]
fn tc_008_absent_corpus_identity_refuses_with_its_own_code() {
    let refusal = ReproductionInputs::nothing_offered()
        .with_seed(Seed::new(7))
        .with_toolchain(toolchain())
        .record()
        .expect_err("a run with no recorded corpus identity is not reproducible");

    only_refusal(refusal, ReproductionRefusal::CorpusIdentityAbsent);
    assert_eq!(refusal.input_kind(), ReproductionInputKind::Corpus);
    assert!(refusal.is_absence());
}

/// Tracing: TC-008, NFR-001-AC-2.
#[test]
#[trace("TC-008", "NFR-001-AC-2")]
fn tc_008_empty_toolchain_identity_refuses_distinctly_from_absent() {
    let refusal = ReproductionInputs::nothing_offered()
        .with_seed(Seed::new(7))
        .with_toolchain(ToolchainIdentity::new(b"", "rustc (unknown)"))
        .with_corpus(corpus())
        .record()
        .expect_err("a present-but-vacuous toolchain identity is not an identity");

    only_refusal(refusal, ReproductionRefusal::ToolchainIdentityEmpty);
    assert_eq!(refusal.input_kind(), ReproductionInputKind::Toolchain);
    assert!(!refusal.is_absence());

    // The decision on record: empty is NOT absent. Both refuse, separately.
    assert_ne!(refusal, ReproductionRefusal::ToolchainIdentityAbsent);
    assert_ne!(
        refusal.code(),
        ReproductionRefusal::ToolchainIdentityAbsent.code()
    );
}

/// Tracing: TC-008, NFR-001-AC-2.
#[test]
#[trace("TC-008", "NFR-001-AC-2")]
fn tc_008_empty_corpus_identity_refuses_distinctly_from_absent() {
    let refusal = ReproductionInputs::nothing_offered()
        .with_seed(Seed::new(7))
        .with_toolchain(toolchain())
        .with_corpus(CorpusIdentity::new(b"", "corpus (unknown)"))
        .record()
        .expect_err("a present-but-vacuous corpus identity is not an identity");

    only_refusal(refusal, ReproductionRefusal::CorpusIdentityEmpty);
    assert_eq!(refusal.input_kind(), ReproductionInputKind::Corpus);
    assert!(!refusal.is_absence());
    assert_ne!(refusal, ReproductionRefusal::CorpusIdentityAbsent);
    assert_ne!(
        refusal.code(),
        ReproductionRefusal::CorpusIdentityAbsent.code()
    );
}

/// Tracing: TC-008, NFR-001-AC-2.
#[test]
#[trace("TC-008", "FR-011-AC-6")]
fn tc_008_absent_seed_never_becomes_zero() {
    // The default-collapse bug in its exact shape: an unsupplied seed must not
    // arrive as the zero a `unwrap_or_default()` would have produced.
    let refused = ReproductionInputs::nothing_offered()
        .with_toolchain(toolchain())
        .with_corpus(corpus())
        .record();
    assert_eq!(refused, Err(ReproductionRefusal::SeedAbsent));

    // Zero is meanwhile a perfectly legitimate *chosen* seed, which is exactly
    // why absence may not be spelled as zero: the two would be indistinguishable.
    let chosen_zero = ReproductionInputs::nothing_offered()
        .with_seed(Seed::new(0))
        .with_toolchain(toolchain())
        .with_corpus(corpus())
        .record()
        .expect("zero is a chosen seed, not an absent one");
    assert_eq!(chosen_zero.seed().value(), 0);

    // A deliberately chosen zero and a deliberately chosen one stay distinct.
    let chosen_one = ReproductionInputs::nothing_offered()
        .with_seed(Seed::new(1))
        .with_toolchain(toolchain())
        .with_corpus(corpus())
        .record()
        .expect("one is a chosen seed");
    assert_ne!(chosen_zero, chosen_one);
}

/// Tracing: TC-008, NFR-001-AC-2.
#[test]
#[trace("TC-008", "NFR-001-AC-2")]
fn tc_008_two_different_identities_never_compare_equal() {
    let base = complete_record();

    // Exactly one mutant per required reproduction input. If a fourth input
    // kind is ever added, the length assertion below fails until it is covered.
    let mutants: [(ReproductionInputKind, ReproductionRecord); 3] = [
        (
            ReproductionInputKind::Seed,
            ReproductionInputs::nothing_offered()
                .with_seed(Seed::new(0x5eed_0000_0000_0002))
                .with_toolchain(toolchain())
                .with_corpus(corpus())
                .record()
                .expect("seed mutant"),
        ),
        (
            ReproductionInputKind::Toolchain,
            ReproductionInputs::nothing_offered()
                .with_seed(Seed::new(0x5eed_0000_0000_0001))
                .with_toolchain(ToolchainIdentity::new(
                    b"rustc-1.99.0-x86_64-unknown-linux-gnu",
                    "rustc 1.98.0 (x86_64-unknown-linux-gnu)",
                ))
                .with_corpus(corpus())
                .record()
                .expect("toolchain mutant"),
        ),
        (
            ReproductionInputKind::Corpus,
            ReproductionInputs::nothing_offered()
                .with_seed(Seed::new(0x5eed_0000_0000_0001))
                .with_toolchain(toolchain())
                .with_corpus(CorpusIdentity::new(
                    b"corpus:9f2c1ad5",
                    "protocol corpus 9f2c1ad4",
                ))
                .record()
                .expect("corpus mutant"),
        ),
    ];
    assert_eq!(mutants.len(), ReproductionInputKind::ALL.len());

    for (kind, mutant) in &mutants {
        assert_ne!(
            base, *mutant,
            "records differing on {kind} compared equal — a defaulted or \
             collapsed identity is the only way this can happen"
        );
        assert_ne!(
            hash_of(&base),
            hash_of(mutant),
            "{kind} mutant hash collided"
        );
        assert_ne!(replay_bytes(&base), replay_bytes(mutant));
    }

    // The mutants are also pairwise distinct, so no two axes collapse together.
    for (i, (left_kind, left)) in mutants.iter().enumerate() {
        for (right_kind, right) in mutants.iter().skip(i + 1) {
            assert_ne!(
                left, right,
                "{left_kind} and {right_kind} mutants collapsed"
            );
        }
    }
}

/// Tracing: TC-008, NFR-001-AC-2.
#[test]
#[trace("TC-008", "FR-011-AC-2")]
fn tc_008_display_text_is_not_identity() {
    // Equal bytes, different display text: the same toolchain.
    let rendered_one = ToolchainIdentity::new(TOOLCHAIN_BYTES, "rustc 1.98.0");
    let rendered_two = ToolchainIdentity::new(TOOLCHAIN_BYTES, "Rust 1.98 (release build)");
    assert_eq!(rendered_one, rendered_two);
    assert_eq!(rendered_one.cmp(&rendered_two), std::cmp::Ordering::Equal);
    assert_ne!(rendered_one.to_string(), rendered_two.to_string());

    // Different bytes, equal display text: different toolchains.
    let bytes_one = CorpusIdentity::new(b"corpus:aaaa", "protocol corpus");
    let bytes_two = CorpusIdentity::new(b"corpus:bbbb", "protocol corpus");
    assert_ne!(bytes_one, bytes_two);
    assert_eq!(bytes_one.to_string(), bytes_two.to_string());

    // The same holds through the whole record: rendering never enters equality.
    let rendered_record = ReproductionInputs::nothing_offered()
        .with_seed(Seed::new(0x5eed_0000_0000_0001))
        .with_toolchain(rendered_two)
        .with_corpus(CorpusIdentity::new(CORPUS_BYTES, "a different rendering"))
        .record()
        .expect("record with different display text");
    assert_eq!(complete_record(), rendered_record);
    assert_eq!(hash_of(&complete_record()), hash_of(&rendered_record));
}

/// Tracing: TC-008, NFR-001-AC-1.
#[test]
#[trace("TC-008", "NFR-001-AC-1")]
fn tc_008_same_seed_same_toolchain_same_bytes() {
    let first = complete_record();
    let second = complete_record();
    assert_eq!(first, second);
    assert_eq!(replay_bytes(&first), replay_bytes(&second));
    assert_eq!(hash_of(&first), hash_of(&second));
}

/// Tracing: TC-008, NFR-001-AC-2.
#[test]
#[trace("TC-008", "NFR-001-AC-2")]
fn tc_008_every_refusal_has_its_own_code_and_kind() {
    assert_eq!(ReproductionRefusal::ALL.len(), 5);

    for (i, left) in ReproductionRefusal::ALL.iter().enumerate() {
        assert!(!left.code().is_empty());
        assert_eq!(left.to_string(), left.code());
        for right in ReproductionRefusal::ALL.iter().skip(i + 1) {
            assert_ne!(left, right);
            assert_ne!(
                left.code(),
                right.code(),
                "{} and {} share a refusal code",
                left.code(),
                right.code()
            );
        }
    }

    // Every required input kind is reachable as an absence, and only the two
    // byte-carrying kinds have an emptiness form.
    for kind in ReproductionInputKind::ALL {
        assert!(
            ReproductionRefusal::ALL
                .iter()
                .any(|r| r.input_kind() == kind && r.is_absence()),
            "{kind} has no absence refusal"
        );
    }
    let empty_kinds: Vec<ReproductionInputKind> = ReproductionRefusal::ALL
        .iter()
        .filter(|r| !r.is_absence())
        .map(|r| r.input_kind())
        .collect();
    assert_eq!(
        empty_kinds,
        vec![
            ReproductionInputKind::Toolchain,
            ReproductionInputKind::Corpus
        ]
    );
}

/// Tracing: TC-008, NFR-001-AC-2.
///
/// A source-inspection test, and deliberately supplementary: the *behaviour* of
/// a defaulted seed is already caught by `tc_008_absent_seed_refuses_with_its_own_code`
/// and `tc_008_absent_seed_never_becomes_zero`. What no runtime path can reach
/// is the construct's absence from code paths that do not exist yet, which is
/// what this test holds closed for every future edit to the module.
#[test]
#[trace("TC-008", "NFR-001-AC-2")]
fn tc_008_reproduction_module_contains_no_default_or_fallback_read() {
    // The bug this chunk exists to prevent, checked in the source itself: a
    // sibling repo shipped `revision.parse().unwrap_or_default()`, which
    // collapsed two different producer revisions to `0` and made two distinct
    // selections compare equal while the test still passed. Prose in this
    // module names those constructs, so only executable lines are scanned.
    const SOURCE: &str = include_str!("../src/repro.rs");
    const BANNED: [&str; 8] = [
        "unwrap_or",
        "impl Default",
        "Default for",
        "FromStr",
        "From<String>",
        "From<&str>",
        "non_exhaustive",
        "_ =>",
    ];

    let mut scanned = 0usize;
    for (number, line) in SOURCE.lines().enumerate() {
        let code = line.trim_start();
        if code.starts_with("//") {
            continue;
        }
        scanned += 1;
        for banned in BANNED {
            assert!(
                !code.contains(banned),
                "src/repro.rs:{}: executable line contains `{banned}`: {code}",
                number + 1
            );
        }
        assert!(
            !(code.starts_with("#[derive(") && code.contains("Default")),
            "src/repro.rs:{}: derives Default",
            number + 1
        );
    }
    assert!(scanned > 100, "source scan read only {scanned} code lines");
}

#[test]
#[trace("TC-008", "TC-009", "NFR-002-AC-1", "NFR-002-AC-3")]
fn tc_009_reproduction_identities_charge_before_retention() {
    let inventory = production_inventory().expect("production inventory");
    let display = "rustc 1.98.0";
    let retained = TOOLCHAIN_BYTES.len() + display.len();

    let short = Budget::nothing_granted()
        .grant(BudgetDimension::InputByte, retained - 1)
        .expect("positive one-short input grant")
        .grant(BudgetDimension::AccountedAllocation, 1)
        .expect("positive allocation grant");
    let mut resources = ResourceAccountant::new(short, &inventory);
    assert!(matches!(
        ToolchainIdentity::new_bounded(TOOLCHAIN_BYTES, display, &mut resources),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::InputByte
                && value.requested() == retained
    ));
    assert_eq!(
        resources
            .usage()
            .consumed(BudgetDimension::AccountedAllocation),
        0
    );

    let two_inputs = retained
        .checked_mul(2)
        .expect("bounded reproduction fixture");
    let one_allocation = Budget::nothing_granted()
        .grant(BudgetDimension::InputByte, two_inputs)
        .expect("positive two-input grant")
        .grant(BudgetDimension::AccountedAllocation, 1)
        .expect("positive one-allocation grant");
    let mut resources = ResourceAccountant::new(one_allocation, &inventory);
    assert!(matches!(
        ToolchainIdentity::new_bounded(TOOLCHAIN_BYTES, display, &mut resources),
        OperationOutcome::Completed(_)
    ));
    assert!(matches!(
        ToolchainIdentity::new_bounded(TOOLCHAIN_BYTES, display, &mut resources),
        OperationOutcome::ResourceIncomplete(value)
            if value.dimension() == BudgetDimension::AccountedAllocation
                && value.consumed() == 1
                && value.requested() == 1
    ));

    let corpus_retained = CORPUS_BYTES.len() + "corpus".len();
    let exact = Budget::nothing_granted()
        .grant(BudgetDimension::InputByte, corpus_retained)
        .expect("positive exact corpus grant")
        .grant(BudgetDimension::AccountedAllocation, 1)
        .expect("positive exact allocation grant");
    let mut fresh = ResourceAccountant::new(exact, &inventory);
    let OperationOutcome::Completed(identity) =
        CorpusIdentity::new_bounded(CORPUS_BYTES, "corpus", &mut fresh)
    else {
        panic!("fresh exact corpus budget must complete");
    };
    assert_eq!(identity.bytes(), CORPUS_BYTES);
}
