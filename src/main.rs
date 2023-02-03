mod config;
mod simulator;
mod cache;
#[cfg(test)]
mod test;

use std::fs::File;
use std::io::{BufReader};
use std::time::Instant;
use clap::Parser;
use crate::config::LayeredCacheConfig;
use crate::simulator::Simulator;

const BUFFER_SIZE: usize = 40 * 4096;

#[cfg(debug_assertions)]
const DEBUG_DEFAULT: bool = true;

#[cfg(not(debug_assertions))]
const DEBUG_DEFAULT: bool = false;

#[derive(Parser, Debug)]
#[command(about = String::from("Cache simulator for CS4202 Practical 1"))]
struct Args {
    config: String,
    trace: String,

    #[arg(short, long)]
    performance: bool,

    #[arg(short, long, default_value_t=DEBUG_DEFAULT)]
    debug: bool
}

fn main() -> Result<(), String> {
    let start = Instant::now();
    let args = Args::parse();
    let config_file = File::open(&args.config).map_err(|e| format!("Couldn't open the config file at path {}: {e}", args.config))?;
    let config: LayeredCacheConfig = serde_json::from_reader(BufReader::new(config_file)).map_err(|e| format!("Couldn't parse the config file: {e}"))?;
    let trace_file = File::open(&args.trace).map_err(|e| format!("Couldn't open the trace file at path {}: {e}", args.trace))?;
    let trace_reader = BufReader::with_capacity(BUFFER_SIZE, trace_file);
    let mut simulator = Simulator::new(config);
    let result = simulator.simulate(trace_reader)?;
    println!("{}", serde_json::to_string_pretty(result).map_err(|e| format!("Couldn't serialise the output {e}"))?);
    let end = Instant::now();
    if args.performance {
        let simulation_time = simulator.get_execution_time();
        let total_time = end - start;
        println!("Simulation time: {}s, {}ns", simulation_time.as_secs(), simulation_time.as_nanos());
        println!("Total execution time (includes initial parsing, configuration, and output): {}s, {}ns", total_time.as_secs(), total_time.as_nanos())
    }
    Ok(())
}