//! Integration test to compare against canned output.

use log::{debug, info, warn};
use std::fs;
use std::io::Write;

#[test]
fn test_compare() {
    let _ = env_logger::try_init();
    let testdir = concat!(env!("CARGO_MANIFEST_DIR"), "/testdata").to_string();
    for entry in fs::read_dir(testdir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if !path.is_file() {
            warn!("skipping non-file {path:?}");
        }
        let filename = entry.file_name().into_string().unwrap();
        match filename.split_once('.') {
            Some((name, "svg")) | Some((name, "SVG")) => {
                info!("process '{name}'");
                let got = skreate::generate(&name).expect("failed to generate SVG");
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
            _ => {
                warn!("skipping non-SVG-file {filename}");
                continue;
            }
        }
    }
}

fn regenerate() -> bool {
    !std::env::var("SKREATE_REGENERATE")
        .unwrap_or_default()
        .is_empty()
}
