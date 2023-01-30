use crate::config::ReplacementPolicyConfig;

pub struct Cache {
    num_sets: u64,
    set_selection_bits: u8,
    set_selection_bit_mask: u64,
    tag_selection_bits: u8,
    tag_selection_bit_mask: u64,
    line_size: u64,
    cache: Vec<CacheLineMetadata>,
    replacement_policy: CacheReplacementPolicy,
}

#[derive(Clone)]
pub struct CacheLineMetadata {
    pub tag: u64,
    pub valid: bool,
}

impl Default for CacheLineMetadata {
    fn default() -> Self {
        Self {
            tag: 0,
            valid: false,
        }
    }
}

// We assume <= 2^64 instructions in the input
pub enum CacheReplacementPolicy {
    RoundRobin(Vec<u64>),
    LeastRecentlyUsed(Vec<u64>, u64),
    LeastFrequentlyUsed(Vec<u64>),
}

impl CacheReplacementPolicy {
    pub fn new(policy: ReplacementPolicyConfig, num_sets: u64, num_lines: u64) -> Self {
        match policy {
            ReplacementPolicyConfig::RoundRobin => Self::RoundRobin(vec![0, num_sets]),
            ReplacementPolicyConfig::LeastRecentlyUsed => Self::LeastRecentlyUsed(vec![0, num_lines], 0),
            ReplacementPolicyConfig::LeadFrequentlyUsed => Self::LeastFrequentlyUsed(vec![0, num_lines])
        }
    }

    pub fn update_on_read(&mut self, cache: &Cache) {
        match self {
            CacheReplacementPolicy::LeastRecentlyUsed(_, _) => { todo!() }
            CacheReplacementPolicy::LeastFrequentlyUsed(_) => { todo!() }
            // Nothing to do for round robin
            CacheReplacementPolicy::RoundRobin(_) => {}
        }
    }

    pub fn update_and_get_line_to_write(&mut self, _cache: &Cache, set: u64, _tag: u64) -> u64 {
        match self {
            CacheReplacementPolicy::RoundRobin(set_indices) => {
                let val = set_indices[set as usize];
                set_indices[set as usize] += 1;
                val
            }
            _ => { todo!() }
        }
    }
}

impl Cache {
    pub fn new(size: u64, line_size: u64, num_sets: u64, policy: ReplacementPolicyConfig) -> Self {
        let set_selection_bits = num_sets.ilog2() as u8;
        let cache_lines = size / line_size;
        Self {
            num_sets,
            set_selection_bits,
            set_selection_bit_mask: (2u64.pow(set_selection_bits as u32)) - 1,
            tag_selection_bits: 64 - set_selection_bits,
            tag_selection_bit_mask: ((2u64.pow(64 - set_selection_bits as u32)) - 1) << set_selection_bits,
            line_size,
            cache: vec![CacheLineMetadata::default(); cache_lines as usize],
            replacement_policy: CacheReplacementPolicy::new(policy, num_sets, cache_lines),
        }
    }

    #[inline(always)]
    fn address_to_set_and_tag(&self, input: u64) -> (u64, u64) {
        (input & self.set_selection_bit_mask, input & self.tag_selection_bit_mask)
    }

    #[inline(always)]
    fn address_to_set(&self, input: u64) -> u64 {
        input & self.set_selection_bit_mask
    }

    // Cache hit is true, cache miss is false
    fn read_and_update(&mut self, input: u64) -> bool {
        let (set, tag) = self.address_to_set_and_tag(input);
        let set_inclusive_lower_bound = set * self.num_sets;
        let set_exclusive_upper_bound = set_inclusive_lower_bound + self.num_sets;
        for line in &mut self.cache[set_inclusive_lower_bound as usize..set_exclusive_upper_bound as usize] {
            // Skip uninitialised lines
            if !line.valid { continue; }
            // Cache hit
            if line.tag == tag {
                // Update replacement policy, report hit
                self.replacement_policy.update_on_read(self);
                return true;
            }
        }
        // Cache miss, update
        let line = self.replacement_policy.update_and_get_line_to_write(self, set, tag);
        self.cache[line as usize] = CacheLineMetadata {
            tag,
            valid: true,
        };
        false
    }

    fn write(&mut self, input: u64) {
        let (set, tag) = self.address_to_set_and_tag(input);
        let line = self.replacement_policy.update_and_get_line_to_write(self, set, tag);
        self.cache[line] = CacheLineMetadata {
            tag,
            valid: true,
        };
    }
}