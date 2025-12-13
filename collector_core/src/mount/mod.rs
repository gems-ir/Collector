//! VSS support module (Windows only).

#[cfg(target_os = "windows")]
mod vss;

#[cfg(target_os = "windows")]
pub use vss::{DriveLetter, Vss, VssSnapshot};

#[cfg(not(target_os = "windows"))]
pub struct VssSnapshot;
