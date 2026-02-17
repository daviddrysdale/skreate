// Copyright 2024-2025 David Drysdale

//! Generate SVG files for examples.

use clap::Parser;
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Parser, Debug)]
struct Options {
    /// Output directory.
    #[arg(short, long)]
    out_dir: String,
}

fn check_dir(dir: &str) -> &Path {
    let path = Path::new(dir);
    if !path.exists() {
        eprintln!("Directory {dir} does not exist.");
        std::process::exit(1)
    }
    if !path.is_dir() {
        eprintln!("Location {dir} is not a directory.");
        std::process::exit(1)
    }
    path
}

fn main() {
    let opts = Options::parse();

    let out_path = check_dir(&opts.out_dir);

    for info in skreate::moves::INFO {
        if !info.visible {
            continue;
        }
        let svg = skreate::generate(info.example)
            .unwrap_or_else(|_| panic!("example for {} does not parse!", info.example));

        let filename = format!("{}.svg", sanitise_file_name::sanitise(info.example));
        let filename = out_path.join(filename);
        println!("Generate {filename:?}");
        let mut svgfile = File::create(filename).expect("failed to create {filename:?}");
        svgfile
            .write_all(svg.as_bytes())
            .expect("failed to write rendered SVG file");
    }
}
