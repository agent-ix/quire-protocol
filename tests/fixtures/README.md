# A four-unit selection fixture

`a-four-unit-selections.json` is B's immutable acceptance selection for
IT-001. It was copied unchanged from the output of A's
`native_protocol_handoff` at `quire-spec-language` commit
`84975dd687d38d9b24eae84b7356e3f24cdc7b42`, then retained here as an
AGPL-3.0-only compatibility fixture. JSON does not support an SPDX header; this
provenance record supplies the applicable license notice.

The runtime test never reads the producer-emitted `expected.json`. It selects
this local fixture before it reads `compiled-protocol.json`, then independently
loads the corresponding source, model, producer, and dependency bytes from the
actual producer directory. Updating this selection requires a deliberate B
review together with A revision, license, and digest provenance.

## Running the IT-001 lane

The lane is `#[ignore]`d by default because it consumes A's real producer output,
which is generated rather than vendored. Generate it once from
`quire-spec-language` at the pinned revision `84975dd`:

```console
CARGO_PROFILE_RELEASE_STRIP=symbols cargo run --locked --offline --release \
  --example native_protocol_handoff -- /tmp/quire-native-handoff-it001
```

Then run the lane against that directory:

```console
CARGO_BUILD_JOBS=1 QUIRE_PROTOCOL_A_FIXTURE=/tmp/quire-native-handoff-it001 \
  flock /tmp/quire-heavy-check.lock \
  cargo test --locked --test it001_native_handoff -- --ignored --test-threads=1
```

No hosted CI runs this lane; per `AGENTS.md` it is a local check.
