use std::fs::File;
use std::io::{BufReader};
use std::time::Instant;
use clap::Parser;
use cachelib::config::LayeredCacheConfig;
use cachelib::io::get_reader;
use cachelib::simulator::Simulator;

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

    #[arg(short, long, default_value_t = DEBUG_DEFAULT)]
    debug: bool,
}

fn main() -> Result<(), String> {
    let start = Instant::now();
    let args = Args::parse();
    let config_file = File::open(&args.config).map_err(|e| format!("Couldn't open the config file at path {}: {e}", args.config))?;
    let config: LayeredCacheConfig = serde_json::from_reader(BufReader::new(config_file)).map_err(|e| format!("Couldn't parse the config file: {e}"))?;
    let mut simulator = Simulator::new(&config);
    let trace_file = File::open(&args.trace).map_err(|e| format!("Couldn't open the trace file at path {}: {e}", args.trace))?;
    let trace_reader = get_reader(trace_file)?;
    let result = simulator.simulate(trace_reader)?;
    println!("{}", serde_json::to_string_pretty(result).map_err(|e| format!("Couldn't serialise the output {e}"))?);
    if args.performance {
        let end = Instant::now();
        let simulation_time = simulator.get_execution_time();
        let total_time = end - start;
        println!("Simulation time: {}s", simulation_time.as_nanos() as f64 / 1e9);
        println!("Total execution time (includes initial parsing, configuration, and output): {}s", total_time.as_nanos() as f64 / 1e9)
    }
    if args.debug {
        #[cfg(debug_assertions)]
        println!("Running the debug binary, debug mode is enabled by default. If benchmarking, do not use this binary, re-compile with the --release argument when using cargo run");
        println!("Parsed input configuration: {config:?}");
        let uninitialised_lines = simulator.get_uninitialised_line_counts();
        let formatted = config.caches
            .iter()
            .map(|c| c.name.clone())
            .zip(uninitialised_lines.iter())
            .map(|(name, count)| format!("{name}: {}", *count))
            .reduce(|a, b| format!("{a}, {b}")).unwrap();
        println!("Uninitialised cache lines by layer: ({formatted})");
        println!("Total uninitialised cache lines: {}", uninitialised_lines.iter().sum::<u64>())
    }
    Ok(())
}