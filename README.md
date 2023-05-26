# CS4202 Practical 1 - Cache Simulator

## Design
The implementation is split into two crates, `cachelib` and `cachesim`. The former can be used a library, with the latter being a small wrapper around the library to implement a command line interface.

As such, the implementation prioritises ease of use and maintenance, while striving to maintain a high performance.

The implementation is written in Rust, and the executable supports most platforms which support memory mapping files as cross-platform APIs are used.

## Safety

All core functionality avoids unsafe code. `cachesim` and the tests use unsafe blocks to memory map files due to limitations across platforms, for more information see the crate documentation for `memmap2` [here](https://docs.rs/memmap2/latest/memmap2/struct.Mmap.html)

## Usage

### Requirements

* Rust 1.67.1
  * Or any greater version, stable channel.
  * `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
* Internet connection
  * To download required packages from crates.io when building the program

### Build and Run

From the `cacheprac` directory use the following command to build and run the executable

`cargo run --release -- <config_path> <trace_path>`

Alternatively, build an executable with

`cargo build --release`

Then run the executable with

`./target/release/cachesim <config_path> <trace_path>`

### Optional Arguments
For additional information, run the executable with the argument `--help`

| Short argument | Long argument | Meaning                                                                                              |
|----------------|---------------|------------------------------------------------------------------------------------------------------|
| -p             | --performance | Outputs the time taken to run the tests, with and without the time taken to load the configurations. |
| -d             | --debug       | Outputs some debug information to stdout. Enabled by default when compiled in debug mode.            |
| -h             | --help        | Show help                                                                                            |

### Running Tests
To run all tests, use

`cargo test`

This will automatically find and run all examples at ./examples, checking against the expected output. (Not in this repository as they're several GB)

As the benchmarks can take a while in debug mode, the `Cargo.toml` file enables optimisation when running tests, but keeps debug assertions and debug information. If any errors are removed this line can be removed to make it easier to use debugging tools.

### Running Benchmarks

There are benchmarks for `cachelib`, which use `criterion` to measure changes in performance. The benchmarking system will automatically benchmark all examples. To run benchmarks, use

`cargo bench`

This may take up to 10 minutes, depending on the machine used.

Details on performance changes will be output to stdout, and graphs can be viewed by opening `./target/criterion/report/index.html` in a browser.

To prevent IO issues adding significant noise to measurements, for benchmarking the entire trace file is read into memory. This isn't an issue for any of the examples, but to support larger files we don't do this for the executable file, memory mapping the file instead.

### Library Documentation
The `cachelib` crate has full rustdoc support for all public methods, and can be generated and viewed using

`cargo doc --no-deps --open`

This will generate the documentation and open it in the default web browser.
