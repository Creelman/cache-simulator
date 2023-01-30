use serde::{Serialize, Deserialize};

#[derive(Deserialize)]
pub struct LayeredCacheConfig {
    caches: Vec<CacheConfig>,
}

#[derive(Deserialize)]
pub struct CacheConfig {
    name: String,
    size: usize,
    line_size: usize,
    kind: CacheKindConfig,
    #[serde(default = "ReplacementPolicy::default")]
    replacement_policy: ReplacementPolicyConfig,
}

#[derive(Deserialize)]
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
    EightWay
}

#[derive(Deserialize)]
pub enum ReplacementPolicyConfig {
    #[serde(alias = "rr")]
    RoundRobin,
    #[serde(alias = "lru")]
    LeastRecentlyUsed,
    #[serde(alias = "lfu")]
    LeadFrequentlyUsed
}

impl Default for ReplacementPolicyConfig {
    fn default() -> Self {
        ReplacementPolicyConfig::RoundRobin
    }
}