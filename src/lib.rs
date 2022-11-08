//! Simple Wrapper around clap cargo that adds some utilities to access the metadata
//!
//! ```ignore
//! # use clap::Parser;
//! # use crate::ClapCargo;
//!
//! pub struct ArgStruct {
//!   #[clap(flatten)]
//!   pub cargo: ClapCargo,
//! }
//! ```

use anyhow::{bail, Result};
use cargo_metadata::{camino::Utf8PathBuf, DependencyKind, Metadata, Package};
use clap_cargo::{Features, Manifest, Workspace};
use heck::ToShoutyKebabCase;
use impls::Merge;
use std::{
    env,
    ffi::OsString,
    path::{Path, PathBuf},
    process::Command,
};

mod cargo_bin;
mod cargo_build;
#[cfg(feature = "std")]
mod cmd;
pub mod impls;

pub use cargo_bin::*;
pub use cargo_build::*;
pub use impls::*;

#[cfg(feature = "std")]
pub use cmd::ToCmd;

/// Combination of all three clap cargo's arg structs and two new ones,
/// [`CargoBuild`] and [`CargoBin`].
#[derive(Default, Clone, Debug, PartialEq, Eq, clap::Args)]
#[non_exhaustive]
pub struct ClapCargo {
    #[clap(flatten)]
    pub features: Features,

    #[clap(flatten)]
    pub manifest: Manifest,

    #[clap(flatten)]
    pub workspace: Workspace,

    #[clap(flatten)]
    pub cargo_bin: CargoBin,

    #[clap(flatten)]
    pub cargo_build: CargoBuild,

    /// Extra arguments passed to cargo after `--`
    #[clap(last = true, name = "CARGO_ARGS")]
    pub slop: Vec<OsString>,
}

impl ClapCargo {
    /// Current metadata for the CLI's context
    pub fn metadata(&self) -> Result<&Metadata> {
        unsafe {
            static mut METADATA: Option<Metadata> = None;
            if METADATA.is_none() {
                let mut metadata_cmd = self.manifest.metadata();
                self.features.forward_metadata(&mut metadata_cmd);
                METADATA = Some(metadata_cmd.exec()?);
            }
            Ok(METADATA.as_ref().unwrap())
        }
    }

    /// Current manifest path in context
    pub fn manifest_path(&self) -> Result<PathBuf> {
        let manifest_path = self
            .manifest
            .manifest_path
            .clone()
            .unwrap_or_else(|| Path::new("./Cargo.toml").to_path_buf());
        Ok(if manifest_path.is_relative() {
            env::current_dir()?.join(manifest_path)
        } else {
            manifest_path
        })
    }

    /// Directory where build artifacts will go
    pub fn target_dir(&self) -> Result<PathBuf> {
        Ok(self.metadata()?.target_directory.clone().into())
    }

    /// Get the current packages that are selected by CLI
    pub fn current_packages(&self) -> Result<Vec<&Package>> {
        let meta = self.metadata()?;
        Ok(self.workspace.partition_packages(meta).0)
    }

    /// All packages referenced
    pub fn packages(&self) -> Result<Vec<&Package>> {
        Ok(self.metadata()?.packages.iter().collect::<Vec<&Package>>())
    }

    /// Add the correct CLI flags to a command
    #[deprecated(note = "use add_args_to_cmd instead")]
    pub fn add_cargo_args(&self, cmd: &mut Command) {
        self.add_args_to_cmd(cmd);
    }

    pub fn get_deps(&self, p: &Package) -> Result<Vec<Utf8PathBuf>> {
        let packages = &self.metadata()?.packages;
        p.dependencies
            .iter()
            .filter(|dep| matches!(dep.kind, DependencyKind::Normal))
            .map(|dep| {
                packages.iter().find(|p| p.name == dep.name).map_or_else(
                    || bail!("could not find {}", dep.name),
                    |p| Ok(p.manifest_path.clone()),
                )
            })
            .collect()
    }

    /// Create a Command builder for cargo
    pub fn cargo_cmd(&self) -> Command {
        let mut cmd = Command::new(self.cargo_bin.bin());
        if cmd.get_program().eq_ignore_ascii_case("cargo") {
            cmd.arg(format!("+{}", self.channel()));
        }
        if self.cargo_build.link_args || self.cargo_build.optimize {
            cmd.env("RUSTFLAGS", "-C link-args=-s");
        }
        cmd
    }

    pub fn channel(&self) -> &str {
        if self.cargo_build.optimize {
            "nightly"
        } else {
            self.cargo_bin.channel()
        }
    }

    #[cfg(feature = "std")]
    pub fn build_cmd(&self) -> Command {
        let mut cmd = self.cargo_cmd();
        self.add_args_to_cmd(cmd.arg("build"));
        cmd
    }

    /// Find package given a name
    pub fn find_package(&self, name: &str) -> Result<Option<&Package>> {
        let mut found_close_pair: Option<&str> = None;
        let package = self.packages()?.into_iter().find(|p| {
            let res = p.name == name;
            if !res && p.name.to_shouty_kebab_case() == name.to_shouty_kebab_case() {
                found_close_pair = Some(&p.name);
            };
            res
        });

        if let (Some(similar_package), None) = (found_close_pair, package) {
            bail!("Found similar package for {name} ~ {similar_package}");
        }

        Ok(package)
    }
}

impl Merge for ClapCargo {
    fn merge(&mut self, other: Self) {
        let Self {
            features,
            manifest,
            workspace,
            cargo_bin,
            cargo_build: build,
            mut slop,
        } = other;
        self.features.merge(features);
        self.manifest.merge(manifest);
        self.workspace.merge(workspace);
        self.cargo_bin.merge(cargo_bin);
        self.cargo_build.merge(build);
        self.slop.append(&mut slop);
    }
}

impl Args for ClapCargo {
    fn to_args(&self) -> Vec<OsString> {
        let mut args = self.workspace.to_args();
        args.extend(self.features.to_args());
        // Can skip non-cargo args
        // args.extend(self.cargo_bin.to_args());
        args.extend(self.cargo_build.to_args());
        args.extend(self.manifest.to_args());
        args
    }
}
