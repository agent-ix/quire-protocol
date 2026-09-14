# Artifact, dependency, and rights policy

The repository contains its authored Rust library and tests alongside project
documentation and specification artifacts. It consumes exact pinned Rust
dependencies and reads the native compiler owner's published handoff from that
dependency's source tree; it copies no external implementation source, fixture
corpus, generated code, or test data into this repository.

Every admitted source profile, compiler artifact, observation, temporal result,
E02 contract, producer result, fixture, and generated counterexample must retain
its origin, exact revision or content digest, applicable license or retention
decision, and immutable identity. A reference does not grant retention rights.
Artifacts that cannot be retained must remain external references with an owned
gap; they cannot be reshaped or replaced with synthetic bytes and represented as
the original evidence.

Before adding a dependency, record its exact version or revision, license,
feature set, provenance, purpose, and compatibility with the selected outbound
`AGPL-3.0-or-later` license. No external choreography implementation is
selected for the initial finite slice. E03 remains the admission gate for any
later specialized library.
