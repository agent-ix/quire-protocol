// SPDX-License-Identifier: AGPL-3.0-or-later
//! A's complete handoffs, emitted fresh by A's own producer.
//!
//! QSL commits its `/1` and `/2` handoffs under `PUBLISHED_V1_HANDOFF` and
//! `PUBLISHED_HANDOFF`, but those copies carry no `dependencies/` bytes: QSL
//! commits no exact-byte dependency fixtures, so the committed sidecars select
//! an empty dependency inventory and the committed offers cannot be admitted.
//! A fresh run of A's `native_protocol_handoff` and `native_protocol_v2_handoff`
//! examples writes the complete handoff, dependency bytes included.
//!
//! B runs those two examples from the exactly pinned QSL source into its own
//! target directory. It keeps no copy of any handoff member, and it derives no
//! expectation from the offer: the sidecar and original inputs are still the
//! producer's own files, read and checksummed exactly as before.
//!
//! QSL-251 asks QSL for a public handoff writer so this module can stop
//! building QSL's examples and walking its source tree.

use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::{
        atomic::{AtomicUsize, Ordering},
        OnceLock,
    },
};

use sha2::{Digest, Sha256};

use quire_spec_language::protocol_artifact::handoff::PUBLISHED_HANDOFF;

/// A's version-1 producer example.
const V1_EXAMPLE: &str = "native_protocol_handoff";
/// A's version-2 producer example.
const V2_EXAMPLE: &str = "native_protocol_v2_handoff";

/// Directories holding one published `/1` and `/2` producer run.
struct Runs {
    v1: PathBuf,
    v2: PathBuf,
}

/// The fresh `/1` handoff directory.
#[allow(dead_code, reason = "each including test target uses one version")]
pub fn v1() -> Result<&'static Path, String> {
    runs().map(|runs| runs.v1.as_path())
}

/// The fresh `/2` handoff directory.
#[allow(dead_code, reason = "each including test target uses one version")]
pub fn v2() -> Result<&'static Path, String> {
    runs().map(|runs| runs.v2.as_path())
}

fn runs() -> Result<&'static Runs, String> {
    static RUNS: OnceLock<Result<Runs, String>> = OnceLock::new();
    RUNS.get_or_init(|| {
        emit_and_publish().map(|published| Runs {
            v1: published.join("v1"),
            v2: published.join("v2"),
        })
    })
    .as_ref()
    .map_err(Clone::clone)
}

/// Build A's producer examples, run both into a private directory, and publish
/// that run. Returns the published directory, which holds `v1/` and `v2/`.
///
/// Every call emits afresh; `v1()`/`v2()` call this once per process.
pub fn emit_and_publish() -> Result<PathBuf, String> {
    // `PUBLISHED_HANDOFF` is `<QSL source>/artifacts/compiled-protocol-v2`.
    let qsl = Path::new(PUBLISHED_HANDOFF)
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| format!("{PUBLISHED_HANDOFF} has no QSL source root"))?;
    let build = producer_target()?;

    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"));
    let status = Command::new(cargo)
        .arg("build")
        .arg("--locked")
        .arg("--manifest-path")
        .arg(qsl.join("Cargo.toml"))
        .args(["--example", V1_EXAMPLE, "--example", V2_EXAMPLE])
        .arg("--target-dir")
        .arg(&build)
        .status()
        .map_err(|error| format!("cannot start cargo for A's producer: {error}"))?;
    if !status.success() {
        return Err(format!("building A's producer examples failed: {status}"));
    }

    let fresh = build.join(format!("fresh-{}-{}", std::process::id(), next_call()));
    remove_if_present(&fresh)?;
    fs::create_dir_all(&fresh)
        .map_err(|error| format!("cannot create {}: {error}", fresh.display()))?;
    let examples = build.join("debug").join("examples");
    for (example, version) in [(V1_EXAMPLE, "v1"), (V2_EXAMPLE, "v2")] {
        let status = Command::new(examples.join(example))
            .arg(fresh.join(version))
            .status()
            .map_err(|error| format!("cannot start A's {example}: {error}"))?;
        if !status.success() {
            return Err(format!("A's {example} failed: {status}"));
        }
    }
    publish(&build, &fresh)
}

/// Publish a fresh run under a directory named by its content:
/// `handoff-<sha256 of v1/SHA256SUMS then v2/SHA256SUMS>`. The inventories
/// cover every member, so equal names mean equal runs. A run that is already
/// published is reused and the fresh copy removed; nothing ever deletes or
/// replaces a published directory, so a process still reading one (including
/// a test that re-executes this binary) is never disturbed.
fn publish(build: &Path, fresh: &Path) -> Result<PathBuf, String> {
    let mut inventories = Sha256::new();
    for version in ["v1", "v2"] {
        let path = fresh.join(version).join("SHA256SUMS");
        let bytes =
            fs::read(&path).map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        inventories.update(&bytes);
    }
    let name: String = inventories
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    let published = build.join(format!("handoff-{name}"));
    if !published.is_dir() && fs::rename(fresh, &published).is_ok() {
        return Ok(published);
    }
    // Already published, possibly by a concurrent process that won the rename.
    if published.is_dir() {
        remove_if_present(fresh)?;
        return Ok(published);
    }
    Err(format!(
        "cannot publish A's producer run at {}",
        published.display()
    ))
}

/// A per-process call counter, so repeated calls use distinct fresh dirs.
fn next_call() -> usize {
    static CALLS: AtomicUsize = AtomicUsize::new(0);
    CALLS.fetch_add(1, Ordering::Relaxed)
}

fn remove_if_present(directory: &Path) -> Result<(), String> {
    match fs::remove_dir_all(directory) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("cannot remove {}: {error}", directory.display())),
    }
}

/// `<target>/qsl-producer`, beside this test binary's own target directory.
fn producer_target() -> Result<PathBuf, String> {
    let executable = std::env::current_exe()
        .map_err(|error| format!("cannot locate the test executable: {error}"))?;
    // `<target>/<profile>/deps/<test binary>`.
    executable
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .map(|target| target.join("qsl-producer"))
        .ok_or_else(|| format!("{} is not under a target directory", executable.display()))
}
