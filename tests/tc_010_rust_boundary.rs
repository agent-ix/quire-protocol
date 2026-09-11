// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-010 / NFR-003: the standing Rust-containment audit.
//!
//! # What this file is
//!
//! Owner directive of 2026-09-07: every first-party production, qualification,
//! generator, validator, canonicalizer, adapter, fixture-audit, test-assertion
//! and command path in this repository is Rust. A Rust wrapper around non-Rust
//! logic is not remediation. This file enforces that on every local gate run,
//! over the whole working tree, so a violation fails the build on the commit
//! that introduces it rather than at a terminal qualification gate.
//!
//! The audit is itself Rust, reads the tree with `std::fs`, and spawns no
//! process: an audit implemented as a shell pipeline would violate the rule it
//! enforces. It also audits its own source file — it is not exempt from itself,
//! which is why every forbidden pattern below is assembled from fragments at
//! run time instead of appearing as one contiguous literal here.
//!
//! # The definition this audit enforces
//!
//! A tracked path is a **non-Rust executable path** when any of the following
//! holds. Each clause is a rule variant of [`Rule`] and is proved to fire by a
//! test at the bottom of this file.
//!
//! 1. Its extension is one an interpreter or shell executes ([`SCRIPT_EXTENSIONS`]).
//! 2. Its first two bytes are an interpreter marker (a shebang), whatever its
//!    extension or lack of one. A Rust inner attribute (`marker` then `[`) is
//!    not an interpreter marker.
//! 3. It carries a Unix execute permission bit and is not a Rust source file.
//! 4. Its file name is a build/command manifest another tool executes
//!    ([`COMMAND_MANIFEST_NAMES`]) — a `Makefile`, a `package.json`, a
//!    `Dockerfile`, a `pyproject.toml`.
//! 5. It is a Rust file that spawns a process ([`SPAWN_NEEDLES`]) — the
//!    Rust-wrapper bypass. A Rust function that shells out to a script is
//!    exactly what the directive forbids, so it is caught here and not excused
//!    by being written in Rust.
//! 6. It is a Rust file that emits an interpreter marker inside a string
//!    literal — a generated executable template.
//! 7. It is a hosted-workflow file carrying an inline command step, or a
//!    trigger other than `workflow_dispatch` (NFR-003-AC-1).
//! 8. Its extension is one this audit does not classify at all. An unknown
//!    extension is reported, not passed: a census that silently admits what it
//!    has no rule for is a tripwire tuned to pass.
//!
//! **Not** executable logic, and deliberately out of scope: `.md` prose, `.toml`
//! and `.lock` configuration, `.json`/`.yaml` data and vendored producer
//! fixtures, `LICENSE`. These carry no interpreter of their own. A fixture is a
//! byte sequence a Rust test reads; it becomes executable logic only under one
//! of the clauses above.
//!
//! # This is a local audit, not the accepted shared checker
//!
//! NFR-003's Verification section requires this repository to consume the
//! shared Rust-native language-boundary checker delivered by
//! `agent-ix/quire-research#64`, and forbids a local replacement. That checker
//! is **not published**: quire-research#64 is open with none of its six
//! acceptance boxes ticked, so there is no versioned structured result to
//! consume today. This file is therefore a repository-scoped standing audit
//! that holds the boundary in the meantime; it does not close the shared-checker
//! clause of NFR-003-AC-3, and this gap is reported rather than papered over.
//! When #64 publishes, the rules below are replaced by a call into it.

use std::{
    collections::BTreeSet,
    fmt, fs,
    path::{Path, PathBuf},
};

use ix_trace_rs::trace;

// ---------------------------------------------------------------------------
// Rule vocabulary
// ---------------------------------------------------------------------------

/// The containment rules, one per acceptance criterion family.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Rule {
    /// NFR-003-AC-1: hosted workflow triggers and inline command steps.
    Workflow,
    /// NFR-003-AC-2: unsafe Rust without an owner disposition.
    UnsafeItem,
    /// NFR-003-AC-3: a non-Rust executable path, including the wrapper bypass.
    ExecutablePath,
    /// NFR-003-AC-4: a local runner, verdict parser, evidence store, method
    /// catalogue, sufficiency or human-disposition surface.
    SharedBoundary,
    /// NFR-003-AC-5: a missing or unreadable `ix_trace_rs::trace` binding.
    TraceBinding,
    /// NFR-003-AC-6: an in-process foreign runtime, a Quoin copy, or a
    /// JavaScript/TypeScript path.
    ForeignRuntime,
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Rule::Workflow => "workflow-trigger",
            Rule::UnsafeItem => "unsafe-item",
            Rule::ExecutablePath => "executable-path",
            Rule::SharedBoundary => "shared-boundary-symbol",
            Rule::TraceBinding => "trace-binding",
            Rule::ForeignRuntime => "foreign-runtime",
        };
        f.write_str(name)
    }
}

/// One census result. A finding names the rule, the subject it was found on,
/// and enough detail to act on it without rerunning the audit.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Finding {
    rule: Rule,
    subject: String,
    detail: String,
}

impl Finding {
    fn new(rule: Rule, subject: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            rule,
            subject: subject.into(),
            detail: detail.into(),
        }
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}: {}", self.rule, self.subject, self.detail)
    }
}

// ---------------------------------------------------------------------------
// Allowlist
// ---------------------------------------------------------------------------

/// A recorded exception. Every entry carries a written reason and a ticket: an
/// unexplained exemption is how a containment rule dies quietly.
struct Exemption {
    rule: Rule,
    subject: &'static str,
    reason: &'static str,
    ticket: &'static str,
}

/// The complete allowlist. Anything not listed here is a build failure.
///
/// The five entries below are one finding: the wasm-bindgen JavaScript bridge
/// reaches the dependency graph transitively and target-gated. It is recorded
/// rather than defined away, because the honest census result is "present,
/// unreachable on our targets", not "absent".
const ALLOWLIST: &[Exemption] = &[
    Exemption {
        rule: Rule::ForeignRuntime,
        subject: "js-sys",
        reason: "Transitive and wasm32-target-gated only: \
                 quire-protocol -> quire-spec-language -> quire-contract-ir -> jsonschema \
                 -> getrandom(wasm32) -> js-sys. `cargo tree -i js-sys` on the host target \
                 resolves to nothing; it is reachable only with --target all. No first-party \
                 path uses it and no JavaScript is executed by this crate.",
        ticket: "agent-ix/quire-research#64",
    },
    Exemption {
        rule: Rule::ForeignRuntime,
        subject: "wasm-bindgen",
        reason: "Same wasm32-gated getrandom path as js-sys; see that entry.",
        ticket: "agent-ix/quire-research#64",
    },
    Exemption {
        rule: Rule::ForeignRuntime,
        subject: "wasm-bindgen-macro",
        reason: "Same wasm32-gated getrandom path as js-sys; see that entry.",
        ticket: "agent-ix/quire-research#64",
    },
    Exemption {
        rule: Rule::ForeignRuntime,
        subject: "wasm-bindgen-macro-support",
        reason: "Same wasm32-gated getrandom path as js-sys; see that entry.",
        ticket: "agent-ix/quire-research#64",
    },
    Exemption {
        rule: Rule::ForeignRuntime,
        subject: "wasm-bindgen-shared",
        reason: "Same wasm32-gated getrandom path as js-sys; see that entry.",
        ticket: "agent-ix/quire-research#64",
    },
];

fn is_allowlisted(finding: &Finding) -> bool {
    ALLOWLIST
        .iter()
        .any(|entry| entry.rule == finding.rule && entry.subject == finding.subject)
}

// ---------------------------------------------------------------------------
// Vocabularies
// ---------------------------------------------------------------------------

/// Extensions an interpreter or shell executes.
const SCRIPT_EXTENSIONS: &[&str] = &[
    "applescript",
    "awk",
    "bash",
    "bat",
    "cjs",
    "cmd",
    "coffee",
    "cts",
    "ex",
    "exs",
    "fish",
    "groovy",
    "jl",
    "js",
    "jsx",
    "ksh",
    "lua",
    "mjs",
    "mts",
    "nu",
    "php",
    "pl",
    "pm",
    "ps1",
    "psm1",
    "py",
    "pyw",
    "rb",
    "sh",
    "tcl",
    "ts",
    "tsx",
    "vbs",
    "zsh",
];

/// Extensions that carry data, prose or configuration and no interpreter.
const INERT_EXTENSIONS: &[&str] = &[
    "csv", "ico", "json", "lock", "md", "png", "svg", "toml", "txt", "yaml", "yml",
];

/// Extensionless file names that carry no interpreter.
const INERT_FILE_NAMES: &[&str] = &["LICENSE", "NOTICE", "CODEOWNERS"];

/// File names another tool executes as a command manifest.
const COMMAND_MANIFEST_NAMES: &[&str] = &[
    "Dockerfile",
    "GNUmakefile",
    "Justfile",
    "Makefile",
    "Taskfile.yml",
    "gulpfile.js",
    "justfile",
    "makefile",
    "noxfile.py",
    "package-lock.json",
    "package.json",
    "pnpm-lock.yaml",
    "pyproject.toml",
    "setup.cfg",
    "setup.py",
    "tox.ini",
];

/// Crates that embed a foreign interpreter, bridge to one, or spawn processes.
const FOREIGN_RUNTIME_CRATES: &[&str] = &[
    "assert_cmd",
    "boa_engine",
    "command-group",
    "cpython",
    "deno_core",
    "duct",
    "js-sys",
    "mlua",
    "node-bindgen",
    "pyo3",
    "quickjs-rs",
    "rlua",
    "rquickjs",
    "run_script",
    "rustpython-vm",
    "subprocess",
    "v8",
    "wasm-bindgen",
    "wasm-bindgen-macro",
    "wasm-bindgen-macro-support",
    "wasm-bindgen-shared",
    "xshell",
];

/// Fragments that, concatenated, name a process spawn in Rust. Split so this
/// file does not match its own rule.
const SPAWN_NEEDLES: &[&[&str]] = &[
    &["Command", "::new"],
    &["process", "::Command"],
    &["Command", "::spawn"],
];

/// Fragments naming a duplicate assurance surface this crate must not own.
const SHARED_BOUNDARY_NEEDLES: &[&[&str]] = &[
    &["Verdict", "Parser"],
    &["Evidence", "Store"],
    &["Method", "Catalog"],
    &["Sufficiency", "Decision"],
    &["Human", "Disposition"],
    &["Local", "Runner"],
    &["Stdout", "Parser"],
];

/// Fragments naming an unsafe item. A doc mention of the word is not a match;
/// only the word followed by an item or block opener is.
const UNSAFE_NEEDLES: &[&[&str]] = &[
    &["unsafe", " {"],
    &["unsafe", " fn"],
    &["unsafe", " impl"],
    &["unsafe", " trait"],
    &["unsafe", " extern"],
];

fn joined(parts: &[&str]) -> String {
    parts.concat()
}

/// The two bytes that open an interpreter line, assembled rather than written.
fn interpreter_marker() -> [u8; 2] {
    // Written as ASCII codes rather than a byte string, because a byte string
    // would put the forbidden sequence into this file and force the audit to
    // exempt itself from its own generated-template rule.
    [0x23, 0x21]
}

/// A string literal that opens with an interpreter marker.
fn template_needle() -> String {
    let marker = interpreter_marker();
    let mut needle = String::from("\"");
    needle.push(marker[0] as char);
    needle.push(marker[1] as char);
    needle
}

// ---------------------------------------------------------------------------
// Tree walking
// ---------------------------------------------------------------------------

/// A `.gitignore` pattern this audit understands.
#[derive(Debug, PartialEq, Eq)]
enum Ignore {
    RootEntry(String),
    Extension(String),
}

/// Parse `.gitignore`, refusing any pattern form the walker cannot honour.
///
/// A silently skipped pattern would hide files from the census, so an
/// unrecognised form fails the audit instead of being ignored.
fn ignore_rules(text: &str) -> Vec<Ignore> {
    let mut rules = Vec::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(rest) = line.strip_prefix('/') {
            let name = rest.trim_end_matches('/');
            assert!(
                !name.is_empty() && !name.contains('/') && !name.contains('*'),
                "TC-010: unsupported root ignore pattern {line:?}; extend the audit"
            );
            rules.push(Ignore::RootEntry(name.to_owned()));
        } else if let Some(ext) = line.strip_prefix("*.") {
            assert!(
                !ext.is_empty() && !ext.contains('/') && !ext.contains('*'),
                "TC-010: unsupported extension ignore pattern {line:?}; extend the audit"
            );
            rules.push(Ignore::Extension(ext.to_owned()));
        } else {
            panic!(
                "TC-010: unsupported .gitignore pattern {line:?}. Extend this walker \
                 rather than letting the pattern hide files from the census."
            );
        }
    }
    rules
}

fn ignored(relative: &str, rules: &[Ignore]) -> bool {
    let first = relative.split('/').next().unwrap_or(relative);
    rules.iter().any(|rule| match rule {
        Ignore::RootEntry(name) => first == name,
        Ignore::Extension(ext) => relative
            .rsplit_once('.')
            .is_some_and(|(_, found)| found == ext),
    })
}

/// One file in the working tree, as the census sees it.
struct TreeFile {
    relative: String,
    bytes: Vec<u8>,
    executable: bool,
}

#[cfg(unix)]
fn executable_bit(metadata: &fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;
    metadata.permissions().mode() & 0o111 != 0
}

#[cfg(not(unix))]
fn executable_bit(_metadata: &fs::Metadata) -> bool {
    false
}

/// Walk `root` in Rust, honouring `.gitignore` and always skipping `.git`.
fn walk(root: &Path) -> Vec<TreeFile> {
    let rules = ignore_rules(&fs::read_to_string(root.join(".gitignore")).unwrap_or_default());
    let mut files = Vec::new();
    walk_into(root, root, &rules, &mut files);
    files.sort_by(|left, right| left.relative.cmp(&right.relative));
    files
}

fn walk_into(root: &Path, dir: &Path, rules: &[Ignore], out: &mut Vec<TreeFile>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|error| panic!("TC-010: read {dir:?}: {error}"));
    for entry in entries {
        let entry = entry.unwrap_or_else(|error| panic!("TC-010: entry in {dir:?}: {error}"));
        let path = entry.path();
        let relative = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        if relative == ".git" || ignored(&relative, rules) {
            continue;
        }
        let metadata = entry
            .metadata()
            .unwrap_or_else(|error| panic!("TC-010: metadata for {path:?}: {error}"));
        if metadata.is_dir() {
            walk_into(root, &path, rules, out);
        } else if metadata.is_file() {
            out.push(TreeFile {
                relative,
                bytes: fs::read(&path)
                    .unwrap_or_else(|error| panic!("TC-010: read {path:?}: {error}")),
                executable: executable_bit(&metadata),
            });
        }
    }
}

// ---------------------------------------------------------------------------
// Inspection
// ---------------------------------------------------------------------------

fn file_name(relative: &str) -> &str {
    relative.rsplit('/').next().unwrap_or(relative)
}

fn extension(relative: &str) -> Option<&str> {
    let name = file_name(relative);
    name.rsplit_once('.')
        .filter(|(stem, _)| !stem.is_empty())
        .map(|(_, ext)| ext)
}

fn has_interpreter_marker(bytes: &[u8], is_rust: bool) -> bool {
    let marker = interpreter_marker();
    if bytes.len() < 2 || bytes[0] != marker[0] || bytes[1] != marker[1] {
        return false;
    }
    // A Rust inner attribute opens with the same two bytes followed by `[`.
    !(is_rust && bytes.get(2) == Some(&b'['))
}

/// Inspect one file. Pure over its inputs, so every rule can be fired against
/// synthetic content without touching the disk.
fn inspect_file(relative: &str, bytes: &[u8], executable: bool) -> Vec<Finding> {
    let mut findings = Vec::new();
    let name = file_name(relative);
    let ext = extension(relative);
    let is_rust = ext == Some("rs");
    let text = String::from_utf8_lossy(bytes);

    if let Some(ext) = ext {
        if SCRIPT_EXTENSIONS.contains(&ext) {
            findings.push(Finding::new(
                Rule::ExecutablePath,
                relative,
                format!("`.{ext}` is an interpreted-language extension"),
            ));
        } else if !is_rust && !INERT_EXTENSIONS.contains(&ext) {
            findings.push(Finding::new(
                Rule::ExecutablePath,
                relative,
                format!(
                    "`.{ext}` is not classified by this audit; classify it as inert or \
                     executable rather than letting it pass unexamined"
                ),
            ));
        }
    } else if !INERT_FILE_NAMES.contains(&name) && !name.starts_with('.') {
        findings.push(Finding::new(
            Rule::ExecutablePath,
            relative,
            "extensionless file is not a known inert name".to_owned(),
        ));
    }

    if COMMAND_MANIFEST_NAMES.contains(&name) {
        findings.push(Finding::new(
            Rule::ExecutablePath,
            relative,
            "build/command manifest executed by a non-Rust tool".to_owned(),
        ));
    }

    if has_interpreter_marker(bytes, is_rust) {
        findings.push(Finding::new(
            Rule::ExecutablePath,
            relative,
            "file opens with an interpreter marker".to_owned(),
        ));
    }

    if executable && !is_rust {
        findings.push(Finding::new(
            Rule::ExecutablePath,
            relative,
            "carries a Unix execute permission bit".to_owned(),
        ));
    }

    if is_rust {
        for parts in SPAWN_NEEDLES {
            let needle = joined(parts);
            if text.contains(&needle) {
                findings.push(Finding::new(
                    Rule::ExecutablePath,
                    relative,
                    format!(
                        "Rust source spawns a process (`{needle}`): a Rust wrapper around \
                         non-Rust logic is not remediation"
                    ),
                ));
            }
        }
        if text.contains(&template_needle()) {
            findings.push(Finding::new(
                Rule::ExecutablePath,
                relative,
                "Rust source emits a string literal opening with an interpreter marker \
                 (generated executable template)"
                    .to_owned(),
            ));
        }
        for parts in UNSAFE_NEEDLES {
            let needle = joined(parts);
            if text.contains(&needle) {
                findings.push(Finding::new(
                    Rule::UnsafeItem,
                    relative,
                    format!("unsafe item `{needle}` without a reviewed owner disposition"),
                ));
            }
        }
        for parts in SHARED_BOUNDARY_NEEDLES {
            let needle = joined(parts);
            if text.contains(&needle) {
                findings.push(Finding::new(
                    Rule::SharedBoundary,
                    relative,
                    format!(
                        "`{needle}` duplicates an Engineering Assurance or Quoin surface \
                         this crate must consume, not own"
                    ),
                ));
            }
        }
        findings.extend(inspect_trace_bindings(relative, &text));
    }

    if is_workflow(relative) {
        findings.extend(inspect_workflow(relative, &text));
    }

    findings
}

/// NFR-003-AC-5. Every `#[test]` carries a bare `#[trace(..)]`, and no binding
/// is path-qualified: the path-qualified form compiles and binds nothing.
fn inspect_trace_bindings(relative: &str, text: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    let qualified = joined(&["#[", "ix_trace_rs", "::trace"]);
    if text.contains(&qualified) {
        findings.push(Finding::new(
            Rule::TraceBinding,
            relative,
            "path-qualified trace attribute: it compiles and binds nothing; \
             import the macro and use the bare form"
                .to_owned(),
        ));
    }

    let mut attributes: Vec<String> = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with("#[") {
            attributes.push(line.to_owned());
            continue;
        }
        if line.starts_with("//") || line.is_empty() {
            continue;
        }
        if line.starts_with("fn ") || line.starts_with("pub fn ") || line.starts_with("async fn ") {
            let is_test = attributes.iter().any(|attr| attr.starts_with("#[test]"));
            let traced = attributes.iter().any(|attr| attr.starts_with("#[trace("));
            if is_test && !traced {
                let name = line
                    .trim_start_matches("pub ")
                    .trim_start_matches("async ")
                    .trim_start_matches("fn ")
                    .split(['(', '<'])
                    .next()
                    .unwrap_or(line);
                findings.push(Finding::new(
                    Rule::TraceBinding,
                    relative,
                    format!("test `{name}` carries no bare trace binding"),
                ));
            }
        }
        attributes.clear();
    }

    findings
}

fn is_workflow(relative: &str) -> bool {
    relative.starts_with(".github/") && matches!(extension(relative), Some("yml") | Some("yaml"))
}

/// NFR-003-AC-1. Hosted workflows declare `workflow_dispatch` and nothing else,
/// and carry no inline command step.
fn inspect_workflow(relative: &str, text: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    let mut triggers: BTreeSet<String> = BTreeSet::new();
    let mut in_on_block = false;
    // Triggers are the keys at the first indent level under `on:`; a deeper key
    // is a filter of its trigger (`branches:` under `push:`), not a trigger.
    let mut trigger_indent: Option<usize> = None;

    for line in text.lines() {
        let trimmed = line.trim_start();
        let indent = line.len() - trimmed.len();

        let step = trimmed.trim_start_matches("- ").trim_end();
        if step.starts_with("run:") {
            findings.push(Finding::new(
                Rule::Workflow,
                relative,
                "inline command step in a hosted workflow".to_owned(),
            ));
        }

        if indent == 0 {
            if let Some(rest) = trimmed.strip_prefix("on:") {
                let rest = rest.trim();
                in_on_block = rest.is_empty();
                trigger_indent = None;
                if !rest.is_empty() {
                    for value in rest.trim_matches(['[', ']']).split(',') {
                        let value = value.trim();
                        if !value.is_empty() {
                            triggers.insert(value.to_owned());
                        }
                    }
                }
                continue;
            }
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                in_on_block = false;
            }
        } else if in_on_block {
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let level = *trigger_indent.get_or_insert(indent);
            if indent > level {
                continue;
            }
            if let Some((key, _)) = trimmed.split_once(':') {
                let key = key.trim_start_matches("- ").trim();
                if !key.is_empty() && !key.starts_with('#') {
                    triggers.insert(key.to_owned());
                }
            } else if let Some(item) = trimmed.strip_prefix("- ") {
                let item = item.trim();
                if !item.is_empty() {
                    triggers.insert(item.to_owned());
                }
            }
        }
    }

    if triggers.is_empty() {
        findings.push(Finding::new(
            Rule::Workflow,
            relative,
            "workflow declares no readable trigger".to_owned(),
        ));
    }
    for trigger in &triggers {
        if trigger != "workflow_dispatch" {
            findings.push(Finding::new(
                Rule::Workflow,
                relative,
                format!("trigger `{trigger}` is not manual dispatch"),
            ));
        }
    }

    findings
}

/// Inspect the dependency inventory: direct manifest keys plus every resolved
/// package in the lock file.
fn inspect_manifest(cargo_toml: &str, cargo_lock: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    let mut section = String::new();
    let mut forbid_declared = false;

    for raw in cargo_toml.lines() {
        let line = raw.trim();
        if line.starts_with('[') {
            section = line.trim_matches(['[', ']']).to_owned();
            continue;
        }
        if section == "lints.rust" && line.starts_with("unsafe_code") && line.contains("forbid") {
            forbid_declared = true;
        }
        if section == "package" && line.starts_with("build") && line.contains('=') {
            findings.push(Finding::new(
                Rule::ExecutablePath,
                "Cargo.toml",
                "package declares a build script command".to_owned(),
            ));
        }
    }

    if !forbid_declared {
        findings.push(Finding::new(
            Rule::UnsafeItem,
            "Cargo.toml",
            "crate does not declare `unsafe_code = \"forbid\"` in [lints.rust]".to_owned(),
        ));
    }

    let mut packages: BTreeSet<&str> = BTreeSet::new();
    for raw in cargo_lock.lines() {
        let line = raw.trim();
        if let Some(rest) = line.strip_prefix("name = ") {
            packages.insert(rest.trim_matches('"'));
        }
    }

    for package in packages {
        if FOREIGN_RUNTIME_CRATES.contains(&package) {
            findings.push(Finding::new(
                Rule::ForeignRuntime,
                package,
                "dependency embeds, bridges to, or spawns a non-Rust runtime".to_owned(),
            ));
        }
        if package.starts_with("quoin") {
            findings.push(Finding::new(
                Rule::SharedBoundary,
                package,
                "Quoin must stay external behind its structured interface, \
                 never a runtime dependency"
                    .to_owned(),
            ));
        }
    }

    findings
}

// ---------------------------------------------------------------------------
// The census
// ---------------------------------------------------------------------------

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Every finding in the current tree, allowlisted entries included.
fn raw_census() -> Vec<Finding> {
    let root = repository_root();
    let mut findings = Vec::new();
    let mut rust_files = 0_usize;
    for file in walk(&root) {
        if extension(&file.relative) == Some("rs") {
            rust_files += 1;
        }
        findings.extend(inspect_file(&file.relative, &file.bytes, file.executable));
    }
    assert!(
        rust_files > 0,
        "TC-010: the walker found no Rust file at all — the audit is bound to nothing"
    );
    findings.extend(inspect_manifest(
        &fs::read_to_string(root.join("Cargo.toml")).expect("TC-010: read Cargo.toml"),
        &fs::read_to_string(root.join("Cargo.lock")).expect("TC-010: read Cargo.lock"),
    ));
    findings.sort();
    findings
}

/// Assert the census is clean for `rule`, printing **every** finding. A high
/// count is a census result, not a reason to weaken the rule, so nothing here
/// truncates, samples, or stops at the first failure.
fn assert_clean(rule: Rule) {
    let all = raw_census();
    let reported: Vec<&Finding> = all
        .iter()
        .filter(|finding| finding.rule == rule && !is_allowlisted(finding))
        .collect();
    if reported.is_empty() {
        return;
    }
    let mut message = format!(
        "TC-010 census: {} {rule} violation(s) in the tree:\n",
        reported.len()
    );
    for finding in reported {
        message.push_str(&format!("  {finding}\n"));
    }
    message.push_str(
        "Each line is a finding to remediate or to allowlist with a written reason and a \
         ticket. Do not narrow the rule to make this pass.",
    );
    panic!("{message}");
}

// ---------------------------------------------------------------------------
// Acceptance tests — one per NFR-003 criterion
// ---------------------------------------------------------------------------

#[trace("NFR-003-AC-1", "TC-010")]
#[test]
fn tc_010_hosted_workflows_are_dispatch_only() {
    assert_clean(Rule::Workflow);
}

#[trace("NFR-003-AC-2", "TC-010")]
#[test]
fn tc_010_no_unsafe_block_without_owner_disposition() {
    assert_clean(Rule::UnsafeItem);
}

#[trace("NFR-003-AC-3", "TC-010")]
#[test]
fn tc_010_every_executable_path_is_rust() {
    assert_clean(Rule::ExecutablePath);
}

#[trace("NFR-003-AC-4", "TC-010")]
#[test]
fn tc_010_no_local_runner_or_evidence_store_in_inventory() {
    assert_clean(Rule::SharedBoundary);
}

#[trace("NFR-003-AC-5", "TC-010")]
#[test]
fn tc_010_every_acceptance_test_carries_a_bare_trace_binding() {
    assert_clean(Rule::TraceBinding);
}

#[trace("NFR-003-AC-6", "TC-010")]
#[test]
fn tc_010_no_quoin_copy_and_no_js_path() {
    assert_clean(Rule::ForeignRuntime);
}

// ---------------------------------------------------------------------------
// Proof that the audit fires
//
// A rule that greps fine and binds to nothing is the failure mode this section
// exists to exclude. Every rule above is fired here against content that should
// trip it, so a passing census is evidence of a clean tree rather than evidence
// of a dead check.
// ---------------------------------------------------------------------------

fn fires(relative: &str, body: &str, rule: Rule) -> Vec<Finding> {
    let found: Vec<Finding> = inspect_file(relative, body.as_bytes(), false)
        .into_iter()
        .filter(|finding| finding.rule == rule)
        .collect();
    assert!(
        !found.is_empty(),
        "TC-010: rule {rule} did not fire on {relative}, which it must reject"
    );
    found
}

#[trace("NFR-003-AC-3", "TC-010")]
#[test]
fn tc_010_audit_rejects_script_extensions_and_markers() {
    let shell = fires("tools/build.sh", "echo hello\n", Rule::ExecutablePath);
    assert!(
        shell
            .iter()
            .any(|finding| finding.detail.contains("interpreted-language extension")),
        "TC-010: `.sh` must be classified as a script, not merely unrecognised, \
         found {shell:?}"
    );
    fires("tools/generate.py", "print(1)\n", Rule::ExecutablePath);
    fires("web/app.ts", "export const a = 1;\n", Rule::ExecutablePath);

    let marker = interpreter_marker();
    let shebang = format!(
        "{}{}/usr/bin/env bash\necho hello\n",
        marker[0] as char, marker[1] as char
    );
    // Extensionless, renamed, and even Rust-suffixed interpreter files are caught.
    fires("tools/release", &shebang, Rule::ExecutablePath);
    fires("tools/notes.md", &shebang, Rule::ExecutablePath);
    fires("src/generate.rs", &shebang, Rule::ExecutablePath);

    fires("package.json", "{}\n", Rule::ExecutablePath);
    fires("Makefile", "all:\n", Rule::ExecutablePath);
    fires("data/corpus.xyz", "anything\n", Rule::ExecutablePath);

    let executable = inspect_file("tests/fixtures/opaque", b"binary bytes", true);
    assert!(
        executable
            .iter()
            .any(|finding| finding.detail.contains("Unix execute permission bit")),
        "TC-010: the execute-bit rule must fire, found {executable:?}"
    );
}

#[trace("NFR-003-AC-3", "TC-010")]
#[test]
fn tc_010_audit_rejects_the_rust_wrapper_bypass() {
    for parts in SPAWN_NEEDLES {
        let body = format!("fn run() {{ let _ = {}(\"sh\"); }}\n", joined(parts));
        fires("src/wrapper.rs", &body, Rule::ExecutablePath);
    }

    let marker = interpreter_marker();
    let template = format!(
        "const SCRIPT: &str = \"{}{}/bin/sh\\necho\";\n",
        marker[0] as char, marker[1] as char
    );
    fires("src/template.rs", &template, Rule::ExecutablePath);

    // A Rust file that only mentions these things in prose is not a violation.
    assert!(
        inspect_file(
            "src/clean.rs",
            b"//! This module spawns nothing and generates no script.\n",
            false
        )
        .is_empty(),
        "TC-010: prose about scripts must not be reported as a violation"
    );
}

#[trace("NFR-003-AC-1", "TC-010")]
#[test]
fn tc_010_audit_rejects_non_dispatch_triggers_and_inline_commands() {
    let pushed = "name: ci\non:\n  push:\n    branches: [main]\njobs: {}\n";
    let found = fires(".github/workflows/ci.yml", pushed, Rule::Workflow);
    assert!(
        found
            .iter()
            .any(|finding| finding.detail.contains("`push`")),
        "TC-010: the automatic trigger must be named, found {found:?}"
    );
    assert!(
        !found
            .iter()
            .any(|finding| finding.detail.contains("`branches`")),
        "TC-010: a filter under a trigger is not itself a trigger, found {found:?}"
    );

    let inline =
        "name: ci\non:\n  workflow_dispatch:\njobs:\n  a:\n    steps:\n      - run: cargo test\n";
    fires(".github/workflows/inline.yml", inline, Rule::Workflow);

    let untriggered = "name: ci\njobs: {}\n";
    fires(".github/workflows/none.yml", untriggered, Rule::Workflow);

    let dispatch = "name: ci\non:\n  workflow_dispatch:\njobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n";
    assert!(
        inspect_file(".github/workflows/ok.yml", dispatch.as_bytes(), false).is_empty(),
        "TC-010: a manual-dispatch workflow with no inline command must pass"
    );
}

#[trace("NFR-003-AC-2", "TC-010")]
#[test]
fn tc_010_audit_rejects_unsafe_items() {
    for parts in UNSAFE_NEEDLES {
        let body = format!("fn f() {{ {} }}\n", joined(parts));
        fires("src/danger.rs", &body, Rule::UnsafeItem);
    }
    assert!(
        inspect_file(
            "src/lint.rs",
            b"//! This crate forbids unsafe_code at the crate root.\n",
            false
        )
        .is_empty(),
        "TC-010: the lint name and prose must not be read as an unsafe item"
    );
}

#[trace("NFR-003-AC-4", "TC-010")]
#[test]
fn tc_010_audit_rejects_duplicate_assurance_surfaces() {
    for parts in SHARED_BOUNDARY_NEEDLES {
        let body = format!("pub struct {} {{}}\n", joined(parts));
        fires("src/local.rs", &body, Rule::SharedBoundary);
    }

    let lock = "[[package]]\nname = \"quoin-core\"\nversion = \"0.1.0\"\n";
    let findings = inspect_manifest("[lints.rust]\nunsafe_code = \"forbid\"\n", lock);
    assert!(
        findings
            .iter()
            .any(|finding| finding.rule == Rule::SharedBoundary),
        "TC-010: a Quoin runtime dependency must be reported, found {findings:?}"
    );
}

#[trace("NFR-003-AC-5", "TC-010")]
#[test]
fn tc_010_missing_trace_binding_fails() {
    let untraced = "#[test]\nfn tc_000_untraced() {}\n";
    let found = fires("tests/untraced.rs", untraced, Rule::TraceBinding);
    assert!(
        found[0].detail.contains("tc_000_untraced"),
        "TC-010: the finding must name the untraced test, found {found:?}"
    );

    let qualified = joined(&["#[", "ix_trace_rs", "::trace"]);
    let body = format!("{qualified}(\"TC-010\")]\n#[test]\nfn tc_000_qualified() {{}}\n");
    fires("tests/qualified.rs", &body, Rule::TraceBinding);

    let traced = "#[trace(\"TC-010\")]\n#[test]\nfn tc_000_traced() {}\n";
    assert!(
        inspect_file("tests/traced.rs", traced.as_bytes(), false).is_empty(),
        "TC-010: a bare-bound test must pass"
    );
    // A misspelled id is rejected earlier still: `ix_trace_rs::trace` validates
    // the id shape at expansion time, so `#[trace("NFR3AC1")]` fails to compile
    // and never reaches this audit.
}

#[trace("NFR-003-AC-6", "TC-010")]
#[test]
fn tc_010_audit_rejects_foreign_runtime_dependencies() {
    let lock = "[[package]]\nname = \"pyo3\"\nversion = \"0.22.0\"\n";
    let findings = inspect_manifest("[lints.rust]\nunsafe_code = \"forbid\"\n", lock);
    assert!(
        findings
            .iter()
            .any(|finding| finding.rule == Rule::ForeignRuntime && finding.subject == "pyo3"),
        "TC-010: an embedded foreign runtime must be reported, found {findings:?}"
    );

    let missing_forbid = inspect_manifest("[package]\nname = \"x\"\n", "");
    assert!(
        missing_forbid
            .iter()
            .any(|finding| finding.rule == Rule::UnsafeItem),
        "TC-010: a manifest without the forbid lint must be reported, found {missing_forbid:?}"
    );

    let build_script = inspect_manifest(
        "[package]\nbuild = \"build.rs\"\n[lints.rust]\nunsafe_code = \"forbid\"\n",
        "",
    );
    assert!(
        build_script
            .iter()
            .any(|finding| finding.rule == Rule::ExecutablePath),
        "TC-010: a declared build script must be reported, found {build_script:?}"
    );
}

#[trace("NFR-003-AC-3", "TC-010")]
#[test]
fn tc_010_walker_reaches_a_planted_violation_on_disk() {
    // The rules above are proved on synthetic content; this proves the walker
    // that feeds them actually reaches a nested file on a real filesystem.
    let root = std::env::temp_dir().join(format!(
        "tc010-walker-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_nanos())
            .unwrap_or_default()
    ));
    let nested = root.join("tools").join("nested");
    fs::create_dir_all(&nested).expect("TC-010: create the planted tree");
    fs::write(root.join(".gitignore"), "/target/\n").expect("TC-010: write .gitignore");
    fs::write(root.join("keep.rs"), "// clean\n").expect("TC-010: write a Rust file");
    fs::write(nested.join("deploy.sh"), "echo hi\n").expect("TC-010: write the planted script");
    fs::create_dir_all(root.join("target")).expect("TC-010: create an ignored directory");
    fs::write(root.join("target").join("ignored.sh"), "echo hi\n")
        .expect("TC-010: write an ignored script");

    let walked = walk(&root);
    let relatives: Vec<&str> = walked.iter().map(|file| file.relative.as_str()).collect();
    assert!(
        relatives.contains(&"tools/nested/deploy.sh"),
        "TC-010: the walker must reach nested files, saw {relatives:?}"
    );
    assert!(
        !relatives.contains(&"target/ignored.sh"),
        "TC-010: the walker must honour .gitignore, saw {relatives:?}"
    );

    let findings: Vec<Finding> = walked
        .iter()
        .flat_map(|file| inspect_file(&file.relative, &file.bytes, file.executable))
        .collect();
    assert!(
        findings
            .iter()
            .any(|finding| finding.subject == "tools/nested/deploy.sh"),
        "TC-010: the planted script must be reported, found {findings:?}"
    );

    fs::remove_dir_all(&root).expect("TC-010: remove the planted tree");
}

#[trace("NFR-003-AC-3", "TC-010")]
#[test]
fn tc_010_gitignore_patterns_are_understood_or_refused() {
    assert_eq!(
        ignore_rules("/target/\n*.profraw\n"),
        vec![
            Ignore::RootEntry("target".to_owned()),
            Ignore::Extension("profraw".to_owned()),
        ]
    );
    let refused = std::panic::catch_unwind(|| ignore_rules("src/**/generated/*.rs\n"));
    assert!(
        refused.is_err(),
        "TC-010: an ignore pattern the walker cannot honour must fail the audit, \
         not silently hide files"
    );
}

#[trace("NFR-003-AC-4", "TC-010")]
#[test]
fn tc_010_every_allowlist_entry_carries_a_reason_and_a_ticket() {
    for entry in ALLOWLIST {
        assert!(
            entry.reason.len() > 40,
            "TC-010: allowlist entry `{}` needs a written reason, not a label",
            entry.subject
        );
        assert!(
            entry.ticket.contains('#'),
            "TC-010: allowlist entry `{}` needs a ticket reference",
            entry.subject
        );
    }
    // Every allowlisted subject must still be a live finding. A stale entry is
    // an exemption nobody can see the cost of.
    let census = raw_census();
    for entry in ALLOWLIST {
        assert!(
            census
                .iter()
                .any(|finding| finding.rule == entry.rule && finding.subject == entry.subject),
            "TC-010: allowlist entry `{}` matches no current finding; remove it",
            entry.subject
        );
    }
}

#[trace("NFR-003-AC-3", "TC-010")]
#[test]
fn tc_010_census_reports_every_finding() {
    // The census surface: run with `-- --nocapture` to read it. Every finding
    // is printed, allowlisted ones included, so the cost of each exemption
    // stays visible and a growing count is never hidden by a passing gate.
    let census = raw_census();
    let mut outstanding = 0_usize;
    for finding in &census {
        let state = if is_allowlisted(finding) {
            "allowlisted"
        } else {
            outstanding += 1;
            "OUTSTANDING"
        };
        println!("TC-010 census [{state}] {finding}");
    }
    println!(
        "TC-010 census: {} finding(s), {} allowlisted, {outstanding} outstanding",
        census.len(),
        census.len() - outstanding
    );
    assert_eq!(
        outstanding, 0,
        "TC-010: the tree carries {outstanding} unexempted containment violation(s); \
         the lines above are the complete census"
    );
}
