use clap::Parser;
use clap_cargo_extra::{Args, ClapCargo};

#[derive(Debug, Parser)]
struct Cli {
    #[clap(flatten)]
    clap_cargo: ClapCargo,
}

fn main() {
    let args = Cli::parse();
    println!("args = {:#?}", args);
    print!("{:?}", args.clap_cargo.to_args())
}
