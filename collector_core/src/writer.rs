use std::io::Write;
use std::path::{Path, PathBuf};

use sysinfo::System;
use tokio::fs::{self, File, OpenOptions};
use tokio::io::AsyncReadExt;
use walkdir::WalkDir;
use zip::{AesMode, ZipWriter, write::SimpleFileOptions};

use crate::error::{CollectorError, Result};
use crate::utils::{FormatSource, normalize_path};

const ZIP_BUFFER_SIZE: usize = 4096;

pub struct Writer {
    base_destination: FormatSource,
    full_destination: FormatSource,
    hostname: String,
}

impl Writer {
    pub fn new<P: AsRef<Path>>(destination_path: P) -> Result<Self> {
        let hostname = System::host_name().unwrap_or_else(|| "unknown".to_string());
        let base = FormatSource::new(destination_path.as_ref());
        let folder_name = format!("Collector_{}", hostname);
        let full = base.join(&folder_name);

        Ok(Self {
            base_destination: base,
            full_destination: full,
            hostname,
        })
    }

    pub fn with_folder_name<P: AsRef<Path>, S: AsRef<str>>(
        destination_path: P,
        folder_name: S,
    ) -> Result<Self> {
        let base = FormatSource::new(destination_path.as_ref());
        let full = base.join(folder_name.as_ref());

        Ok(Self {
            base_destination: base,
            full_destination: full,
            hostname: folder_name.as_ref().to_string(),
        })
    }

    pub fn base_destination(&self) -> &FormatSource {
        &self.base_destination
    }

    pub fn full_destination(&self) -> &FormatSource {
        &self.full_destination
    }

    pub fn hostname(&self) -> &str {
        &self.hostname
    }

    pub fn get_file_path<S: AsRef<str>>(&self, relative_path: S) -> PathBuf {
        let normalized = normalize_path(relative_path.as_ref());
        self.full_destination.join(&normalized).to_path_buf()
    }

    pub fn get_file_path_string<S: AsRef<str>>(&self, relative_path: S) -> String {
        self.get_file_path(relative_path)
            .to_string_lossy()
            .to_string()
    }

    pub async fn create_file<S: AsRef<str>>(&self, relative_path: S) -> Result<File> {
        let file_path = self.get_file_path(&relative_path);
        self.create_parent_dirs(&relative_path).await?;

        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&file_path)
            .await
            .map_err(|e| CollectorError::FileWrite {
                path: file_path,
                source: e,
            })
    }

    pub async fn create_parent_dirs<S: AsRef<str>>(&self, relative_path: S) -> Result<()> {
        let mut dir_path = self.get_file_path(&relative_path);
        dir_path.pop();

        if !dir_path.exists() {
            fs::create_dir_all(&dir_path)
                .await
                .map_err(|e| CollectorError::DirectoryCreate {
                    path: dir_path,
                    source: e,
                })?;
        }

        Ok(())
    }

    pub async fn create_directory<S: AsRef<str>>(&self, relative_path: S) -> Result<()> {
        let dir_path = self.get_file_path(&relative_path);

        if !dir_path.exists() {
            fs::create_dir_all(&dir_path)
                .await
                .map_err(|e| CollectorError::DirectoryCreate {
                    path: dir_path,
                    source: e,
                })?;
        }

        Ok(())
    }

    pub async fn create_archive(&self, password: Option<String>) -> Result<()> {
        let zip_name = format!("Collector_{}.zip", self.hostname);
        let zip_path = self.base_destination.join(&zip_name).to_path_buf();

        let file = std::fs::File::create(&zip_path).map_err(|e| CollectorError::ZipCreation {
            path: zip_path.clone(),
            reason: e.to_string(),
        })?;

        let mut zip = ZipWriter::new(file);

        let mut options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o644);

        if let Some(ref pwd) = password {
            options = options.with_aes_encryption(AesMode::Aes256, pwd);
        }

        let full_dest_path = self.full_destination.to_path_buf();
        let mut buffer = vec![0u8; ZIP_BUFFER_SIZE];

        for entry in WalkDir::new(&full_dest_path) {
            let entry = entry.map_err(|e| CollectorError::ZipCreation {
                path: zip_path.clone(),
                reason: e.to_string(),
            })?;

            let path = entry.path();
            let relative_path =
                path.strip_prefix(&full_dest_path)
                    .map_err(|e| CollectorError::ZipCreation {
                        path: path.to_path_buf(),
                        reason: e.to_string(),
                    })?;

            if path.is_file() {
                zip.start_file_from_path(relative_path, options.clone())?;

                let mut file =
                    fs::File::open(path)
                        .await
                        .map_err(|e| CollectorError::FileRead {
                            path: path.to_path_buf(),
                            source: e,
                        })?;

                loop {
                    let bytes_read =
                        file.read(&mut buffer)
                            .await
                            .map_err(|e| CollectorError::FileRead {
                                path: path.to_path_buf(),
                                source: e,
                            })?;

                    if bytes_read == 0 {
                        break;
                    }

                    zip.write_all(&buffer[..bytes_read])?;
                }
            } else if !relative_path.as_os_str().is_empty() {
                zip.add_directory_from_path(relative_path, options.clone())?;
            }
        }

        zip.finish()?;

        fs::remove_dir_all(&full_dest_path)
            .await
            .map_err(|e| CollectorError::DirectoryCreate {
                path: full_dest_path,
                source: e,
            })?;

        log::info!("Created ZIP: {}", zip_path.display());
        Ok(())
    }

    pub fn csv_log_path(&self) -> PathBuf {
        self.get_file_path("Collector_copy.csv")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_writer_new() {
        let writer = Writer::new("./output");
        assert!(writer.is_ok());
    }

    #[test]
    fn test_writer_with_folder_name() {
        let writer = Writer::with_folder_name("./output", "CustomFolder").unwrap();
        assert!(
            writer
                .full_destination()
                .to_string_lossy()
                .contains("CustomFolder")
        );
    }

    #[test]
    fn test_get_file_path_normalization() {
        let writer = Writer::with_folder_name("./output", "Test").unwrap();

        let path = writer.get_file_path("C:\\Windows\\System32\\file.txt");
        let path_str = path.to_string_lossy();

        assert!(!path_str.contains(':'));
        assert!(path_str.contains("Windows"));
        assert!(path_str.contains("file.txt"));
    }

    #[tokio::test]
    async fn test_create_file() {
        let temp_dir = tempdir().unwrap();
        let writer = Writer::with_folder_name(temp_dir.path(), "Test").unwrap();

        let result = writer.create_file("test/nested/file.txt").await;
        assert!(result.is_ok());

        let file_path = writer.get_file_path("test/nested/file.txt");
        assert!(file_path.exists());
    }

    #[tokio::test]
    async fn test_create_directory() {
        let temp_dir = tempdir().unwrap();
        let writer = Writer::with_folder_name(temp_dir.path(), "Test").unwrap();

        let result = writer.create_directory("test/nested/dir").await;
        assert!(result.is_ok());

        let dir_path = writer.get_file_path("test/nested/dir");
        assert!(dir_path.is_dir());
    }

    #[test]
    fn test_csv_log_path() {
        let writer = Writer::with_folder_name("./output", "Test").unwrap();
        let csv_path = writer.csv_log_path();

        assert!(csv_path.to_string_lossy().contains("Collector_copy.csv"));
    }
}
