use std::fs::File;
use std::io::{BufReader, Read};
use std::time::Instant;
use serde::{Deserialize, Serialize};
use crate::cache::Cache;
use crate::config::{CacheConfig, CacheKindConfig, LayeredCacheConfig};

pub const LINE_SIZE: usize = 40;
const ADDRESS_OFFSET: usize = 17;
const ADDRESS_SIZE: usize = 16;
const ADDRESS_UPPER: usize = ADDRESS_OFFSET + ADDRESS_SIZE;
const RW_MODE: usize = ADDRESS_UPPER + 1;
const SIZE: usize = RW_MODE + 2;

pub struct Simulator {
    caches: Vec<Cache>,
    result: LayeredCacheResult,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct LayeredCacheResult {
    main_memory_accesses: u64,
    caches: Vec<CacheResult>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct CacheResult {
    name: String,
    hits: u64,
    misses: u64,
}

impl Simulator {
    pub fn new(config: LayeredCacheConfig) -> Self {
        let caches: Vec<Cache> = config.caches.iter().map(Self::config_to_cache).collect();
        let result = LayeredCacheResult {
            main_memory_accesses: 0,
            caches: config.caches.iter().map(|cache| CacheResult {
                hits: 0,
                misses: 0,
                name: cache.name.clone(),
            }).collect(),
        };
        Self {
            caches,
            result,
        }
    }

    #[inline(always)]
    fn read(&mut self, address: u64, size: u16) {
        // Assume line size doesn't decrease with level
        let alignment_diff = address % self.caches[0].line_size;
        let mut current_aligned_address = address - alignment_diff;
        while current_aligned_address < (address + size as u64) as u64 {
            let mut cache_index = 0;
            let mut found = false;
            while cache_index < self.caches.len() {
                let line_size = self.caches[cache_index].line_size;
                if self.caches[cache_index].read_and_update_line(current_aligned_address - current_aligned_address % line_size) {
                    // Hit
                    found = true;
                    self.result.caches[cache_index].hits += 1;
                    break;
                } else {
                    // Miss
                    self.result.caches[cache_index].misses += 1;
                }
                cache_index += 1;
            }
            if !found {
                self.result.main_memory_accesses += 1;
            }
            current_aligned_address += self.caches[0].line_size
        }
    }


    pub fn simulate<Source: Read>(&mut self, mut reader: BufReader<Source>) -> Result<&LayeredCacheResult, String> {
        if reader.buffer().len() % LINE_SIZE != 0 {
            return Err(String::from("Buffer is incorrectly aligned"));
        }
        let start = Instant::now();
        let mut buffer = [0u8; LINE_SIZE];
        loop {
            let num_bytes = reader.read(&mut buffer).map_err(|e| format!("Couldn't read from the input source: {e}"))?;
            if num_bytes == 0 {
                let end = Instant::now();
                eprintln!("Time: {}", (end - start).as_secs_f64());
                println!("Count: {}", self.caches.iter().map(|c| c.cache.iter().filter(|a| !a.valid).count()).sum::<usize>());
                return Ok(&self.result);
            }
            debug_assert!(num_bytes == 40 && buffer[39] == b'\n');
            let addr_as_str = std::str::from_utf8(&buffer[ADDRESS_OFFSET..ADDRESS_UPPER]).map_err(|e| format!("Parsing address error: {e}"))?;
            let address = u64::from_str_radix(addr_as_str, 16).map_err(|e| format!("Parsing address error: {e}"))?;
            let size_as_str = std::str::from_utf8(&buffer[SIZE..LINE_SIZE - 1]).map_err(|e| format!("Parsing size error: {e}"))?;
            let size = u16::from_str_radix(size_as_str, 10).map_err(|e| format!("Parsing size error: {e}"))?;
            let is_read = buffer[RW_MODE] == b'R';
            if is_read {
                self.read(address, size);
            } else {
                debug_assert!(buffer[RW_MODE] == b'W');
                self.read(address, size)
            }
        }
    }

    fn config_to_cache(config: &CacheConfig) -> Cache {
        let num_lines = config.size / config.line_size;
        match config.kind {
            CacheKindConfig::Direct => {
                Cache::new(config.size, config.line_size, num_lines, config.replacement_policy)
            }
            CacheKindConfig::Full => {
                Cache::new(config.size, config.line_size, 1, config.replacement_policy)
            }
            CacheKindConfig::TwoWay => {
                Cache::new(config.size, config.line_size, num_lines / 2, config.replacement_policy)
            }
            CacheKindConfig::FourWay => {
                Cache::new(config.size, config.line_size, num_lines / 4, config.replacement_policy)
            }
            CacheKindConfig::EightWay => {
                Cache::new(config.size, config.line_size, num_lines / 8, config.replacement_policy)
            }
        }
    }
}



