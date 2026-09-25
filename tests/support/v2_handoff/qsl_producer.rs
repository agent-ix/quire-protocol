// SPDX-License-Identifier: AGPL-3.0-or-later
//! A's complete handoffs, emitted fresh by A's own public writer.
//!
//! QSL commits its `/1` and `/2` handoffs under `PUBLISHED_V1_HANDOFF` and
//! `PUBLISHED_HANDOFF`, but those copies carry no `dependencies/` bytes: QSL
//! commits no exact-byte dependency fixtures, so the committed sidecars select
//! an empty dependency inventory and the committed offers cannot be admitted.
//! QSL-251's `protocol_artifact::handoff::{write_v1, write_v2}` (feature
//! `handoff-writer`, enabled on this crate's dev-dependency) write the complete
//! handoff, dependency bytes included, in-process.
//!
//! B keeps no copy of any handoff member, and it derives no expectation from
//! the offer: the sidecar and original inputs are still the producer's own
//! files, read and checksummed exactly as before.

use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicUsize, Ordering},
        OnceLock,
    },
};

use quire_spec_language::protocol_artifact::handoff::{write_v1, write_v2, PUBLISHED_HANDOFF};

/// Directories holding one `/1` and `/2` producer run.
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
        emit().map(|run| Runs {
            v1: run.join("v1"),
            v2: run.join("v2"),
        })
    })
    .as_ref()
    .map_err(Clone::clone)
}

/// Write A's `/1` and `/2` handoffs into a directory private to this call,
/// beside this test binary's target directory. Returns the directory, which
/// holds `v1/` and `v2/`.
///
/// Every call writes afresh; `v1()`/`v2()` call this once per process.
#[allow(dead_code, reason = "each including test target uses one version")]
pub fn emit() -> Result<PathBuf, String> {
    static CALLS: AtomicUsize = AtomicUsize::new(0);
    let run = producer_target()?.join(format!(
        "run-{}-{}",
        std::process::id(),
        CALLS.fetch_add(1, Ordering::Relaxed)
    ));
    match fs::remove_dir_all(&run) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(format!("cannot remove {}: {error}", run.display())),
    }
    fs::create_dir_all(&run)
        .map_err(|error| format!("cannot create {}: {error}", run.display()))?;
    write_v1(&run.join("v1")).map_err(|error| format!("A's write_v1 failed: {error}"))?;
    write_v2(&run.join("v2")).map_err(|error| format!("A's write_v2 failed: {error}"))?;
    Ok(run)
}

/// The exactly pinned QSL source root.
///
/// `PUBLISHED_HANDOFF` is `<QSL source>/artifacts/compiled-protocol-v2`. The
/// lanes still compare the written handoff's authored sources against it.
#[allow(dead_code, reason = "each including test target uses a subset")]
pub fn qsl_root() -> Result<&'static Path, String> {
    Path::new(PUBLISHED_HANDOFF)
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| format!("{PUBLISHED_HANDOFF} has no QSL source root"))
}

/// `<target>/qsl-producer`, beside this test binary's own target directory.
pub fn producer_target() -> Result<PathBuf, String> {
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
