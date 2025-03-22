// Copyright 2024-2025 David Drysdale

//! Command line driver for diagram generation.
use anyhow::Result;
use log::debug;
use std::io::Write;

fn main() -> Result<()> {
    env_logger::init();

    let infile = std::env::args().nth(1);
    debug!("processing input from {infile:?}");
    let mut reader: Box<dyn std::io::Read> = match infile.as_deref() {
        None | Some("-") => Box::new(std::io::stdin()),
        Some(f) => Box::new(std::fs::File::open(f)?),
    };

    let mut input = String::new();
    reader.read_to_string(&mut input)?;
    let output = skreate::generate(&input)?;
    std::io::stdout().write_all(&output.into_bytes())?;
    Ok(())
}
