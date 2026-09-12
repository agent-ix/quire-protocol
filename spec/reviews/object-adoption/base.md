---
id: SR-052
title: "Base review of the upstream domain object adoption"
type: SpecReview
analysis: base
scope: "FR-010..FR-018, FR-002, FR-003, FR-006, FR-008, TC-014, ADR-001, TM-001, spec.md §16"
review_set: subset
---
## Summary

The base checklist was applied to the nine new object requirements, the four
behavioural requirements that replaced local definitions with upstream
references, the new test case and architecture decision, and the matrix and
reference-index updates. Identifier hygiene and edge symmetry are clean: FR-010
declares eight `contains` edges and exactly eight artifacts declare the matching
`part_of`; every `TC-NNN` cited by an object criterion exists; all 45
`ix://agent-ix/quire-specification` targets resolve against that repository's
`origin/main`. The substantive gaps are at the verification seam — object
criteria name test cases that do not name them back — and in how the matrix
records the new layer.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | The object criteria claim backing from test cases that do not reciprocate. Thirty-three of the sixty-six new criteria verify through `Test (TC-001)`, `TC-008`, `TC-009`, `TC-010`, `TC-011` or `TC-013`, but every one of those test cases was authored against a behavioural requirement and its Traceability names only that requirement — TC-013 states "Verifies FR-009-AC-1 through FR-009-AC-7" and says nothing of FR-012's nine criteria. The reference is one-directional, so `quire coverage` will not mint a backing symbol for the object criterion and the matrix row can read as covered while nothing exercises it. Only TC-014 was authored for an object requirement and traces back to it. | FR-012..FR-018 Acceptance Criteria; TC-001, TC-008, TC-009, TC-010, TC-011, TC-013 Traceability | correct-requirement-no-evidence |
| FND-002 | medium | TM-001's new Domain Object Coverage table records `see artifact` in the Acceptance Criteria column for seven of nine rows, where the behavioural table enumerates every criterion by identifier. A reader cannot tell from the matrix how many criteria an object requirement carries or which are unbacked, which is the one question the matrix exists to answer. FR-010 and FR-011 are enumerated; the rest are not. | TM-001 Domain Object Coverage | correct-requirement-no-evidence |
| FND-003 | medium | Sixteen of the sixty-six new criteria verify by `Inspection` and five by `Analysis`, and none of the twenty-one names what would falsify it. For a layer whose whole purpose is to make identity and ordering checkable, an inspection criterion that does not state its negative control is indistinguishable from an assertion. FR-010-AC-1 through AC-3 are the clearest case: each asks whether an artifact set has a property, with no stated procedure for demonstrating that it does not. | FR-010-AC-1..AC-3; FR-011-AC-1, AC-7; FR-012..FR-018 Inspection criteria | correct-requirement-no-evidence |
| FND-004 | low | `FR-010`'s Entities summary table and its `contains` edges are two enumerations of one set, maintained by hand. They agree today — eight and eight — but nothing checks that they continue to, and the object-type guide states that entities are linked by edges with the list as an optional summary. The duplication is the same class of defect the adoption exists to remove, one level up. | FR-010 Entities; FR-010 frontmatter | wrong-requirement |
| FND-005 | low | `FR-010-AC-2` cites the literal placeholder `ix://agent-ix/quire-specification/FR-2NN` inside its criterion text. The intent is clear and it is prose rather than a relationship target, so nothing dangles, but a criterion whose text contains an unresolvable reference cannot be mechanically checked against the reference set it describes. | FR-010-AC-2 | wrong-requirement |

## What the checklist found clean

Recorded because these were the failure modes most likely in a change of this
shape, and they did not occur:

- **Identifier hygiene.** FR-010 through FR-018 are sequential with no gap or
  duplicate. FR-010 and FR-011 reuse identifiers a deleted scratch draft had
  claimed; that draft was untracked and never validated, so no released artifact
  referenced them.
- **Edge symmetry.** Eight `contains`, eight `part_of`, no orphan and no
  one-sided edge.
- **Reference resolution.** All 45 distinct `ix://agent-ix/quire-specification`
  targets resolve against `origin/main` @ `3c913c2`. The only apparent miss is
  the `FR-2NN` placeholder of FND-005, which is prose.
- **Verification vocabulary.** Every `Verification` cell is `Inspection`,
  `Analysis` or `Test (…)`; no cell carries an unrecognised method.
- **No local redefinition.** No protocol, temporal or observation noun is defined
  locally in the adopted requirements; each resolves to an upstream reference.

## Remediation

FND-001 requires the six existing test cases to declare the object criteria they
back, or those criteria to cite test cases authored for them. FND-002 requires
TM-001 to enumerate object criteria as it does behavioural ones. FND-003 requires
each inspection and analysis criterion to state its negative control. FND-004 and
FND-005 are cosmetic and can travel with the next content change.

None is blocked on an external producer; all five are B-ownable now.

## Remediation applied

FND-001 and FND-002 are closed in the same change; FND-003 through FND-005 are
carried.

- **FND-001.** TC-001, TC-008, TC-009, TC-010, TC-011 and TC-013 each now declare,
  in their own Traceability section, the object requirements whose criteria cite
  them, and state that a change which stops exercising those criteria is a change
  to their coverage. The reference resolves in both directions, so a backing
  symbol can be minted for an object criterion rather than the matrix row reading
  covered while nothing exercises it.
- **FND-002.** TM-001's Domain Object Coverage table enumerates every object
  requirement's criterion range and every test case that backs it, matching the
  behavioural table's shape. The totals are stated: sixty-six criteria across
  nine requirements, twenty-one verified by inspection or analysis.

FND-003 stands and is the most substantive of the five: twenty-one inspection and
analysis criteria still state no negative control. It is carried deliberately
rather than closed cheaply, because writing a negative control per criterion is
authoring work, not an edit — and FR-010-AC-1 through AC-3 in particular need a
procedure for demonstrating that the artifact set *lacks* the property, which
does not exist yet.

## Second remediation pass, after SR-050 and SR-051

Both reviews landed after the first pass. Between them and an independent audit
they exposed a defect in how this adoption was performed, not only in what it
produced, and that is recorded here rather than absorbed.

### The method defect

The "Adopted vocabulary" and "Where the upstream vocabulary does not yet reach"
sections were written from a subagent's *characterisations* of upstream
artifacts rather than from the artifacts. Two consequences followed:

1. **A fabricated quotation.** FR-006 attributed to `FR-243-settlement-basis` a
   sentence that artifact does not contain, and said "three requirements" where
   it says four. SR-050 FND-001 caught it; SR-051 FND-004 added that the sentence
   resembles one in FR-258. FR-243 also *does* name the disputed member,
   `unavailable`, where this repository said it "never names the fifth".
2. **A wrong characterisation that became a boundary breach.** FR-015 declared a
   closed four-member digest-domain vocabulary on the stated ground that
   `FR-201-canonical-identity-domain` "types the domain label kind" while
   selecting no member. FR-201 is `object: enumeration`, publishes nine members,
   and states "This artifact *is* the vocabulary of digest domains." Two of the
   four minted members re-spelled members it already publishes. SR-051 FND-001
   is correct that this is the second-locally-authored-profile failure one level
   below protocol semantics.

An audit of every quotation in this repository against upstream `origin/main`
found three further attributions to FR-235, FR-237 and FR-238 that paraphrased
inside quotation marks. Everything else — FR-224, FR-225, FR-226, FR-240,
FR-211..FR-215, FR-271..FR-274, FR-259, FR-261, FR-263 — checks out verbatim.

### Applied

- **FR-015 withdrawn and retyped.** It is now `object: value_object` titled
  "Digest domain selection": this repository selects `raw-artifact-digest` and
  `quire.verification.jcs` from FR-201's published vocabulary and mints nothing.
  FR-015-AC-5 makes the prohibition explicit. The correction is recorded in the
  artifact rather than silently rewritten.
- **The fabricated and paraphrased quotations are replaced with the artifacts'
  own words** in FR-003 and FR-006, and the upstream issue that carried the
  fabrication is corrected in a comment rather than by editing its body, so the
  error stays visible (quire-specification#29).
- **FR-014's injectivity criterion was a tautology.** `refusal_axis` is a key
  component, so "distinct axes carry distinct identities" could never fail and
  the shared-code defect stayed invisible behind it (SR-050 FND-002). AC-1 now
  states the obligation over the observed producer interior — the FR-271 code
  and FR-272 cause the envelope carries — which can fail.
- **FR-010-AC-2 over-claimed.** It asserted repository-wide noun adoption while
  FR-004, FR-005, FR-007, NFR-001, IT-002 and IT-003 had no upstream reference
  at all (SR-050 FND-003, SR-051 FND-005). FR-010 now carries an adoption-status
  table and the criterion asserts nothing about requirements not yet passed.
- **The reverse failure is closed.** FR-008 bound its refusal interior to a
  catalog FR-270 describes as "Proposed, not adopted or implemented … source
  material here, not an adopted selection", with no manifest entry and no
  reference index row (SR-051 FND-002). The catalog is now an FR-009 entry with
  status `unresolved`, cited in FR-008 and in spec.md §16.
- **ADR-001's denominator corrected** to 0 of 11: eleven candidate rows, not the
  nine-requirement denominator (SR-051 FND-006). The result is unchanged.

### Carried

FND-003 of this review (twenty-one inspection and analysis criteria with no
stated negative control), SR-050 FND-005 through FND-014 and SR-051 FND-003,
FND-007 through FND-010. The substantive one among them is SR-050 FND-005: FR-011
marks authority inapplicable for kind *value* while FR-013 and FR-014 both
declare *value* with an authority. That is a real inconsistency in the discipline
this repository just adopted, and it should be settled before the FR-003..FR-008
decomposition states thirty more requirements against it.
