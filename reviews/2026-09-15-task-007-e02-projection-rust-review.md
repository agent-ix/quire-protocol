---
id: SR-118
title: "Rust review of the Task-007 E02 projection"
type: SpecReview
analysis: code-review
scope: "Task-007 / quire-protocol#9, FR-007/FR-034/FR-035 and TC-007"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-protocol/Task-007, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-007, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-034, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-035, type: reviews }
  - { target: ix://agent-ix/quire-protocol/TC-007, type: reviews }
---
# SR-118: Rust Review of the Task-007 E02 Projection

## Summary

Reviewed the request-bound qverify resolution, canonical-source validation,
FR-062 mapping, projection-identity preimage, qverify strict-reader boundary,
typed refusal routing and TC-007 corpus. All in-scope Rust findings were fixed.

## Verdict

**PASS.** The projection is source-bound, exhaustive over the selected closed
source enums, accepted by qverify's exact strict reader, and compiled against
the owner-published dependency-compatible revision. Task-008/#10 separately
owns Protocol-wide accounting and bound integration for this public operation.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | Resolved: qverify `VerificationErrorCode` used a wildcard mapping to `finding-loss`, so a new variant at a later exact pin could compile and silently acquire the wrong Protocol refusal. The match names every owner variant exhaustively; promotion to `293e342` deliberately maps its four model-domain variants to conservative `finding-loss` and traces those decisions in TC-007. | FR-035-AC-6/7; TC-007 | implementation-bug-despite-evidence |
| FND-002 | medium | Resolved: the frozen dependency, source-module and conditional-compilation inventories did not yet name qverify, its transitive closure, `projection.rs`, or the new `cfg(test)` site. The full gate exposed each omission and the exact inventories now include them. Quire CLI #90 separately records that its Rust scanner ignores rustfmt-formatted multiline trace attributes; short stacked compiler-checked attributes retain the exact bindings until the scanner is fixed. | NFR-004; NFR-006; TC-012 | implementation-bug-despite-evidence |
| FND-003 | low | Resolved: TC-007 initially asserted only the derived-ID shape and a subset of target fields. It now independently recomputes the normative qverify JCS preimage, compares every selected target field, mutates every identity-preimage member, and checks producer/adapter provenance. | FR-034-AC-1..7; TC-007 | correct-requirement-no-evidence |
| FND-004 | low | Resolved: the first mapping corpus omitted several closed source members. It now covers all execution, truth, activation, participation, assumption, observation-adequacy and dependence-relation members plus simultaneous findings. | FR-035-AC-1..5; TC-007 | correct-requirement-no-evidence |

## Review Checks

| Check | Result |
| --- | --- |
| Ownership | qverify supplies generated references and `TechniqueResult`, the E02 version constant, RFC 8785/JCS operations, resource-envelope reader and strict contract validator. Protocol declares only its mapping/context/refusal boundary. |
| Identity | The result identity uses the exact FR-062 object and semantic `bindingRefs` order. Source, every target-binding preimage member, target contract and adapter mutations change identity or refuse. |
| Mapping | Closed Rust enums are exhaustively matched. Independent finding axes remain separate, all four source dependence relations survive, and no protocol adequacy or sufficiency value enters E02. |
| Wire construction | E02 deliberately declares open finding objects. Protocol constructs that owner-defined open detail as JSON, immediately validates it through qverify, retains the exact canonical bytes, and separately exposes qverify's generated typed view; it defines no mirror wire struct or enum. |
| Panic and unsafe | No changed request path contains input-controlled indexing, `unwrap`, unsafe code, recursion, async bridge, lock or callback. Remaining `expect` sites are compile-time nonempty constants and catalog/stage registrations. |
| Bounds | Source reading and target rereading use their owners' bounds. Protocol accountant integration, pre-allocation charges and resource-incomplete mapping remain explicitly owned by Task-008/#10 and are not claimed here. |
| Tests | Four bare-traced TC-007 symbols cover the exact identity, total mapping, simultaneous findings, all target/source mutations, owner strict reread and the seven-code refusal boundary. Every FR-007, FR-034 and FR-035 criterion is compiler-checked and visible to Quire despite the native scanner regression tracked by [quire-cli#90](https://github.com/agent-ix/quire-cli/issues/90). |

## Verification

Against exact qverify `293e342c6f5e2c69738956ff5165f75761b0364a` and transitive
Engineering Assurance `9237ce50ce19c6d6c937705bfde4b64adcc9d16b`, the exact
formatting, warnings-denied all-target Clippy and complete-test local gate
passes over one unchanged tracked state. Quire reports 245/245 documents
grammar-clean; native Quoin `0.23.1-86-gc721d71` reports no findings. The
manifest, lockfile and rights record contain no temporary path source.
