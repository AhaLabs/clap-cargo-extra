//! Cargo flags for selecting crates in a wasm.

use std::{env, ffi::OsString};

use crate::{impls::Merge, Args};

#[derive(Default, Clone, Debug, PartialEq, Eq, clap::Args)]
#[non_exhaustive]
pub struct CargoBin {
    /// stable, beta, nightly, and custom
    /// default: stable
    #[clap(long)]
    channel: Option<String>,
}

impl CargoBin {
    pub fn bin(&self) -> String {
        env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned())
    }

    pub fn channel(&self) -> &str {
        self.channel.as_deref().unwrap_or("stable")
    }
}

impl Merge for CargoBin {
    fn merge(&mut self, other: Self) {
        if self.channel.is_none() {
            self.channel = other.channel;
        }
    }
}

impl Args for CargoBin {
    fn to_args(&self) -> Vec<OsString> {
        let mut args: Vec<OsString> = Vec::with_capacity(2);
        if let Some(channel) = self.channel.as_ref() {
            args.push("--channel".into());
            args.push(channel.into());
        }
        args
    }
}
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn verify_app() {
        #[derive(Debug, clap::StructOpt)]
        struct Cli {
            #[clap(flatten)]
            toolchain: CargoBin,
        }

        use clap::CommandFactory;
        Cli::command().debug_assert()
    }
}
