use std::error::Error;
use std::fs;
use regex::Regex;

/// The path for sample inputs
pub const SAMPLE_INPUTS_PATH: &str = "examples/sample-inputs";

/// The path for sample outputs
pub const SAMPLE_OUTPUTS_PATH: &str = "examples/sample-outputs";

/// The path for trace files
pub const TRACE_FILES_PATH: &str = "examples/trace-files";

/// Convenience struct for test cases
pub struct TestCasePaths {
    pub config: String,
    pub trace: String,
    pub output: String
}

/// Reads all files in the output directory, splits via regex, and outputs test cases with fully
/// qualified paths to the input config, trace file, and output file.
pub fn get_configs() -> Result<Vec<TestCasePaths>, Box<dyn Error>> {
    let mut out = Vec::new();
    let output_file_directory = fs::read_dir(SAMPLE_OUTPUTS_PATH)?;
    let output_pattern = Regex::new(r"output-(?P<trace>[0-9a-zA-Z_]+)-(?P<config>[0-9a-zA-Z_]+)\.json")?;
    let mut files = output_file_directory.into_iter()
        .filter(|a| output_pattern.is_match(&a.as_ref().unwrap().file_name().into_string().unwrap()))
        .map(|a| a.unwrap())
        .collect::<Vec<_>>();
    files.sort_by_key(|a| a.file_name());
    for file in files {
        // Get file name
        let file_name = file.file_name().into_string().map_err(|e| format!("Can't convert OS string ({e:?}) to standard string"))?;
        // Get components of name
        let tokens = output_pattern.captures(&file_name).ok_or("Couldn't parse the file name".to_string())?;
        let trace_file_path = tokens.get(1).ok_or("Couldn't get the trace file from the output file name".to_string())?.as_str();
        let config_file_path = tokens.get(2).ok_or("Couldn't get the config file from the output file name".to_string())?.as_str();
        // Get input files
        let trace_file = format!("{TRACE_FILES_PATH}/{trace_file_path}.out");
        let config_file = format!("{SAMPLE_INPUTS_PATH}/{config_file_path}.json");
        // Read expected output
        let expected_output_file = format!("{SAMPLE_OUTPUTS_PATH}/{file_name}");
        out.push(TestCasePaths {
            config: config_file,
            trace: trace_file,
            output: expected_output_file,
        })
    }
    Ok(out)
}
