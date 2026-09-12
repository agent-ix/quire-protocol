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
