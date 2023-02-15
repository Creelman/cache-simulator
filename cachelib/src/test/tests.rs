use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use memmap2::{Advice, Mmap};
use crate::config::{LayeredCacheConfig};
use crate::simulator::{LayeredCacheResult, Simulator};
use crate::util::{get_configs};

#[test]
fn run_all_examples() -> Result<(), Box<dyn Error>> {
    for test in get_configs()? {
        // Get file name
        println!("Running test for {}", test.output);
        // Get input files
        let trace_file = File::open(test.trace)?;
        let config_file = File::open(test.config)?;
        // Read expected output
        let expected_output_file = File::open(test.output.clone())?;
        let expected_output: LayeredCacheResult = serde_json::from_reader(BufReader::new(expected_output_file))?;
        // Simulate!
        let config: LayeredCacheConfig = serde_json::from_reader(BufReader::new(config_file))?;
        let mut simulator = Simulator::new(&config);
        let mmap = unsafe {
            let m = Mmap::map(&trace_file).map_err(|e| format!("Couldn't memory map the file: {e}"))?;
            m.advise(Advice::Sequential).map_err(|e| format!("Failed to provide access advice to the OS, {e}"))?;
            m
        };
        let result = simulator.simulate(&mmap)?;
        assert_eq!(*result, expected_output);
        // Check results
        let time = simulator.get_execution_time();
        println!("Success for {}, time: {}", test.output, time.as_nanos() as f64 / 1e9);
    }
    Ok(())
}
