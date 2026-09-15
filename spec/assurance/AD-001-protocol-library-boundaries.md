---
id: AD-001
title: Complete Protocol implementation architecture
type: ArchitectureDescription
status: accepted
owner: agent-ix-protocol-owner
system: quire-protocol Rust library
relationships:
  - target: ix://agent-ix/quire-protocol/ADR-002
    type: references
  - target: ix://agent-ix/quire-protocol/ADR-004
    type: references
  - target: ix://agent-ix/quire-protocol/ADR-005
    type: references
  - target: ix://agent-ix/quire-protocol/ADR-006
    type: references
  - target: ix://agent-ix/quire-protocol/ADR-007
    type: references
  - target: ix://agent-ix/quire-protocol/ADR-009
    type: references
  - target: ix://agent-ix/quire-protocol/FR-001
    type: realizes
  - target: ix://agent-ix/quire-protocol/FR-002
    type: realizes
  - target: ix://agent-ix/quire-protocol/FR-003
    type: realizes
  - target: ix://agent-ix/quire-protocol/FR-004
    type: realizes
  - target: ix://agent-ix/quire-protocol/FR-005
    type: realizes
  - target: ix://agent-ix/quire-protocol/FR-006
    type: realizes
  - target: ix://agent-ix/quire-protocol/FR-007
    type: realizes
  - target: ix://agent-ix/quire-protocol/FR-008
    type: realizes
  - target: ix://agent-ix/quire-protocol/FR-009
    type: realizes
---
# Complete Protocol implementation architecture

## System Boundary

`quire-protocol` is the pure Rust consumer and assessment library between exact
external producer contracts and two derived output boundaries. It resolves the
contracts selected for one request, admits a versioned QSL compiled protocol,
binds its static requirements to exact QObs runtime values, evaluates
compensation and conformance without performing business work, produces strict
canonical Protocol results, and optionally projects those results to E02.

External responsibility is explicit:

| Responsibility outside Protocol | Owner |
| --- | --- |
| grammar, source compiler, compiled protocol and native-temporal evaluator | QSL |
| observation collection and observation authority | QObs |
| TL syntax/evaluation and native/TL correspondence | TL libraries / Contract-IR |
| producer-process execution | Engineering Assurance |
| evidence storage/indexing and catalogue operations | Quoin |
| verification-catalogue semantics | owning verification system |
| sufficiency decision and human disposition | human/campaign owner |

The architecture is complete for Tasks 002 through 013 and 017 through 018. Those tasks
implement these decisions and SHALL NOT introduce another stage model, identity
vocabulary, fallback backend, resource guard, external adapter shape or result
assembly path. A future incompatible owner contract is an explicit unresolved
dependency, not authority to approximate or redesign it locally.

## Views

The following views are jointly normative: topology fixes dependency direction;
contract resolution fixes authority; static/runtime linking fixes identity;
stateful assessment fixes transactionality; result/projection fixes output
ownership; and refusal/resource/reproduction fixes cross-cutting behavior.

### Subsystem topology

```text
contract resolution
  |-- QSL /1 -> version-1 admission -> static link regression
  `-- QSL /2 -> version-2 admission -> static definition + runtime bindings
                                             |
                                             v
                                      exact linked graph
                                             |
QObs runtime subjects/relationships --------+----> compensation
QObs authority facts -----------------------+          |
QSL native-temporal results + FR-300 bindings -+        v
                                         conformance kernel
                                             |
                                             v
                                      claim dispositions
                                             |
                                             v
                                   canonical Protocol results
                                      |                 | \
                              Contract-IR view      E02 projection \
                                      |                                \
                         QCI predicate projection                 Quoin intake
                                      |
                         QCI native/TL valuation + result join

Engineering Assurance request -> Protocol adapter -> Engineering Assurance result
```

Bounds, refusals, version provenance and reproduction evidence are cross-cutting
services. They do not form alternate entry points around the typed path.

### Contract resolution and capability view

The caller supplies one manifest plus the exact offered revisions, interfaces
and artifact bytes. Resolution performs no network, VCS, process or filesystem
discovery. It returns constructor-private `ResolvedArtifact<C>` and
`ResolvedInterface<C>` capabilities bound to the resolution/request identity.
Every integration adapter that reads an external value requires the capability
for that exact owner contract version; the frozen `/1` facade exception below is
not an integration adapter and admits no other external value.

Manifest keys name owner contract versions, not repositories or informal roles.
The manifest may comprise QSL compiled-protocol `/1` and `/2`, QSL native-
temporal request/result contracts, QObs runtime and authority contracts,
Engineering Assurance producer-execution contracts, accepted E02 draft-2, and
the accepted Quoin protocol-scenario intake contract. Each operation declares
and resolves only the exact contract versions it consumes; no request is forced
to offer both QSL versions or an unused integration. Protocol-owned binding,
result, mapping and scenario-record contracts have no external entry. QCI is
downstream of the Protocol result map and therefore has no Protocol intake
entry.

The exact presently selected entry vocabulary is:

| Adapter | Required contract keys |
| --- | --- |
| QSL compiled `/1` | `quire.compiled-protocol/1` |
| QSL compiled `/2` | `quire.compiled-protocol/2` |
| QSL temporal | `quire.native-temporal-request/v1`, `quire.native-temporal-result/v1` |
| QObs runtime/authority | `quire.observation.record/v2`, `population/v2`, `explicit-members/v1`, `position-ledger/v1`, `clock-binding/v1`, `capture-environment/v1`, `progress-assertion/v1`, `closure-assertion/v1`, `completeness-assertion/v1`, `result-availability/v1` under the QObs owner namespace |
| Engineering Assurance | `engineering-assurance.producer-execution-request/v1`, `engineering-assurance.producer-execution-result/v1` |
| E02 | `quire-verification/e02-draft-2` |
| Quoin | the not-yet-accepted Protocol scenario-intake contract |

The adapter source compiles an independent accepted owner, contract-version and
revision selection for every interface-only entry. Resolution compares that
selection before minting a capability; a manifest assertion cannot authenticate
the linked implementation. Artifact capabilities also retain the exact verified
bytes. Neither capability may be reused for another request or offered value.
Required keys are validated in lexicographic key order for deterministic first-
cause reporting, never as causal precedence.

An adapter declares its required keys independently of the offered manifest. An
offer cannot authorize itself. Resolution has exactly the `pinned` and
`unresolved` outcomes defined by FR-009. Missing, provisional, unknown,
digest-mismatched or exhausted resolution never yields a partial capability.

### Admission, binding and graph view

The public path is version explicit:

- `intake::v1::admit` and `model::v1::link_static` own the `/1` compatibility
  lane.
- `intake::v2::admit` and `model::v2::link` own full assessment intake.
- `assessment::Engine::evaluate_batch` and `apply_incremental` accept only a
  linked version-2 context.
- `output::read_result`, `output::mapped_result_view`,
  `output::e02::project` and `qualification::compose_scenario_evidence` are the
  outbound boundaries.

The already published root `admit_and_link(bytes, expected, limits)` remains an
explicit legacy `/1` compatibility facade with its frozen signature and refusal
set. It delegates only to `/1` admission and static linking and is not an
integration boundary: it cannot consume QObs, QSL native-temporal, Engineering
Assurance, E02 or Quoin values. New integration and full-assessment entry points
require resolved capabilities. This compatibility exception is narrow and
testable; it neither mints a capability nor bypasses resolution for any external
value used by an assessment.

Version-2 linking consumes one admitted QSL package, one accepted
`quire.protocol.binding-definition/1`, one
`quire.protocol.runtime-binding/1`, and a caller-owned definition registry. The
static definition targets one exact admitted declaration and retains its
independent artifact selection, each role's exact FR-217 internality, and each
slot's selected producer authority, producer-local kind and correspondence.
Relationship slots additionally retain their selected declaration and ordered
endpoint requirements. The runtime set targets that exact static definition and
contains imported QObs `QualifiedSubject`, `Relationship` and
`RequiredRelationship` values; it defines no local copy of their wire shape.

Static requirements and runtime instances use non-interchangeable Rust types.
Workflow, role, participant, component, endpoint, channel, message, send,
receive, delivery, attempt, effect, relationship and node-occurrence identities
remain distinct. Runtime wrappers retain the complete FR-287 key and use private
constructors that validate the slot. A source handle, table ordinal, display
name or static requirement never becomes a runtime identity.

The graph projector matches QSL's closed typed enums exhaustively and retains
the complete typed declaration. It never serializes to generic JSON and deletes
selected keys. Exact QSL `ProtocolNumber`, rational, interval, value expression,
type, guard, branch, join, iteration, channel, compensation and finish content
is retained. Numeric fingerprints are forbidden as consistency content.
Presentation metadata and source loci are stored separately from semantic
equality, while genuine semantic selections remain in the FR-288 closure.

The FR-288 key is componentwise: source document, source-qualified declaration,
formal owner/clause and complete semantic selection closure. The definition
registry maps that key to complete typed consistency content. Exact replay is a
no-op; same key with different content refuses transactionally. It is never a
flat graph hash.

All five channel premises remain explicit. FIFO retains its exact channel,
binder, typed key-expression definition, selected type and semantic
dependencies. Runtime partitioning pairs that channel with the exact owner-
evaluated value. Equal values on different channels remain distinct, and no
FIFO partition or input order creates a causal edge.

Opaque runtime identity bytes never pass through UTF-8 conversion. Where the
existing string-only canonical-result contract must carry a runtime reference,
the adapter uses a versioned reversible string: a fixed domain prefix followed
by lowercase hexadecimal encoding of the RFC 8785 canonical object containing
the complete authority key, kind bytes and identity bytes. The decoder verifies
the prefix, canonical bytes and byte-exact round trip before use.

### Stateful assessment view

`assessment::Engine` is created for one exact linked assessment and cannot be
retargeted to another request. It borrows or owns the immutable definition
registry and contains only that assessment's compensation ledgers and
incremental conformance state. Every retained item is charged to the same
finite request budget before commit; completing and dropping the Engine releases
the state, so bounded requests cannot accumulate lifetime-unbounded retained
state across Engine instances. Operations compute into
private candidates and commit only a complete accepted transition. A refusal,
unsupported construct, contradiction or resource-incomplete outcome leaves the
prior externally observable typed snapshot, canonical serialization and
resource-accounting semantics unchanged. Tests compare an owned snapshot before
and after the operation and then replay the same accepted continuation; they do
not inspect Rust object bytes, allocation addresses or padding.

Compensation requests are derived from the linked graph and runtime bindings.
Their registration key is the FR-288 definition plus declaration-local
compensation handle paired with the concrete FR-287 effect. Distinct admitted
attempts count from the initial attempt, exact replay counts once, and excess
retry refuses transactionally. Only the selected commit node creates a cutoff;
`commit never` creates none. Missing evidence is incomplete, while foreign,
ambiguous or contradictory identity is a refusal. Assessment invokes no
business operation or compensation.

Recovery evaluation accepts no public callback. The production evaluator walks
only the admitted QSL typed recovery expression selected by the registration,
using its resolved state/temporal capability and exact captured values. A
recognized expression evaluates under the request accountant; an unknown or
out-of-profile expression returns the typed unsupported disposition, and a
source evaluator refusal or resource-incomplete outcome is retained without
conversion. Test-only relation doubles remain crate-private and cannot enter a
production Engine.

One deterministic `ConformanceKernel` derives obligations from the linked graph
and consumes the compensation snapshot, QObs authority facts, QSL native-
temporal results and complete QSpec FR-300 activation bindings. The binding is
validated against the exact linked definition/control before its clause subject
and semantic trigger derive an FR-230 obligation; a table coordinate, timestamp,
arrival order or caller-built model cannot substitute. Batch and incremental APIs
are frontends over that kernel.
Incremental readiness follows authored causal prerequisites and exact semantic
keys, never timestamp, arrival or map order. Replaying an accepted prefix is
idempotent; a late contradiction creates the specified immutable correction or
refusal without rewriting an earlier result.

Claims consume the conformance report. The finite global-conformance claim is
derived directly. The other closed claim kinds retain their independent
disposition and exact fragment/backend/premise selections. Production exposes
no caller-provided backend callback: a claim without an accepted local backend
is explicitly unsupported, and no fallback runs. Requested claim order is
preserved; one claim's failure cannot promote, suppress or reorder a sibling.

An `AssessmentBundle` retains the linked definition reference, compensation
snapshot, conformance report, ordered claim outcomes and complete resource and
provenance records. Each admitted requested claim has exactly one ordered
`ClaimOutcome`: either a strict-reader-valid canonical FR-241 row (including the
semantic `completed`, `refused`, `unsupported`, `failed` and
`resource-incomplete` dispositions) or a claim-local `ProtocolRefusal` showing
that no valid row could be constructed. Independent sibling outcomes survive.
The public operation returns `Result<AssessmentBundle, ProtocolRefusal>`;
resolution, admission, linking or whole-assessment invariant failure is the
outer refusal and returns no bundle. Resource exhaustion after a claim is
admitted is the canonical `resource-incomplete` disposition, not an outer
refusal. Projection has its own `Result` and never changes its valid source
result. This two-level algebra prevents a semantic `refused` row, a protocol
construction failure and resource incompleteness from being collapsed.

### Output and integration view

Activation, assumptions, adequacy, participation, claim kind/strength,
settlement and decision-support values have one shared internal domain owner.
Compensation, conformance, claims and result assembly use those types rather
than redeclaring wire-shaped equivalents. Canonical result assembly is crate-
private and accepts only validated subsystem outputs. The strict reader remains
the constructor for `ValidatedResult` from external bytes.

The borrowed `MappedResultView` is the sole Contract-IR result boundary. QCI's
accepted FR-025 predicate bridge consumes Protocol leaf-result views to build
checked Boolean valuations; QCI FR-026 then constructs sibling QSL-native and TL
inputs from those valuations and joins the two formula-wide owner results. A
Protocol predicate-result view is not itself a native-temporal result. Protocol
consumes QSL native-temporal results directly and never consumes QCI's join;
reversing that edge would create a package and semantic-authority cycle.

E02 projection accepts a `ValidatedResult` and the complete FR-062 obligation,
plan, task, run, profile, binding, dependency-closure, execution-scope and
adapter context. It implements the accepted total mapping, constructs the exact
derived identity, and passes its candidate through qverify's strict draft-2
reader before returning it. It emits no sufficiency decision.

Engineering Assurance owns producer execution: an EA request enters the
Protocol adapter and the typed EA result returns to that adapter. This flow is
independent of E02 projection. Protocol composes the FR-017 scenario record
containing the full or partial resolution state, owner of every consumed value,
typed outcome identity, every granted/consumed budget dimension and compiler-
checked trace binding. It passes that value through an accepted Quoin protocol-
scenario intake operation and does not store, reload, index, score or dispose
it.

### Refusal, resource and reproduction view

Every public failure returns one shared envelope with an outcome identity,
actual originating stage, exact affected identity when available, prerequisites,
resource usage and one or more typed causes. A cause is either a Protocol-owned
closed cause, an exact retained source-owner cause with contract/code/locus and
opaque payload, or `unsupported-source-contract`. Unknown diagnostic catalog
versions are never interpreted by spelling similarity. A runtime value without
a source locus names its exact slot or entry ordinal and does not manufacture a
span.

The closed Protocol refusal domains are `resolution`, `admission`,
`representation`, `assessment`, `result`, `projection` and `evidence`. Stages
map exactly to them: `contract-resolution` to `resolution`; `admission` to
`admission`; `representation` to `representation`; `compensation`,
`conformance` and `claim-dispatch` to `assessment`; `result-assembly` and
`result-read` to `result`; `projection` to `projection`; and
`evidence-handoff` to `evidence`. A stage/domain mismatch is itself invalid and
cannot be repaired from text.

`bounds::inventory::ResourceAccountant` is the only Protocol request accountant.
The parallel public `CountGuard`/`RunGuard` API is internalized. Every Protocol-
owned allocation, retention, queue, traversal, operation and output site has one
inventory entry and one pre-work charge. Owner maxima and positive caller grants
combine by the stricter inclusive bound. Deadlines are checked before and after
each indivisible unit, and no later unit starts after expiry.

External owners keep their own dimensions and accounting versions. Protocol
passes their exact limits, retains their returned usage separately, and never
mirrors or sums an owner accumulation rule. QSL `f1700a9` publishes
`protocol_artifact::work::Dimension::accumulation()`; the QSL adapter calls that
accessor and removes Protocol's current local mirror. Scenario evidence records
each owner/version/dimension separately.

Reproduction retains exact revisions, contract versions, original artifact
digests, binding and input identities, observations, progress/closure facts,
budgets, corpus, seed, toolchain and ordered outcomes. Equivalent presentation
or causally independent arrival permutations reproduce identical semantic
outcomes and canonical bytes. Every named semantic mutation changes the
affected identity or refuses.

## Decisions

1. Preserve the substantial existing linker, compensation, conformance, claim,
   result and bounds kernels, but make their cross-stage production constructors
   crate-private and connect them through typed transitions. Independent hand-
   built stage models are not a supported architecture.
2. Keep `/1` as an executable compatibility lane and `/2` as the only full
   assessment lane. Unknown versions refuse without fallback.
3. Import owner types and require accepted capabilities at crossings. Protocol
   owns only its binding, linked graph, assessment, result, refusal and evidence
   composition values.
4. Use RFC 8785 only in identity/digest domains whose owning requirement selects
   it. Componentwise keys and exact producer bytes are never replaced by a JCS
   hash or locally canonicalized copy.
5. Keep QObs facts flowing into Protocol, Protocol results flowing into QCI and
   E02, Engineering Assurance executing producers, and Quoin retaining evidence.
   No reverse edge or local substitute is allowed.
6. Apply one transactional state model, one refusal envelope and one Protocol
   resource accountant across the path.
7. Advanced projection, realizability, refinement and composition algorithms
   remain unsupported until their accepted owner contracts exist; unsupported
   never means false, satisfied or successful.

## Conformance Rules

- A downstream production constructor must accept the preceding stage's
  constructor-private value; accepting a structurally similar caller-built
  value is non-conforming.
- Every closed owner enum is matched exhaustively with no wildcard or fallback.
- No semantic equality depends on hashes narrower than the retained source
  value, display text, filesystem path, source span, input order or timestamp.
- Exact replay is idempotent; conflicting replay is transactional refusal.
- Pre-assessment and whole-assessment fatal failures expose no partial success.
  Each admitted claim has exactly one canonical semantic disposition or one
  claim-local construction refusal, and sibling outcomes remain independent.
- Every public operation has one resource-inventory owner and every emitted
  refusal carries its actual stage.
- QCI, E02, Engineering Assurance and Quoin are exercised only through their
  accepted versioned directions above.

## Risks

- **E02 owner — immediate:** qverify implements
  `quire-verification/e02-draft-2`, but its interface-decision document still
  calls the cross-repository contract provisional while its prerequisite
  tickets are closed. Task-007 remains blocked until the owner reconciles that
  contradiction and asserts the exact accepted revision; tracked by
  `agent-ix/quire-verification#32`.
- **Quoin owner — immediate:** Quoin now has a Rust workspace and evidence
  storage, but no structured operation can carry FR-017's complete protocol-
  scenario record. Task-018/Task-011 remain blocked on that exact intake
  contract; tracked by `agent-ix/quoin#519`.
- **Compensation authority — immediate:** QSpec FR-236 publishes five outcome
  labels but does not partition them between per-attempt and aggregate recovery
  facts. The affected Task-003 mapping cases remain blocked on an accepted owner
  ruling; Protocol will not infer a partition. Tracked by
  `agent-ix/quire-specification#57`; other compensation work can land.
- **Contract-IR owner — delivered:** QCI #71 / PR #78 (`69ec82b`) implements
  FR-026. Its native/TL join consumes QSL-native and TL result views, while its
  FR-025 predicate bridge consumes Protocol leaf-result views. It is downstream
  evidence for QS #1, not a remaining Protocol production dependency.
- **Protocol owner — Task-008:** the current code exposes two resource-guard
  architectures and must converge on the inventoried accountant.
- **Dependency owner — every consuming task:** accepted git pins, licenses and
  artifact rights must be recorded before use. `publish = false` and absence of
  a registry release are intentional and not blockers.
