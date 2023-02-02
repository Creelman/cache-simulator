use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use regex::Regex;
use crate::config::{CacheConfig, LayeredCacheConfig};
use crate::simulator::{LayeredCacheResult, Simulator};

const SAMPLE_INPUTS_PATH: &str = "/cs/studres/CS4202/Coursework/P1-CacheSim/sample-inputs";
const SAMPLE_OUTPUTS_PATH: &str = "/cs/studres/CS4202/Coursework/P1-CacheSim/sample-outputs";
const TRACE_FILES_PATH: &str = "/cs/studres/CS4202/Coursework/P1-CacheSim/trace-files";

#[test]
fn run_all_examples() -> Result<(), Box<dyn Error>> {
    let output_file_directory = fs::read_dir(SAMPLE_OUTPUTS_PATH).unwrap();
    let output_pattern = Regex::new(r"output-(?P<trace>[0-9a-zA-Z_]+)-(?P<config>[0-9a-zA-Z_]+)\.json").unwrap();
    thing();
    for f in output_file_directory.into_iter().filter(|a| output_pattern.is_match(&a.as_ref().unwrap().file_name().into_string().unwrap())) {
        let file = f.unwrap();
        let file_name = file.file_name().into_string().unwrap();
        let tokens = output_pattern.captures(&file_name).unwrap();
        let x = tokens.get(0).unwrap();
        let trace_file_path = tokens.get(1).unwrap().as_str();
        let config_file_path = tokens.get(2).unwrap().as_str();
        let trace_file = File::open(format!("{}/{}.out", TRACE_FILES_PATH, trace_file_path)).unwrap();
        let config_file = File::open(format!("{}/{}.json", SAMPLE_INPUTS_PATH, config_file_path)).unwrap();
        let expected_output_file = File::open(format!("{}/{}", SAMPLE_OUTPUTS_PATH, file_name)).unwrap();
        let expected_output: LayeredCacheResult = serde_json::from_reader(BufReader::new(expected_output_file)).unwrap();
        let config: LayeredCacheConfig = serde_json::from_reader(BufReader::new(config_file)).unwrap();
        let mut simulator = Simulator::new(config);
        let result = simulator.simulate(BufReader::with_capacity(40_000, trace_file)).unwrap();
        assert_eq!(*result, expected_output);
    }
    Ok(())
}

#[test]
fn thing() {}
