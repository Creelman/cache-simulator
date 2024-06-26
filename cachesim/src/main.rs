use std::fs::File;
use std::io::{BufReader};
use std::time::Instant;
use clap::Parser;
use cachelib::config::LayeredCacheConfig;
use cachelib::simulator::Simulator;
use memmap2::{Advice, Mmap};

#[cfg(debug_assertions)]
const DEBUG_DEFAULT: bool = true;

#[cfg(not(debug_assertions))]
const DEBUG_DEFAULT: bool = false;

#[derive(Parser, Debug)]
#[command(about)]
/// Cache simulator for CS4202 Practical 1
struct Args {
    /// The path to the JSON configuration file
    config: String,

    /// The path to the trace file
    trace: String,

    /// Output performance statistics
    #[arg(short, long)]
    performance: bool,

    /// Output debug information
    #[arg(short, long, default_value_t = DEBUG_DEFAULT)]
    debug: bool,
}

fn main() -> Result<(), String> {
    let start = Instant::now();
    let args = Args::parse();
    let config_file = File::open(&args.config).map_err(|e| format!("Couldn't open the config file at path {}: {e}", args.config))?;
    let config: LayeredCacheConfig = serde_json::from_reader(BufReader::new(config_file)).map_err(|e| format!("Couldn't parse the config file: {e}"))?;
    if config.caches.is_empty() {
        return Err("The provided file is valid, but the list of caches was empty".to_string())
    }
    let mut simulator = Simulator::new(&config);
    let trace_file = File::open(&args.trace).map_err(|e| format!("Couldn't open the trace file at path {}: {e}", args.trace))?;
    // MMap for speed. If we wanted more portability we could use a BufReader and repeatedly call
    // simulate - this is the main reason simulate explicitly supports multiple calls to simulate
    let map = unsafe {
        let m = Mmap::map(&trace_file).map_err(|e| format!("Couldn't memory map the file: {e}"))?;
        m.advise(Advice::Sequential).map_err(|e| format!("Failed to provide access advice to the OS, {e}"))?;
        m
    };
    let result = simulator.simulate(map.as_ref())?;
    println!("{}", serde_json::to_string_pretty(result).map_err(|e| format!("Couldn't serialise the output {e}"))?);
    // Output performance characteristics
    if args.performance {
        let end = Instant::now();
        let simulation_time = simulator.get_execution_time();
        let total_time = end - start;
        println!("Simulation time: {}s", simulation_time.as_nanos() as f64 / 1e9);
        println!("Total execution time (includes initial parsing, configuration, and output): {}s", total_time.as_nanos() as f64 / 1e9)
    }
    // Output debug characteristics
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