// SPDX-License-Identifier: AGPL-3.0-or-later
//! IT-001 SC-05/Task-009: the dependency and source inventory of this consumer.
//!
//! This lane reads B's own manifest, lockfile and library source as data. It
//! proves, without running the producers, that the protocol path uses exactly
//! the selected native/formal frontend, promoted observation owner, accepted
//! E02 owner, and B's explicitly admitted serialization/schema carriers.

use std::{collections::BTreeSet, fs, path::Path};

use ix_trace_rs::trace;

const MANIFEST: &str = include_str!("../Cargo.toml");
const LOCKFILE: &str = include_str!("../Cargo.lock");
const RIGHTS_RECORD: &str = include_str!("../DEPENDENCIES.md");
const LIBRARY: &str = include_str!("../src/lib.rs");
const INTAKE_V1: &str = include_str!("../src/intake/v1.rs");
const INTAKE_V2: &str = include_str!("../src/intake/v2.rs");
const MODEL_V1: &str = include_str!("../src/model/v1.rs");

/// A's accepted reader. Its exact revision is whatever the manifest pins; this
/// lane checks that every crate from A's repository resolves at that one
/// revision, not which revision it is.
const FRONTEND: &str = "quire-spec-language";
/// A's foundation crate (ADR-011 layer F), from the same repository and
/// revision as `FRONTEND`. It owns `ByteDigest`, `Source` and `SourceIdentity`,
/// which A's reader returns, so the two must resolve one commit to unify.
const FRONTEND_FOUNDATION: &str = "qsl-foundation";
/// A's repository, as it appears in manifest and lockfile sources.
const FRONTEND_REPOSITORY: &str = "agent-ix/quire-spec-language";

/// F's promoted observation owner supplies typed progress, closure and
/// completeness views. B consumes those views and does not mirror them.
const OBSERVATION_OWNER: &str = "quire-observation";

/// E02's accepted owner crate supplies the target types, JCS implementation,
/// and strict reader. B maps into that interface and does not mirror it.
const E02_OWNER: &str = "quire-verification-contracts";
const E02_REVISION: &str = "da8d7272051b1cf02768ec459539551414fd6c21";

/// Engineering Assurance owns the claim-strength vocabulary (EA FR-022). B
/// imports its `ClaimStrength` through the serde-only `claim-strength` feature
/// and declares no strength enum of its own.
const CLAIM_STRENGTH_OWNER: &str = "engineering-assurance";
const CLAIM_STRENGTH_REVISION: &str = "712f4ff76c2b1e01d26d10efd97a0eef0235defa";

/// Serialization carriers read members A has already published and serialize
/// B's own selections through A's `Serialize` impls; they carry no schema,
/// grammar or wire decoder.
const SERIALISATION: &str = "serde_json";
const SERIALISATION_DERIVE: &str = "serde";

/// The pinned, local-only closed-schema validator used by hostile-byte readers.
const SCHEMA_VALIDATOR: &str = "jsonschema";
/// Fixed implementation used to stream bounded request-identity preimages.
const STREAMING_DIGEST: &str = "sha2";

/// The library's frozen module set, established in one commit by the chunk
/// facade (C-00). This is an exact set, not a filter: a module that appears
/// without being added here fails the lane, so no unlisted module can carry
/// protocol semantics.
const LIBRARY_MODULES: &[&str] = &[
    "assessment.rs",
    "bounds",
    "claims",
    "closure.rs",
    "compensation",
    "conformance",
    "delivery_index.rs",
    "digest",
    "ids.rs",
    "intake",
    "inventory",
    "lib.rs",
    "link",
    "model",
    "outcome.rs",
    "producer_contracts.rs",
    "projection.rs",
    "refusal",
    "repro.rs",
    "result",
    "version_lock",
    "version_lock.rs",
];

/// Every package the resolved graph is permitted to contain, B's own crate
/// aside. This is the frozen inventory, not a filter: a package that appears
/// without being added here fails the lane, so a second frontend, an alternate
/// decoder or a foreign runtime cannot enter unannounced.
const INVENTORY: &[&str] = &[
    // A's own workspace crates, split out of `quire-spec-language` (ADR-011),
    // all resolved from A's repository at the one pinned revision; see
    // `it_001_resolves_one_frontend_in_the_whole_graph`.
    "qsl-attrs",
    "qsl-cst",
    "qsl-eval",
    "qsl-forms",
    "qsl-foundation",
    "qsl-package",
    "qsl-replay",
    "qsl-semantics",
    "quire-exact",
    // Third repositories' crates reaching the graph through A at that
    // revision; B did not choose them.
    "agent-ix-extraction-frontend",
    "agent-ix-semantic-ir",
    "quire-canonical",
    "quire-rs",
    "ahash",
    "aho-corasick",
    "allocator-api2",
    "anstream",
    "anstyle",
    "anstyle-parse",
    "anstyle-query",
    "anstyle-wincon",
    "anyhow",
    "ar_archive_writer",
    "autocfg",
    "base64",
    "bit-set",
    "bit-vec",
    "bitflags",
    "block-buffer",
    "borrow-or-share",
    "bstr",
    "bumpalo",
    "bytecount",
    "cc",
    "cfg-if",
    "chrono",
    "clap",
    "clap_builder",
    "clap_derive",
    "clap_lex",
    "colorchoice",
    "cpufeatures",
    "crossbeam-deque",
    "crossbeam-epoch",
    "crossbeam-utils",
    "crypto-common",
    "data-encoding",
    "deranged",
    "digest",
    "displaydoc",
    "dyn-clone",
    "either",
    "email_address",
    "engineering-assurance",
    "equivalent",
    "errno",
    "fancy-regex",
    "fastrand",
    "find-msvc-tools",
    "fluent-uri",
    "fnv",
    "foldhash",
    "form_urlencoded",
    "fraction",
    "futures-core",
    "futures-task",
    "futures-util",
    "generator",
    "generic-array",
    "getrandom",
    "globset",
    "hashbrown",
    "heck",
    "icu_collections",
    "icu_locale_core",
    "icu_normalizer",
    "icu_normalizer_data",
    "icu_properties",
    "icu_properties_data",
    "icu_provider",
    "idna",
    "idna_adapter",
    "ignore",
    "indexmap",
    "is_terminal_polyfill",
    "iso8601",
    "itoa",
    "ix-trace-rs",
    "js-sys",
    "jsonschema",
    "jsonschema-regex",
    "jsonschema-value",
    "lazy_static",
    "libc",
    "libyaml-rs",
    "linux-raw-sys",
    "litemap",
    "lock_api",
    "log",
    "logos",
    "logos-codegen",
    "logos-derive",
    "loom",
    "matchers",
    "memchr",
    "micromap",
    "nom",
    "nu-ansi-term",
    "num",
    "num-bigint",
    "num-cmp",
    "num-complex",
    "num-conv",
    "num-integer",
    "num-iter",
    "num-rational",
    "num-traits",
    "object",
    "once_cell",
    "once_cell_polyfill",
    "outref",
    "parking_lot",
    "parking_lot_core",
    "percent-encoding",
    "pin-project-lite",
    "potential_utf",
    "powerfmt",
    "proc-macro2",
    "psm",
    "quire-contract-model",
    "quire-observation",
    "quire-spec-language",
    "quire-verification-contracts",
    "quote",
    "r-efi",
    "rayon",
    "rayon-core",
    "redox_syscall",
    "ref-cast",
    "ref-cast-impl",
    "referencing",
    "regex",
    "regex-automata",
    "regex-syntax",
    "regress",
    "rustix",
    "rustversion",
    "ryu",
    "ryu-js",
    "same-file",
    "schemars",
    "schemars_derive",
    "scoped-tls",
    "scopeguard",
    "semver",
    "serde",
    "serde_core",
    "serde_derive",
    "serde_derive_internals",
    "serde_json",
    "serde_json_canonicalizer",
    "serde_path_to_error",
    "serde_stacker",
    "serde_tokenstream",
    "sha2",
    "sharded-slab",
    "shlex",
    "slab",
    "smallvec",
    "stable_deref_trait",
    "stacker",
    "strsim",
    "strum",
    "strum_macros",
    "syn",
    "synstructure",
    "tempfile",
    "thiserror",
    "thiserror-impl",
    "thread_local",
    "time",
    "time-core",
    "time-macros",
    "tinystr",
    "tinyvec",
    "tracing",
    "tracing-core",
    "tracing-log",
    "tracing-subscriber",
    "typenum",
    "typify",
    "typify-impl",
    "typify-macro",
    "unicode-general-category",
    "unicode-ident",
    "unicode-normalization",
    "url",
    "utf8_iter",
    "utf8parse",
    "uuid",
    "uuid-simd",
    "valuable",
    "version_check",
    "vsimd",
    "walkdir",
    "wasi",
    "wasip2",
    "wasm-bindgen",
    "wasm-bindgen-macro",
    "wasm-bindgen-macro-support",
    "wasm-bindgen-shared",
    "winapi",
    "winapi-i686-pc-windows-gnu",
    "winapi-util",
    "winapi-x86_64-pc-windows-gnu",
    "windows-link",
    "windows-result",
    "windows-sys",
    "wit-bindgen",
    "writeable",
    "yaml_serde",
    "yoke",
    "yoke-derive",
    "zerocopy",
    "zerocopy-derive",
    "zerofrom",
    "zerofrom-derive",
    "zerotrie",
    "zerovec",
    "zerovec-derive",
    "zmij",
];

fn manifest_section(name: &str) -> &'static str {
    let start = MANIFEST
        .find(name)
        .unwrap_or_else(|| panic!("manifest declares {name}"));
    let rest = &MANIFEST[start + name.len()..];
    rest.find("\n[").map_or(rest, |end| &rest[..end])
}

fn declared_packages(section: &str) -> BTreeSet<&str> {
    section
        .lines()
        .filter_map(|line| line.split_once(" = "))
        .map(|(name, _)| name.trim())
        .filter(|name| !name.is_empty())
        .collect()
}

/// Tracing: IT-001, Task-009.
#[test]
#[trace("IT-001", "Task-009")]
fn it_001_declares_only_selected_owner_dependencies() {
    // The library links exactly one protocol frontend, the promoted observation
    // owner, the accepted E02 owner, and the carriers required to emit,
    // digest, and strictly validate B's own closed documents. No alternate parser,
    // evaluator or runtime can enter the protocol path unannounced.
    assert_eq!(
        declared_packages(manifest_section("[dependencies]")),
        BTreeSet::from([
            FRONTEND,
            FRONTEND_FOUNDATION,
            OBSERVATION_OWNER,
            E02_OWNER,
            CLAIM_STRENGTH_OWNER,
            SERIALISATION,
            SERIALISATION_DERIVE,
            SCHEMA_VALIDATOR,
            STREAMING_DIGEST,
        ])
    );
    let revision = frontend_revision();
    for package in [FRONTEND, FRONTEND_FOUNDATION] {
        let declaration = manifest_section("[dependencies]")
            .lines()
            .find(|line| line.starts_with(&format!("{package} = ")))
            .unwrap_or_else(|| panic!("manifest declares {package}"));
        assert!(
            declaration.contains(FRONTEND_REPOSITORY)
                && declaration.contains(&format!(r#"rev = "{revision}""#)),
            "{package} must be pinned to A's repository at the reader's exact revision: {declaration}"
        );
    }

    // Development dependencies are harness-only: tracing, the cycle-free
    // Contract Model types used to construct the native handoff, and JSON
    // fixtures. The Producer interface 1.2 static-bundle admission used by
    // fixtures is `quire_observation`'s own `StaticProducerBundle` (a
    // production dependency already), not a separate FCD crate. None is
    // reachable from the published library.
    assert_eq!(
        declared_packages(manifest_section("[dev-dependencies]")),
        BTreeSet::from([
            "ix-trace-rs",
            "quire-contract-ir",
            // QSL-251: the same pinned frontend again, with its test-only
            // `handoff-writer` feature, for the tests' in-process producer.
            FRONTEND,
            "serde_json",
        ])
    );
}

/// Tracing: IT-001, Task-009.
#[test]
#[trace("IT-001", "Task-009")]
fn it_001_resolves_one_frontend_in_the_whole_graph() {
    let mut resolved = BTreeSet::new();
    for line in LOCKFILE.lines() {
        if let Some(name) = line.strip_prefix("name = ") {
            resolved.insert(name.trim().trim_matches('"'));
        }
    }
    assert!(
        resolved.remove("quire-protocol"),
        "the lockfile locks B itself"
    );

    let unexpected = resolved
        .difference(&INVENTORY.iter().copied().collect())
        .copied()
        .collect::<Vec<_>>();
    assert!(
        unexpected.is_empty(),
        "the resolved graph gained packages outside the frozen inventory: {unexpected:?}"
    );

    // Every resolved package from A's repository comes from one source, at the
    // pinned revision: there is no second copy, fork or alternate revision of
    // the native frontend anywhere in the graph. A's workspace crates share
    // that one source line.
    // The baseline producer named in the inventory above comes from a third
    // repository, so the one-frontend claim would be empty if it could be a
    // second reader under another name. It cannot: it is not from A's
    // repository, which is what the assertion below counts.
    let revision = frontend_revision();
    let from_a = LOCKFILE
        .lines()
        .filter(|line| line.starts_with("source = ") && line.contains(FRONTEND_REPOSITORY))
        .collect::<BTreeSet<_>>();
    assert_eq!(from_a.len(), 1, "got {from_a:?}");
    assert!(
        from_a.iter().all(|source| source.contains(&revision)),
        "got {from_a:?}"
    );
}

/// The exact revision the manifest pins `FRONTEND` to.
fn frontend_revision() -> String {
    let declaration = manifest_section("[dependencies]")
        .lines()
        .find(|line| line.starts_with(&format!("{FRONTEND} = ")))
        .unwrap_or_else(|| panic!("manifest declares {FRONTEND}"));
    let (_, rest) = declaration
        .split_once(r#"rev = ""#)
        .unwrap_or_else(|| panic!("{FRONTEND} is pinned by rev: {declaration}"));
    let (revision, _) = rest
        .split_once('"')
        .unwrap_or_else(|| panic!("{FRONTEND} rev is quoted: {declaration}"));
    assert_eq!(
        revision.len(),
        40,
        "{FRONTEND} rev is a full commit: {revision}"
    );
    revision.to_owned()
}

/// Tracing: TC-013 and NFR-006-AC-4.
///
/// `engineering-assurance` is a direct dependency again (TL-206), for the
/// claim-strength vocabulary alone. It must resolve at exactly one revision:
/// a second revision would carry a distinct `ClaimStrength` type that does not
/// unify with the one B emits.
#[test]
#[trace("TC-013", "NFR-006-AC-4")]
fn tc_013_records_the_exact_e02_git_closure_in_all_three_sources() {
    assert!(
        manifest_section("[dependencies]").contains(E02_REVISION),
        "the direct E02 dependency must select the accepted compatible revision"
    );

    let locked_sources = LOCKFILE
        .lines()
        .filter(|line| {
            line.starts_with("source = ") && line.contains(E02_OWNER) && line.contains(E02_REVISION)
        })
        .count();
    assert_eq!(locked_sources, 1, "lock source drift for {E02_OWNER}");
    assert!(
        RIGHTS_RECORD.contains(&format!("| `{E02_OWNER}` | `{E02_REVISION}` |")),
        "rights record drift for {E02_OWNER}"
    );

    assert!(
        manifest_section("[dependencies]").contains(CLAIM_STRENGTH_REVISION),
        "the direct claim-strength owner dependency must select the pinned revision"
    );
    let claim_strength_packages = LOCKFILE
        .split("[[package]]")
        .filter(|package| {
            package
                .lines()
                .any(|line| line == format!(r#"name = "{CLAIM_STRENGTH_OWNER}""#))
        })
        .collect::<Vec<_>>();
    assert_eq!(
        claim_strength_packages.len(),
        1,
        "one locked {CLAIM_STRENGTH_OWNER} package in the graph, got {claim_strength_packages:?}"
    );
    assert!(
        claim_strength_packages[0].contains(CLAIM_STRENGTH_REVISION),
        "got {claim_strength_packages:?}"
    );
    assert!(
        RIGHTS_RECORD.contains(&format!(
            "| `{CLAIM_STRENGTH_OWNER}` | `{CLAIM_STRENGTH_REVISION}` |"
        )),
        "rights record drift for {CLAIM_STRENGTH_OWNER}"
    );
}

/// Tracing: IT-001, Task-009.
#[test]
#[trace("IT-001", "Task-009")]
fn it_001_library_facade_only_delegates_frontend_admission() {
    // The frozen module set, so no unlisted subsystem can carry protocol
    // semantics.
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = fs::read_dir(&source)
        .expect("library source directory")
        .map(|entry| entry.expect("library source entry").file_name())
        .collect::<Vec<_>>();
    files.sort();
    let frozen = LIBRARY_MODULES
        .iter()
        .map(std::ffi::OsString::from)
        .collect::<Vec<_>>();
    assert_eq!(files, frozen);

    // The frozen crate-root facade has one `/1` compatibility function. Its
    // whole body enters the explicit version-1 intake module; the full `/2`
    // assessment path may not be reintroduced at this root.
    //
    // This counts `lib.rs` alone, and it is an exact count rather than a
    // ceiling: a second function here would hide a version-selection decision
    // in the compatibility surface.
    let functions = LIBRARY.match_indices("fn ").count();
    assert_eq!(
        functions, 1,
        "the root facade must expose only the frozen version-1 compatibility path"
    );
    assert!(
        LIBRARY.contains("intake::v1::admit(bytes, expected, limits)"),
        "the version-1 facade must delegate directly to its explicit intake module"
    );
    assert!(
        !LIBRARY.contains("admit_and_link_v2"),
        "the root facade must not offer a version-2 compatibility alias"
    );
    assert!(
        INTAKE_V1.contains("quire_spec_language::protocol_artifact::read(bytes, expected, limits)"),
        "the version-1 intake must be a bare call into A's reader"
    );
    assert!(
        INTAKE_V2.contains("v2::read(bytes, expected, limits)"),
        "the version-2 intake must be a bare call into A's version-2 reader"
    );
    assert!(
        MODEL_V1.contains("crate::link::link(package, declaration)"),
        "the version-1 model must delegate static linking to the admitted legacy graph"
    );

    for line in LIBRARY.lines() {
        let line = line.trim();
        let is_reexport = line.starts_with("pub use ")
            || line.starts_with("quire_spec_language::")
            || line.starts_with("ExpectedForeignSource")
            || line.is_empty();
        let is_prose = line.starts_with("//") || line.starts_with("#!");
        assert!(
            is_reexport || is_prose || !line.contains("quire_spec_language"),
            "unexpected use of A's crate outside a re-export or the delegation: {line}"
        );
    }
}
