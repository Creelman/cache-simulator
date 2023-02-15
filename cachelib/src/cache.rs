use crate::replacement_policies::{LeastFrequentlyUsed, LeastRecentlyUsed, NoPolicy, ReplacementPolicy, RoundRobin};

/// A generic trait for caches
///
/// Technically not required as we're using static dispatch to speed things up instead of dyn Cache,
/// but this gives flexibility for the future with no overhead
///
/// The trait assumes that ensuring reads spanning multiple cache lines are split properly is the
/// responsibility of the caller
pub trait CacheTrait {

    /// Converts an address into a set and a tag. Both respect cache line alignment.
    ///
    /// The set is aligned such that it can be used as an index to a collection of sets
    ///
    /// The tag is not re-aligned as this isn't required
    ///
    /// # Arguments
    ///
    /// * `input`:
    ///
    /// returns: (u64, u64)
    fn address_to_set_and_tag(&self, input: u64) -> (u64, u64);


    /// Tries to read a cache line, returning true on a cache hit, and false otherwise
    ///
    /// On both hits and misses, the implementation must update any internal buffers, replacement
    /// policies, or other cache metadata
    ///
    /// # Arguments
    ///
    /// * `input`: The address of the read. Note this is for the line at that address, hence no size
    /// argument
    ///
    /// returns: bool
    fn read_and_update_line(&mut self, input: u64) -> bool;

    /// Gets the bit mask used to align the address
    fn get_alignment_bit_mask(&self) -> u64;

    /// Gets the line size used by this cache
    fn get_line_size(&self) -> u64;

    /// Gets the number of uninitialised cache lines. Useful for analysing cache performance or
    /// debugging
    fn get_uninitialised_line_count(&self) -> usize;
}

/// A generic cache implementation, parameterised by a replacement policy
///
/// The general approach here is to have one solid implementation which is easy to maintain and
/// expand with more replacement policies without compromising too much on performance
///
/// To facilitate this we rely on Rust's monomorphisation and the inlining of the replacement policy
/// functions to provide performance, which should be close to on par with writing specialised
/// implementations for each cache type
///
/// We could take this further by adding constant generics for the line size and the size of the
/// cache, which is _just about_ tractable if we say these values are both relatively small powers
/// of two, but it would increase compile times more than I'd like, and either reduces flexibility,
/// or requires adding another *almost* identical implementation
///
/// Note that for optimisation reasons the cache assumes that accessing 0 is not possible, as it
/// would cause an error on most systems
pub struct Cache<R: ReplacementPolicy>
{
    set_selection_bit_mask: u64,
    tag_selection_bit_mask: u64,
    cache_alignment_bit_mask: u64,
    line_size: u64,
    cache: Vec<u64>,
    replacement_policy: R,
    cache_alignment_bits: u8,
    set_size: u64,
}

impl<R: ReplacementPolicy> Cache<R> {
    pub fn new(size: u64, line_size: u64, num_sets: u64, policy: R) -> Self {
        let cache_alignment_bits = line_size.trailing_zeros() as u8;
        let set_selection_bits = num_sets.trailing_zeros() as u8;
        let cache_lines = size / line_size;
        Self {
            set_size: cache_lines / num_sets,
            set_selection_bit_mask: (num_sets - 1) << cache_alignment_bits,
            tag_selection_bit_mask: ((1 << (u64::BITS - set_selection_bits as u32 - cache_alignment_bits as u32)) - 1) << (cache_alignment_bits + set_selection_bits),
            cache_alignment_bit_mask: !((1 << (cache_alignment_bits as u32)) - 1),
            line_size,
            cache_alignment_bits,
            cache: vec![0; cache_lines as usize],
            replacement_policy: policy,
        }
    }
}

impl<R: ReplacementPolicy> CacheTrait for Cache<R> {

    fn address_to_set_and_tag(&self, input: u64) -> (u64, u64) {
        (((input & self.set_selection_bit_mask) >> self.cache_alignment_bits), input & (self.tag_selection_bit_mask))
    }

    // Cache hit is true, cache miss is false
    fn read_and_update_line(&mut self, input: u64) -> bool {
        let (set, tag) = self.address_to_set_and_tag(input);
        let set_inclusive_lower_bound = set * self.set_size;
        let set_exclusive_upper_bound = set_inclusive_lower_bound + self.set_size;
        // Only search the relevant set
        let mut x = set_inclusive_lower_bound;
        while x < set_exclusive_upper_bound {
            // Cache hit
            if self.cache[x as usize] == tag {
                // Update replacement policy, report hit
                self.replacement_policy.update_on_read(x);
                return true;
            }
            x += 1;
        }
        // Cache miss, update
        let line = self.replacement_policy.get_new_line(set_inclusive_lower_bound, set, self.set_size);
        self.cache[line as usize] = tag;
        false
    }
    fn get_alignment_bit_mask(&self) -> u64 {
        self.cache_alignment_bit_mask
    }
    fn get_line_size(&self) -> u64 {
        self.line_size
    }
    fn get_uninitialised_line_count(&self) -> usize {
        self.cache.iter().filter(|a| **a == 0).count()
    }
}

/// Enum for all 4 types of cache provided by the library
///
/// Using trait objects in Rust reduces boilerplate, but it is surprisingly slow, as this is
/// completely opaque to the compiler
///
/// For most cases this isn't an issue, but for our use case we would be de-referencing for each
/// line in the input file, which imposes significant overhead
///
/// It's much faster to explicitly branch on all implementations, as the compiler can reason about
/// the concrete types, perform function inlining etc
pub enum GenericCache {
    RoundRobin(Cache<RoundRobin>),
    LeastRecentlyUsed(Cache<LeastRecentlyUsed>),
    LeastFrequentlyUsed(Cache<LeastFrequentlyUsed>),
    NoPolicy(Cache<NoPolicy>),
}

impl From<Cache<RoundRobin>> for GenericCache {
    fn from(value: Cache<RoundRobin>) -> Self {
        Self::RoundRobin(value)
    }
}

impl From<Cache<LeastRecentlyUsed>> for GenericCache {
    fn from(value: Cache<LeastRecentlyUsed>) -> Self {
        Self::LeastRecentlyUsed(value)
    }
}

impl From<Cache<LeastFrequentlyUsed>> for GenericCache {
    fn from(value: Cache<LeastFrequentlyUsed>) -> Self {
        Self::LeastFrequentlyUsed(value)
    }
}

impl From<Cache<NoPolicy>> for GenericCache {
    fn from(value: Cache<NoPolicy>) -> Self {
        Self::NoPolicy(value)
    }
}

impl CacheTrait for GenericCache {
    fn address_to_set_and_tag(&self, input: u64) -> (u64, u64) {
        match self {
            GenericCache::RoundRobin(c) => c.address_to_set_and_tag(input),
            GenericCache::LeastRecentlyUsed(c) => c.address_to_set_and_tag(input),
            GenericCache::LeastFrequentlyUsed(c) => c.address_to_set_and_tag(input),
            GenericCache::NoPolicy(c) => c.address_to_set_and_tag(input)
        }
    }

    fn read_and_update_line(&mut self, input: u64) -> bool {
        match self {
            GenericCache::RoundRobin(c) => c.read_and_update_line(input),
            GenericCache::LeastRecentlyUsed(c) => c.read_and_update_line(input),
            GenericCache::LeastFrequentlyUsed(c) => c.read_and_update_line(input),
            GenericCache::NoPolicy(c) => c.read_and_update_line(input)
        }
    }

    fn get_alignment_bit_mask(&self) -> u64 {
        match self {
            GenericCache::RoundRobin(c) => c.get_alignment_bit_mask(),
            GenericCache::LeastRecentlyUsed(c) => c.get_alignment_bit_mask(),
            GenericCache::LeastFrequentlyUsed(c) => c.get_alignment_bit_mask(),
            GenericCache::NoPolicy(c) => c.get_alignment_bit_mask()
        }
    }

    fn get_line_size(&self) -> u64 {
        match self {
            GenericCache::RoundRobin(c) => c.get_line_size(),
            GenericCache::LeastRecentlyUsed(c) => c.get_line_size(),
            GenericCache::LeastFrequentlyUsed(c) => c.get_line_size(),
            GenericCache::NoPolicy(c) => c.get_line_size()
        }
    }

    fn get_uninitialised_line_count(&self) -> usize {
        match self {
            GenericCache::RoundRobin(c) => c.get_uninitialised_line_count(),
            GenericCache::LeastRecentlyUsed(c) => c.get_uninitialised_line_count(),
            GenericCache::LeastFrequentlyUsed(c) => c.get_uninitialised_line_count(),
            GenericCache::NoPolicy(c) => c.get_uninitialised_line_count()
        }
    }
}