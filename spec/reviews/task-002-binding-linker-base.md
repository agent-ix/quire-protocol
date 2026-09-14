---
id: SR-087
title: "Base review of Task-002 exact binding linker"
type: SpecReview
analysis: base
scope: "FR-002, TC-002, Task-002 and TM-001 against Protocol 2cc2439, QSL 7931a00, QSpec 99951ee and QObs 93bc565"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-002, type: reviews }
  - { target: ix://agent-ix/quire-protocol/TC-002, type: reviews }
  - { target: ix://agent-ix/quire-protocol/Task-002, type: reviews }
---
# SR-087: Base review of Task-002 exact binding linker

## Summary

Task-002 was reviewed against the implemented static linker, the exact pinned
compiled-protocol Rust records, the accepted Quire Specification PR #41/#42
objects and Quire Observation PR #20's constructor-guarded FR-287/FR-262
values. The revised requirement now defines one coherent linker subsystem
without treating static slots as runtime subjects or reimplementing compiler
admission.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Resolved: the first draft let a QSL `BindingKind` imply a QObs producer-local `SubjectKind`, although FR-210 expressly supplies no universal producer wire enum. The accepted static binding definition now carries an independently selected slot-to-producer-kind/correspondence fact, and the linker is forbidden to infer the mapping from equal spelling. | FR-002 Inputs/Behavior; TC-002 |
| FND-002 | high | Resolved: role internality was mixed into the runtime set even though FR-217 makes it static FR-288 consistency content and FR-288 excludes runtime values. The reviewed boundary separates an accepted static binding definition from the runtime subject/relationship set; only the static selection participates in FR-288. | FR-002 Inputs/Behavior/AC-4 |
| FND-003 | high | Resolved: same-FR-288/different-content refusal was required without an input capable of remembering prior content. The definition registry is now an explicit input and Task-002 owns the cross-link contradiction check. | FR-002 Inputs; Task-002 |
| FND-004 | medium | Resolved: the first relationship rule retained a second key without considering its slot. FR-262 requires two distinct candidates for one required slot to be ambiguous; two keys remain distinct only when they satisfy distinct slots. | FR-002 Behavior; TC-002 |
| FND-005 | medium | Resolved: the first draft required every runtime failure to have a source locus although imported FR-287 values carry no locus. Diagnostics now name the exact static handle or runtime-entry ordinal, retain a real originating locus when present, and never manufacture one. | FR-002 Outputs/AC-3 |
| FND-006 | medium | The complete FR-288 selection is derivable from the admitted version-2 package plus the accepted static binding definition: exact source artifact identity/revision/raw digest, source-qualified declaration handle, requirement/clause, language, profile, model, dependency and correspondence selections are all present. Concrete runtime subjects, observations and resource ceilings remain excluded. | QSL `protocol_artifact::v2::AdmittedPackage`; FR-288 |
| FND-007 | low | The current `ControlShape`, numeric fingerprint and FIFO position tuple are explicitly partial. Task-002 replaces them with complete typed consistency content and an owner-typed generic FIFO partition; no new digest domain or approximate evaluator is introduced. | `src/link/mod.rs`; FR-002; Task-002 |
| FND-008 | low | QSL #85 asks about a sixth duplication premise under superseded wording. Accepted FR-216 has exactly five premises and derives duplication permission from delivery cardinality, so #85 is a tracking cleanup rather than an implementation dependency. | FR-216; QSL #85 |

## Reviewed subsystem boundary

- Compiled-protocol admission, typed graph validation and numeric parsing stay
  in QSL. Protocol consumes only an admitted version-2 capability.
- Observation owns construction of authority-qualified subjects and producer-
  validated relationships. Protocol imports those opaque values and binds them
  to static requirements; it authors no duplicate observation wire record.
- The accepted static binding definition owns only Protocol's consumer-side
  selection boundary: contract version, exact selected artifact, role
  classification and slot compatibility. It does not mint runtime identities.
- The definition registry detects one qualified key associated with different
  static consistency content. It does not hash or flatten the graph.
- FIFO evaluation remains with the value owner. Protocol retains the typed
  expression definition and pairs a channel with an owner-typed evaluated value
  without interpreting or approximating it.

## Decision

No blocking ownership or interface ambiguity remains for Task-002. The Rust
implementation may proceed in the order recorded in Task-002, with all new
claims remaining planned until executable trace-bound evidence exists.
