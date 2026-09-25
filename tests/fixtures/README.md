# IT-001 handoffs

B keeps no copy of A's handoffs or selection sidecars. QSL's committed handoff
copies (`PUBLISHED_V1_HANDOFF`, `PUBLISHED_HANDOFF`) carry no `dependencies/`
bytes and select an empty dependency inventory, so they cannot be admitted.
The IT-001 lanes therefore run A's own `native_protocol_handoff` and
`native_protocol_v2_handoff` examples from the exactly pinned
`quire-spec-language` source, once per test process, into
`<target-dir>/qsl-producer/` (`tests/support/v2_handoff/qsl_producer.rs`). The
first run builds those examples, which takes a few minutes.

Each lane reads the producer's own selection sidecar (`expected.json`,
`expected-v2.json`) before it reads the offer, checks every retained input
against its selected digest, and never derives an expectation from the offered
bytes.

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
the fresh `native_protocol_v2_handoff` output described above and imports
`protocol_artifact::handoff`'s member-name and mutation-format constants. `tests/it001_v2_handoff.rs` verifies the published
checksums before decoding, then runs the base offer, all 28 published mutations,
both cross-version submissions, and exact/one-short/fresh probes for every
`protocol_artifact::Dimension::ALL` member under the default local gate.
