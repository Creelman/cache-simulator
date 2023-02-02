use serde::{Serialize, Deserialize};

#[derive(Deserialize)]
pub struct LayeredCacheConfig {
    pub caches: Vec<CacheConfig>,
}

#[derive(Deserialize)]
pub struct CacheConfig {
    pub name: String,
    pub size: u64,
    pub line_size: u64,
    pub kind: CacheKindConfig,
    #[serde(default = "ReplacementPolicyConfig::default")]
    pub replacement_policy: ReplacementPolicyConfig,
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

#[derive(Copy, Clone, Deserialize)]
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