use serde::Deserialize;

/// A cache configuration with multiple layers
#[derive(Debug, Deserialize)]
pub struct LayeredCacheConfig {
    pub caches: Vec<CacheConfig>,
}

/// A configuration for a single cache
#[derive(Debug, Deserialize)]
pub struct CacheConfig {
    pub name: String,
    pub size: u64,
    pub line_size: u64,
    pub kind: CacheKindConfig,
    #[serde(default = "ReplacementPolicyConfig::default")]
    pub replacement_policy: ReplacementPolicyConfig,
}

/// The kind of cache - direct, full, 2way, 4way, or 8way
#[derive(Debug, Deserialize)]
pub enum CacheKindConfig {
    #[serde(alias = "direct")]
    Direct,
    #[serde(alias = "full")]
    Full,
    #[serde(alias = "2way")]
    TwoWay,
    #[serde(alias = "4way")]
    FourWay,
    #[serde(alias = "8way")]
    EightWay,
}

/// The replacement policy, if applicable - round robin, lru, or lfu. Defaults to round robin.
#[derive(Debug, Copy, Clone, Deserialize)]
pub enum ReplacementPolicyConfig {
    #[serde(alias = "rr")]
    RoundRobin,
    #[serde(alias = "lru")]
    LeastRecentlyUsed,
    #[serde(alias = "lfu")]
    LeastFrequentlyUsed,
}

impl Default for ReplacementPolicyConfig {
    fn default() -> Self {
        ReplacementPolicyConfig::RoundRobin
    }
}
