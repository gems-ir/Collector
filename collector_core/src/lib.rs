//! Fast and secure artifact collector for digital forensics.

pub mod error;
pub mod resource;
pub mod platform;
pub mod writer;
pub mod csv;
pub mod utils;

mod extract;

#[cfg(target_os = "windows")]
pub mod mount;

pub mod prelude {
    pub use crate::error::{CollectorError, Result};
    pub use crate::platform::{ArtifactCollector, CollectionStats, VssCollector};
    pub use crate::resource::{ResourcesParser, YamlArtifact, YamlParser};
    pub use crate::writer::Writer;
    pub use crate::csv::{CsvLogFile, CsvLogItem};
    pub use crate::utils::{is_admin, require_admin, FormatSource};
}

// Re-exports for convenience
pub use error::{CollectorError, Result};
pub use platform::{ArtifactCollector, CollectionStats, VssCollector};
pub use resource::{ResourcesParser, YamlParser};
