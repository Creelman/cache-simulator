pub trait ReplacementPolicy {
    fn update_on_read(&mut self, cache_index: u64);
    fn get_new_line(&mut self, set_lower_bound_index: u64, set: u64, cache_lines_per_set: u64) -> u64;
}

#[derive(Default)]
pub struct NoPolicy;

impl ReplacementPolicy for NoPolicy {
    fn update_on_read(&mut self, _: u64) {}

    fn get_new_line(&mut self, set_lower_bound_index: u64, _set: u64, _cache_lines_per_set: u64) -> u64 {
        set_lower_bound_index
    }
}

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
        let val = set_lower_bound_index as u64 + *set_index;
        *set_index = (*set_index + 1) % cache_lines_per_set;
        val
    }
}

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
        let slice = &mut self.last_used_times[slb..slb + cache_lines_per_set as usize];
        let (index, value) = slice.iter_mut().enumerate().min_by(|(_, v1), (_, v2)| v1.cmp(v2)).unwrap();
        *value = self.time;
        self.time += 1;
        (slb + index) as u64
    }
}

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
        let slice = &mut self.usages[slb..slb + cache_lines_per_set as usize];
        let (index, value) = slice.iter_mut().enumerate().min_by(|(_, v1), (_, v2)| v1.cmp(v2)).unwrap();
        *value = 1;
        (slb + index) as u64
    }
}