//! Cargo flags for selecting crates in a wasm.

use std::env;

#[derive(Default, Clone, Debug, PartialEq, Eq, clap::Args)]
#[non_exhaustive]
pub struct CargoBin {
    /// stable, beta, nightly, and custom
    #[clap(long, default_value = "stable")]
    pub channel: String,
}

impl CargoBin {
    pub fn bin(&self) -> String {
        env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned())
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
