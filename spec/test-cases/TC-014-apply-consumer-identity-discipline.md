---
id: TC-014
title: "Apply the consumer identity discipline"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/FR-011
    type: verifies
---
# TC-014: Apply the consumer identity discipline

## Description

Verify FR-011's identity discipline against B-owned fixtures: that presentation
never participates in identity, that a producer-authored identity is copied and
never repaired, that a consumer-authored identity stays inert outside this
boundary, that digest domains do not collapse, and that absence never defaults.

This test case requires no external producer delivery. Every fixture is B-owned,
so the discipline is provable here rather than only through integrations that
wait on someone else.

## Preconditions

B-owned fixtures only. No fixture is an artifact emitted by an external
producer, and no assertion depends on an external revision being published.

## Test Data

Object instances differing only in display name, source path, span, message
text, locale or rendering; producer-authored identities with absent and with
malformed components; consumer-authored identities offered as producer
references and as producer selectors; identical byte sequences carried in each
digest domain `FR-015` selects; and keys with an absent component alongside keys
with a defaulted, empty and present component in the same position.

## Test Procedure

1. Compare object instances differing only in presentation.
   - TC-014-SC-01: identity, equality and ordering position are unchanged by a
     different display name, source path, span, message text, locale or
     rendering; and two instances sharing a display name with different keys
     compare unequal.
2. Submit a producer-authored identity that is absent, then one that is
   malformed.
   - TC-014-SC-02: an admitted producer-authored identity is byte-equal to the
     bytes it arrived as; deriving, repairing, normalizing or recomputing one
     returns a typed refusal rather than a repaired identity.
3. Offer a consumer-authored identity as a producer reference in a canonical
   result, then as a selector to a producer.
   - TC-014-SC-03: both are refused, and no canonical result carries a
     consumer-authored identity in a producer reference position.
4. Present one byte sequence in each selected digest domain and compare across
   every ordered pair.
   - TC-014-SC-04: for both ordered pairs of the two members `FR-015` selects
     from `FR-201`'s published vocabulary, a component from one domain is
     unequal to a component from another over identical bytes, in both
     directions.
5. Compare an absent key component against a defaulted, an empty and a present
   component.
   - TC-014-SC-05: absence compares equal only to absence; it never equals a
     default, an empty value, or a present component, and an operation requiring
     the component refuses rather than assuming one.
6. Inspect every object artifact this repository specifies.
   - TC-014-SC-06: each declares one of the three upstream kinds and one
     authority, declares no fourth kind, and — where its collections admit
     duplicate, surplus, missing or reordered defects — declares a unique key
     and a total order, or names the owner that has not declared one and the
     exact unsettled question.

## Expected Results

Identity is determined by the declared key and nothing else. Presentation is
inert, absence is explicit, digest domains stay separate, producer identities are
copied rather than reconstructed, and consumer identities do not escape the
boundary.

## Metadata

- Priority: P0
- Type: Property
- Automation: Automated; B-owned fixtures only, no external dependency

## Traceability

Verifies FR-011-AC-1 through FR-011-AC-7.
