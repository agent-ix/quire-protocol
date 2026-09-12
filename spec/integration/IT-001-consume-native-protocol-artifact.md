---
id: IT-001
title: "Consume the native compiler protocol artifact"
type: IT
relationships:
  - target: ix://agent-ix/quire-protocol/FR-001
    type: verifies
  - target: ix://agent-ix/quire-protocol/FR-002
    type: verifies
  - target: ix://agent-ix/quire-protocol/NFR-002
    type: verifies
---
# IT-001: Consume the native compiler protocol artifact

## Objective

Verify that the native Quire compiler's selected protocol artifact is admitted
and linked through the public Rust boundary without a copied parser, alternate
formal frontend, or identity reinterpretation, and that each published contract
version is consumed only through its own version-explicit entry point.

## Target Integration

The producer is the A-owned native Quire compiler artifact interface. The
consumer is this library's strict protocol admission and linking API. The `/1`
and `/2` compiled-protocol contracts are two separate integrations that share
this test; neither may stand in for the other.

## Preconditions

The consumer has independently selected and retained the compiler contract,
baseline, producer, native-source, model-source, and dependency bytes before it
inspects an offered package. The native compiler publishes versioned
compiled-protocol contracts. The existing `/1` candidate retains its
`quire.protocol.numeric/1` wire profile and was generated at immutable revision
`5690c2b29c155ec72e21f5a7bb00fe1146b6f71a` (PR72). The `/2` intake is a
separate contract: B must receive A's exact
`protocol_artifact::v2::AdmittedPackage` and independently constructed
`protocol_artifact::v2::Expected` and `protocol_artifact::v2::ExpectedTemporal`
inputs through `protocol_artifact::v2::read`, plus the original offered artifact
bytes. Neither version may select, repair, upgrade, or downgrade the other, and
neither may be detected from the offered bytes.

This candidate fixture is integration evidence only. Its producer PR and the
remaining acceptance/traceability item in `quire-spec-language#40` must close
before it can establish a promoted baseline or complete FR-001/FR-002.

## Inputs

For `/1`: the retained four-unit compiled package; independently selected
original native sources, model source, admitted model, and dependency closure.

For `/2`: A's raw emitted package bytes and external artifact reference;
B-retained original, unrecanonicalized source, model, definition, profile,
configuration, dependency, producer, artifact, and result selections; the
B-retained temporal profile, clock alternative, and clock parameter selections
bound by that package; and A's typed `v2::AdmittedPackage` report and refusal
boundary.

Both lanes use one-axis mutations of offered wire/version, seal,
producer/source/dependency/baseline identity, feature, a binding actually
referenced by the package, numeric representation, numeric domain, and digest
domain. The `/2` lane additionally uses one-axis mutations of the temporal
profile, clock alternative, and clock parameter selections; temporal binding
sets that are missing, surplus, duplicated, reordered, owned by a foreign
owner, or out of the declared range; recanonicalized source, model, dependency
and configuration inputs; and digest-domain substitution across the
producer-canonical, native-raw, compiled-artifact, and protocol-result domains.
Numeric inputs include fields whose declared domains admit `i64::MIN`,
`i64::MAX`, and reduced exact rationals `(i64::MAX, i64::MAX - 1)` and
`(i64::MIN, i64::MAX)`. Resource inputs hold one `/2` input constant and
vary the budget: exactly the amount that input consumes on each declared limit
dimension, one unit short of it, and a fresh budget at the exact value.

## Test Procedure

1. Construct `Expected` only from B-selected retained inputs, then submit the
   exact producer package through `quire_protocol::admit_and_link`.
   - IT-001-SC-01: admission retains artifact, contract, baseline, producer,
     native-source, model-source, and dependency identities unchanged; the
     producer's `expected.json` is not read as authorization.
2. Inspect the public admitted package and its source/declaration links.
   - IT-001-SC-02: all four source units, every declaration/reference, and the
     actual `Workflow.wide`/`Workflow.exact` binding path agree with the selected
     producer inputs.
3. Reseal each single-axis mutation and submit it to the same public API.
   - IT-001-SC-03: a changed wire/version, artifact seal, producer/source/
     dependency/baseline identity or digest, unknown feature, or referenced
     binding returns a typed refusal before a linked package is exposed.
4. Submit signed-64 and exact-rational endpoint values and invalid or lossy
   representations through their declared fields.
   - IT-001-SC-04: admitted endpoint values canonically round-trip byte for
     byte; bare JSON numbers, non-canonical forms, signed-64 or field-domain
     overflow, non-positive or unreduced denominators, wrong numeric kinds, and
     producer/result digest-domain substitution refuse before linking.
5. Inspect the resolved direct and transitive consumer path and the public API
   call used by the integration harness.
   - IT-001-SC-05: the evidence positively identifies the A-owned reader as the
     only native compiler reader and identifies no second parser, formal
     frontend, or foreign runtime in the admitted path.
6. Submit the exact `/2` bytes and an independently retained `/2` selection,
   built as `v2::Expected` and `v2::ExpectedTemporal` without reading A's
   offered bytes, through `quire_protocol::admit_and_link_v2`; submit each
   version's bytes at the other version's entry point; and reseal one changed
   `/2` selection axis at a time.
   - IT-001-SC-06: `/2` retains the exact A-provided artifact bytes and external
     artifact reference through linking and returns A's `v2::AdmittedPackage`
     only for the independently selected `/2` inputs. `/1` bytes at
     `admit_and_link_v2` and `/2` bytes at `admit_and_link` each return a typed
     refusal with no auto-detection, upgrade, or downgrade. A changed artifact,
     contract, baseline, producer, source, model, dependency, definition,
     configuration, result, temporal-profile, clock-alternative, or
     clock-parameter selection refuses before a package is exposed, as does a
     resealed change to the offered wire/version, seal, feature, referenced
     binding, numeric representation, or numeric domain. Each refusal is
     distinguishable from the others by its typed identity; one shared cause
     across two axes does not discharge this scenario.
   - IT-001-SC-06a: `admit_and_link` is unchanged by the addition of `/2`,
     shown by re-running SC-01 through SC-05 on the same `/1` fixture at the
     revision that adds `/2` and obtaining the same admitted identities and the
     same typed refusal per axis as at the revision before it, with
     `admit_and_link`'s signature and refusal set unchanged.
7. Substitute each digest domain for another across the same `/2` selection, and
   submit recanonicalized source, model, dependency and configuration inputs.
   - IT-001-SC-07: a producer-canonical, native-raw, compiled-artifact, or
     protocol-result digest presented in another domain's slot returns a typed
     refusal, and a producer input that B recanonicalized instead of copying
     byte for byte refuses rather than admitting an equivalent form.
8. Submit `/2` temporal binding sets that are missing, surplus, duplicated,
   reordered, owned by a foreign owner, or out of the declared range.
   - IT-001-SC-08: each returns a typed refusal that names the offending
     temporal binding axis before a linked package is exposed; a correctly
     ordered, singly owned, in-range binding set admits.
9. Submit one unchanged `/2` input under a budget set exactly to what that
   input consumes on each declared limit dimension, then under a budget one unit
   short of it, then resubmit the same input under a fresh budget at the exact
   value.
   - IT-001-SC-09: the exactly sufficient budget admits, the one-short budget
     returns that dimension's typed resource refusal before a linked package is
     exposed, and the fresh-budget resubmission of the same bytes admits, so no
     budget is consumed or retained across a refused attempt.

## Expected Results

Only the exact independently selected package for its explicit contract version
is admitted and linked, through that version's own entry point. Every mismatch
is a typed refusal before a linked package is exposed, and every admitted
numeric endpoint re-encodes to the offered canonical bytes. The integration uses
the A-owned public reader directly; it does not make the candidate producer
revision, display names, or fixture sidecar self-authorizing. Admission and
linking establish no PT02, observation-adequacy, replay-agreement, evaluation,
or campaign-completion result.

## Metadata

- Priority: P0
- Target Integration: native compiler artifact to protocol library
- Automation: Automated fixture lane; promotion remains blocked by the external
  compiler-contract acceptance gate

## Dependencies

The A-owned frozen profile and each versioned compiled-protocol artifact contract
must be accepted. `quire-spec-language` PR72 at `5690c2b` supplies the signed-64
`/1` endpoint candidate. `quire-spec-language#40` owns `/2` contract
acceptance, traceability, and the public typed boundary.

Before the `/2` lane can run, A must provide: an immutable revision to pin; the
raw `/2` fixture bytes and their external artifact reference; the independent
selection recipe B needs to rederive `v2::Expected` and `v2::ExpectedTemporal`
without reading the offered bytes; the original, unrecanonicalized producer
input corpus; the per-axis mutation fixtures and their documented refusal
expectations; and the exact public names, constructors, limits, report and
refusal types of `protocol_artifact::v2`. A prose definition, native source
file, emitted sidecar, hand-authored bytes, or a `/1` value cannot substitute
for either compiled interface because this repository owns no parser.

## Notes

The generated `/1` fixture is at
`/tmp/quire-native-handoff-signed64-endpoints/`. Its embedded producer ELF
digest changes on rebuild and is intentionally not pinned; B derives all stable
selections from the retained generated inputs. The complete positive parser-set
proof for SC-05 is still an open integration obligation, not an absence-of-known
bad-dependencies claim.

The `/2` lane's fixture loader, mutation corpus, and scenario enumeration are
authored ahead of A's delivery. They add no `[features]` entry and no `cfg`
attribute, so the crate stays single-lane under NFR-004-AC-7: the corpus builder
compiles and is checked on every gate run, and the parts that need A's boundary
live in one `#[ignore]` lane that fails explicitly rather than passing vacuously.
Until A's revision is pinned, SC-06 through SC-09 assert nothing about A's
behaviour and remain unexecuted; the `/1` SC-01 through SC-05 lane is unchanged.

## Traceability

Verifies FR-001 and FR-002. SC-07 and SC-09 additionally verify NFR-002: the
digest domains that must stay separate, and the declared `/2` resource limits at
their exact bound, one unit short, and on fresh retry.
