//! Integration test to compare against canned output.

use log::{debug, info};
use std::ffi::OsStr;
use std::fs::{self, read_dir, File};
use std::io::{Read, Write};

#[test]
fn test_compare() {
    let _ = env_logger::try_init();
    let doc_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../doc/generated").to_string();
    let doc_dir = std::path::PathBuf::from(doc_dir);
    for info in skreate::moves::INFO {
        let name = info.name;
        let input = info.example;
        let path = doc_dir.join(format!("{input}.svg"));
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

#[test]
fn test_parse_success() {
    let _ = env_logger::try_init();
    let extension = OsStr::new("skate");
    let eg_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/testdata/pass").to_string();
    let eg_path = std::path::PathBuf::from(eg_dir);

    for entry in read_dir(eg_path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if !path.is_file() || path.extension() != Some(extension) {
            continue;
        }
        let mut reader = File::open(path.clone()).unwrap();
        let mut input = String::new();
        reader.read_to_string(&mut input).unwrap();
        let result = skreate::generate(&input);
        assert!(result.is_ok());
        info!("file '{path:?}' parsed successfully");
    }
}

fn regenerate() -> bool {
    !std::env::var("SKREATE_REGENERATE")
        .unwrap_or_default()
        .is_empty()
}
