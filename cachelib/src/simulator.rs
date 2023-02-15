use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use crate::cache::{Cache, CacheTrait, GenericCache};
use crate::config::{CacheConfig, CacheKindConfig, LayeredCacheConfig, ReplacementPolicyConfig};
use crate::hex::HEX_LOOKUP;
use crate::replacement_policies::{LeastFrequentlyUsed, LeastRecentlyUsed, NoPolicy, RoundRobin};

const LINE_SIZE: usize = 40;
const ADDRESS_OFFSET: usize = 17;
const ADDRESS_SIZE: usize = 16;
const ADDRESS_UPPER: usize = ADDRESS_OFFSET + ADDRESS_SIZE;
const RW_MODE: usize = ADDRESS_UPPER + 1;
const SIZE: usize = RW_MODE + 2;

/// The simulator handles line alignment when using the caches, and collects results.
///
/// It supports calling simulate multiple times, and will update the time taken to simulate and the
/// results accordingly
pub struct Simulator {
    caches: Vec<GenericCache>,
    result: LayeredCacheResult,
    simulation_time: Duration,
}

/// The result of a cache simulation. Can be serialised to the required output format
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct LayeredCacheResult {
    main_memory_accesses: u64,
    caches: Vec<CacheResult>,
}

/// The result for an individual cache. Can be serialised to the required output format
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct CacheResult {
    name: String,
    hits: u64,
    misses: u64,
}

impl Simulator {

    /// Creates a new simulator for a given configuration
    ///
    /// # Arguments
    ///
    /// * `config`: A cache configuration, usually resulting from parsing JSON
    ///
    /// returns: Simulator
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


    /// Reads a value from memory, at a given address with a given size
    ///
    /// The simulator will handle splitting the read so caches can be checked for each relevant line
    ///
    /// # Arguments
    ///
    /// * `address`: The address of the read
    /// * `size`: The size of the read in bytes
    ///
    /// returns: (), internally the result is updated
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


    /// Simulates the cache using a reference to a byte array.
    ///
    /// The byte array must follow the specified format and must have a length which is a multiple
    /// of 40 (not contain partial lines).
    ///
    /// For speed, we don't verify the input format; if the input format may be invalid it should be
    /// validated before using this function. While it won't panic, it may produce incorrect results
    ///
    /// Note that reads from the byte array are *guaranteed to be sequential*. This means that when
    /// using something like mmap, one can advise the operating system that sequential reads will be
    /// used, which can increase read performance
    ///
    /// # Arguments
    ///
    /// * `bytes`: The input byte array
    ///
    /// returns: Result<&LayeredCacheResult, String>
    pub fn simulate(&mut self, bytes: &[u8]) -> Result<&LayeredCacheResult, String> {
        assert_eq!(bytes.len() % 40, 0);
        let start = Instant::now();
        let mut i: usize = 0;
        while i < bytes.len() {
            // Alias for clarity, no overhead when compiled
            let buffer = &bytes[i..i + 40];
            // Re-implemented, as parse and from_str_radix end up being the bottleneck for smaller caches
            let address = parse_address((&buffer[ADDRESS_OFFSET..ADDRESS_UPPER]).try_into().unwrap());
            let size = parse_size((&buffer[SIZE..LINE_SIZE - 1]).try_into().unwrap());
            self.read(address, size);
            i += 40;
        }
        let end = Instant::now();
        self.simulation_time += end - start;
        // Main memory accesses are whatever misses the last cache
        self.result.main_memory_accesses = self.result.caches.last().unwrap().misses;
        Ok(&self.result)
    }

    /// Gets the wall-clock execution time for processing
    pub fn get_execution_time(&self) -> &Duration {
        &self.simulation_time
    }

    /// Gets the number of initialised lines for each cache
    pub fn get_uninitialised_line_counts(&self) -> Vec<u64> {
        self.caches.iter().map(|x| x.get_uninitialised_line_count() as u64).collect()
    }

    /// Creates a new cache from a cache configuration
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
}

/// Parses a 64-bit value from a 16 byte hexadecimal address
///
/// For caches which do not require large lookup times, such as direct or 2way, parsing the
/// address with the standard library becomes the bottleneck by a significant margin, so we
/// use a custom implementation.
///
/// This is significantly faster than using the standard library, but omits checks for the input
/// format. While it is guaranteed not to panic, if the input format is incorrect it may produce
/// incorrect results.
///
/// This function makes use of a lookup table of 2^16 bytes, which performs lookups for each
/// pair of hex values. This gets unrolled by the compiler, and has been shown to be
/// significantly faster than individual lookups of each byte, or branching approaches
///
/// The lookup table is defined in the hex module, which is automatically generated at compile
/// time. We use build.rs for this instead of a const fn in this module as build.rs is much
/// faster to run and the result can be cached across multiple compilations. In addition,
/// using const fn takes too long and the interpreter times out.
///
/// While the lookup table is relatively large, only a small fraction of it (256 entries) are ever
/// accessed, assuming the input is well-formed. This prevents it taking up too much of the cache;
/// only the fragments of it which are useful (and largely sequential!) are ever accessed and
///stored
///
/// # Arguments
///
/// * `buf`: The byte buffer
///
/// returns: u64
///
/// # Examples
///
/// ```
/// use cachelib::simulator::parse_address;
/// let address = b"000000000000000A";
/// assert_eq!(parse_address(&address), 10)
/// ```
pub fn parse_address(buf: &[u8; 16]) -> u64 {
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


/// This exists for the same reasons as parse_address, but uses simple multiplication instead of
/// a lookup table
///
/// The performance difference isn't as large as it is for parse_address as the input is smaller,
/// but it's enough to have a significant impact
///
/// # Arguments
///
/// * `buf`: The input
///
/// returns: u16
///
/// # Examples
///
/// ```
/// use cachelib::simulator::parse_size;
/// let size = b"010";
/// assert_eq!(parse_size(&size), 10);
/// ```
pub fn parse_size(buf: &[u8; 3]) -> u16 {
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
