use std::fs::File;
use std::io::{BufReader, Read};
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use cachelib::config::LayeredCacheConfig;
use cachelib::simulator::Simulator;
use cachelib::util::get_configs;

/// Benchmark experimenting
pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Examples");

    get_configs()
        .unwrap()
        .iter()
        .for_each(|case| {
            let config_file = File::open(case.config.clone()).unwrap();
            // Ignoring expected output
            let config: LayeredCacheConfig = serde_json::from_reader(BufReader::new(config_file)).unwrap();
            let mut trace_file = File::open(case.trace.clone()).unwrap();
            let mut buf = Vec::new();
            // For the purposes of this we aren't interested in IO effects, and the given examples,
            // while large, are small enough to fit into memory
            trace_file.read_to_end(&mut buf).unwrap();
            group.bench_with_input(BenchmarkId::new("Example: ", case.output.clone()), &(config, buf), |bench, (conf, buf)| {
                bench.iter(|| {
                    Simulator::new(conf).simulate(buf).unwrap();
                });
            });
        });
}

criterion_group!(
    name = benches;
    config = Criterion::default().significance_level(0.1).sample_size(10);
    targets = criterion_benchmark
);
criterion_main!(benches);