use clap::Parser;
use serde_json::json;
use skreate::moves;
use std::ffi::OsStr;
use std::fs::{read_dir, File};
use std::io::Write;
use std::path::Path;

const TEMPLATE: &str = "template";

#[derive(Parser, Debug)]
struct Options {
    /// Input file.
    #[arg(short, long)]
    in_file: String,
    /// Output directory.
    #[arg(short, long)]
    out_dir: String,
    /// Directory holding examples.
    #[arg(short, long)]
    eg_dir: Option<String>,
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
    let extension = OsStr::new("skate");
    let opts = Options::parse();

    let out_path = check_dir(&opts.out_dir);

    let mut hbs = handlebars::Handlebars::new();
    hbs.register_template_file(TEMPLATE, &opts.in_file)
        .expect(&format!("failed to load template at {}", opts.in_file));

    let mut examples: Vec<String> = Vec::new();
    if let Some(eg_dir) = &opts.eg_dir {
        let eg_path = check_dir(eg_dir);
        for entry in read_dir(eg_path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() && Some(extension) == path.extension() {
                examples.push(entry.file_name().into_string().unwrap());
            }
        }
    }
    examples.sort();

    let infos = moves::info();
    let json = json!({"infos": &infos, "examples": &examples});
    let filename = out_path.join("manual.html");
    let mut outfile = File::create(filename).expect("failed to create {filename:?}");
    outfile
        .write_all(
            hbs.render(TEMPLATE, &json)
                .expect("failed to render")
                .as_bytes(),
        )
        .expect("failed to write rendered manual");

    // Also generate a sample SVG file for each move.
    for info in infos {
        if !info.visible {
            continue;
        }
        let svg = skreate::generate(info.example)
            .expect(&format!("example for {} does not parse!", info.example));
        let filename = out_path.join(format!("{}.svg", info.name));
        let mut svgfile = File::create(filename).expect("failed to create {filename:?}");
        svgfile
            .write_all(svg.as_bytes())
            .expect("failed to write rendered SVG file");
    }
}
