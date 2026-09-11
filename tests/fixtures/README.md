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
