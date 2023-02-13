pub mod io;
pub mod cache;
pub mod config;
pub mod replacement_policies;
pub mod simulator;
// Generated from the build script
mod hex {
    include!(concat!(env!("OUT_DIR"), "/hex.rs"));
}
#[cfg(test)]
mod test;
pub mod util;