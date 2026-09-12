---
id: IT-001
title: "Consume the native compiler protocol artifact"
type: IT
relationships:
  - target: ix://agent-ix/quire-protocol/FR-001
    type: verifies
  - target: ix://agent-ix/quire-protocol/FR-002
    type: verifies
---
# IT-001: Consume the native compiler protocol artifact

## Objective

Verify that the native Quire compiler's selected protocol artifact is admitted
and linked through the public Rust boundary without a copied parser, alternate
formal frontend, or identity reinterpretation.

## Target Integration

The producer is the A-owned native Quire compiler artifact interface. The
consumer is this library's strict protocol admission and linking API.

## Preconditions

The consumer has independently selected and retained the compiler contract,
baseline, producer, native-source, model-source, and dependency bytes before it
inspects an offered package. The native compiler publishes versioned
compiled-protocol contracts. The existing `/1` candidate retains its
`quire.protocol.numeric/1` wire profile and was generated at immutable revision
`5690c2b29c155ec72e21f5a7bb00fe1146b6f71a` (PR72). The `/2` intake is a
separate contract: B must receive A's exact
`protocol_artifact::v2::AdmittedPackage` and independently constructed
`protocol_artifact::v2::ExpectedSelection` inputs, plus the original offered
artifact bytes. Neither version may select, repair, or upgrade the other.

This candidate fixture is integration evidence only. Its producer PR and the
remaining acceptance/traceability item in `quire-spec-language#40` must close
before it can establish a promoted baseline or complete FR-001/FR-002.

## Inputs

For `/1`: the retained four-unit compiled package; independently selected
original native sources, model source, admitted model, and dependency closure.
For `/2`: A's raw emitted package bytes; B-retained original source, model,
definition, profile, configuration, producer, artifact, and result selections;
and A's admitted typed package/expected-selection boundary. Both lanes use
one-axis mutations of offered wire/version, seal, producer/source/dependency/
baseline identity, feature, a binding actually referenced by the package,
numeric representation, numeric domain, and digest domain. Numeric inputs
include fields whose declared domains admit `i64::MIN`, `i64::MAX`, and reduced
exact rationals `(i64::MAX, i64::MAX - 1)` and `(i64::MIN, i64::MAX)`.

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
6. Submit the exact `/2` bytes and independently retained `/2` selection through
   `quire_protocol::admit_and_link_v2`; submit each version at the other
   version's API and reseal one changed `/2` selection axis at a time.
   - IT-001-SC-06: `/2` retains the exact A-provided artifact bytes and returns
     A's `/2` admitted package only for the independently selected `/2` inputs;
     `/1` at the `/2` intake and `/2` at the `/1` intake refuse without
     auto-upgrade. Changed definition, profile, configuration, source, model,
     producer, artifact, or result digest-domain selections, and any
     recanonicalized producer input, refuse before a package is exposed.

## Expected Results

Only the exact independently selected package for its explicit contract version
is admitted and linked. Every mismatch is a typed refusal before a linked
package is exposed, and every admitted numeric endpoint re-encodes to the
offered canonical bytes. The integration uses the A-owned public reader
directly; it does not make the candidate producer revision, display names, or
fixture sidecar self-authorizing. Admission establishes no PT02, observation
adequacy, evaluation, or campaign-completion result.

## Metadata

- Priority: P0
- Target Integration: native compiler artifact to protocol library
- Automation: Automated fixture lane; promotion remains blocked by the external
  compiler-contract acceptance gate

## Dependencies

The A-owned frozen profile and each versioned compiled-protocol artifact contract
must be accepted. `quire-spec-language` PR72 at `5690c2b` supplies the signed-64
`/1` endpoint candidate. `quire-spec-language#40` owns `/2` contract
acceptance, traceability, and the public typed boundary. Before the `/2` lane
can run, A must provide immutable fixture bytes and artifact reference; original
unrecanonicalized producer inputs and their selections; exact public names and
constructors for `v2::AdmittedPackage`, `v2::ExpectedSelection`, limits and
refusal/report types; and documented mutation/refusal expectations for each
selection axis. A prose definition, native source file, emitted sidecar, or a
`/1` value cannot substitute for either compiled interface because this
repository owns no parser.

## Notes

The generated fixture is at
`/tmp/quire-native-handoff-signed64-endpoints/`. Its embedded producer ELF
digest changes on rebuild and is intentionally not pinned; B derives all stable
selections from the retained generated inputs. The complete positive parser-set
proof for SC-05 is still an open integration obligation, not an absence-of-known
bad-dependencies claim.

## Traceability

Verifies FR-001 and FR-002.
