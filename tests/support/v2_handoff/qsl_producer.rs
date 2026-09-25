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

use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::OnceLock,
};

use quire_spec_language::protocol_artifact::handoff::PUBLISHED_HANDOFF;

/// A's version-1 producer example.
const V1_EXAMPLE: &str = "native_protocol_handoff";
/// A's version-2 producer example.
const V2_EXAMPLE: &str = "native_protocol_v2_handoff";
/// Prefix of each per-process output directory under the producer target dir.
const RUN_PREFIX: &str = "handoff-";

/// Directories holding one fresh `/1` and `/2` producer run.
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
    RUNS.get_or_init(run).as_ref().map_err(Clone::clone)
}

fn run() -> Result<Runs, String> {
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

    sweep_previous_runs(&build);
    let output = build.join(format!("{RUN_PREFIX}{}", std::process::id()));
    if output.exists() {
        fs::remove_dir_all(&output)
            .map_err(|error| format!("cannot clear {}: {error}", output.display()))?;
    }
    fs::create_dir_all(&output)
        .map_err(|error| format!("cannot create {}: {error}", output.display()))?;

    let examples = build.join("debug").join("examples");
    let runs = Runs {
        v1: output.join("v1"),
        v2: output.join("v2"),
    };
    for (example, directory) in [(V1_EXAMPLE, &runs.v1), (V2_EXAMPLE, &runs.v2)] {
        let status = Command::new(examples.join(example))
            .arg(directory)
            .status()
            .map_err(|error| format!("cannot start A's {example}: {error}"))?;
        if !status.success() {
            return Err(format!("A's {example} failed: {status}"));
        }
    }
    Ok(runs)
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

/// Remove earlier processes' outputs. Cargo runs one test binary at a time,
/// so no other live process reads them.
fn sweep_previous_runs(build: &Path) {
    let Ok(entries) = fs::read_dir(build) else {
        return;
    };
    let own = format!("{RUN_PREFIX}{}", std::process::id());
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with(RUN_PREFIX) && name != own {
            let _ = fs::remove_dir_all(entry.path());
        }
    }
}
