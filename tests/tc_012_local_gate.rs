// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-012/Task-014: executable definition of the repository-local gate.

use ix_trace_rs::trace;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const FORMAT_COMMAND: &str =
    "CARGO_BUILD_JOBS=1 flock /tmp/quire-heavy-check.lock cargo fmt --all -- --check";
const CLIPPY_COMMAND: &str = "CARGO_BUILD_JOBS=1 flock /tmp/quire-heavy-check.lock cargo clippy --locked --all-targets --target-dir target-codex-backends -- -D warnings";
const TEST_COMMAND: &str = "CARGO_BUILD_JOBS=1 flock /tmp/quire-heavy-check.lock cargo test --locked --target-dir target-codex-backends";
const DOC_COMMAND: &str = "CARGO_BUILD_JOBS=1 flock /tmp/quire-heavy-check.lock env RUSTDOCFLAGS=\"-D warnings\" cargo doc --locked --no-deps --target-dir target-codex-backends";
const ADMITTED_TARGET: &str = "x86_64-unknown-linux-gnu";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum GateDefect {
    CommandSet,
    CommandOrder,
    MissingRisk,
    Concurrency,
    WarningChannel,
    MissingAllTargets,
    UndeclaredFeature,
    MissingFeatureLane,
    PhantomFeature,
    UnselectedCondition,
    SharedTarget,
    ChangedHead,
    ChangedTrackedState,
}

impl GateDefect {
    const ALL: [Self; 13] = [
        Self::CommandSet,
        Self::CommandOrder,
        Self::MissingRisk,
        Self::Concurrency,
        Self::WarningChannel,
        Self::MissingAllTargets,
        Self::UndeclaredFeature,
        Self::MissingFeatureLane,
        Self::PhantomFeature,
        Self::UnselectedCondition,
        Self::SharedTarget,
        Self::ChangedHead,
        Self::ChangedTrackedState,
    ];

    const fn code(self) -> &'static str {
        match self {
            Self::CommandSet => "gate-command-set",
            Self::CommandOrder => "gate-command-order",
            Self::MissingRisk => "gate-risk-unbound",
            Self::Concurrency => "gate-not-serialized",
            Self::WarningChannel => "gate-warning-channel-missing",
            Self::MissingAllTargets => "gate-all-targets-missing",
            Self::UndeclaredFeature => "gate-feature-undeclared",
            Self::MissingFeatureLane => "gate-feature-lane-missing",
            Self::PhantomFeature => "gate-feature-phantom",
            Self::UnselectedCondition => "gate-condition-unselected",
            Self::SharedTarget => "gate-target-directory-shared",
            Self::ChangedHead => "gate-head-changed",
            Self::ChangedTrackedState => "gate-tracked-state-changed",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CommandKind {
    Format,
    Clippy,
    Test,
    Doc,
}

#[derive(Clone, Debug)]
struct Command {
    kind: CommandKind,
    text: String,
    risk_bound: bool,
    build_jobs: u8,
    lock: String,
    all_targets: bool,
    deny_warnings: bool,
    target_directory: Option<String>,
    requested_features: BTreeSet<String>,
}

impl Command {
    fn from_documented_line(text: &str, agents: &str) -> Self {
        let kind = if text.contains("cargo fmt") {
            CommandKind::Format
        } else if text.contains("cargo clippy") {
            CommandKind::Clippy
        } else if text.contains("cargo test") {
            CommandKind::Test
        } else if text.contains("cargo doc") {
            CommandKind::Doc
        } else {
            panic!("the gate block contains an unknown command")
        };
        let risk_bound = match kind {
            CommandKind::Format => agents.contains("formatting command detects formatting drift"),
            CommandKind::Clippy => agents.contains("Clippy command detects\n  lint violations"),
            CommandKind::Test => agents.contains("test command detects behavioural regressions"),
            CommandKind::Doc => agents.contains("doc command detects undocumented public API"),
        };
        let build_jobs = if text.starts_with("CARGO_BUILD_JOBS=1 ") {
            1
        } else {
            0
        };
        let lock = if text.contains("flock /tmp/quire-heavy-check.lock ") {
            "/tmp/quire-heavy-check.lock"
        } else {
            ""
        };
        let target_directory = text
            .split_once("--target-dir ")
            .and_then(|(_, tail)| tail.split_whitespace().next())
            .map(str::to_owned);
        Self {
            kind,
            text: text.to_owned(),
            risk_bound,
            build_jobs,
            lock: lock.to_owned(),
            all_targets: text.contains("--all-targets"),
            deny_warnings: text.ends_with("-D warnings"),
            target_directory,
            requested_features: requested_features(text),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Condition {
    Test { path: String },
    Target { path: String, predicate: String },
    Feature { path: String, name: String },
}

#[derive(Clone, Debug)]
struct GatePolicy {
    commands: Vec<Command>,
    rust_lints: bool,
    clippy_lints: bool,
    declared_features: BTreeSet<String>,
    optional_dependency: bool,
    feature_lanes: Vec<BTreeSet<String>>,
    conditions: Vec<Condition>,
    admitted_target: String,
    first_head: String,
    final_head: String,
    first_state: String,
    final_state: String,
}

impl GatePolicy {
    fn repository() -> Self {
        let agents = include_str!("../AGENTS.md");
        let cargo = include_str!("../Cargo.toml");
        let commands = agents
            .lines()
            .map(str::trim)
            .filter(|line| line.starts_with("CARGO_BUILD_JOBS=1 flock "))
            .map(|line| line.trim_end_matches('`'))
            .map(|line| Command::from_documented_line(line, agents))
            .collect();

        Self {
            commands,
            rust_lints: cargo.contains("[lints.rust]")
                && cargo.contains("unsafe_code = \"forbid\""),
            clippy_lints: cargo.contains("[lints.clippy]") && cargo.contains("all = \"deny\""),
            declared_features: manifest_features(cargo),
            optional_dependency: cargo.lines().any(|line| {
                line.chars()
                    .filter(|character| !character.is_whitespace())
                    .collect::<String>()
                    .contains("optional=true")
            }),
            feature_lanes: vec![BTreeSet::new()],
            conditions: conditional_inventory(Path::new(env!("CARGO_MANIFEST_DIR"))),
            admitted_target: ADMITTED_TARGET.to_owned(),
            first_head: "candidate-head".to_owned(),
            final_head: "candidate-head".to_owned(),
            first_state: "clean-tracked-state".to_owned(),
            final_state: "clean-tracked-state".to_owned(),
        }
    }

    fn defects(&self) -> BTreeSet<GateDefect> {
        let mut defects = BTreeSet::new();
        let expected_kinds = [
            CommandKind::Format,
            CommandKind::Clippy,
            CommandKind::Test,
            CommandKind::Doc,
        ];
        let actual_kinds: Vec<_> = self.commands.iter().map(|command| command.kind).collect();
        let expected_text = [FORMAT_COMMAND, CLIPPY_COMMAND, TEST_COMMAND, DOC_COMMAND];

        if actual_kinds.len() != expected_kinds.len()
            || actual_kinds
                .iter()
                .any(|kind| !expected_kinds.contains(kind))
            || self
                .commands
                .iter()
                .any(|command| !expected_text.contains(&command.text.as_str()))
        {
            defects.insert(GateDefect::CommandSet);
        } else if actual_kinds != expected_kinds {
            defects.insert(GateDefect::CommandOrder);
        }
        if self.commands.iter().any(|command| !command.risk_bound) {
            defects.insert(GateDefect::MissingRisk);
        }
        if self
            .commands
            .iter()
            .any(|command| command.build_jobs != 1 || command.lock != "/tmp/quire-heavy-check.lock")
        {
            defects.insert(GateDefect::Concurrency);
        }

        let clippy = self.command(CommandKind::Clippy);
        if !self.rust_lints
            || !self.clippy_lints
            || clippy.is_none_or(|command| !command.deny_warnings)
        {
            defects.insert(GateDefect::WarningChannel);
        }
        if clippy.is_none_or(|command| !command.all_targets) {
            defects.insert(GateDefect::MissingAllTargets);
        }
        if self.commands.iter().any(|command| {
            command
                .requested_features
                .iter()
                .any(|feature| !self.declared_features.contains(feature))
        }) {
            defects.insert(GateDefect::UndeclaredFeature);
        }

        let referenced_features: BTreeSet<&str> = self
            .conditions
            .iter()
            .filter_map(|condition| match condition {
                Condition::Feature { name, .. } => Some(name.as_str()),
                Condition::Test { .. } | Condition::Target { .. } => None,
            })
            .collect();
        if referenced_features
            .iter()
            .any(|feature| !self.declared_features.contains(*feature))
        {
            defects.insert(GateDefect::UndeclaredFeature);
        }
        if self
            .declared_features
            .iter()
            .any(|feature| !referenced_features.contains(feature.as_str()))
        {
            defects.insert(GateDefect::PhantomFeature);
        }

        let has_feature_surface = !self.declared_features.is_empty()
            || self.optional_dependency
            || !referenced_features.is_empty();
        let has_minimal_lane = self.feature_lanes.iter().any(BTreeSet::is_empty);
        let has_opposing_lane = self.feature_lanes.len() >= 2
            && self.feature_lanes.iter().any(|lane| !lane.is_empty())
            && self
                .declared_features
                .iter()
                .all(|feature| self.feature_lanes.iter().any(|lane| lane.contains(feature)));
        if (has_feature_surface && (!has_minimal_lane || !has_opposing_lane))
            || (!has_feature_surface
                && (self.feature_lanes.len() != 1 || !self.feature_lanes[0].is_empty()))
        {
            defects.insert(GateDefect::MissingFeatureLane);
        }

        let test_selected = self.command(CommandKind::Test).is_some();
        for condition in &self.conditions {
            let selected = match condition {
                Condition::Test { .. } => test_selected,
                Condition::Target { predicate, .. } => {
                    predicate == "unix" && self.admitted_target == ADMITTED_TARGET
                }
                Condition::Feature { .. } => true,
            };
            if !selected {
                defects.insert(GateDefect::UnselectedCondition);
            }
        }

        if self.commands.iter().any(|command| {
            matches!(
                command.kind,
                CommandKind::Clippy | CommandKind::Test | CommandKind::Doc
            ) && command.target_directory.as_deref() != Some("target-codex-backends")
        }) {
            defects.insert(GateDefect::SharedTarget);
        }
        if self.first_head != self.final_head {
            defects.insert(GateDefect::ChangedHead);
        }
        if self.first_state != self.final_state {
            defects.insert(GateDefect::ChangedTrackedState);
        }
        defects
    }

    fn command(&self, kind: CommandKind) -> Option<&Command> {
        self.commands.iter().find(|command| command.kind == kind)
    }
}

fn requested_features(command: &str) -> BTreeSet<String> {
    command
        .split_once("--features ")
        .and_then(|(_, tail)| tail.split_whitespace().next())
        .into_iter()
        .flat_map(|features| features.split(','))
        .map(str::to_owned)
        .collect()
}

fn manifest_features(cargo: &str) -> BTreeSet<String> {
    let mut in_features = false;
    cargo
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.starts_with('[') {
                in_features = line == "[features]";
                return None;
            }
            if in_features && !line.is_empty() && !line.starts_with('#') {
                return line.split_once('=').map(|(name, _)| name.trim().to_owned());
            }
            None
        })
        .collect()
}

fn conditional_inventory(root: &Path) -> Vec<Condition> {
    let mut rust_files = Vec::new();
    collect_rust_files(&root.join("src"), &mut rust_files);
    collect_rust_files(&root.join("tests"), &mut rust_files);
    rust_files.sort();

    let attribute = ["#[", "cfg("].concat();
    let crate_attribute = ["#![", "cfg("].concat();
    let macro_call = ["cfg", "!("].concat();
    let mut conditions = Vec::new();
    for path in rust_files {
        let text = fs::read_to_string(&path).expect("tracked Rust source must be readable");
        let relative = path
            .strip_prefix(root)
            .expect("inventory path must remain below the repository root")
            .display()
            .to_string();
        for line in text.lines().map(str::trim) {
            if !(line.starts_with(&attribute)
                || line.starts_with(&crate_attribute)
                || line.contains(&macro_call))
            {
                continue;
            }
            if line.contains("feature =") {
                let name = line
                    .split('"')
                    .nth(1)
                    .expect("feature conditions must use one literal feature name");
                conditions.push(Condition::Feature {
                    path: relative.clone(),
                    name: name.to_owned(),
                });
            } else if line.contains("cfg(test)") {
                conditions.push(Condition::Test {
                    path: relative.clone(),
                });
            } else if line.contains("cfg(unix)") {
                conditions.push(Condition::Target {
                    path: relative.clone(),
                    predicate: "unix".to_owned(),
                });
            } else {
                conditions.push(Condition::Target {
                    path: relative.clone(),
                    predicate: line.to_owned(),
                });
            }
        }
    }
    conditions
}

fn collect_rust_files(directory: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(directory).expect("source directory must be readable") {
        let path = entry.expect("source entry must be readable").path();
        if path.is_dir() {
            collect_rust_files(&path, files);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
}

fn one_defect(mut policy: GatePolicy, mutate: impl FnOnce(&mut GatePolicy)) -> GateDefect {
    mutate(&mut policy);
    let defects = policy.defects();
    assert_eq!(
        defects.len(),
        1,
        "negative control must isolate one policy axis"
    );
    *defects.iter().next().expect("one defect was asserted")
}

#[test]
#[trace("NFR-004-AC-1", "NFR-004-AC-2", "Task-014")]
fn tc_012_the_current_command_set_and_risks_are_exact() {
    let policy = GatePolicy::repository();
    assert!(policy.defects().is_empty());
    assert_eq!(
        one_defect(policy.clone(), |candidate| {
            candidate.commands.remove(0);
        }),
        GateDefect::CommandSet
    );
    assert_eq!(
        one_defect(policy.clone(), |candidate| candidate.commands.swap(0, 1)),
        GateDefect::CommandOrder
    );
    assert_eq!(
        one_defect(policy, |candidate| candidate.commands[0].risk_bound = false),
        GateDefect::MissingRisk
    );
}

#[test]
#[trace("NFR-004-AC-3", "Task-014")]
fn tc_012_every_heavy_command_is_serialized() {
    let policy = GatePolicy::repository();
    assert_eq!(
        one_defect(policy.clone(), |candidate| candidate.commands[0]
            .build_jobs = 2),
        GateDefect::Concurrency
    );
    assert_eq!(
        one_defect(policy, |candidate| candidate.commands[2].lock.clear()),
        GateDefect::Concurrency
    );
}

#[test]
#[trace("NFR-004-AC-4", "Task-014")]
fn tc_012_both_warning_channels_are_independently_required() {
    let policy = GatePolicy::repository();
    assert_eq!(
        one_defect(policy.clone(), |candidate| candidate.rust_lints = false),
        GateDefect::WarningChannel
    );
    assert_eq!(
        one_defect(policy.clone(), |candidate| candidate.clippy_lints = false),
        GateDefect::WarningChannel
    );
    assert_eq!(
        one_defect(policy, |candidate| {
            candidate.commands[1].deny_warnings = false;
        }),
        GateDefect::WarningChannel
    );
}

#[test]
#[trace("NFR-004-AC-5", "Task-014")]
fn tc_012_the_current_crate_has_one_cargo_feature_lane() {
    let policy = GatePolicy::repository();
    assert!(policy.declared_features.is_empty());
    assert!(!policy.optional_dependency);
    assert_eq!(policy.feature_lanes, [BTreeSet::new()]);
    assert!(policy.defects().is_empty());
}

#[test]
#[trace("NFR-004-AC-6", "Task-014")]
fn tc_012_an_undeclared_feature_is_its_own_defect() {
    let policy = GatePolicy::repository();
    assert_eq!(
        one_defect(policy, |candidate| {
            candidate.commands[2]
                .requested_features
                .insert("quire-extraction".to_owned());
        }),
        GateDefect::UndeclaredFeature
    );
}

#[test]
#[trace("NFR-004-AC-7", "Task-014")]
fn tc_012_a_new_feature_surface_requires_opposing_lanes() {
    let mut policy = GatePolicy::repository();
    policy.declared_features.insert("fast-path".to_owned());
    policy.conditions.push(Condition::Feature {
        path: "src/lib.rs".to_owned(),
        name: "fast-path".to_owned(),
    });
    assert_eq!(policy.defects(), [GateDefect::MissingFeatureLane].into());

    policy
        .feature_lanes
        .push(["fast-path".to_owned()].into_iter().collect());
    assert!(policy.defects().is_empty());

    let mut optional = GatePolicy::repository();
    optional.optional_dependency = true;
    optional.feature_lanes.push(BTreeSet::new());
    assert_eq!(
        optional.defects(),
        [GateDefect::MissingFeatureLane].into(),
        "two empty lanes do not exercise an optional dependency"
    );
    optional
        .feature_lanes
        .push(["implicit-dependency".to_owned()].into_iter().collect());
    assert!(optional.defects().is_empty());
}

#[test]
#[trace("NFR-004-AC-8", "Task-014")]
fn tc_012_every_conditional_site_is_selected_and_phantom_features_refuse() {
    let policy = GatePolicy::repository();
    assert_eq!(
        policy.conditions,
        [
            Condition::Test {
                path: "src/assessment.rs".to_owned(),
            },
            Condition::Test {
                path: "src/claims/dispatch.rs".to_owned(),
            },
            Condition::Test {
                path: "src/claims/mod.rs".to_owned(),
            },
            Condition::Test {
                path: "src/conformance/kernel.rs".to_owned(),
            },
            Condition::Test {
                path: "src/conformance/mod.rs".to_owned(),
            },
            Condition::Test {
                path: "src/link/interleaving.rs".to_owned(),
            },
            Condition::Test {
                path: "src/link/mod.rs".to_owned(),
            },
            Condition::Test {
                path: "src/link/mod.rs".to_owned(),
            },
            Condition::Test {
                path: "src/producer_contracts.rs".to_owned(),
            },
            Condition::Test {
                path: "src/projection.rs".to_owned(),
            },
            Condition::Test {
                path: "src/refusal/mod.rs".to_owned(),
            },
            Condition::Test {
                path: "src/result/mod.rs".to_owned(),
            },
            Condition::Test {
                path: "src/version_lock.rs".to_owned(),
            },
            Condition::Target {
                path: "tests/it001_v2_handoff.rs".to_owned(),
                predicate: "unix".to_owned(),
            },
        ]
    );
    assert!(policy.defects().is_empty());

    assert_eq!(
        one_defect(policy.clone(), |candidate| {
            candidate.commands[1].all_targets = false;
        }),
        GateDefect::MissingAllTargets
    );
    let mut unselected = policy.clone();
    unselected.conditions.push(Condition::Target {
        path: "tests/windows_only.rs".to_owned(),
        predicate: "windows".to_owned(),
    });
    assert_eq!(
        unselected.defects(),
        [GateDefect::UnselectedCondition].into()
    );

    let mut phantom = policy;
    phantom.declared_features.insert("phantom".to_owned());
    phantom
        .feature_lanes
        .push(["phantom".to_owned()].into_iter().collect());
    assert_eq!(phantom.defects(), [GateDefect::PhantomFeature].into());

    let mut undeclared_condition = GatePolicy::repository();
    undeclared_condition.conditions.push(Condition::Feature {
        path: "src/lib.rs".to_owned(),
        name: "ghost".to_owned(),
    });
    undeclared_condition
        .feature_lanes
        .push(["ghost".to_owned()].into_iter().collect());
    assert_eq!(
        undeclared_condition.defects(),
        [GateDefect::UndeclaredFeature].into()
    );
}

#[test]
#[trace("NFR-004-AC-9", "Task-014")]
fn tc_012_a_shared_or_foreign_target_directory_is_its_own_defect() {
    let policy = GatePolicy::repository();
    assert_eq!(
        one_defect(policy, |candidate| {
            candidate.commands[2].target_directory = Some("/tmp/shared-target".to_owned());
        }),
        GateDefect::SharedTarget
    );
}

#[test]
#[trace("NFR-004-AC-10", "Task-014")]
fn tc_012_a_moving_head_or_tracked_state_is_its_own_defect() {
    let policy = GatePolicy::repository();
    assert_eq!(
        one_defect(policy.clone(), |candidate| candidate.final_head =
            "different".to_owned()),
        GateDefect::ChangedHead
    );
    assert_eq!(
        one_defect(policy, |candidate| candidate.final_state =
            "different".to_owned()),
        GateDefect::ChangedTrackedState
    );
}

#[test]
#[trace(
    "NFR-004-AC-1",
    "NFR-004-AC-3",
    "NFR-004-AC-4",
    "NFR-004-AC-6",
    "NFR-004-AC-7",
    "NFR-004-AC-8",
    "NFR-004-AC-9",
    "NFR-004-AC-10",
    "Task-014"
)]
fn tc_012_each_gate_defect_has_one_stable_identity() {
    let identities: BTreeSet<_> = GateDefect::ALL.into_iter().map(GateDefect::code).collect();
    assert_eq!(identities.len(), GateDefect::ALL.len());
    assert!(identities
        .iter()
        .all(|identity| identity.starts_with("gate-")));
}
