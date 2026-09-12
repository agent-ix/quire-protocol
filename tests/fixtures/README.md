# A four-unit selection fixture

`a-four-unit-selections.json` is B's reviewed acceptance selection for IT-001.
It was re-derived against the original sources, admitted model, contract and
dependency bytes produced by `native_protocol_handoff` at
`quire-spec-language` PR72 revision
`5690c2b29c155ec72e21f5a7bb00fe1146b6f71a`, then retained here as an
AGPL-3.0-only compatibility fixture. JSON does not support an SPDX header; this
provenance record supplies the applicable license notice.

The runtime test never reads the producer-emitted `expected.json`. It selects
this local fixture before it reads `compiled-protocol.json`, then independently
loads the corresponding source, model, producer, and dependency bytes from the
actual producer directory. The emitted package seal contains the producer ELF
digest and therefore changes when that binary is rebuilt; it is not a durable
fixture pin. Updating this selection requires a deliberate B review together
with A revision, license, and digest provenance.

## Running the IT-001 lane

The lane is `#[ignore]`d by default because it consumes A's real producer output,
which is generated rather than vendored. Use the already generated endpoint
fixture at `/tmp/quire-native-handoff-signed64-endpoints/`, or regenerate it
from `quire-spec-language` at PR72 revision `5690c2b`:

```console
CARGO_PROFILE_RELEASE_STRIP=symbols cargo run --locked --offline --release \
  --example native_protocol_handoff -- /tmp/quire-native-handoff-signed64-endpoints
```

Then run the lane against that directory:

```console
CARGO_BUILD_JOBS=1 QUIRE_PROTOCOL_A_FIXTURE=/tmp/quire-native-handoff-signed64-endpoints \
  flock /tmp/quire-heavy-check.lock \
  cargo test --locked --test it001_native_handoff -- --ignored --test-threads=1
```

No hosted CI runs this lane; per `AGENTS.md` it is a local check.
