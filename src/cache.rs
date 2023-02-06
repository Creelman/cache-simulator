use crate::config::ReplacementPolicyConfig;

pub struct Cache {
    set_selection_bit_mask: u64,
    tag_selection_bit_mask: u64,
    cache_alignment_bit_mask: u64,
    line_size: u64,
    cache: Vec<CacheLineMetadata>,
    replacement_policy: CacheReplacementPolicy,
    cache_alignment_bits: u8,
    set_size: u64,
}

#[derive(Clone, Debug, Default)]
pub struct CacheLineMetadata {
    pub tag: u64,
}

// We assume <= 2^64 instructions in the input
pub enum CacheReplacementPolicy {
    RoundRobin(Vec<u64>),
    LeastRecentlyUsed(Vec<u64>, u64),
    LeastFrequentlyUsed(Vec<u64>),
    // Used for direct mapped, not technically necessary but saves a few cycles and some memory
    None
}

impl CacheReplacementPolicy {
    pub fn new(policy: Option<ReplacementPolicyConfig>, num_sets: u64, num_lines: u64) -> Self {
        match policy {
            Some(ReplacementPolicyConfig::RoundRobin) => Self::RoundRobin(vec![0; num_sets as usize]),
            Some(ReplacementPolicyConfig::LeastRecentlyUsed) => Self::LeastRecentlyUsed(vec![0; num_lines as usize], 1),
            Some(ReplacementPolicyConfig::LeastFrequentlyUsed) => Self::LeastFrequentlyUsed(vec![0; num_lines as usize]),
            None => Self::None
        }
    }

    pub fn update_on_read(&mut self, cache_index: u64) {
        match self {
            CacheReplacementPolicy::LeastRecentlyUsed(times, current) => {
                times[cache_index as usize] = *current;
                *current += 1;
            }
            CacheReplacementPolicy::LeastFrequentlyUsed(usages_map) => {
                usages_map[cache_index as usize] += 1;
            }
            // Nothing to do for round robin or none
            _ => {}
        }
    }

    pub fn update_and_get_line_to_write(&mut self, cache_lines_per_set: u64, set: u64) -> u64 {
        let set_offset = (set * cache_lines_per_set) as usize;
        match self {
            CacheReplacementPolicy::RoundRobin(set_indices) => {
                let set_index = &mut set_indices[set as usize];
                let val = set_offset as u64 + *set_index;
                *set_index = (*set_index + 1) % cache_lines_per_set;
                val
            }
            CacheReplacementPolicy::LeastFrequentlyUsed(usages_map) => {
                let slice = &mut usages_map[set_offset..set_offset + cache_lines_per_set as usize];
                let (index, value) = slice.iter_mut().enumerate().min_by(|(_, v1), (_, v2)| v1.cmp(v2)).unwrap();
                *value = 1;
                (set_offset + index) as u64
            }
            CacheReplacementPolicy::LeastRecentlyUsed(times, current) => {
                let slice = &mut times[set_offset..set_offset + cache_lines_per_set as usize];
                let (index, value) = slice.iter_mut().enumerate().min_by(|(_, v1), (_, v2)| v1.cmp(v2)).unwrap();
                *value = *current;
                *current += 1;
                (set_offset + index) as u64
            }
            CacheReplacementPolicy::None => set_offset as u64
        }
    }
}

impl Cache {
    pub fn new(size: u64, line_size: u64, num_sets: u64, policy: Option<ReplacementPolicyConfig>) -> Self {
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
            cache: vec![CacheLineMetadata::default(); cache_lines as usize],
            replacement_policy: CacheReplacementPolicy::new(policy, num_sets, cache_lines),
        }
    }

    #[inline(always)]
    fn address_to_set_and_tag(&self, input: u64) -> (u64, u64) {
        (((input & self.set_selection_bit_mask) >> self.cache_alignment_bits), input & (self.tag_selection_bit_mask))
    }

    #[inline(never)]
    // Cache hit is true, cache miss is false
    pub fn read_and_update_line(&mut self, input: u64) -> bool {
        let (set, tag) = self.address_to_set_and_tag(input);
        let set_inclusive_lower_bound = set * self.set_size;
        let set_exclusive_upper_bound = set_inclusive_lower_bound + self.set_size;
        // Only search the relevant set
        for (index, line) in &mut self.cache[set_inclusive_lower_bound as usize..set_exclusive_upper_bound as usize].iter().enumerate() {
            // Cache hit
            if line.tag == tag {
                // Update replacement policy, report hit
                self.replacement_policy.update_on_read(set_inclusive_lower_bound + index as u64);
                return true;
            }
        }
        // Cache miss, update
        let line = self.replacement_policy.update_and_get_line_to_write(self.set_size, set);
        self.cache[line as usize] = CacheLineMetadata {
            tag
        };
        false
    }

    pub fn get_alignment_bit_mask(&self) -> u64 {
        self.cache_alignment_bit_mask
    }

    pub fn get_line_size(&self) -> u64 {
        self.line_size
    }

    pub fn get_uninitialised_line_count(&self) -> usize {
        self.cache.iter().filter(|a| a.tag == 0).count()
    }
}