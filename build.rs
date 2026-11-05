// SPDX-License-Identifier: AGPL-3.0-or-later
//! Pass the Cargo-selected E02 dependency revision to the compile-time lock adapter.
//!
//! The dependency follows `branch = "main"`; the commit Cargo selected is the
//! one `Cargo.lock` records, so that is where it is read from.

use std::{env, fs, path::PathBuf};

fn main() {
    let manifest_path = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let lock = fs::read_to_string(manifest_path.join("Cargo.lock")).expect("Cargo.lock");
    let package = lock
        .split("[[package]]")
        .find(|package| {
            package
                .lines()
                .any(|line| line == r#"name = "quire-verification-contracts""#)
        })
        .expect("direct E02 owner dependency is locked");
    let revision = package
        .lines()
        .find_map(|line| line.strip_prefix("source = "))
        .and_then(|source| source.trim_matches('"').rsplit_once('#'))
        .map(|(_, revision)| revision)
        .filter(|revision| {
            revision.len() == 40
                && revision
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        })
        .expect("direct E02 dependency has a Cargo-selected full revision");

    println!("cargo:rerun-if-changed=Cargo.lock");
    println!("cargo:rustc-env=QVERIFY_E02_REVISION={revision}");
}
