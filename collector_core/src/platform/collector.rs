use std::io::Read;
use std::path::PathBuf;

use filetime::FileTime;
use glob::glob;
use sha1::{Digest, Sha1};
use tokio::task::spawn_blocking;

use crate::csv::{CsvLogFile, CsvLogItem};
use crate::error::{CollectorError, Result};
use crate::extract::extract_file;
use crate::utils::{FormatSource, HASH_BUFFER_SIZE, require_admin};
use crate::writer::Writer;

#[cfg(target_os = "windows")]
use crate::mount::VssSnapshot;

/// Collection statistics
#[derive(Debug, Clone, Default)]
pub struct CollectionStats {
    pub files_collected: u64,
    pub bytes_collected: u64,
    pub filesystem_extractions: u64,
    pub ntfs_extractions: u64,
    pub failed_extractions: u64,
    pub patterns_processed: u64,
}

impl CollectionStats {
    /// Merge another stats into this one
    pub fn merge(&mut self, other: &CollectionStats) {
        self.files_collected += other.files_collected;
        self.bytes_collected += other.bytes_collected;
        self.filesystem_extractions += other.filesystem_extractions;
        self.ntfs_extractions += other.ntfs_extractions;
        self.failed_extractions += other.failed_extractions;
        self.patterns_processed += other.patterns_processed;
    }
}

/// Main artifact collector
pub struct ArtifactCollector {
    source_directory: FormatSource,
    artifact_patterns: Vec<String>,
    writer: Writer,
    csv_logger: CsvLogFile,
    stats: CollectionStats,
    #[cfg(target_os = "windows")]
    vss_snapshot: Option<VssSnapshot>,
}

impl ArtifactCollector {
    /// Create a new collector
    pub async fn new<S, D>(source: S, destination: D, patterns: Vec<String>) -> Result<Self>
    where
        S: Into<PathBuf>,
        D: Into<PathBuf>,
    {
        let source_path = source.into();
        let dest_path = destination.into();

        let writer = Writer::new(&dest_path)?;
        let csv_path = writer.csv_log_path();
        writer.create_file("Collector_copy.csv").await?;
        let csv_logger = CsvLogFile::new(&csv_path).await?;

        Ok(Self {
            source_directory: FormatSource::new(source_path),
            artifact_patterns: patterns,
            writer,
            csv_logger,
            stats: CollectionStats::default(),
            #[cfg(target_os = "windows")]
            vss_snapshot: None,
        })
    }

    /// Set VSS snapshot for Windows
    #[cfg(target_os = "windows")]
    pub fn with_vss_snapshot(mut self, snapshot: VssSnapshot) -> Self {
        self.vss_snapshot = Some(snapshot);
        self
    }

    /// Get current statistics
    pub fn stats(&self) -> &CollectionStats {
        &self.stats
    }

    /// Get the writer
    pub fn writer(&self) -> &Writer {
        &self.writer
    }

    /// Count total files matching all patterns (before collection)
    pub fn count_files(&self) -> u64 {
        self.get_all_files().len() as u64
    }

    /// Get all files matching patterns
    fn get_all_files(&self) -> Vec<PathBuf> {
        let mut files = Vec::new();

        for pattern in &self.artifact_patterns {
            let normalized = pattern.trim_start_matches('\\').trim_start_matches('/');
            let source_pattern = self.source_directory.join(normalized).to_string_lossy();

            if let Ok(entries) = glob(&source_pattern) {
                files.extend(entries.filter_map(|e| e.ok()).filter(|p| p.is_file()));
            }
        }

        files
    }

    /// Collect all artifacts (no progress callback)
    pub async fn collect(&mut self) -> Result<CollectionStats> {
        self.collect_internal::<fn(u64, u64, &str)>(None).await
    }

    /// Collect all artifacts with progress callback
    pub async fn collect_with_progress<F>(&mut self, callback: F) -> Result<CollectionStats>
    where
        F: Fn(u64, u64, &str) + Send + Sync + 'static,
    {
        self.collect_internal(Some(callback)).await
    }

    /// Internal collection implementation
    async fn collect_internal<F>(&mut self, callback: Option<F>) -> Result<CollectionStats>
    where
        F: Fn(u64, u64, &str),
    {
        require_admin()?;

        log::info!("Starting collection from {}", self.source_directory);

        let files = self.get_all_files();
        let total = files.len() as u64;

        log::info!("Found {} files to collect", total);

        for (index, file) in files.iter().enumerate() {
            let current = index as u64 + 1;

            if let Some(ref cb) = callback {
                cb(current, total, &file.to_string_lossy());
            }

            if let Err(e) = self.process_file(file).await {
                log::error!("Failed to process {}: {}", file.display(), e);
                self.stats.failed_extractions += 1;
            }
        }

        log::info!(
            "Collection complete: {} files ({} bytes)",
            self.stats.files_collected,
            self.stats.bytes_collected
        );

        Ok(self.stats.clone())
    }

    /// Create ZIP archive
    pub async fn create_archive(&self, password: Option<String>) -> Result<()> {
        log::info!("Creating ZIP archive...");
        self.writer.create_archive(password).await
    }

    /// Process a single file
    async fn process_file(&mut self, source_path: &PathBuf) -> Result<()> {
        let relative_path = self.get_relative_path(source_path);
        let mut output_file = self.writer.create_file(&relative_path).await?;

        #[cfg(target_os = "windows")]
        let (bytes, used_ntfs) =
            extract_file(source_path, &mut output_file, self.vss_snapshot.as_ref()).await?;

        #[cfg(not(target_os = "windows"))]
        let (bytes, used_ntfs) = extract_file(source_path, &mut output_file, None).await?;

        self.stats.files_collected += 1;
        self.stats.bytes_collected += bytes;

        if used_ntfs {
            self.stats.ntfs_extractions += 1;
        } else {
            self.stats.filesystem_extractions += 1;
        }

        self.log_extraction(source_path, &relative_path, used_ntfs)
            .await?;
        Ok(())
    }

    /// Get relative path for destination
    fn get_relative_path(&self, source_path: &PathBuf) -> String {
        #[cfg(target_os = "windows")]
        {
            if let Some(ref vss) = self.vss_snapshot {
                if let Some(snapshot_id) = vss.snapshot_id() {
                    let path_str = source_path.to_string_lossy();
                    let source_str = self.source_directory.to_string_lossy();
                    return path_str.replace(&source_str, snapshot_id);
                }
            }
        }
        source_path.to_string_lossy().to_string()
    }

    /// Log extraction to CSV
    async fn log_extraction(
        &mut self,
        source: &PathBuf,
        destination: &str,
        from_ntfs: bool,
    ) -> Result<()> {
        let dest_path = self.writer.get_file_path(destination);

        let metadata = std::fs::metadata(&dest_path).map_err(|e| CollectorError::FileRead {
            path: dest_path.clone(),
            source: e,
        })?;

        let mtime = FileTime::from_last_modification_time(&metadata);
        let atime = FileTime::from_last_access_time(&metadata);

        // Hash computed in blocking thread (CPU-bound)
        let hash = Self::calculate_hash_blocking(dest_path.clone()).await?;

        let log_item = CsvLogItem::with_paths(
            source.to_string_lossy().to_string(),
            dest_path.to_string_lossy().to_string(),
        )
        .with_hash(hash)
        .with_ntfs_flag(from_ntfs)
        .with_timestamps(mtime.to_string(), atime.to_string())
        .with_size(metadata.len());

        self.csv_logger.add_row(log_item).await
    }

    /// Calculate SHA1 hash in a blocking thread (CPU-bound operation)
    async fn calculate_hash_blocking(path: PathBuf) -> Result<String> {
        spawn_blocking(move || {
            let mut file = std::fs::File::open(&path).map_err(|e| CollectorError::FileRead {
                path: path.clone(),
                source: e,
            })?;

            let mut hasher = Sha1::new();
            let mut buffer = [0u8; HASH_BUFFER_SIZE];

            loop {
                let bytes_read = file
                    .read(&mut buffer)
                    .map_err(|e| CollectorError::FileRead {
                        path: path.clone(),
                        source: e,
                    })?;

                if bytes_read == 0 {
                    break;
                }

                hasher.update(&buffer[..bytes_read]);
            }

            Ok(hex::encode(hasher.finalize()))
        })
        .await
        .map_err(|e| CollectorError::CollectionFailed(format!("Hash task failed: {}", e)))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collection_stats_default() {
        let stats = CollectionStats::default();
        assert_eq!(stats.files_collected, 0);
        assert_eq!(stats.bytes_collected, 0);
    }

    #[test]
    fn test_collection_stats_merge() {
        let mut stats1 = CollectionStats {
            files_collected: 10,
            bytes_collected: 1000,
            ..Default::default()
        };

        let stats2 = CollectionStats {
            files_collected: 5,
            bytes_collected: 500,
            ntfs_extractions: 3,
            ..Default::default()
        };

        stats1.merge(&stats2);

        assert_eq!(stats1.files_collected, 15);
        assert_eq!(stats1.bytes_collected, 1500);
        assert_eq!(stats1.ntfs_extractions, 3);
    }

    #[tokio::test]
    async fn test_artifact_collector_new() {
        let temp_dir = tempfile::tempdir().unwrap();
        let source = temp_dir.path().join("source");
        let dest = temp_dir.path().join("dest");

        std::fs::create_dir_all(&source).unwrap();
        std::fs::create_dir_all(&dest).unwrap();

        let collector = ArtifactCollector::new(&source, &dest, vec!["*.txt".to_string()]).await;

        assert!(collector.is_ok());
    }
}
