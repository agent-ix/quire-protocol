# A four-unit selection fixture

`a-four-unit-selections.json` is B's reviewed acceptance selection for IT-001.
It is derived against the original sources, admitted model, contract and
dependency bytes produced by `native_protocol_handoff` at `quire-spec-language`
`6c8c401bb807a417215fae7593ef27fd9bab051e` — the QSL revision pinned by this
crate — and retained
here as an AGPL-3.0-only compatibility fixture. JSON does not support an SPDX
header; this provenance record supplies the applicable license notice.

The runtime test never reads the producer-emitted `expected.json`. It selects
this local fixture before it reads `compiled-protocol.json`, then independently
loads the corresponding source, model, producer, and dependency bytes from the
actual producer directory.

QSL #112 replaced the ambient build-specific address with the committed,
owner-exported `PUBLISHED_V1_HANDOFF` and member-name constants. QSL #114 then
gave `Service` and `Provider` distinct admitted model authorities. The selected
version-1 package digest is
`sha256:fb00f406798997dae14c6de1b5e61c184c354f5470aa926dde8813e707e50cce`.
Protocol retains its own reviewed selection snapshot, but obtains the offered
bytes and every selected dependency from QSL's exactly pinned crate source.
It neither vendors the handoff nor invokes the producer during tests.

## Running the IT-001 version-1 lane

The complete lane is default-running and needs no environment variable:

```console
CARGO_BUILD_JOBS=1 flock /tmp/quire-heavy-check.lock \
  cargo test --locked --offline --target-dir target-codex-backends \
  --test it001_native_handoff -- --test-threads=1
```

It admits through `intake::v1::admit`, executes the local
`model::v1::link_static` boundary, retains every selected identity, checks the
positive reader/dependency inventory, and runs the independent refusal and
numeric-boundary controls. No hosted CI runs this lane.

## SC-06a historical compatibility evidence

The same committed version-1 test and selection blobs were executed at the
actual Protocol revision before version 2 (`0e5ec29207d0339e829dfbc6b660fd40e857d563`)
and the revision adding it (`5ce9070e7fb74b3a292c96c9478859b1c5fa3231`).
Their matching QSL revisions were `5690c2b29c155ec72e21f5a7bb00fe1146b6f71a`
and `d03e4ca560eb34463db5e541b2db5947d628b620`. Both isolated local runs passed
all 23 historical SC-01..SC-05 cases with no failure or ignore. The Protocol
test blob was `96b7ecc816b43ea2328c032d8c5b843ef000cc29` and the selection blob was
`309af51873ef103a2331edf6e361aad85170b6d0` at both revisions. Source analysis
shows the `/1` facade signature and body unchanged; the QSL `/1` public error,
invalid, unsupported, expectation, limit and admitted-package definitions were
unchanged while the new `/2` module was added separately.

## Version 2 published handoff

The version 2 lane also needs no environment variable or vendored fixture. It loads
`protocol_artifact::handoff::PUBLISHED_HANDOFF` from the exactly pinned
`quire-spec-language` source and imports that module's member-name and
mutation-format constants. `tests/it001_v2_handoff.rs` verifies the published
checksums before decoding, then runs the base offer, all 28 published mutations,
both cross-version submissions, and exact/one-short/fresh probes for every
`protocol_artifact::Dimension::ALL` member under the default local gate.
