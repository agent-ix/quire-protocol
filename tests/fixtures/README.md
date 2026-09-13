# A four-unit selection fixture

`a-four-unit-selections.json` is B's reviewed acceptance selection for IT-001.
It is derived against the original sources, admitted model, contract and
dependency bytes produced by `native_protocol_handoff` at `quire-spec-language`
`04254a31071cc2d4c97e5c0707e1414141e55d89` — A's promoted `main` — and retained
here as an AGPL-3.0-only compatibility fixture. JSON does not support an SPDX
header; this provenance record supplies the applicable license notice.

The runtime test never reads the producer-emitted `expected.json`. It selects
this local fixture before it reads `compiled-protocol.json`, then independently
loads the corresponding source, model, producer, and dependency bytes from the
actual producer directory.

## The fixture is build-specific, and that is a defect

An earlier revision of this file said the producer ELF digest "is not a durable
fixture pin". That was false in a way worth stating rather than deleting: the
fixture pins that digest in two places, and it is transitively inside the
artifact digest as well. Rebuilding A's producer under a different toolchain,
profile or strip setting changes all three values, so this fixture is pinned to
one *build* of A, not to one *revision* of A.

That was demonstrated, not inferred. Regenerating with the documented
`CARGO_PROFILE_RELEASE_STRIP=symbols` recipe changed both the producer ELF
digest and the artifact digest that contains it. Updating this selection
therefore requires a deliberate B review together with A revision, license, and
digest provenance — and will keep requiring one on every producer rebuild until
the selection addresses A's producer by something reproducible. That addressing
gap is tracked as `agent-ix/quire-spec-language#78`.

## Correction: the fixture was an unreproducible hybrid

Before the regeneration recorded below, this fixture combined digests derived
from two different A revisions, and `quire-spec-language` PR72 — named here as
the source of `5690c2b` — is closed with `mergedAt: null`. The revision it
described was never on A's `main`, so no one could reproduce the fixture from
what this file said. Twenty-three of the lane's twenty-four tests had
consequently never executed. All twenty-four execute at `04254a3`.

## Regenerating the fixture

From a checkout of `quire-spec-language` at `04254a3`, with its own target
directory:

```console
CARGO_PROFILE_RELEASE_STRIP=symbols CARGO_BUILD_JOBS=1 \
  CARGO_TARGET_DIR=<a-checkout>/target \
  cargo run --locked --offline --release \
  --example native_protocol_handoff -- <handoff-dir>
```

Three digests in `a-four-unit-selections.json` are then rederived from their own
original bytes, never from the producer's `expected.json`:

| Field | Bytes hashed |
| --- | --- |
| `artifact/digest` | `<handoff-dir>/compiled-protocol.json` |
| `contract/digest` | A's `docs/compiled-protocol-v1.md` at `04254a3` |
| producer identity | `<handoff-dir>/dependencies/0.bin` |

## Running the IT-001 lane

The lane is `#[ignore]`d by default because it consumes A's real producer
output, which is generated rather than vendored:

```console
CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=<this-checkout>/target \
  QUIRE_PROTOCOL_A_FIXTURE=<handoff-dir> \
  flock /tmp/quire-heavy-check.lock \
  cargo test --locked --test it001_native_handoff -- --ignored --test-threads=1
```

No hosted CI runs this lane; per `AGENTS.md` it is a local check.
