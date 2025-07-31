mod utils;
mod csv;
pub mod resource;
mod writer;
mod extract;


#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::Collect;

#[cfg(target_family = "windows")]
mod windows;
#[cfg(target_family = "windows")]
pub mod windows_vss;
#[cfg(target_family = "windows")]
pub use windows::Collect;
#[cfg(target_family = "windows")]
mod mount;
