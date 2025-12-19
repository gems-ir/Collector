use std::path::PathBuf;
use thiserror::Error;

/// Main error type for collector operations.
#[derive(Error, Debug)]
pub enum CollectorError {
    // I/O Errors
    #[error("Failed to read file '{path}': {source}")]
    FileRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to write file '{path}': {source}")]
    FileWrite {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to create directory '{path}': {source}")]
    DirectoryCreate {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Path does not exist: {0}")]
    PathNotFound(PathBuf),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    // Permission Errors
    #[error("Administrator/root privileges required")]
    InsufficientPrivileges,

    // NTFS Errors
    #[error("NTFS extraction failed for '{path}': {reason}")]
    NtfsExtraction { path: PathBuf, reason: String },

    #[error("Invalid drive letter: {0}")]
    InvalidDriveLetter(String),

    #[error("NTFS error: {0}")]
    NtfsError(String),

    #[error("Sector reader error: {0}")]
    SectorReaderError(String),

    // VSS Errors
    #[error("VSS operation failed: {0}")]
    VssOperation(String),

    #[error("No VSS snapshots found for drive '{0}'")]
    NoVssSnapshots(String),

    #[error("VSS COM initialization failed: {0}")]
    VssComInit(String),

    #[error("Failed to mount VSS snapshot: {0}")]
    VssMountFailed(String),

    // Resource Errors
    #[error("Failed to parse resource '{path}': {reason}")]
    ResourceParse { path: PathBuf, reason: String },

    #[error("Resource not found: '{0}'")]
    ResourceNotFound(String),

    #[error("Invalid resource '{name}': {reason}")]
    InvalidResource { name: String, reason: String },

    #[error("Resources directory not found: {0}")]
    ResourcesDirectoryNotFound(PathBuf),

    #[error("No resources specified")]
    NoResourcesSpecified,

    // ZIP Errors
    #[error("ZIP operation failed: {0}")]
    ZipError(#[from] zip::result::ZipError),

    #[error("Failed to create ZIP '{path}': {reason}")]
    ZipCreation { path: PathBuf, reason: String },

    // CSV Errors
    #[error("CSV error: {0}")]
    CsvError(String),

    // Pattern Errors
    #[error("Invalid pattern '{pattern}': {reason}")]
    InvalidPattern { pattern: String, reason: String },

    #[error("Glob error: {0}")]
    GlobError(#[from] glob::GlobError),

    #[error("Pattern error: {0}")]
    PatternError(#[from] glob::PatternError),

    // Config Errors
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Invalid source path: {0}")]
    InvalidSourcePath(PathBuf),

    #[error("Invalid destination path: {0}")]
    InvalidDestinationPath(PathBuf),

    // Collection Errors
    #[error("Collection failed: {0}")]
    CollectionFailed(String),

    #[error("No files matched patterns")]
    NoFilesMatched,

    #[error("Failed to extract '{source_path}' to '{destination}': {reason}")]
    ExtractionFailed {
        source_path: PathBuf,
        destination: PathBuf,
        reason: String,
    },
}

pub type Result<T> = std::result::Result<T, CollectorError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = CollectorError::PathNotFound(PathBuf::from("/some/path"));
        assert!(err.to_string().contains("/some/path"));
    }

    #[test]
    fn test_error_chain() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = CollectorError::FileRead {
            path: PathBuf::from("/test"),
            source: io_err,
        };
        assert!(err.to_string().contains("/test"));
    }

    #[test]
    fn test_insufficient_privileges_error() {
        let err = CollectorError::InsufficientPrivileges;
        assert!(err.to_string().contains("Administrator"));
    }

    #[test]
    fn test_resource_not_found_error() {
        let err = CollectorError::ResourceNotFound("MFT".to_string());
        assert!(err.to_string().contains("MFT"));
    }
}
