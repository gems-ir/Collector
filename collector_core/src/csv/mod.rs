//! CSV logging for artifact collection.

use chrono::Utc;
use csv_async::{AsyncReader, AsyncSerializer, AsyncWriterBuilder};
use serde::Serialize;
use std::path::Path;
use tokio::fs::{File, OpenOptions};

use crate::error::{CollectorError, Result};

#[derive(Debug, Serialize, Clone)]
pub struct CsvLogItem {
    pub collect_time: String,
    pub source_file: String,
    pub destination_file: String,
    pub hash_sha1: String,
    pub from_ntfs: bool,
    pub modified_time: String,
    pub access_time: String,
    pub file_size: u64,
}

impl Default for CsvLogItem {
    fn default() -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            collect_time: now.clone(),
            source_file: String::new(),
            destination_file: String::new(),
            hash_sha1: String::new(),
            from_ntfs: false,
            modified_time: now.clone(),
            access_time: now,
            file_size: 0,
        }
    }
}

impl CsvLogItem {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_paths<S: Into<String>, D: Into<String>>(source: S, destination: D) -> Self {
        Self {
            source_file: source.into(),
            destination_file: destination.into(),
            ..Default::default()
        }
    }

    pub fn with_hash(mut self, hash: String) -> Self {
        self.hash_sha1 = hash;
        self
    }

    pub fn with_ntfs_flag(mut self, from_ntfs: bool) -> Self {
        self.from_ntfs = from_ntfs;
        self
    }

    pub fn with_timestamps(mut self, modified: String, access: String) -> Self {
        self.modified_time = modified;
        self.access_time = access;
        self
    }

    pub fn with_size(mut self, size: u64) -> Self {
        self.file_size = size;
        self
    }
}

pub struct CsvLogFile {
    csv_writer: AsyncSerializer<File>,
    file_path: String,
}

impl CsvLogFile {
    pub async fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_str = path.as_ref().to_string_lossy().to_string();

        let csv_file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&path)
            .await
            .map_err(|e| CollectorError::CsvError(format!("Failed to open CSV: {}", e)))?;

        let has_headers = Self::check_has_headers(&path_str).await;

        let csv_writer = if has_headers {
            let mut builder = AsyncWriterBuilder::new();
            builder.has_headers(false);
            builder.create_serializer(csv_file)
        } else {
            AsyncWriterBuilder::new().create_serializer(csv_file)
        };

        Ok(Self {
            csv_writer,
            file_path: path_str,
        })
    }

    pub async fn add_row(&mut self, item: CsvLogItem) -> Result<()> {
        self.csv_writer
            .serialize(item)
            .await
            .map_err(|e| CollectorError::CsvError(format!("Failed to write row: {}", e)))?;
        Ok(())
    }

    pub fn file_path(&self) -> &str {
        &self.file_path
    }

    async fn check_has_headers(path: &str) -> bool {
        let path = Path::new(path);
        if !path.exists() {
            return false;
        }

        let Ok(csv_file) = OpenOptions::new().read(true).open(path).await else {
            return false;
        };

        let mut reader = AsyncReader::from_reader(csv_file);
        matches!(reader.headers().await, Ok(h) if !h.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_log_item_default() {
        let item = CsvLogItem::default();
        assert!(item.source_file.is_empty());
        assert!(item.destination_file.is_empty());
        assert!(!item.from_ntfs);
    }

    #[test]
    fn test_csv_log_item_with_paths() {
        let item = CsvLogItem::with_paths("source.txt", "dest.txt");
        assert_eq!(item.source_file, "source.txt");
        assert_eq!(item.destination_file, "dest.txt");
    }

    #[test]
    fn test_csv_log_item_builder_pattern() {
        let item = CsvLogItem::with_paths("src", "dst")
            .with_hash("abc123".to_string())
            .with_ntfs_flag(true)
            .with_size(1024);

        assert_eq!(item.source_file, "src");
        assert_eq!(item.hash_sha1, "abc123");
        assert!(item.from_ntfs);
        assert_eq!(item.file_size, 1024);
    }

    #[tokio::test]
    async fn test_csv_log_file_creation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let csv_path = temp_dir.path().join("test.csv");

        let logger = CsvLogFile::new(&csv_path).await;
        assert!(logger.is_ok());
        assert!(csv_path.exists());
    }

    #[tokio::test]
    async fn test_csv_log_file_add_row() {
        let temp_dir = tempfile::tempdir().unwrap();
        let csv_path = temp_dir.path().join("test_rows.csv");

        let mut logger = CsvLogFile::new(&csv_path).await.unwrap();
        let item = CsvLogItem::with_paths("source", "dest");

        let result = logger.add_row(item).await;
        assert!(result.is_ok());
    }
}
