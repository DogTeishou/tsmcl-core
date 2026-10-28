pub mod vanilla_launch;

#[cfg(test)]
pub mod tests;

pub const LAUNCHER_VERSION: &str = env!("CARGO_PKG_VERSION");
const LAUNCHER_VERSION_TYEP: &str = "TSMCL_Core";

pub use crate::launch::vanilla_launch::VanillaLauncher;