//! Integration test to compare against canned output.

use log::{debug, info};
use std::fs;
use std::io::Write;

#[test]
fn test_compare() {
    let _ = env_logger::try_init();
    let doc_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../doc/generated").to_string();
    let doc_dir = std::path::PathBuf::from(doc_dir);
    for info in skreate::moves::info() {
        let name = info.name;
        let input = info.example;
        let path = doc_dir.join(format!("{name}.svg"));
        info!("process '{input}' for {path:?}");
        let got = skreate::generate(input).expect("failed to generate SVG");
        if regenerate() {
            info!("regenerate '{path:?}'");
            let mut outfile = fs::File::create(path).unwrap();
            outfile.write_all(&got.into_bytes()).unwrap();
        } else {
            debug!("compare output with '{path:?}'");
            let want = fs::read_to_string(path)
                .unwrap_or_else(|_e| panic!("failed to find file for {name}"));
            assert_eq!(got.trim(), want.trim(), "for '{name}'");
        }
    }
}

fn regenerate() -> bool {
    !std::env::var("SKREATE_REGENERATE")
        .unwrap_or_default()
        .is_empty()
}
