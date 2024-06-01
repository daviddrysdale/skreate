//! Integration test to compare against canned output.

use log::{debug, info};
use std::fs;
use std::io::Write;

#[test]
fn test_compare() {
    let _ = env_logger::try_init();
    let testdir = concat!(env!("CARGO_MANIFEST_DIR"), "/testdata").to_string();
    let testdir = std::path::PathBuf::from(testdir);
    for name in skreate::moves::ids() {
        let path = testdir.join(format!("{name}.svg"));
        info!("process '{name}' for {path:?}");
        let got = skreate::generate(name).expect("failed to generate SVG");
        if regenerate() {
            info!("regenerate '{path:?}'");
            let mut outfile = fs::File::create(path).unwrap();
            outfile.write_all(&got.into_bytes()).unwrap();
        } else {
            debug!("compare output with '{path:?}'");
            let want = fs::read_to_string(path).unwrap();
            assert_eq!(got.trim(), want.trim(), "for '{name}'");
        }
    }
}

fn regenerate() -> bool {
    !std::env::var("SKREATE_REGENERATE")
        .unwrap_or_default()
        .is_empty()
}
