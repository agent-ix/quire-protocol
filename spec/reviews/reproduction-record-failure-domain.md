---
id: SR-035
title: "Failure-domain review of the reproduction record (chunk C-31)"
type: SpecReview
analysis: failure-domain
scope: "NFR-001-AC-1..AC-3 and issue #10 box 5, as implemented by `src/repro.rs` and `tests/tc_008_reproduction_record.rs` on `agent-b/c31-repro-identities` over base 48db0398d60523f15eb97978d919673cf27cde40"
review_set: partial
relationships:
  - { target: ix://agent-ix/quire-protocol/NFR-001, type: reviews }
---
## Summary

Chunk C-31 records the three identities that make a local run reproducible —
deterministic seed, toolchain identity, corpus identity — as typed fields on one
record, and makes the absence of any of them a refusal with its own stable code.
The failure domain under review is **silent identity defaulting**: a default
seed, an inferred toolchain or an assumed corpus produces a record that looks
complete, compares equal to other equally defaulted records, and converts an
unreproducible run into one that claims to be reproducible.

The review found no unresolved defaulting path in the module. Two structural
gaps are recorded as follow-ups rather than fixed here, because they belong to
chunks that own the files concerned.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Resolved: absence is never spelled as a value. No type in `src/repro.rs` implements `Default`, and every read of an offered input is a `match` whose `None` arm returns a refusal. Nine `compile_fail` doc-tests prove the absent `Default`, `FromStr` and `From<String>` impls, and each was verified to fail for its intended diagnostic (E0599 `default`, E0277 `FromStr`, E0308) rather than for an incidental typo. | NFR-001-AC-2; TC-008 |
| FND-002 | high | Resolved: zero is a legitimate chosen seed, so absence may not be represented by zero. `tc_008_absent_seed_never_becomes_zero` asserts both halves — an unsupplied seed refuses as `reproduction-seed-absent`, and a deliberately chosen `Seed::new(0)` is admitted and stays distinct from `Seed::new(1)`. This is the exact shape of the sibling-repo defect in which `revision.parse().unwrap_or_default()` collapsed two different producer revisions to `0` and made two distinct selections compare equal while the test still passed. | NFR-001-AC-2; TC-008 |
| FND-003 | high | Resolved: present-but-empty is not absent. An identity supplied with no bytes refuses under its own code (`reproduction-toolchain-identity-empty`, `reproduction-corpus-identity-empty`), distinct from the absence codes. The decision is documented in the module header and asserted in both directions. A seed has no empty form, which is stated rather than left implicit. | NFR-001-AC-2; TC-008 |
| FND-004 | medium | Resolved: display text cannot enter identity. `ToolchainIdentity` and `CorpusIdentity` hand-write `PartialEq`, `Eq`, `PartialOrd`, `Ord` and `Hash` over the producer's exact bytes alone, following the pattern chunk C-07 established for protocol identities. Equal bytes with different rendering compare equal and hash equal; equal rendering with different bytes compares unequal. | NFR-001-AC-3; TC-008 |
| FND-005 | medium | Resolved: two runs with different identities never compare equal. One mutant per required input kind is built and asserted unequal to the base record by `==`, by `Hash`, and by a deterministic replay driven only from the record; the mutants are also pairwise unequal, so no two axes collapse into one. The mutant count is asserted against `ReproductionInputKind::ALL`, so a fourth reproduction input fails the test until it is covered. | NFR-001-AC-2; TC-008 |
| FND-006 | low | Resolved: the enums are closed, not `#[non_exhaustive]`, and the reason is documented in the module header and the pull-request body. Weighing C-07's precedent: a `#[non_exhaustive]` enum forces downstream wildcard arms, and a wildcard arm silently absorbs a variant added later — which would defeat the independent assertability this chunk exists to deliver. Every `match` in the module is exhaustive with no `_` arm, and a static test rejects `_ =>` on any executable line. | NFR-001; TC-008 |
| FND-007 | medium | Open, routed to chunk C-01: this module defines its own `ReproductionRefusal` enum with B-owned stable codes rather than an envelope, because `src/refusal/` is owned by chunk C-01 (SR-032) and is unlanded on this base. The five codes are expected to map into `ProtocolRefusal` in the `result` domain once C-01 lands. No envelope was built here. | FR-008-AC-1 |
| FND-008 | medium | Open, routed to chunk C-30: `ToolchainIdentity` and `CorpusIdentity` retain caller-supplied `Vec<u8>` and `String` with no length bound, so they are unbounded retention sites under NFR-002-AC-3. They are flagged here rather than bounded, because the resource-site inventory and its boundary cases belong to C-30 over C-29's guard framework. | NFR-002-AC-3 |
| FND-009 | low | Open, routed to chunk C-26: the record is standalone in `src/repro.rs` and is not yet embedded as provenance on the canonical protocol result, because `src/result/` is owned by C-26 and is unlanded. NFR-001-AC-1's requirement that equivalent presentations produce "complete identical provenance" therefore remains planned, not executed. | FR-006-AC-1; NFR-001-AC-1 |

## Coverage assessed

| Criterion | Status | Evidence |
| --- | --- | --- |
| NFR-001-AC-1 | Partial | `tc_008_same_seed_same_toolchain_same_bytes` proves two runs described by equal records replay byte-identically and hash identically. Map-order, locale, allocation-layout and arrival-order permutations are out of this chunk's scope and remain planned. |
| NFR-001-AC-2 | Partial | Five independently asserted refusals plus a three-axis mutation set cover the seed, toolchain and corpus identities. The remaining named identities of NFR-001-AC-2 (source, baseline/profile, model, compiled protocol, request, binding, input, observation, closure axes, producer, implementation, backend) are owned by other chunks. |
| NFR-001-AC-3 | Partial | Display text is proved not to be identity for both run identities and for the whole record. Declared canonical ordering of unordered semantic sets is not in this chunk's scope. |

## Gates run

All gates were run locally and serially in this worktree with an explicit
`--target-dir ./target`, one Cargo job, and warnings denied. Repo B declares no
`[features]` table, so it has exactly one lane; no second lane was invented.

- `cargo fmt --all -- --check` — clean.
- `cargo clippy -j 1 --all-targets --target-dir ./target -- -D warnings` — clean.
- `cargo test -j 1 --target-dir ./target -- --test-threads=1` — 12 passed in
  `tc_008_reproduction_record`, 10 doc-tests passed (1 positive control, 9
  `compile_fail`), 0 unit tests, 9 pre-existing `it001_native_handoff` tests
  ignored for want of A's producer fixture. 0 failed.

## Mutant kill evidence

Two guard-removal mutants were applied to `src/repro.rs`, run, and reverted.

| Mutant | Tests that failed | Intended? |
| --- | --- | --- |
| `None => Seed::new(0)` replacing the seed refusal | `tc_008_absent_seed_refuses_with_its_own_code`, `tc_008_absent_seed_never_becomes_zero` | Yes — exactly the two tests assigned to that guard, and no others. |
| `None => self.seed.unwrap_or(Seed::new(0))` | the two above, plus `tc_008_reproduction_module_contains_no_default_or_fallback_read` | Yes — the static audit names the offending line and the banned construct. |
