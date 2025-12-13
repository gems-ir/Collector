//! Fast and secure artifact collector for digital forensics.

pub mod csv;
pub mod error;
pub mod platform;
pub mod resource;
pub mod utils;
pub mod writer;

mod extract;

#[cfg(target_os = "windows")]
pub mod mount;

pub mod prelude {
    pub use crate::csv::{CsvLogFile, CsvLogItem};
    pub use crate::error::{CollectorError, Result};
    pub use crate::platform::{ArtifactCollector, CollectionStats, VssCollector};
    pub use crate::resource::{ResourcesParser, YamlArtifact, YamlParser};
    pub use crate::utils::{FormatSource, is_admin, require_admin};
    pub use crate::writer::Writer;
}

// Re-exports for convenience
pub use error::{CollectorError, Result};
pub use platform::{ArtifactCollector, CollectionStats, VssCollector};
pub use resource::{ResourcesParser, YamlParser};
