---
id: FR-015
title: "Digest domain selection"
type: FR
object: value_object
relationships:
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

`FR-201` is `object: value_object`. It publishes nine members —
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
| `quire.verification.jcs` | This repository's protocol-result digest, over RFC 8785 canonical UTF-8 bytes. |

Where a producer reference arrives already carrying an upstream domain label,
that label is retained exactly and is never re-labelled into another member.

## What remains genuinely open, and is upstream's to decide

`FR-259`, `FR-261` and `FR-263` each record that no requirement selects a domain
member for the digests they carry. That is a real gap, and it licenses *asking*
upstream which member applies — it does not license minting one. Raised on
`quire-specification`; until it is answered, a digest whose domain is unselected
is compared only against a digest of a selected domain, which is unequal, so the
comparison refuses rather than guessing.

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

## Unreconciled divergence with the implementation enum

`digest::producer::DigestDomain` names five members — `ProducerArtifactBytes`,
`ProducerSource`, `ProducerModel`, `ProducerConfig`, `ProtocolResultJcs` — none
spelled as `FR-201` publishes them, and the `/2` lane's test enum names four
more. Neither conforms to the selection above.

Reconciling them is implementation work this requirement does not perform and
must not pre-empt by renaming either side here. What this requirement fixes is
the specified vocabulary: it is upstream's, selected from, not minted.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-015-AC-1 | Every digest domain label this repository uses is a member `FR-201` publishes, spelled exactly as published, and no label originates here. | Inspection |
| FR-015-AC-2 | A producer reference arriving with an upstream domain label retains that label exactly through admission, linking and result production, and is never re-labelled. | Test (TC-001, TC-014) |
| FR-015-AC-3 | A digest under one selected member compares unequal to a digest under another over identical bytes, in both directions. | Test (TC-014) |
| FR-015-AC-4 | A digest whose domain is unselected compares only to absence; a comparison requiring a domain refuses rather than defaulting to a member. | Test (TC-014) |
| FR-015-AC-5 | No requirement, test or implementation in this repository declares a digest domain vocabulary of its own. | Inspection |

## Dependencies

- `ix://agent-ix/quire-specification/FR-201` publishes the vocabulary and the
  cross-domain inequality rule.
- [FR-011](./FR-011-consumer-identity-discipline.md) supplies the identity
  discipline.
- [FR-001](./FR-001-admit-selected-protocol-profile.md) forbids reinterpreting a
  producer digest through another domain.
