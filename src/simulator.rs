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
// Switched from [u8] to [u64] to save on assembly cast instructions
const HEX_LOOKUP: [u64; 256] = generate_hex_lookup_table();

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
        // Main memory access are whatever misses the last cache
        self.result.main_memory_accesses = self.result.caches.last().unwrap().misses;
    }


    pub fn simulate<Source: Read>(&mut self, mut reader: Source) -> Result<&LayeredCacheResult, String> {
        let start = Instant::now();
        let mut buffer = [0u8; LINE_SIZE];
        loop {
            let num_bytes = reader.read(&mut buffer).unwrap();
            if num_bytes == 0 {
                let end = Instant::now();
                self.simulation_time += end - start;
                return Ok(&self.result);
            }
            debug_assert!(num_bytes == 40 && buffer[39] == b'\n');

            // Re-implemented, as parse and from_str_radix end up being the bottleneck for smaller caches
            let address = Self::parse_address((&buffer[ADDRESS_OFFSET..ADDRESS_UPPER]).try_into().unwrap());
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

    fn config_to_cache(config: &CacheConfig) -> Cache {
        let num_lines = config.size / config.line_size;
        match config.kind {
            CacheKindConfig::Direct => {
                Cache::new(config.size, config.line_size, num_lines, None)
            }
            CacheKindConfig::Full => {
                Cache::new(config.size, config.line_size, 1, Some(config.replacement_policy))
            }
            CacheKindConfig::TwoWay => {
                Cache::new(config.size, config.line_size, num_lines / 2, Some(config.replacement_policy))
            }
            CacheKindConfig::FourWay => {
                Cache::new(config.size, config.line_size, num_lines / 4, Some(config.replacement_policy))
            }
            CacheKindConfig::EightWay => {
                Cache::new(config.size, config.line_size, num_lines / 8, Some(config.replacement_policy))
            }
        }
    }

    // Way faster than stdlib, assumes input is well formed
    pub fn parse_address(buf: &[u8; 16]) -> u64 {
        // This is completely unrolled by compiler, output is branch-less (verified on 1.67.0)
        let res = buf.iter()
            .rev()
            .enumerate()
            .map(|(idx, a)| (HEX_LOOKUP[*a as usize] as u64) << (idx * 4))
            .reduce(|a, b| a | b)
            .unwrap();
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

    #[inline(never)]
    // Marked unsafe as it assumes input is well formed
    fn parse_size(buf: &[u8; 3]) -> u16 {
        let mut res: u16 = 0;
        res += 1u16 * (buf[2] - b'0') as u16;
        res += 10u16 * (buf[1] - b'0') as u16;
        res += 100u16 * (buf[0] - b'0') as u16;
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

// Cached at compile time
const fn generate_hex_lookup_table() -> [u64; 256] {
    let mut output = [0u64; 256];
    let mut input = 0;
    while input < u8::MAX {
        output[input as usize] = if (input) >= b'0' && input <= b'9' {
            input - b'0'
        } else if input >= b'A' && input <= b'F' {
            input - b'A' + 10
        } else if input >= b'a' && input <= b'f' {
            input - b'a' + 10
        } else {
            0
        } as u64;
        input += 1;
    }
    output
}



