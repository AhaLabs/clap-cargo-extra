use std::ffi::OsString;

use clap_cargo::{Features, Manifest, Workspace};

/// Trait for generating args. [`OsString`] is used since some args may be paths.
pub trait Args {
    fn to_args(&self) -> Vec<OsString>;
}

/// Trait for merging Arg structs. First struct takes precedence over second struct.
pub trait Merge {
    fn merge(&mut self, other: Self);
}

impl Args for Features {
    fn to_args(&self) -> Vec<OsString> {
        let mut args: Vec<OsString> = Vec::with_capacity(4);
        if self.all_features {
            args.push("--all-features".into());
        }
        if self.no_default_features {
            args.push("--no-default-features".into());
        }
        if !self.features.is_empty() {
            args.push("--features".into());
            args.push(self.features.join(" ").into())
        }
        args
    }
}

impl Merge for Features {
    fn merge(&mut self, other: Self) {
        let Features {
            all_features,
            no_default_features,
            mut features,
            ..
        } = other;
        self.all_features = self.all_features || all_features;
        self.no_default_features = self.no_default_features || no_default_features;
        self.features.append(&mut features);
    }
}

impl Args for Manifest {
    fn to_args(&self) -> Vec<OsString> {
        let mut args: Vec<OsString> = Vec::with_capacity(4);
        if let Some(manifest_path) = self.manifest_path.as_ref() {
            args.push("--manifest-path".into());
            args.push(manifest_path.clone().into());
        }
        args
    }
}

impl Merge for Manifest {
    fn merge(&mut self, other: Self) {
        if self.manifest_path.is_none() {
            self.manifest_path = other.manifest_path;
        }
    }
}

impl Args for Workspace {
    fn to_args(&self) -> Vec<OsString> {
        let mut args: Vec<OsString> = Vec::with_capacity(4);
        if self.workspace || self.all {
            args.push("--workspace".into());
        }
        for package in &self.package {
            args.push("--package".into());
            args.push(package.into());
        }
        for exclude in &self.exclude {
            args.push("--exclude".into());
            args.push(exclude.into());
        }
        args
    }
}

impl Merge for Workspace {
    fn merge(&mut self, other: Self) {
        let Workspace {
            mut package,
            workspace,
            all,
            mut exclude,
            ..
        } = other;
        self.package.append(&mut package);
        self.workspace = self.workspace || workspace;
        self.all = self.all || all;
        self.exclude.append(&mut exclude);
    }
}

// impl IntoIterator for Workspace {
//     type Item = String;
//     type IntoIter = std::vec::IntoIter<Self::Item>;

//     fn into_iter(self) -> Self::IntoIter {
//         self.to_args().into_iter()
//     }
// }
