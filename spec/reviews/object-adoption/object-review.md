---
id: SR-050
title: "gap-analysis review of the domain object layer and its upstream adoption"
type: SpecReview
analysis: gap-analysis
scope: "spec/functional/FR-010 through FR-018; the Adopted vocabulary sections of spec/functional/FR-002, FR-003, FR-006, FR-008; spec/functional/FR-004, FR-005, FR-007 as unadopted comparators; spec/test-cases/TC-014; spec/tests.md; spec/spec.md §16; against ix://agent-ix/quire-specification spec/objects/ at origin/main 3c913c2"
review_set: subset
---
# SR-050: Domain object layer and upstream adoption

## Summary

This review audited the nine new object requirements FR-010..FR-018 against the
six-axis identity discipline of `ix://agent-ix/quire-specification/FR-200` and
the exemplar `FR-204-declared-object-identity`, audited the four adopted-vocabulary
sections for residual local definitions of protocol, temporal and observation
nouns, audited the `contains`/`part_of`/`specifies` edge set for consistency, and
spot-checked the cited upstream open gaps against the upstream artifacts' own
text.

What holds. FR-011 through FR-018 each address all six axes explicitly — identity
kind, key, key order, causal order, absence and digest domain — with none silently
omitted. The `contains`/`part_of` set is complete and consistent in both
directions: FR-010 declares eight `contains` edges and each of FR-011..FR-018
declares the matching `part_of` back to FR-010, with no orphan and no dangling
edge. FR-010 carries no identity axes of its own, which matches upstream's own
domain artifact FR-200 rather than departing from it. Every upstream gap citation
sampled resolved verbatim: FR-224 ("establishes no component that distinguishes
one delivery from another"), FR-225 ("A retried attempt is required to be retained
as a second subject, and the prose supplies no component that separates it from
the first"), FR-226 (the declared key "is constant across two effect claims ... so
two duplicate effects compare equal under it"), FR-235, FR-237, FR-238, FR-240
("no requirement declares a closed vocabulary. No member list is invented here";
"Participation is unenumerated at the source"), FR-252 (its key carries no
declaration component, exactly as FR-018 relies on), FR-259, FR-261, FR-263
("No requirement states the identity domain of the membership, configuration or
closure digests"), FR-273 (six versus eleven stages, both quoted, neither
preferred) and FR-211..FR-215 (each "the prose does not fix the key components").
FR-014's deliberate exclusion of scenario identity from the key is the right call
at the level it is argued. FR-015's refusal to call the production enum a
refinement of the specified membership is correct and is demonstrated below.

What does not hold. One upstream quotation in FR-006 is fabricated. FR-014's key
makes the very injectivity check the artifact exists to enforce a tautology.
FR-010-AC-2's central claim — that every protocol, temporal and observation noun
now resolves to an upstream reference — is falsified inside this repository by
FR-004, FR-005 and FR-007, which received no adoption pass at all. Seven of the
nine object requirements have no verification edge from any test case. FR-011's
own reconciliation table is contradicted by FR-013 and FR-014 two files later.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | FR-006 attributes to upstream FR-243 a quotation that does not exist in it — "do not state the same membership. Two list five members, one lists four and never names the fifth." — and says three requirements where FR-243 says four ("Four requirements state this vocabulary and one of them lists only four members"). | FR-006 "Where the upstream vocabulary does not yet reach"; `ix://agent-ix/quire-specification/FR-243` | wrong-requirement |
| FND-002 | high | FR-014 names `refusal_axis` as a key component, so distinct axes are distinct identities by construction and FR-014-AC-1 cannot fail; the shared-error-code defect the artifact exists to prevent is undetectable at the key, and only AC-3's producer-enum mapping retains any force. | FR-014 "Key", FR-014-AC-1, FR-014-AC-4, NFR-005 | wrong-requirement |
| FND-003 | high | FR-010-AC-2 and spec/tests.md both assert that every protocol, temporal and observation noun resolves to an `ix://` reference; FR-004, FR-005 and FR-007 carry no object reference at all, and FR-007-AC-4 restates `FR-283`'s closed refusal set and FR-007-AC-2 restates `FR-249`'s derived projection identity locally. | FR-010-AC-2, spec/tests.md, FR-004, FR-005, FR-007; `ix://agent-ix/quire-specification/FR-249`, `FR-283` | wrong-requirement |
| FND-004 | high | FR-010 and FR-012..FR-018 carry no `verifies` edge from any test case or integration test — TC-014 binds FR-011 alone — while FR-012's eight ACs all cite TC-013, whose own Traceability verifies FR-009-AC-1..AC-7 and nothing else. | spec/test-cases/*, FR-012-AC-1..AC-8, TC-013 | correct-requirement-no-evidence |
| FND-005 | medium | FR-011's reconciliation table marks authority "not applicable" for identity kind *value*, yet FR-013 declares kind *value* with authority consumer/producer per side and FR-014 declares "*value*, authority *consumer*"; the authority axis is orthogonal over {authored, derived} only, so the orthogonality claim is overstated and two artifacts declare a pair FR-011 says cannot exist. | FR-011 "Reconciliation with the upstream trichotomy", FR-013, FR-014, FR-015, FR-016 | wrong-requirement |
| FND-006 | medium | FR-018's key is (`delivery_root`, `producer_revision`) but `delivery_root` appears in no Properties entry, contradicting "Every property is a component of the index this repository writes" and making identity depend on ambient load context that FR-200 and FR-204 forbid. | FR-018 "Properties", FR-018 "Key", FR-018-AC-7; `ix://agent-ix/quire-specification/FR-204` | wrong-requirement |
| FND-007 | medium | FR-016's key is the dimension label alone and the artifact states authority is not applicable and the budget is consumer-authored, but NFR-002 enforces "every authored protocol limit" alongside caller-supplied budgets and its Verification requires each inventory entry to identify its "limit authority"; the key cannot tell an authored protocol maximum from a caller budget on the same dimension. | FR-016 "Key", FR-016 "Identity kind", NFR-002 Statement and Verification | wrong-requirement |
| FND-008 | medium | No local requirement restates FR-200-AC-1's obligation that every contained object address all six axes with non-applicability stated rather than omitted; FR-010's three ACs do not mention it and FR-011-AC-1 covers only kind and authority, so the discipline the layer is built on is unenforced here. | FR-010-AC-1..AC-3, FR-011-AC-1; `ix://agent-ix/quire-specification/FR-200`-AC-1 | missing-requirement |
| FND-009 | medium | The new Domain Object Coverage table records "see artifact" instead of AC identifiers for FR-012..FR-018, names TC-006 for FR-015 which FR-015's own ACs never cite, and records FR-017 as Inspection-only although its ACs cite TC-008, TC-009, TC-010, TC-013, IT-002 and IT-003. | spec/tests.md "Domain Object Coverage", FR-015-AC-1..AC-7, FR-017-AC-1..AC-8 | correct-requirement-no-evidence |
| FND-010 | medium | Upstream authors a `references` back-edge from each behavioral requirement to the object artifact that extracts it (FR-009 → FR-204; thirteen on FR-061); FR-001, FR-008, FR-009, NFR-002 and NFR-005 carry no such edge to FR-012..FR-017, so `specifies` — which has no inverse — leaves the layer reachable in one direction only. | FR-001, FR-008, FR-009, NFR-002, NFR-005; `ix://agent-ix/quire-specification/FR-009`, `FR-061` | missing-requirement |
| FND-011 | medium | FR-014's `outcome_kind` admits "Completed, empty", which carries no refusal axis, while its Absence clause states that an outcome carrying no axis "is reported as an unassigned axis"; a legitimate empty success is therefore reported as a defect under the declared rule. | FR-014 "The three-way outcome distinction", FR-014 "Absence", FR-014-AC-5, FR-006 | wrong-requirement |
| FND-012 | low | FR-015's refusal to call the production enum a refinement is correct, but it names no owner for the undecided axis, which FR-011's ordering obligation requires, and `digest::producer::DigestDomain` has no member separating `native-raw` from `producer-canonical` or `compiled-artifact`, so FR-015-AC-2's twelve ordered pairs cannot be exercised on the production path. | FR-015 "Unreconciled divergence", FR-015-AC-2, FR-011 "Ordering obligation", src/digest/producer.rs, tests/support/v2/mod.rs | correct-requirement-no-evidence |
| FND-013 | low | The edge type for the same extraction relation is inconsistent: FR-012, FR-013 and FR-016 use `specifies` toward FR-009 and NFR-002 while FR-018 uses `depends_on` toward FR-009 and FR-014 uses `depends_on` toward NFR-005, though both extract those requirements' prose exactly as the others do. | FR-012, FR-013, FR-014, FR-016, FR-018 frontmatter | wrong-requirement |
| FND-014 | low | FR-006's Description requires completeness and provenance to be kept separate from every other axis, but the adopted-vocabulary table maps neither to an upstream object and the gap section cites no upstream absence for either, so two of the axes the requirement enumerates are neither adopted nor recorded as unreachable. | FR-006 Description, FR-006 "Adopted vocabulary" | missing-requirement |

## Verdict on the two named questions

**Item 1 — six axes, and whether the keys discriminate.** All six axes are
declared in FR-011 through FR-018, and in FR-010 the absence of identity axes
matches upstream's own domain artifact. Correctness is weaker than presence.
FR-012's (`owner`, `contract_version`) key discriminates exactly what FR-009-AC-5
needs discriminated and treats revision, digest and status as state, which is
right. FR-017's (`integration`, `scenario`) key is sound, with `integration`
redundant but harmless. FR-013's provenance-first key is the correct application
of FR-204 at the consumer boundary. FR-015's label key is exact-match and closed,
and its absence rule is right. Three keys fail. FR-014's key contains the very
component whose distinctness it obliges, so the collision check is vacuous
(FND-002) — the exclusion of scenario identity is correct reasoning that
nevertheless does not rescue a key that admits `refusal_axis` itself. FR-016's
label-only key cannot separate NFR-002's two limit authorities (FND-007).
FR-018's key depends on a `delivery_root` that is not a component of the object
(FND-006).

**Item 5 — digest domain refinement.** FR-015's call is correct. A refinement
requires a total function from the five production members onto the four
specified ones; none exists. `ProtocolResultJcs` → `protocol-result` is the only
forced mapping. `ProducerArtifactBytes` is `compiled-artifact` if the axis is
artifact kind and `native-raw` if it is canonicalization regime.
`ProducerSource`, `ProducerModel` and `ProducerConfig` each map to
`producer-canonical` by subject and to `native-raw` by regime, and the production
enum has no member at all that distinguishes bytes as delivered from the
producer's own canonical form — which is exactly the distinction IT-001-SC-07 and
FR-015-AC-6 turn on. Declaring a refinement would assert a correspondence no
artifact establishes. The residue FR-015 leaves is that it names no owner for the
decision (FND-012) and does not record that the production enum, as it stands,
cannot exercise its own twelve-ordered-pair obligation.
