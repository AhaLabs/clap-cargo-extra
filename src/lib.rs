//! Simple Wrapper around clap cargo that adds some utilities to the
//!
//! ```
//! pub struct ArgStruct {
//!   #[clap(flatten)]
//!   pub cargo: ClapCargo,
//! }
//! ```

use anyhow::{bail, Result};
use cargo_metadata::{camino::Utf8PathBuf, DependencyKind, Metadata, Package};
use clap_cargo::{Features, Manifest, Workspace};
use heck::ToShoutyKebabCase;
use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

mod cargo_bin;
pub use cargo_bin::*;

/// Combination of all three clap cargo's arg structs
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

    /// Compile release build (default is debug)
    #[clap(long)]
    pub release: bool,

    /// Add additional nightly features for optimizing
    #[clap(long)]
    pub optimize: bool,

    /// provide target, otherwise builds all targets
    #[clap(long)]
    pub target: Option<String>,

    #[clap(long)]
    pub link_args: bool,
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
                // println!("{:#?}", METADATA)
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
    pub fn add_cargo_args(&self, cmd: &mut Command) {
        if let Some(manifest_path) = &self.manifest.manifest_path {
            cmd.arg("--manifest-path");
            cmd.arg(manifest_path);
        }
        if self.features.no_default_features {
            cmd.arg("--no-default-features");
        }
        if self.features.all_features {
            cmd.arg("--all-features");
        } else {
            for feature in &self.features.features {
                cmd.arg("--features");
                cmd.arg(feature);
            }
        }
        for pack in &self.workspace.exclude {
            cmd.arg("--exclude");
            cmd.arg(pack);
        }
        if self.workspace.workspace || self.workspace.all {
            cmd.arg("--workspace");
        } else if !self.workspace.package.is_empty() {
            self.workspace.package.iter().for_each(|p| {
                cmd.arg("-p");
                cmd.arg(p);
            })
        }
    }

    pub fn get_deps(&self, p: &Package) -> Result<Vec<Utf8PathBuf>> {
        let packages = &self.metadata()?.packages;
        let res = p
            .dependencies
            .iter()
            .filter_map(|dep| {
                matches!(dep.kind, DependencyKind::Normal).then(|| {
                    packages
                        .iter()
                        .find(|p| p.name == dep.name)
                        .unwrap_or_else(|| panic!("could not find {}", dep.name))
                        .manifest_path
                        .clone()
                })
            })
            .collect();
        Ok(res)
    }

    /// Create a Command builder for cargo
    pub fn cargo_cmd(&self) -> Command {
        let mut cmd = Command::new(self.cargo_bin.bin());
        if cmd.get_program().eq_ignore_ascii_case("cargo") {
            cmd.arg(format!("+{}", self.channel()));
        }
        if self.link_args || self.optimize {
            cmd.env("RUSTFLAGS", "-C link-args=-s");
        }
        cmd
    }

    pub fn channel(&self) -> &str {
        if self.optimize {
            "nightly"
        } else {
            &self.cargo_bin.channel
        }
    }

    pub fn build_cmd(&self) -> Command {
        let mut cmd = self.cargo_cmd();
        cmd.arg("build");
        if let Some(target) = self.target.as_ref() {
            cmd.arg("--target");
            cmd.arg(target);
        } else {
            cmd.arg("--all-targets");
        }
        self.add_cargo_args(&mut cmd);
        if self.release {
            cmd.arg("--release");
        }
        if self.optimize {
            cmd.arg("-Z=build-std=std,panic_abort");
            cmd.arg("-Z=build-std-features=panic_immediate_abort");
        }
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
