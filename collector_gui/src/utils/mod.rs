#[cfg(target_os = "linux")]
mod values_linux;
#[cfg(target_os = "windows")]
mod values_windows;

#[cfg(target_os = "linux")]
pub use values_linux::*;

#[cfg(target_os = "windows")]
pub use values_windows::*;
