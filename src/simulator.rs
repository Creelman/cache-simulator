use std::hint::unreachable_unchecked;
use std::io::{Read};
use std::time::{Duration, Instant};
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
    simulation_time: Duration,
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
    pub fn new(config: &LayeredCacheConfig) -> Self {
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
            simulation_time: Duration::new(0, 0),
        }
    }

    #[inline(always)]
    fn read(&mut self, address: u64, size: u16) {
        // Assume line size doesn't decrease with level
        let lowest_line_size = self.caches[0].get_line_size();
        let alignment_diff = address & !self.caches[0].get_alignment_bit_mask();
        let mut current_aligned_address = address - alignment_diff;
        while current_aligned_address < (address + size as u64) {
            let mut cache_index = 0;
            while cache_index < self.caches.len() {
                if self.caches[cache_index].read_and_update_line(current_aligned_address) {
                    // Hit
                    self.result.caches[cache_index].hits += 1;
                    break;
                } else {
                    // Miss
                    self.result.caches[cache_index].misses += 1;
                }
                cache_index += 1;
            }
            current_aligned_address += lowest_line_size;
        }
        // Main memory access are whatever misses the last cache
        self.result.main_memory_accesses = self.result.caches.last().unwrap().misses;
    }


    pub fn simulate<Source: Read>(&mut self, mut reader: Source) -> Result<&LayeredCacheResult, String> {
        let start = Instant::now();
        let mut buffer = [0u8; LINE_SIZE];
        loop {
            let num_bytes = reader.read(&mut buffer).map_err(|e| format!("Couldn't read from the input source: {e}"))?;
            if num_bytes == 0 {
                let end = Instant::now();
                self.simulation_time += end - start;
                return Ok(&self.result);
            }
            debug_assert!(num_bytes == 40 && buffer[39] == b'\n');
            let address;
            let size;
            unsafe {
                address = Self::parse_address((&buffer[ADDRESS_OFFSET..ADDRESS_UPPER]).try_into().unwrap());
                size = Self::parse_size((&buffer[SIZE..LINE_SIZE - 1]).try_into().unwrap())
            }
            self.read(address, size);
        }
    }

    pub fn get_execution_time(&self) -> &Duration {
        &self.simulation_time
    }

    pub fn get_uninitialised_line_counts(&self) -> Vec<u64> {
        self.caches.iter().map(|x| x.get_uninitialised_line_count() as u64).collect()
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

    // Way faster than stdlib, but unsafe
    #[inline(always)]
    unsafe fn parse_address(buf: &[u8; 16]) -> u64 {
        // Known size array is unrolled by compiler
        let mut res: u64 = 0;
        for (i, char) in buf.iter().rev().enumerate() {
            let bytes = if (*char) <= b'9' {
                *char - b'0'
            } else {
                *char - b'a' + 10
            };
            res |= ((bytes as u64) << (i * 4));
        }
        debug_assert_eq!(
            {
                let addr_as_str = std::str::from_utf8(buf).unwrap();
                let address = u64::from_str_radix(addr_as_str, 16).unwrap();
                address
            },
            res
        );
        res
    }

    unsafe fn parse_size(buf: &[u8; 3]) -> u16 {
        let mut res: u16 = 0;
        for (i, char) in buf.iter().rev().enumerate() {
            res += (10u16.pow(i as u32)) * ((*char - b'0') as u16);
        }
        debug_assert_eq!(
            {
                let size_as_str = std::str::from_utf8(buf).unwrap();
                let size = size_as_str.parse::<u16>().unwrap();
                size
            },
            res
        );
        res
    }
}



