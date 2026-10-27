pub mod vanilla_launch;

#[cfg(test)]
pub mod tests;

pub const LAUNCHER_VERSION: &str = env!("CARGO_PKG_VERSION");

pub use crate::launch::vanilla_launch::VanillaLauncher;