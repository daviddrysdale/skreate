// Copyright 2024-2025 David Drysdale

//! Command line driver for diagram generation.
use anyhow::Result;
use clap::Parser;
use log::debug;
use std::io::Write;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Opts {
    /// Where to read input; '-' means use stdin.
    r#infile: String,

    /// Where to emit output; stdout if not specified.
    #[arg(short, long = "out")]
    outfile: Option<String>,
}

fn main() -> Result<()> {
    env_logger::init();
    let opts = Opts::parse();

    let infile = std::env::args().nth(1);
    debug!("processing input from {infile:?}");
    let mut reader: Box<dyn std::io::Read> = match opts.infile.as_ref() {
        "-" => Box::new(std::io::stdin()),
        f => Box::new(std::fs::File::open(f)?),
    };

    let mut input = String::new();
    reader.read_to_string(&mut input)?;
    let output = skreate::generate(&input)?;

    let mut writer: Box<dyn std::io::Write> = match opts.outfile {
        None => Box::new(std::io::stdout()),
        Some(f) => Box::new(std::fs::File::create(f)?),
    };
    writer.write_all(&output.into_bytes())?;
    Ok(())
}
