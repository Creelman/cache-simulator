mod config;
mod simulator;
mod cache;
#[cfg(test)]
mod test;
mod parallel_sim;

use std::error::Error;
use std::fs::File;
use std::io::{BufReader};
use std::sync::mpsc;
use std::thread;
use std::thread::{JoinHandle, Thread};
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

    #[arg(short, long, default_value_t = DEBUG_DEFAULT)]
    debug: bool,
}

fn main() -> Result<(), String> {
    //test().map_err(|e| e.to_string())?;
    let start = Instant::now();
    let args = Args::parse();
    let config_file = File::open(&args.config).map_err(|e| format!("Couldn't open the config file at path {}: {e}", args.config))?;
    let config: LayeredCacheConfig = serde_json::from_reader(BufReader::new(config_file)).map_err(|e| format!("Couldn't parse the config file: {e}"))?;
    let trace_file = File::open(&args.trace).map_err(|e| format!("Couldn't open the trace file at path {}: {e}", args.trace))?;
    let trace_reader = BufReader::with_capacity(BUFFER_SIZE, trace_file);
    let mut simulator = Simulator::new(&config);
    let result = simulator.simulate(trace_reader)?;
    println!("{}", serde_json::to_string_pretty(result).map_err(|e| format!("Couldn't serialise the output {e}"))?);
    let end = Instant::now();
    if args.performance {
        let simulation_time = simulator.get_execution_time();
        let total_time = end - start;
        println!("Simulation time: {}s", simulation_time.as_nanos() as f64 / 1e9);
        println!("Total execution time (includes initial parsing, configuration, and output): {}s", total_time.as_nanos() as f64 / 1e9)
    }
    if args.debug {
        #[cfg(debug_assertions)]
        println!("Running the debug binary, debug mode is enabled by default. If benchmarking do not use this binary, re-compile with the --release argument");
        let uninitialised_lines = simulator.get_uninitialised_line_counts();
        let formatted= config.caches
            .iter()
            .map(|c| c.name.clone())
            .zip(uninitialised_lines.iter())
            .map(|(name, count)| format!("{name}: {}", *count))
            .reduce(|a, b|format!("{a}, {b}")).unwrap();
        println!("Uninitialised cache lines by layer: ({formatted})");
        println!("Total uninitialised cache lines: {}", uninitialised_lines.iter().sum::<u64>())
    }
    Ok(())
}

fn test() -> Result<(), Box<dyn Error>>{
    let start = Instant::now();
    let mut x: u64 = 20_000_000;
    let (snd, rec) = mpsc::channel::<(u64, u64)>();
    let xx = x;
    let thread: JoinHandle<Result<(), String>> = thread::spawn(move  || {
        let mut i = 0;
        while i < xx {
            snd.send((i, xx)).map_err(|e| e.to_string())?;
            i += 1;
        }
        Ok(())
    });
    while x > 0 {
        rec.recv()?;
        x -= 1;
    }
    thread.join().map_err(|_| "Join err")??;
    let end = Instant::now();
    println!("{:?}", end - start);
    Ok(())
}