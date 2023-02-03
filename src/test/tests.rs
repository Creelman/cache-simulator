use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use regex::Regex;
use crate::config::{LayeredCacheConfig};
use crate::simulator::{LayeredCacheResult, Simulator};

const SAMPLE_INPUTS_PATH: &str = "/cs/studres/CS4202/Coursework/P1-CacheSim/sample-inputs";
const SAMPLE_OUTPUTS_PATH: &str = "/cs/studres/CS4202/Coursework/P1-CacheSim/sample-outputs";
const TRACE_FILES_PATH: &str = "/cs/studres/CS4202/Coursework/P1-CacheSim/trace-files";

#[test]
fn run_all_examples() -> Result<(), Box<dyn Error>> {
    let output_file_directory = fs::read_dir(SAMPLE_OUTPUTS_PATH)?;
    let output_pattern = Regex::new(r"output-(?P<trace>[0-9a-zA-Z_]+)-(?P<config>[0-9a-zA-Z_]+)\.json")?;
    // Sort files for consistency
    let mut files = output_file_directory.into_iter()
        .filter(|a| output_pattern.is_match(&a.as_ref().unwrap().file_name().into_string().unwrap()))
        .map(|a| a.unwrap())
        .collect::<Vec<_>>();
    files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
    for file in files {
        // Get file name
        let file_name = file.file_name().into_string().map_err(|e| format!("Can't convert OS string ({e:?}) to standard string"))?;
        // Get components of name
        let tokens = output_pattern.captures(&file_name).ok_or("Couldn't parse the file name".to_string())?;
        let trace_file_path = tokens.get(1).ok_or("Couldn't get the trace file from the output file name".to_string())?.as_str();
        let config_file_path = tokens.get(2).ok_or("Couldn't get the config file from the output file name".to_string())?.as_str();
        // Get input files
        let trace_file = File::open(format!("{TRACE_FILES_PATH}/{trace_file_path}.out"))?;
        let config_file = File::open(format!("{SAMPLE_INPUTS_PATH}/{config_file_path}.json"))?;
        // Read expected output
        let expected_output_file = File::open(format!("{SAMPLE_OUTPUTS_PATH}/{file_name}"))?;
        let expected_output: LayeredCacheResult = serde_json::from_reader(BufReader::new(expected_output_file))?;
        // Simulate!
        let config: LayeredCacheConfig = serde_json::from_reader(BufReader::new(config_file))?;
        let mut simulator = Simulator::new(config);
        let result = simulator.simulate(BufReader::with_capacity(40_000, trace_file))?;
        // Check results
        assert_eq!(*result, expected_output);
        let time = simulator.get_execution_time();
        println!("Success for {file_name}, time: {}", time.as_nanos() as f64 / 1e9);
    }
    Ok(())
}
