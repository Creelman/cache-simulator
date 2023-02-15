/// A generic trait for implementing new replacement policies. Can be used to parameterise a Cache.
pub trait ReplacementPolicy {
    /// Updates the policy when a cache line is read
    ///
    /// Not applicable for some policies, a default which does nothing is provided
    ///
    /// # Arguments
    ///
    /// * `cache_index`: The index of the cache line which was read
    ///
    /// returns: ()
    ///
    fn update_on_read(&mut self, _cache_index: u64) {}


    /// Used by the cache to get a line number when a new line needs added to the cache.
    ///
    /// Implementations should assume that when this method is called, the cache line has been
    /// replaced
    ///
    /// # Arguments
    ///
    /// * `set_lower_bound_index`: The lower bound for the cache lines of the set. This is equal to
    /// set * cache_lines_per_set, but this allows it to be cached, as it is already known by the
    /// cache
    /// * `set`: The cache set
    /// * `cache_lines_per_set`: The number of cache lines per set
    ///
    /// returns: u64
    fn get_new_line(&mut self, set_lower_bound_index: u64, set: u64, cache_lines_per_set: u64) -> u64;
}

#[derive(Default)]
/// NoPolicy is used for direct mapped caches. It does nothing when updating on read, and simply
/// returns the set lower bound index when a new line is requested
///
/// As the generic cache implementation is monomorphised, the compiler can completely optimise this
/// away, removing the need for a separate implementation
pub struct NoPolicy;

impl ReplacementPolicy for NoPolicy {
    fn update_on_read(&mut self, _: u64) {}

    fn get_new_line(&mut self, set_lower_bound_index: u64, _set: u64, _cache_lines_per_set: u64) -> u64 {
        set_lower_bound_index
    }
}

/// Standard round robin replacement policy, which keeps separate indices for each set
pub struct RoundRobin {
    set_indices: Vec<u64>,
}

impl RoundRobin {
    pub fn new(num_sets: u64) -> Self {
        Self {
            set_indices: vec![0; num_sets as usize]
        }
    }
}

impl ReplacementPolicy for RoundRobin {
    fn update_on_read(&mut self, _: u64) {}

    fn get_new_line(&mut self, set_lower_bound_index: u64, set: u64, cache_lines_per_set: u64) -> u64 {
        let set_index = &mut self.set_indices[set as usize];
        let val = set_lower_bound_index + *set_index;
        *set_index = (*set_index + 1) % cache_lines_per_set;
        val
    }
}

/// Least Recently Used replacement policy
///
/// This implementation keeps track of when each line was last used, and also keeps track of a
/// logical clock, which is updated each time a line is used. This saves comparisons during search
/// for a new line, we already know what the timestamp should be
pub struct LeastRecentlyUsed {
    last_used_times: Vec<u64>,
    // Tracking logical time means we have fewer comparisons when finding a new line
    time: u64
}

impl LeastRecentlyUsed {
    pub fn new(num_lines: u64) -> Self {
        Self {
            last_used_times: vec![0; num_lines as usize],
            time: 0,
        }
    }
}

impl ReplacementPolicy for LeastRecentlyUsed {
    fn update_on_read(&mut self, cache_index: u64) {
        self.last_used_times[cache_index as usize] = self.time;
        self.time += 1;
    }

    fn get_new_line(&mut self, set_lower_bound_index: u64, _set: u64, cache_lines_per_set: u64) -> u64 {
        let slb = set_lower_bound_index as usize;
        let mut index = slb;
        let mut min_value = u64::MAX;
        let mut min_index = usize::MAX;
        while index < slb + cache_lines_per_set as usize {
            if self.last_used_times[index] < min_value {
                min_value = self.last_used_times[index];
                min_index = index;
            }
            index += 1;
        }
        self.last_used_times[min_index] = self.time;
        self.time += 1;
        (min_index) as u64
    }
}

/// Least frequently used replacement policy
pub struct LeastFrequentlyUsed {
    usages: Vec<u64>
}

impl LeastFrequentlyUsed {
    pub fn new(num_lines: u64) -> Self {
        Self {
            usages: vec![0; num_lines as usize]
        }
    }
}

impl ReplacementPolicy for LeastFrequentlyUsed {
    fn update_on_read(&mut self, cache_index: u64) {
        self.usages[cache_index as usize] += 1;
    }

    fn get_new_line(&mut self, set_lower_bound_index: u64, _set: u64, cache_lines_per_set: u64) -> u64 {
        let slb = set_lower_bound_index as usize;
        let mut index = slb;
        // Iterators surprisingly inefficient here, doing it manually halves the processing time for full_lfu
        // I believe the compiler can't see through .enumerate properly
        let mut min_value = u64::MAX;
        let mut min_index = usize::MAX;
        while index < slb + cache_lines_per_set as usize {
            if self.usages[index] < min_value {
                min_value = self.usages[index];
                min_index = index;
            }
            index += 1;
        }
        self.usages[min_index] = 1;
        (min_index) as u64
    }
}