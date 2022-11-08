//! Cargo flags for selecting crates in a workspace.

use std::ffi::OsString;

use crate::{impls::Merge, Args};

#[derive(Default, Clone, Debug, PartialEq, Eq, clap::Args)]
#[non_exhaustive]
pub struct CargoBuild {
    /// Add additional nightly features for optimizing
    #[clap(long)]
    pub optimize: bool,

    /// Build for the target triple
    #[clap(long, name = "TRIPE")]
    pub target: Option<String>,

    /// Build all targets
    #[clap(long)]
    pub all_targets: bool,

    #[clap(long)]
    pub link_args: bool,

    /// Build artifacts in release mode, with optimizations
    #[clap(long, short = 'r')]
    pub release: bool,

    /// Build artifacts with the specified profile
    #[clap(long, name = "PROFILE_NAME")]
    pub profile: Option<String>,
}

impl CargoBuild {
    pub fn profile(&self) -> &str {
        self.profile
            .as_deref()
            .unwrap_or_else(|| self.release_or_debug())
    }

    fn release_or_debug(&self) -> &str {
        if self.release {
            "release"
        } else {
            "debug"
        }
    }
}

impl Merge for CargoBuild {
    fn merge(&mut self, other: CargoBuild) {
        let CargoBuild {
            optimize,
            target,
            all_targets,
            link_args,
            release,
            profile,
        } = other;
        self.optimize = self.optimize || optimize;
        if self.target.is_none() {
            self.target = target;
        }
        self.all_targets = self.all_targets || all_targets;
        self.link_args = self.link_args || link_args;
        self.release = self.release || release;
        if self.profile.is_none() {
            self.profile = profile;
        }
    }
}

impl Args for CargoBuild {
    fn to_args(&self) -> Vec<OsString> {
        let mut args: Vec<OsString> = Vec::new();
        if self.optimize {
            args.push("-Z=build-std=std,panic_abort".into());
            args.push("-Z=build-std-features=panic_immediate_abort".into());
        }
        if let Some(target) = &self.target {
            args.push("--target".into());
            args.push(target.into());
        }
        if self.all_targets {
            args.push("--all-targets".into());
        }
        if self.release {
            args.push("--release".into());
        }
        if let Some(profile) = &self.profile {
            args.push("--profile".into());
            args.push(profile.into());
        }
        args
    }
}
