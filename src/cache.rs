use crate::replacement_policies::{LeastFrequentlyUsed, LeastRecentlyUsed, NoPolicy, ReplacementPolicy, RoundRobin};

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

// Relatively boilerplate heavy, but much faster than using trait objects
pub enum GenericCache {
    RoundRobin(Cache<RoundRobin>),
    LeastRecentlyUsed(Cache<LeastRecentlyUsed>),
    LeastFrequentlyUsed(Cache<LeastFrequentlyUsed>),
    None(Cache<NoPolicy>),
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
        Self::None(value)
    }
}

#[derive(Clone, Debug, Default)]
pub struct CacheLineMetadata {
    pub tag: u64,
}

impl<R: ReplacementPolicy> Cache<R> {
    pub fn new(size: u64, line_size: u64, num_sets: u64, policy: R) -> Self {
        let cache_alignment_bits = line_size.ilog2() as u8;
        let set_selection_bits = num_sets.ilog2() as u8;
        let cache_lines = size / line_size;
        Self {
            set_size: cache_lines / num_sets,
            set_selection_bit_mask: ((2u64.pow(set_selection_bits as u32)) - 1) << cache_alignment_bits,
            tag_selection_bit_mask: (2u64.pow(u64::BITS - set_selection_bits as u32 - cache_alignment_bits as u32) - 1) << (cache_alignment_bits + set_selection_bits),
            cache_alignment_bit_mask: !((2u64.pow(cache_alignment_bits as u32)) - 1),
            line_size,
            cache_alignment_bits,
            cache: vec![0; cache_lines as usize],
            replacement_policy: policy,
        }
    }
}

impl<R: ReplacementPolicy> CacheTrait for Cache<R> {
    #[inline(always)]
    fn address_to_set_and_tag(&self, input: u64) -> (u64, u64) {
        (((input & self.set_selection_bit_mask) >> self.cache_alignment_bits), input & (self.tag_selection_bit_mask))
    }
    #[inline(never)]
    // Cache hit is true, cache miss is false
    fn read_and_update_line(&mut self, input: u64) -> bool {
        let (set, tag) = self.address_to_set_and_tag(input);
        let set_inclusive_lower_bound = set * self.set_size;
        let set_exclusive_upper_bound = set_inclusive_lower_bound + self.set_size;
        // Only search the relevant set
        for (index, line) in &mut self.cache[set_inclusive_lower_bound as usize..set_exclusive_upper_bound as usize].iter().enumerate() {
            // Cache hit
            if *line == tag {
                // Update replacement policy, report hit
                self.replacement_policy.update_on_read(set_inclusive_lower_bound + index as u64);
                return true;
            }
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

pub trait CacheTrait {
    fn address_to_set_and_tag(&self, input: u64) -> (u64, u64);
    // Cache hit is true, cache miss is false
    fn read_and_update_line(&mut self, input: u64) -> bool;
    fn get_alignment_bit_mask(&self) -> u64;
    fn get_line_size(&self) -> u64;
    fn get_uninitialised_line_count(&self) -> usize;
}

impl CacheTrait for GenericCache {
    fn address_to_set_and_tag(&self, input: u64) -> (u64, u64) {
        match self {
            GenericCache::RoundRobin(c) => c.address_to_set_and_tag(input),
            GenericCache::LeastRecentlyUsed(c) => c.address_to_set_and_tag(input),
            GenericCache::LeastFrequentlyUsed(c) => c.address_to_set_and_tag(input),
            GenericCache::None(c) => c.address_to_set_and_tag(input)
        }
    }

    fn read_and_update_line(&mut self, input: u64) -> bool {
        match self {
            GenericCache::RoundRobin(c) => c.read_and_update_line(input),
            GenericCache::LeastRecentlyUsed(c) => c.read_and_update_line(input),
            GenericCache::LeastFrequentlyUsed(c) => c.read_and_update_line(input),
            GenericCache::None(c) => c.read_and_update_line(input)
        }
    }

    fn get_alignment_bit_mask(&self) -> u64 {
        match self {
            GenericCache::RoundRobin(c) => c.get_alignment_bit_mask(),
            GenericCache::LeastRecentlyUsed(c) => c.get_alignment_bit_mask(),
            GenericCache::LeastFrequentlyUsed(c) => c.get_alignment_bit_mask(),
            GenericCache::None(c) => c.get_alignment_bit_mask()
        }
    }

    fn get_line_size(&self) -> u64 {
        match self {
            GenericCache::RoundRobin(c) => c.get_line_size(),
            GenericCache::LeastRecentlyUsed(c) => c.get_line_size(),
            GenericCache::LeastFrequentlyUsed(c) => c.get_line_size(),
            GenericCache::None(c) => c.get_line_size()
        }
    }

    fn get_uninitialised_line_count(&self) -> usize {
        match self {
            GenericCache::RoundRobin(c) => c.get_uninitialised_line_count(),
            GenericCache::LeastRecentlyUsed(c) => c.get_uninitialised_line_count(),
            GenericCache::LeastFrequentlyUsed(c) => c.get_uninitialised_line_count(),
            GenericCache::None(c) => c.get_uninitialised_line_count()
        }
    }
}