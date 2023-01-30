mod config;
mod simulator;
mod cache;

use std::fs::File;
use std::io::BufReader;
use clap::Parser;
use crate::config::LayeredCacheConfig;

#[derive(Parser, Debug)]
#[command(about = String::from("Cache simulator for CS4202 Practical 1"))]
struct Args {
    config: String,
    trace: String,
}

fn main() -> Result<(), String> {
    let args = Args::parse();
    let config_file = File::open(&args.config).map_err(|e| format!("Couldn't open the config file at path {}: {e}", args.config))?;
    let config: LayeredCacheConfig = serde_json::from_reader(BufReader::new(config_file)).map_err(|e| format!("Couldn't parse the config file: {e}"))?;
    let trace_file = File::open(&args.trace).map_err(|e| format!("Couldn't open the trace file at path {}: {e}", args.trace))?;
    let trace_reader = BufReader::new(trace_file);

    Ok(())
}
