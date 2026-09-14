---
id: FR-015
title: "Digest domain selection"
type: FR
object: value_object
relationships:
  - target: ix://agent-ix/quire-protocol/ADR-003
    type: references
  - target: ix://agent-ix/quire-protocol/ADR-006
    type: references
  - target: ix://agent-ix/quire-protocol/US-005
    type: implements
  - target: ix://agent-ix/quire-protocol/FR-010
    type: part_of
  - target: ix://agent-ix/quire-protocol/FR-011
    type: depends_on
  - target: ix://agent-ix/quire-protocol/FR-001
    type: specifies
  - target: ix://agent-ix/quire-specification/FR-201
    type: depends_on
---
# FR-015: Digest domain selection

## Description

When this repository compares two digests, it SHALL compare them under domain labels selected from `FR-201-canonical-identity-domain`.
This repository SHALL mint no digest domain label of its own.

## Correction: this requirement previously minted a competing vocabulary

An earlier revision of this artifact declared a closed four-member set —
`producer-canonical`, `native-raw`, `compiled-artifact`, `protocol-result` — on
the stated ground that `FR-201` "types the domain label kind" while selecting no
member. That characterisation was wrong, and the requirement built on it was a
boundary breach.

`FR-201` is `object: enumeration`. It publishes nine members —
`ir-canonical`, `ir-bound`, `ir-semantic`, `filament-fingerprint`,
`package-fingerprint`, `lock-file-digest`, `raw-artifact-digest`,
`typed-expression-identity`, `quire.verification.jcs` — and states of itself:
"This artifact *is* the vocabulary of digest domains."

Two of the four minted members re-spelled members upstream already publishes:
`native-raw` against `raw-artifact-digest`, and `protocol-result` against
`quire.verification.jcs`. Both vocabularies make spelling load-bearing and both
make a different label compare unequal over identical bytes. A closed local set
therefore could not represent an upstream-labelled digest at all, and this
repository's own result digest would have compared unequal to the same bytes
under the ecosystem's own verification label.

That is the "second locally authored profile" FR-001 exists to refuse, one level
below protocol semantics. The local set is withdrawn.

## Properties

- **label** (identity component) — one member of `FR-201`'s published
  vocabulary, spelled exactly as published. Spelling is load-bearing: a
  different label compares unequal over identical bytes.
- **selection** — which member this repository uses for which of its own
  digests. The selection is this repository's; the vocabulary it selects from is
  not.

This repository selects, and does not extend:

| Selected member | Used for |
| --- | --- |
| `raw-artifact-digest` | The exact supplied bytes of an offered artifact, including layout, as delivered and never recanonicalised. |
| `quire.verification.jcs` | This repository's protocol-result digest, over RFC 8785 canonical UTF-8 bytes. Also the E02 projection identity `FR-007` derives -- see the note below. |

Where a producer reference arrives already carrying an upstream domain label,
that label is retained exactly and is never re-labelled into another member.

### The E02 projection identity needs no additional member

`FR-007` derives a projection identity distinct from the canonical result's own
identity, and it would be easy to read that as needing a tenth domain member.
It does not. `FR-201` separates the two concepts under the heading "Preimage
selection is per-use, not per-domain", and records that `FR-062` takes the
derived result identity "over a distinct projection-identity preimage under
`quire.protocol.e02-projection-identity/v1-draft.1`". That identifier names a
*preimage*, not a domain: the domain stays `quire.verification.jcs` and the
preimage differs. So `FR-007`'s two digests are distinguished by preimage while
sharing one selected member, and this table stays at two entries.

This is recorded because the opposite reading was reached independently during
this campaign, and acting on it would have minted the tenth member -- repeating
exactly the breach the correction above withdraws.

## Owner selections consumed by this repository

Accepted QSpec #40 / PR #44 closes the earlier owner-side uncertainty. `FR-259`
selects `raw-artifact-digest` for native source and bridge-definition bytes;
`FR-261` selects it for binding-contract definition bytes; and `FR-263` selects
it for Producer-interface, configuration and closure definition bytes while
using `quire.verification.jcs` for derived population and membership-rule
identities. Protocol retains those exact labels and does not broaden either
selection. A future digest slot for which its owner has selected no member still
compares only to absence and refuses when a comparison requires a domain.

## Identity and comparison

- **Identity kind** — *value*, per [FR-011](./FR-011-consumer-identity-discipline.md).
  Authority *producer*: the vocabulary is upstream's and this repository selects
  from it.
- **Key** — the upstream member label itself, matched exactly as `FR-201`
  publishes it. Spelling is load-bearing.
- **Key order** — lexicographic over the label, for collection comparison only.
  It carries no causal meaning and expresses no precedence between members.
- **Causal order** — none.
- **Absence** — an unselected domain compares only to absence. It is never
  defaulted to a member, and a comparison requiring a domain refuses.
- **Digest domain** — the label is the domain; it is not itself digested.

## Implemented vocabulary

`digest::producer::DigestDomain` now has exactly the two selected variants
`RawArtifact` and `VerificationJcs`, whose public spellings are
`raw-artifact-digest` and `quire.verification.jcs`. The earlier five-member
local vocabulary and the version-2 test mirror were removed. Task-006 carries
these shared domain values through the final composed result path rather than
introducing another enum.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-015-AC-1 | Every digest domain label this repository uses is a member `FR-201` publishes, spelled exactly as published, and no label originates here. | Inspection |
| FR-015-AC-2 | A producer reference arriving with an upstream domain label retains that label exactly through admission, linking and result production, and is never re-labelled. | Test (TC-001, TC-006, TC-014) |
| FR-015-AC-3 | A digest under one selected member compares unequal to a digest under another over identical bytes, in both directions. | Test (TC-006, TC-014) |
| FR-015-AC-4 | A digest whose domain is unselected compares only to absence; a comparison requiring a domain refuses rather than defaulting to a member. | Test (TC-006, TC-014) |
| FR-015-AC-5 | No requirement, test or implementation in this repository declares a digest domain vocabulary of its own. | Inspection |

## Dependencies

- `ix://agent-ix/quire-specification/FR-201` publishes the vocabulary and the
  cross-domain inequality rule.
- [FR-011](./FR-011-consumer-identity-discipline.md) supplies the identity
  discipline.
- [FR-001](./FR-001-admit-selected-protocol-profile.md) forbids reinterpreting a
  producer digest through another domain.
