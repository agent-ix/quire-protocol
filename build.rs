// SPDX-License-Identifier: AGPL-3.0-or-later
//! Pass the Cargo-selected E02 dependency revision to the compile-time lock adapter.

use std::{env, fs, path::PathBuf};

fn main() {
    let manifest_path = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let manifest = fs::read_to_string(manifest_path.join("Cargo.toml")).expect("Cargo.toml");
    let dependencies = manifest
        .split("[dependencies]")
        .nth(1)
        .and_then(|section| section.split("\n[").next())
        .expect("dependencies section");
    let declaration = dependencies
        .lines()
        .find(|line| line.starts_with("quire-verification-contracts = "))
        .expect("direct E02 owner dependency");
    let revision = declaration
        .split_once("rev = \"")
        .and_then(|(_, suffix)| suffix.split_once('"').map(|(revision, _)| revision))
        .filter(|revision| {
            revision.len() == 40
                && revision
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        })
        .expect("direct E02 dependency has a Cargo-selected full revision");

    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=Cargo.lock");
    println!("cargo:rustc-env=QVERIFY_E02_REVISION={revision}");
}
