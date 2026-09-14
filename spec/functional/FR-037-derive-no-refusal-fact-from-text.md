---
id: FR-037
title: "Derive no refusal fact from display text"
type: FR
relationships:
  - target: ix://agent-ix/quire-protocol/FR-008
    type: part_of
  - target: ix://agent-ix/quire-protocol/FR-014
    type: depends_on
  - target: ix://agent-ix/quire-protocol/FR-036
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-274
    type: references
  - target: ix://agent-ix/quire-specification/FR-271
    type: references
---
# FR-037: Derive no refusal fact from display text

## Description

When a refusal is constructed, compared or acted on, the library SHALL derive its code, originating stage, affected identity, location, equality comparison and retry decision from typed fields alone.
The library SHALL leave source context that is unavailable explicitly unavailable.
The library SHALL NOT reconstruct a location by searching display names, paths or message text.
The library SHALL NOT treat a difference in display text, locale or cause rendering as a difference in refusal identity.

## Inputs

One typed refusal envelope as [FR-036](./FR-036-carry-complete-refusal-envelope.md)
declares it, together with whatever human-readable rendering accompanies it:
message text, locale, display names, display paths.

## Outputs

The same envelope's typed values, unchanged by any substitution of the
accompanying rendering; and, where source context is unavailable, an explicit
unavailability rather than a synthesized location.

## Behavior

The library SHALL compute refusal identity, equality and retry decisions over typed fields only.
The library SHALL produce byte-identical typed values when the same refusal is rendered under a different locale, a different message string or a different cause rendering.
The library SHALL NOT accept byte zero, an empty path, a repeated display name or a message match as a location.
The library SHALL NOT repair a shared typed interior across two distinct refusal axes by differing display text, because injectivity is evaluated over the typed interior only.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-037-AC-1 | Changing display text, locale or cause rendering changes no refusal identity, no code, no equality comparison and no retry decision. | Test (TC-011) |
| FR-037-AC-2 | No code, originating stage, affected identity, location, equality comparison or retry decision is derived from human-readable text; removing every message string from an envelope changes none of those values. | Test (TC-011) |
| FR-037-AC-3 | Unavailable source context remains explicitly unavailable, and neither byte zero, nor an empty path, nor a repeated display name, nor message text becomes an invented location. | Test (TC-011) |
| FR-037-AC-4 | Two distinct refusal axes whose envelopes carry one shared code and one shared typed cause fail FR-014-AC-1's pairwise check, and differing message text, locale or rendering between them repairs neither the collision nor the check. | Test (TC-011) |
| FR-037-AC-5 | An unavailable location and an available location compare unequal, and an unavailable location compares only to absence rather than to any rendered spelling. | Test (TC-011) |

## Adopted vocabulary

The location kinds this requirement refuses to synthesize are upstream's
`ix://agent-ix/quire-specification/FR-274`, and the code vocabulary it refuses to
match on text is `FR-271`. Both are adopted by reference. The identity this
requirement protects is
[FR-014](./FR-014-typed-outcome-identity.md)'s typed outcome identity: kind
*value*, authority *consumer*, key (`integration`, `refusal_axis`,
`outcome_kind`), in which, as FR-014 records, display text, locale, cause
rendering, message wording, path and span never participate.

Text-independence is what makes FR-014's injectivity check falsifiable. If a
collision between two axes could be dissolved by rendering them differently, one
shared producer code would satisfy every negative scenario row while appearing
distinct. FR-037-AC-4 states the opposite explicitly and can fail.

The local location is either a typed source locus, a typed runtime slot or entry
ordinal, or explicit unavailability. A source contract that publishes no typed
locus cannot be repaired with FR-274 prose or display data. A retained source
code is compared only within its exact selected source contract.

## Dependencies

- [FR-008](./FR-008-return-typed-refusals.md) owns the envelope; this requirement
  is `part_of` it.
- [FR-036](./FR-036-carry-complete-refusal-envelope.md) obliges the fields this
  requirement forbids deriving from text.
- [FR-014](./FR-014-typed-outcome-identity.md) owns the identity and the
  injectivity obligation FR-037-AC-4 protects.

## Status

Draft. The envelope implementation is partial; Task-006 completes the remaining
public-path coverage and TC-011.
