//! # CacheLib
//!
//! Cachelib is a library for arbitrary cache simulation
//!
//! It provides a generic cache implementation which can be parameterised by a replacement policy,
//! and provides a simulator to use these caches on inputs using the format in the specification
//!
//! While designed to accommodate high performance, it prioritises flexibility, being easy to
//! maintain and expand with new policies

/// Contains the implementation of the cache, and a utility enum for the existing cache types
pub mod cache;

/// Contains definitions for the JSON input format, which can be used with the provided replacement
/// policies
pub mod config;

/// Contains the provided replacement policies, with a trait for implementing custom replacement
/// policies
pub mod replacement_policies;

/// Contains the simulator used to simulate a program with a given cache configuration
pub mod simulator;
// Generated from the build.rs, private
mod hex {
    include!(concat!(env!("OUT_DIR"), "/hex.rs"));
}
#[cfg(test)]
mod test;

/// Contains utilities for running tests and benchmarks.
pub mod util;