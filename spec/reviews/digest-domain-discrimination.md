---
id: SR-027
title: "Code review — IT-001 digest-domain discrimination and the admission refusal layer"
type: SpecReview
analysis: code-review
scope: "tests/it001_native_handoff.rs at 681e25a, and src/digest/, src/inventory/, tests/tc_006_digest_domains.rs on agent-b/it001-digest-domain-discrimination"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-protocol/AP-001, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-001, type: references }
  - { target: ix://agent-ix/quire-protocol/IT-001, type: references }
---
## Summary

`tests/it001_native_handoff.rs` on `main` asserted a JSON parse failure for a
cross-domain digest substitution and asserted one shared `Invalid::Selection`
code for three different refusal axes, so four landed assertions passed for the
wrong reason. This review covers those defects and the B-owned discriminating
layer that remediates them.

## Verdict

**FAIL** — two high findings, both shipped on `main` at 681e25a and both
remediated on this branch; the gate records them because they were merged, and
the remediation is re-verified against A's real producer fixture.

## Assurance Context

Profile `AP-001` (`spec/assurance/AP-001-protocol-implementation.md`),
`profile_version` 0.2, `review_policy.mode: require`, `operations` includes
`code-review`. Evaluated source revision: `agent-b/it001-digest-domain-discrimination`
over baseline `681e25a` with `agent-b/c00-lib-facade` (`48db039`) merged in.

The profile's `impact-identity-substitution` scenario — "changed source,
dependency, observation, profile, result, or adapter bytes retain an accepted
identity" — is the impact these findings sit under: a cross-domain digest
substitution is exactly an identity substitution that survives a syntactic
check.

Unavailable selected context: no measurement, exception, independence or
producer-reliance evidence was selected or supplied for this review, and none
is claimed. Active exceptions: none. Structural-gap evidence: none selected.
The `--features quire-extraction` lane named in the standing gate directive does
not exist in this repository (issue #31, open); no lane was invented to make it
report green.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Cross-domain digest case asserted `Error::Json`, a parse failure, so a well-formed substitution would pass it untouched | tests/it001_native_handoff.rs:496 |
| FND-002 | high | Producer identity, baseline selection state and baseline digest all asserted the single shared `Invalid::Selection` code | tests/it001_native_handoff.rs:344 |
| FND-003 | medium | Per-source, per-dependency and per-model digest slots are not guarded; only the three top-level selection references are | src/inventory/map.rs:96 |
| FND-004 | medium | Inventory construction panicked on a library path through `expect` on a serialisation and a domain invariant | src/inventory/mod.rs:89 |
| FND-005 | medium | The pre-admission read parsed untrusted offer bytes with no ceiling of its own | src/inventory/map.rs:77 |
| FND-006 | medium | `ResultDigestRefusal::Divergent` carried a canonicalisation error message in the field that holds a digest | src/inventory/mod.rs:151 |
| FND-007 | medium | A's `Error`, `Invalid` and `Unsupported` are `#[non_exhaustive]`, so a wildcard arm is compulsory and the compiler cannot force a new variant to be classified | src/inventory/map.rs:122 |
| FND-008 | low | A slot the inventory holds no selection for was reported as a divergence it never compared | src/inventory/mod.rs:110 |
| FND-009 | low | A digest text with no algorithm label was reported as an empty foreign label | src/inventory/mod.rs:132 |
| FND-010 | low | The RFC 8785 canonicaliser refuses JSON numbers rather than implementing ECMAScript number serialisation | src/digest/jcs.rs:60 |

## Disposition

- **FND-001, FND-002** — fixed. Each of the four assertions now names its own
  B-owned variant, and a new case offers a well-formed `sha256:` value from the
  protocol-result domain. Measured before the fix: A alone returns
  `Invalid(Selection)` for that offer, the same code as two other axes.
- **FND-004, FND-005, FND-006, FND-008, FND-009** — fixed in this branch during
  review: `InventoryError` replaces both library-path panics, the pre-admission
  read is bounded by the caller's own `limits.payload_bytes`, a
  `ResultDigestRefusal::Uncanonicalizable` variant carries the canonicalisation
  error, and `DigestClassification::UnselectedSlot` and
  `OfferedDomain::Unlabelled` replace the two silent fallbacks.
- **FND-003** — open. The per-entry slots need the independently selected
  per-source and per-dependency inventory that the fixture-builder chunk lands.
  The scope limit is stated in the guard's own doc comment rather than left
  implied.
- **FND-007** — open and mitigated, not closed. The compulsory arm produces the
  separately assertable `AdmissionRefusal::UnclassifiedSource`, never a success
  and never a neighbouring code, and
  `tc_006_classification_names_every_known_source_variant` fails if any A
  variant published at the pinned revision reaches it. A variant added in a
  future A revision is caught by that test only after the pin moves.
- **FND-010** — open by design. Approximating ECMAScript number serialisation
  would produce a wrong canonical form that still looks well formed, which is
  the failure class this crate exists to refuse.

## Gates

Run locally, serially, one Cargo job, warnings denied, in the worktree's own
target directory:

- `cargo fmt --all -- --check` — clean.
- `RUSTFLAGS=-Dwarnings cargo clippy -j 1 --all-targets -- -D warnings` — clean.
- `RUSTFLAGS=-Dwarnings cargo test -j 1 -- --test-threads=1` — 11 passed,
  0 failed, 10 ignored.
- `QUIRE_PROTOCOL_A_FIXTURE=… cargo test --test it001_native_handoff --
  --ignored --test-threads=1` — 10 passed, 0 failed, 0 ignored, against A's real
  producer output at the pinned revision `84975dd`.

No hosted workflow was enabled; hosted runs stay `workflow_dispatch` only.
