// use assert_cmd::prelude::*;
// use assert_fs::TempDir;
use clap::Parser;
use clap_cargo_extra::{impls::Merge, CargoBuild, ClapCargo};
use std::ffi::OsString;

#[derive(Debug, Parser)]
struct Cli {
    #[clap(flatten)]
    clap_cargo: ClapCargo,
}

fn try_parse_from<I, T>(itr: I) -> Result<ClapCargo, Box<dyn std::error::Error>>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    Ok(Cli::try_parse_from(itr)?.clap_cargo)
}

fn custom() -> Option<String> {
    Some("custom".to_string())
}

const WASM32_UNKNOWN_UNKNOWN: &str = "wasm32-unknown-unknown";

#[test]
fn build_release() -> Result<(), Box<dyn std::error::Error>> {
    let args = try_parse_from(&["", "--release"])?;
    assert_eq!(args.cargo_build.release, true);
    Ok(())
}

#[test]
fn build_profile() -> Result<(), Box<dyn std::error::Error>> {
    let args = try_parse_from(&["", "--profile", "custom"])?;
    let custom = custom();
    assert_eq!(args.cargo_build.profile, custom);
    let mut build = CargoBuild::default();
    build.profile = custom;
    assert_eq!(args.cargo_build, build);
    Ok(())
}

#[test]
fn build_profile_and_release() -> Result<(), Box<dyn std::error::Error>> {
    let args = try_parse_from(&["", "--profile", "custom", "--release"])?;
    let custom = custom();
    assert_eq!(args.cargo_build.release, true);
    assert_eq!(args.cargo_build.profile, custom);
    let mut build = CargoBuild::default();
    build.profile = custom;
    build.release = true;
    assert_eq!(args.cargo_build, build);
    Ok(())
}

#[test]
fn build_profile_and_release_merge() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = try_parse_from(&["", "--profile", "custom"])?;
    let args2 = try_parse_from(&["", "--release"])?;
    args.merge(args2);
    let custom = custom();
    assert_eq!(args.cargo_build.release, true);
    assert_eq!(args.cargo_build.profile, custom);
    let mut build = CargoBuild::default();
    build.profile = custom;
    build.release = true;
    assert_eq!(args.cargo_build, build);
    Ok(())
}

#[test]
fn build_target() -> Result<(), Box<dyn std::error::Error>> {
    let args = try_parse_from(&["", "--target", WASM32_UNKNOWN_UNKNOWN])?;
    let target = Some(WASM32_UNKNOWN_UNKNOWN.to_string());
    assert_eq!(args.cargo_build.target, target);
    let mut build = CargoBuild::default();
    build.target = target;
    assert_eq!(args.cargo_build, build);
    Ok(())
}

#[test]
fn slop() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = vec!["", "--"];
    let slop = vec!["--flag", "--extra_arg", "test"];
    args.extend(slop.iter());
    let args = try_parse_from(&args)?;
    assert_eq!(args.slop, slop);
    let mut clap_cargo = ClapCargo::default();
    clap_cargo.slop = slop.iter().map(Into::into).collect();
    assert_eq!(args, clap_cargo);
    Ok(())
}
