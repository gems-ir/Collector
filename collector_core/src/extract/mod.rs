//! File extraction module.

#[cfg(target_os = "windows")]
mod lowfs;
#[cfg(target_os = "windows")]
mod sector_reader;

mod get;

pub use get::*;
// extract_via_filesystem, extract_file
// cfg(windows) : extract_via_ntfs, , extract_file, get_drive_letter,
