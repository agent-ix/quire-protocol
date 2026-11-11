---
id: SR-2127051
title: "TL-388 PR #150 code-review (rust-review lane)"
type: SpecReview
analysis: code-review
review_set: subset
scope: "agent-ix/quire-protocol; TL-388; PR #150 head 98e95d0; src/deadline.rs, tests/tc_154_settle_deadline_captures.rs, tests/support/observation_owner.rs"
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-100, type: reviews }
---
## Summary

Ticket: TL-388. Code review with the Rust lane (rust-review, rust-style loaded) folded in, scoped to `git diff origin/main...HEAD` of PR #150 at head 98e95d0. Focused checks run once on the head: `cargo clippy --locked --all-targets -- -D warnings` clean; `cargo test --locked --test tc_154_settle_deadline_captures` 23 passed, 0 failed; `RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps` clean (no broken intra-doc link). No full gate run.

## Examined

- FR-100 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-11 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-13 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-14 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-15 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-16 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-17 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-18 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)

## Verdict

**FAIL — findings must be fixed before merge.** The anchor is read from the owner's `observation::View::anchor()` with the clock-family ranges honoured, the old caller-anchor `Deadline::of(.., anchor: i128, ..)` path is gone with every caller and test moved (no compatibility twin), `WaitFact::authority` no longer takes a deadline, refused facts leave the wait unchanged, two `after` candidates are refused with `AmbiguousAnchor`, every new test uses `#[trace("TC-154", "FR-100-AC-n")]` with no `/// Trace:` line, and no vendoring, shim or new pin is introduced. Defects: the no-`after` truth value under closed progress contradicts QSpec FR-061/FR-242; the `after` observation's clock identity is never compared with the wait's clock; and a private enum mirrors the owner's `quire_observation::Anchor`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | With no admitted `after` observation, `judge` returns truth `pending` whenever completeness is `complete`, including with progress `closed` (asserted by the AC-15 test). QSpec FR-061 (origin/main) allows `pending` only when activation is triggered and decision-scope progress is open; an untriggered wait under complete input and closed progress is activation `inactive` (truth `unavailable`), and with open progress it is at best activation-unknown (truth `unavailable`). The value is not new but it is assigned where the owner forbids it. | src/deadline.rs:811-818; tests/tc_154_settle_deadline_captures.rs:776-798 |
| FND-002 | medium | `AfterAnchor::of` reads only `payload().anchor()` and never compares `payload().clock().identity` with the wait's `clock_identity`, so an `after` observation on another clock of the same family is accepted and its instant is used as the origin of a window on the wait's clock. Progress views are refused for the same mismatch (AC-11); the `after` observation is not. | src/deadline.rs:234-248, 721-741 |
| FND-003 | medium | The private `enum Anchor { EventPosition(u64), FixedSample(u64), Timestamp(i128) }` mirrors the owner's published `quire_observation::Anchor` (EventPosition / FixedSample{index, epoch_nanos, period_nanos} / TimestampNanos), dropping the fixed-sample epoch and period. CLAUDE.md: an owner-published type is imported, never mirrored. Parse the view's `AnchorRef` into `quire_observation::Anchor` and match on it. | src/deadline.rs:214-219, 250-271 |
| FND-004 | low | `AfterAnchor::of` maps an unparseable owner anchor text to `WindowOutOfRange`. Owner canonical text always fits u64/i128, so the branch is unreachable for a valid view and, if reached, reports a window-range failure for what is a malformed owner payload. It has no test. | src/deadline.rs:228-248 |
| FND-005 | low | `Wait::new` gained a refusal (`UndeclaredAwait` when `declaration.control` is not `key.branch`) that no test exercises. | src/deadline.rs:662-673 |
| FND-006 | low | `Wait::new` and `assess` take `family: TemporalBoundary`, a full boundary value whose numbers are ignored; only the variant is read. A caller can pass any bounds without effect, which reads as if the bounds mattered. | src/deadline.rs:662-668, 868-875 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-007 | low | `Supersession::contradicted_premise` is still documented as "the authority fact that closed the wait", but an `AfterObservation` supersession sets it to whatever fact produced the earlier pre-anchor result (a response observation or an authority fact). Update the doc. | src/deadline.rs:590, 818 |

## Dispositions

Round 1, fix commit 138d713 (reviewed as `git diff 98e95d0..138d713`).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | 138d713: `judge` returns `unavailable` for every progress/completeness case before the anchor; AC-15 test crosses open/closed x complete/incomplete/contradicted |
| FND-002 | fixed | 138d713: `AfterAnchor` carries `clock().identity`; `Deadline::of` refuses a mismatch with `AnchorNotOnClock` (AC-19 test) |
| FND-003 | fixed | 138d713: private `Anchor` deleted; `AfterAnchor` holds `quire_observation::Anchor` with epoch and period |
| FND-004 | fixed | 138d713: malformed owner text is now `WaitRefusal::AnchorMalformed`, documented as unreachable for a validated view |
| FND-005 | fixed | 138d713: AC-12 test asserts `Wait::new` refuses a declaration that is not the key's await |
| FND-006 | fixed | 138d713: `family` is the new `ClockFamily` tag; quire-observation publishes no data-free family enum (`TemporalBoundary`/`ClockRange` carry bounds), so this is not a mirror |
