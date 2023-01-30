use serde::Serialize;
use crate::cache::Cache;
use crate::config::LayeredCacheConfig;

pub struct Simulator {
    caches: Vec<Cache>,

}

#[derive(Serialize)]
pub struct LayeredCacheResult {
    main_memory_accesses: u64,
    caches: Vec<LayeredCacheResult>
}

#[derive(Serialize)]
pub struct CacheResult {
    name: String,
    hits: u64,
    misses: u64
}

impl Simulator {
    pub fn new(config: LayeredCacheConfig) -> Self {
        todo!()
    }
}

