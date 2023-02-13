use std::io::{Read};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use crate::cache::{Cache, CacheTrait, GenericCache};
use crate::config::{CacheConfig, CacheKindConfig, LayeredCacheConfig, ReplacementPolicyConfig};
use crate::hex::HEX_LOOKUP;
use crate::replacement_policies::{LeastFrequentlyUsed, LeastRecentlyUsed, NoPolicy, RoundRobin};

pub const LINE_SIZE: usize = 40;
const ADDRESS_OFFSET: usize = 17;
const ADDRESS_SIZE: usize = 16;
const ADDRESS_UPPER: usize = ADDRESS_OFFSET + ADDRESS_SIZE;
const RW_MODE: usize = ADDRESS_UPPER + 1;
const SIZE: usize = RW_MODE + 2;
// Originally [u64] to save casts, profiling shows its better to cast [u8], due to better caching effects

pub struct Simulator {
    caches: Vec<GenericCache>,
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
        let caches: Vec<GenericCache> = config.caches.iter().map(Self::config_to_cache).collect();
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

    fn read(&mut self, address: u64, size: u16) {
        // Assume line size doesn't decrease with level
        let first_cache = self.caches.first().unwrap();
        let lowest_line_size = first_cache.get_line_size();
        let alignment_diff = address & !first_cache.get_alignment_bit_mask();
        let mut current_aligned_address = address - alignment_diff;
        while current_aligned_address < (address + size as u64) {
            for (cache, res) in self.caches.iter_mut().zip(&mut self.result.caches) {
                if cache.read_and_update_line(current_aligned_address) {
                    // Hit
                    res.hits += 1;
                    break;
                } else {
                    // Miss
                    res.misses += 1;
                }
            }
            current_aligned_address += lowest_line_size;
        }
    }


    pub fn simulate<Source: Read>(&mut self, mut reader: Source) -> Result<&LayeredCacheResult, String> {
        let start = Instant::now();
        let mut buffer = [0u8; LINE_SIZE];
        loop {
            let num_bytes = reader.read(&mut buffer).unwrap_or(0);
            if num_bytes == 0 {
                let end = Instant::now();
                self.simulation_time += end - start;
                // Main memory accesses are whatever misses the last cache
                self.result.main_memory_accesses = self.result.caches.last().unwrap().misses;
                return Ok(&self.result);
            }
            // Re-implemented, as parse and from_str_radix end up being the bottleneck for smaller caches
            let address = Self::parse_address(&buffer[ADDRESS_OFFSET..ADDRESS_UPPER]);
            let size = Self::parse_size((&buffer[SIZE..LINE_SIZE - 1]).try_into().unwrap());
            self.read(address, size);
        }
    }

    pub fn get_execution_time(&self) -> &Duration {
        &self.simulation_time
    }

    pub fn get_uninitialised_line_counts(&self) -> Vec<u64> {
        self.caches.iter().map(|x| x.get_uninitialised_line_count() as u64).collect()
    }

    fn config_to_cache(config: &CacheConfig) -> GenericCache {
        let num_lines = config.size / config.line_size;
        let num_sets = match config.kind {
            CacheKindConfig::Direct => {
                num_lines
            }
            CacheKindConfig::Full => {
                1
            }
            CacheKindConfig::TwoWay => {
                num_lines / 2
            }
            CacheKindConfig::FourWay => {
                num_lines / 4
            }
            CacheKindConfig::EightWay => {
                num_lines / 8
            }
        };
        if num_sets == num_lines {
            GenericCache::from(Cache::new(config.size, config.line_size, num_sets, NoPolicy::default()))
        } else {
            match config.replacement_policy {
                ReplacementPolicyConfig::RoundRobin => {
                    GenericCache::from(Cache::new(config.size, config.line_size, num_sets, RoundRobin::new(num_sets)))
                }
                ReplacementPolicyConfig::LeastRecentlyUsed => {
                    GenericCache::from(Cache::new(config.size, config.line_size, num_sets, LeastRecentlyUsed::new(num_lines)))
                }
                ReplacementPolicyConfig::LeastFrequentlyUsed => {
                    GenericCache::from(Cache::new(config.size, config.line_size, num_sets, LeastFrequentlyUsed::new(num_lines)))
                }
            }
        }
    }


    // Way faster than stdlib, but omits error checking
    // Why is this faster with inline(never)???
    pub fn parse_address(buf: &[u8]) -> u64 {
        let mut res: u64 = 0;
        let mut x = 0;
        while x < 15 {
            res <<= 8;
            res |= HEX_LOOKUP[buf[x] as usize][buf[x + 1] as usize] as u64;
            x += 2;
        }
        debug_assert_eq!(
            {
                let addr_as_str = std::str::from_utf8(buf).unwrap();
                u64::from_str_radix(addr_as_str, 16).unwrap()
            },
            res
        );
        res
    }

    // Assumes input is well formed, also faster than stdlib
    fn parse_size(buf: &[u8; 3]) -> u16 {
        let mut res = (buf[2] - b'0') as u16;
        res += 10u16 * (buf[1] - b'0') as u16;
        res += 100u16 * (buf[0] - b'0') as u16;
        debug_assert_eq!(
            {
                let size_as_str = std::str::from_utf8(buf).unwrap();
                size_as_str.parse::<u16>().unwrap()
            },
            res
        );
        res
    }
}
