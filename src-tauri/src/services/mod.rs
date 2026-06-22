pub mod audio;
pub mod brightness;
pub mod metrics;
pub mod network;
pub mod power;
pub mod power_supply;

#[cfg(target_os = "linux")]
pub mod mpris;
