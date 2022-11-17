use clap_cargo_extra::ClapCargo;
use std::path::PathBuf;

#[test]
fn multi_level_deps() {
    let mut c = ClapCargo::default();
    // let mp = PathBuf::from("./tests/fixtures/two-dep/Cargo.toml");
    let mp = PathBuf::from("./tests/fixtures/Cargo.toml");
    c.manifest.manifest_path = Some(mp);
    let m = c.metadata().unwrap();
    for p in m.workspace_packages() {
        let expected = match p.name.as_str() {
            "zero-dep" => 0,
            "single-dep" => 1,
            "double-dep" => 2,
            "triple-dep" => 3,
            _ => panic!(),
        };
        let deps = c
            .get_deps(p, cargo_metadata::DependencyKind::Normal)
            .unwrap();
        assert_eq!(deps.len(), expected);
    }
}
