use std::fs::File;
use std::io::{BufReader};
use criterion::{criterion_group, criterion_main, Criterion};
use cachelib::config::LayeredCacheConfig;
use cachelib::io::get_reader;
use cachelib::simulator::Simulator;
use cachelib::util::get_configs;
use criterion_cycles_per_byte::CyclesPerByte;

pub fn criterion_benchmark(c: &mut Criterion<CyclesPerByte>) {
    get_configs()
        .unwrap()
        .iter()
        .for_each(|case| {
            let config_file = File::open(case.config.clone()).unwrap();
            // Ignoring expected output
            let config: LayeredCacheConfig = serde_json::from_reader(BufReader::new(config_file)).unwrap();
            c.bench_function(&(format!("Test Output File: {}", case.output)), move |b| {
                let mut simulator = Simulator::new(&config);
                b.iter(move || {
                    let trace_file = File::open(case.trace.clone()).unwrap();
                    let trace_reader = get_reader(trace_file).unwrap();
                    simulator.simulate(trace_reader).unwrap();
                })
            });
        });
}

criterion_group!(
    name = benches;
    config = Criterion::default().significance_level(0.1).sample_size(10).with_measurement(CyclesPerByte);
    targets = criterion_benchmark
);
criterion_main!(benches);