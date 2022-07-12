//! Simple Wrapper around clap cargo that adds some utilities to the
//!
//! ```
//! pub struct ArgStruct {
//!   #[clap(flatten)]
//!   pub cargo: ClapCargo,
//! }
//! ```

use anyhow::Result;
use cargo_metadata::{camino::Utf8PathBuf, Dependency, DependencyKind, Metadata, Package};
use clap_cargo::{Features, Manifest, Workspace};
use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Default, Clone, Debug, PartialEq, Eq, clap::Args)]
#[non_exhaustive]
pub struct ClapCargo {
    #[clap(flatten)]
    pub features: Features,

    #[clap(flatten)]
    pub manifest: Manifest,

    #[clap(flatten)]
    pub workspace: Workspace,
}

impl ClapCargo {
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

    pub fn target_dir(&self) -> Result<PathBuf> {
        Ok(self.metadata()?.target_directory.clone().into())
    }

    pub fn packages(&self) -> Result<Vec<&Package>> {
        let meta = self.metadata()?;
        Ok(self.workspace.partition_packages(meta).0)
    }

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
        Ok(p.dependencies
            .iter()
            .filter(|dep| matches!(dep.kind, DependencyKind::Normal))
            .map(|dep| {
                self.metadata()
                    .unwrap()
                    .clone()
                    .packages
                    .into_iter()
                    .find(|p| p.name == dep.name)
                    .expect(&format!("could find {}", dep.name))
                    .manifest_path
            })
            .collect())
    }

    pub fn cargo_cmd() -> Command {
        Command::new(cargo())
    }
}

fn cargo() -> String {
    env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned())
}

#[derive(Debug)]
pub enum DepKind {
    Path(Utf8PathBuf),
    Source(String),
    Registry(String),
}

pub struct Dep {
    pub name: String,
    kind: DepKind,
}

impl Dep {
    pub fn new(dep: &Dependency) -> Self {
        println!("{:?}, {:?}, {:?}", dep.path, dep.source, dep.registry);
        let kind = if let Some(path) = dep.path.as_ref() {
            DepKind::Path(path.clone())
        } else if let Some(source) = dep.source.as_ref() {
            DepKind::Source(source.clone())
        } else {
            DepKind::Registry(dep.registry.as_ref().unwrap().clone())
        };
        let name = dep.name.clone();
        Self { kind, name }
    }

    pub fn resolve(&self) -> Result<String> {
        println!("{:?}", self.kind);
        Ok(String::new())
    }
}
