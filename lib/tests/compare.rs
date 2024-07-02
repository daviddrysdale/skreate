//! Integration test to compare against canned output.

use log::{debug, info};
use std::collections::HashSet;
use std::fs;
use std::io::Write;

/// List of move names that have canned SVG files.
fn movelist() -> HashSet<&'static str> {
    let mut reg = HashSet::new();
    reg.insert("BB");
    reg.insert("BF");
    reg.insert("LB");
    reg.insert("LBI-Rk");
    reg.insert("LBI");
    reg.insert("LBI3");
    reg.insert("LBO-Rk");
    reg.insert("LBO");
    reg.insert("LBO3");
    reg.insert("LF");
    reg.insert("LFI-Rk");
    reg.insert("LFI");
    reg.insert("LFI3");
    reg.insert("LFO-Rk");
    reg.insert("LFO");
    reg.insert("LFO3");
    reg.insert("RB");
    reg.insert("RBI-Rk");
    reg.insert("RBI");
    reg.insert("RBI3");
    reg.insert("RBO-Rk");
    reg.insert("RBO");
    reg.insert("RBO3");
    reg.insert("RF");
    reg.insert("RFI-Rk");
    reg.insert("RFI");
    reg.insert("RFI3");
    reg.insert("RFO-Rk");
    reg.insert("RFO");
    reg.insert("RFO3");
    reg.insert("xb-LBI-Rk");
    reg.insert("xb-LBI");
    reg.insert("xb-LBI3");
    reg.insert("xb-LBO-Rk");
    reg.insert("xb-LBO");
    reg.insert("xb-LBO3");
    reg.insert("xb-RBI-Rk");
    reg.insert("xb-RBI");
    reg.insert("xb-RBI3");
    reg.insert("xb-RBO-Rk");
    reg.insert("xb-RBO");
    reg.insert("xb-RBO3");
    reg.insert("xf-LFI-Rk");
    reg.insert("xf-LFI");
    reg.insert("xf-LFI3");
    reg.insert("xf-LFO-Rk");
    reg.insert("xf-LFO");
    reg.insert("xf-LFO3");
    reg.insert("xf-RFI-Rk");
    reg.insert("xf-RFI");
    reg.insert("xf-RFI3");
    reg.insert("xf-RFO-Rk");
    reg.insert("xf-RFO");
    reg.insert("xf-RFO3");
    reg
}

#[test]
fn test_compare() {
    let _ = env_logger::try_init();
    let testdir = concat!(env!("CARGO_MANIFEST_DIR"), "/testdata").to_string();
    let testdir = std::path::PathBuf::from(testdir);
    for name in movelist() {
        let path = testdir.join(format!("{name}.svg"));
        info!("process '{name}' for {path:?}");
        let got = skreate::generate(name).expect("failed to generate SVG");
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
