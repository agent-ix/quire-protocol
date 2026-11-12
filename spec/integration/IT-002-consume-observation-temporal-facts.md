---
id: IT-002
title: "Evaluate compensation and temporal evidence from recipe R"
type: IT
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-003, type: verifies }
  - { target: ix://agent-ix/quire-protocol/FR-004, type: verifies }
  - { target: ix://agent-ix/quire-protocol/FR-006, type: verifies }
  - { target: ix://agent-ix/quire-protocol/FR-045, type: verifies }
  - { target: ix://agent-ix/quire-protocol/FR-046, type: verifies }
  - { target: ix://agent-ix/quire-protocol/FR-047, type: verifies }
---
# IT-002: Evaluate compensation and temporal evidence from recipe R

## Objective

Verify that compensation and temporal evidence evaluation use the admitted
checked package from recipe **R**, QSL's published evaluation entries and
caller-supplied observation evidence. The library preserves owner outcomes and
does not claim to construct the caller's `EvidenceRequest`.

## Target Integration

The producer is QSL's checked-package handoff and evaluation API. The caller
supplies the observation values and the `EvidenceRequest` required by QSL
FR-516. The consumer is this library's compensation assessment, temporal
evidence path and result assembly. QSL owns the checked expressions, their
evaluation, the evidence request and its call causes; this repository owns the
compensation state machine, result mapping and typed refusal propagation.

## Owner Boundaries

| Owner | Supplies | This repository must not |
| --- | --- | --- |
| QSL checked-package and evaluation APIs | Recipe R, checked nodes, temporal clauses, slot outcomes, evidence outcomes and call refusals | Re-evaluate expressions, create an unsupported input, or rename an owner cause |
| Caller | Observation evidence and a corresponding `EvidenceRequest` | Be represented as though its request-construction step were library behavior |
| This repository | Compensation transitions, retry count, charge reporting and result/refusal mapping | Convert unavailable, undefined, refused or incomplete values to Boolean truth |

## Preconditions

The admitted handoff for **R** is available through IT-001. The QSL dependency
provides the public checked-package slot and temporal-evidence evaluation
entries. Test requests are constructed by the caller with values suitable for
R's checked nodes. The integration records each QSL outcome directly and does
not add a second reader, expression evaluator or temporal profile.

## Inputs

R's admitted `Full` and `Partial` compensation rows, true and false values for
their reachable guard and retry slots, unavailable and wrongly typed values,
measured meter usage, caller-built temporal `EvidenceRequest` values for R's
four temporal clauses, the six reachable QSL call-refusal causes, and missing
or withdrawn temporal anchors.

## Test Procedure

1. Evaluate the admitted compensation guards and retry relations with true and
   false supplies, an unavailable binder, a wrongly typed value, and a meter
   one unit below measured use.
   - IT-002-SC-01: each reachable QSL outcome maps to its declared compensation
     successor; non-Boolean outcomes never become Boolean truth. R has no input
     for the unsupported-construct outcome, so this integration makes no such
     claim. R's recovery-predicate rows remain unexecutable while QSL's bare
     projection refuses their comprehension binder.
2. For each retry relation in R, use a declared attempts maximum of `n` and
   exercise `n` attempts followed by attempt `n + 1`.
   - IT-002-SC-02: exactly `n` attempts are admitted, including the initial
     attempt, and `n + 1` returns the retry-bound refusal.
3. Run the compensation assessment in a denied-syscall harness and inspect its
   dependency tree.
   - IT-002-SC-03: assessment completes with no denied I/O syscall and the
     compensation module has no I/O dependency.
4. For each temporal clause in R, construct the `EvidenceRequest` at the caller
   boundary, pass it to QSL's evidence entry and record the result fields that
   this library returns.
   - IT-002-SC-04: activation state, truth, settlement and decision support
     equal QSL's returned values. The test verifies the supplied request at the
     call boundary; no mapping from observation evidence to request is assigned
     to this library.
5. Submit the six reachable refusal requests: missing trigger, unknown
   trigger, unknown occurrence, missing activation mapping, unresolved clause
   root and binding mismatch.
   - IT-002-SC-05: the call refusal and its cause are preserved unchanged, no
     charge is recorded, and caller ledgers remain unchanged. These six are the
     complete set of causes reachable from a caller-built request over an
     admitted package in R; causes that require a rejected package state are
     outside the scenario.
6. Submit requests with an unavailable anchor and a withdrawn anchor.
   - IT-002-SC-06: each settles as a missing non-value and never as Boolean
     truth.
7. Exercise the call through the current QSL evidence interface and compare the
   position values supplied by the assessment with those read by QSL.
   - IT-002-SC-07: `EvidenceRequest` and `EvidenceResult` cross the call as
     in-memory values, and the evaluation reads the position values the
     assessment supplied. The scenario does not define a serialized contract
     for either value.

## Expected Results

Every tested result follows from an admitted R package, caller-supplied
evidence and QSL's current public API. The library preserves owner outcomes and
charges; it does not manufacture unsupported values or observation-to-request
semantics. Missing and withdrawn anchors cannot produce Boolean truth.

## Metadata

- Priority: P0
- Target Integration: checked-package compensation and temporal evaluation
- Automation: Automated local fixture lane through the QSL dependency selected by Cargo

## Dependencies

IT-001's admitted recipe **R**, QSL's package-only slot and temporal evidence
entries, and the caller's observation evidence and `EvidenceRequest`. The
current consumer contract is QSL FR-516 and its checked-package evaluation
surface. The library's own result and compensation semantics remain in FR-003,
FR-004 and FR-045.

## Retired Scenario Counterparts

The former IT-002 procedure mixed QObs runtime/authority inventories, retired
QSL native-temporal request/result contracts, global replay premises, canonical
result assembly, manifest resolution and retained integration evidence. The
current recipe-R integration narrows the QSL-owned evaluation obligations;
each former scenario is accounted for below.

| Retired scenario | Recipe R counterpart or reason it ends |
| --- | --- |
| IT-002-SC-01 | IT-002-SC-01 and IT-002-SC-04 exercise checked compensation and temporal outcomes over R. The previous pinned producer inventory is retired. |
| IT-002-SC-02 | IT-002-SC-01/02/05 cover reachable compensation and temporal outcomes and refusals; the retired activation-binding producer contract is not consumed; the live activation binding is FR-479. |
| IT-002-SC-03 | Ends: R and these evaluation entries do not supply the former QObs scope/progress/closure product; result assembly obligations remain in FR-003/FR-004 and their own evidence. |
| IT-002-SC-04 | Ends: R does not provide complete-global-conformance execution, branch or workflow premise histories; this QSL checked-package integration does not synthesize those inputs. |
| IT-002-SC-05a..05d | Ends: the old batch/incremental replay and equal-time permutation inputs were owned by retired temporal procedures; R's temporal calls are exercised in IT-002-SC-04/06. |
| IT-002-SC-06 | IT-001-SC-04 tests reader limits; IT-002-SC-01 and IT-002-SC-04 exercise evaluation meter behavior. The former combined-history resource scenario is no longer one evaluation boundary. |
| IT-002-SC-07a..07b | Ends: per-scenario retained manifest state and integration-wide refusal identity uniqueness were properties of the retired manifest/evidence procedure, not the R evaluation API. |
| IT-002-SC-08 | Ends: the former QObs/QSL contract inventory and schema-digest manifest are not inputs to recipe R. FR-009 continues to govern the interfaces the current integration actually consumes. |

## Traceability

Verifies FR-003, FR-004, FR-006, FR-045, FR-046 and FR-047. IT-002-SC-01..03
verify compensation behavior and FR-045; IT-002-SC-04..07 verify FR-046. The
counterpart table and the current R-based procedures verify FR-047-AC-3.
