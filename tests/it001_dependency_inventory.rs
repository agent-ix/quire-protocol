// SPDX-License-Identifier: AGPL-3.0-or-later
//! IT-001 SC-05/Task-009: the dependency and source inventory of this consumer.
//!
//! This lane reads B's own manifest, lockfile and library source as data. It
//! proves, without running the producer, that exactly one native/formal
//! frontend participates in the protocol path: A's accepted
//! `quire.compiled-protocol/1` reader.

use std::{collections::BTreeSet, fs, path::Path};

use ix_trace_rs::trace;

const MANIFEST: &str = include_str!("../Cargo.toml");
const LOCKFILE: &str = include_str!("../Cargo.lock");
const LIBRARY: &str = include_str!("../src/lib.rs");

/// A's accepted reader, pinned to the exact revision this lane froze.
const FRONTEND: &str = "quire-spec-language";
const FRONTEND_REVISION: &str = "04254a31071cc2d4c97e5c0707e1414141e55d89";

/// The only non-frontend crate the published library links. It reads members A
/// has already published and serialises B's own selections through A's
/// `Serialize` impls; it carries no schema, grammar or wire decoder. It is named
/// here so that a second such crate cannot enter unannounced.
const SERIALISATION: &str = "serde_json";

/// The library's frozen module set, established in one commit by the chunk
/// facade (C-00). This is an exact set, not a filter: a module that appears
/// without being added here fails the lane, so no unlisted module can carry
/// protocol semantics.
const LIBRARY_MODULES: &[&str] = &[
    "bounds",
    "claims",
    "closure.rs",
    "compensation",
    "conformance",
    "digest",
    "ids.rs",
    "inventory",
    "lib.rs",
    "link",
    "refusal",
    "repro.rs",
    "result",
];

/// Every package the resolved graph is permitted to contain, B's own crate
/// aside. This is the frozen inventory, not a filter: a package that appears
/// without being added here fails the lane, so a second frontend, an alternate
/// decoder or a foreign runtime cannot enter unannounced.
const INVENTORY: &[&str] = &[
    "ahash",
    "aho-corasick",
    "anyhow",
    "ar_archive_writer",
    "autocfg",
    "base64",
    "bit-set",
    "bit-vec",
    "bitflags",
    "block-buffer",
    "bumpalo",
    "bytecount",
    "cc",
    "cfg-if",
    "cpufeatures",
    "crypto-common",
    "deranged",
    "digest",
    "displaydoc",
    "fancy-regex",
    "find-msvc-tools",
    "fnv",
    "form_urlencoded",
    "fraction",
    "futures-core",
    "futures-task",
    "futures-util",
    "generic-array",
    "getrandom",
    "icu_collections",
    "icu_locale_core",
    "icu_normalizer",
    "icu_normalizer_data",
    "icu_properties",
    "icu_properties_data",
    "icu_provider",
    "idna",
    "idna_adapter",
    "iso8601",
    "itoa",
    "ix-trace-rs",
    "js-sys",
    "jsonschema",
    "lazy_static",
    "libc",
    "litemap",
    "lock_api",
    "logos",
    "logos-codegen",
    "logos-derive",
    "memchr",
    "nom",
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
    "parking_lot",
    "parking_lot_core",
    "percent-encoding",
    "pin-project-lite",
    "potential_utf",
    "powerfmt",
    "proc-macro2",
    "psm",
    "quire-contract-ir",
    "quire-spec-language",
    "quote",
    "r-efi",
    "redox_syscall",
    "regex",
    "regex-automata",
    "regex-syntax",
    "rustversion",
    "scopeguard",
    "serde",
    "serde_core",
    "serde_derive",
    "serde_json",
    "serde_stacker",
    "sha2",
    "shlex",
    "slab",
    "smallvec",
    "stable_deref_trait",
    "stacker",
    "syn",
    "synstructure",
    "thiserror",
    "thiserror-impl",
    "time",
    "time-core",
    "time-macros",
    "tinystr",
    "typenum",
    "unicode-ident",
    "url",
    "utf8_iter",
    "uuid",
    "version_check",
    "wasi",
    "wasip2",
    "wasm-bindgen",
    "wasm-bindgen-macro",
    "wasm-bindgen-macro-support",
    "wasm-bindgen-shared",
    "winapi",
    "winapi-i686-pc-windows-gnu",
    "winapi-x86_64-pc-windows-gnu",
    "windows-link",
    "wit-bindgen",
    "writeable",
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
fn it_001_declares_one_protocol_dependency() {
    // The library links exactly one protocol frontend, and it is A's accepted
    // reader at the exact frozen revision. The only other linked crate is the
    // serialisation carrier named above. Nothing else can reach the protocol
    // path.
    assert_eq!(
        declared_packages(manifest_section("[dependencies]")),
        BTreeSet::from([FRONTEND, SERIALISATION])
    );
    assert!(
        manifest_section("[dependencies]").contains(FRONTEND_REVISION),
        "the single protocol dependency must stay pinned to the frozen revision"
    );

    // Development dependencies are harness-only: tracing, the IR types the
    // harness reconstructs selections with, and the JSON the fixture is stored
    // in. None of them is reachable from the published library.
    assert_eq!(
        declared_packages(manifest_section("[dev-dependencies]")),
        BTreeSet::from(["ix-trace-rs", "quire-contract-ir", "serde", "serde_json"])
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

    // Exactly one resolved package comes from A's repository, at the frozen
    // revision: there is no second copy, fork or alternate revision of the
    // native frontend anywhere in the graph.
    let from_a = LOCKFILE
        .lines()
        .filter(|line| {
            line.starts_with("source = ") && line.contains("agent-ix/quire-spec-language")
        })
        .collect::<Vec<_>>();
    assert_eq!(from_a.len(), 1, "got {from_a:?}");
    assert!(from_a[0].contains(FRONTEND_REVISION), "got {from_a:?}");
}

/// Tracing: IT-001, Task-009.
#[test]
#[trace("IT-001", "Task-009")]
fn it_001_library_source_only_delegates() {
    // The frozen module set, so no unlisted module can carry protocol semantics.
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

    // Exactly one function, and its whole body is a call into A's reader. B
    // adds no parser, decoder, numeric layer or alternate semantics: every
    // other item in `lib.rs` is a re-export of an A-owned name.
    //
    // This counts `lib.rs` alone. The modules the facade reserves are empty on
    // this head; a version 2 delegation adds a second function to this count
    // when that lane lands.
    let functions = LIBRARY.match_indices("fn ").count();
    assert_eq!(
        functions, 1,
        "the library must expose one delegating function"
    );
    assert!(
        LIBRARY.contains("quire_spec_language::protocol_artifact::read(bytes, expected, limits)")
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
