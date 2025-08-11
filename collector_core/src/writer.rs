use crate::utils::FormatSource;

use anyhow::Result;
use std::io::Write;
use std::path::{Path, PathBuf};
use sysinfo::System;
use tokio::{fs::{self, File, OpenOptions}, io::AsyncReadExt};
use walkdir::WalkDir;
use zip::{write::SimpleFileOptions, AesMode, ZipWriter};

#[derive(Debug, Clone)]
pub struct Writer {
    base_destination: FormatSource,
    full_destination: FormatSource,
}

impl Writer {
    pub fn new<P: AsRef<Path> + AsRef<str>>(destination_path: P) -> Self {
        let mut base_destination_formatting = FormatSource::from(destination_path);
        let hostname = System::host_name().unwrap();
        let formatted_path = base_destination_formatting.push(&format!("Collector_{}", hostname));
        Writer {
            base_destination: base_destination_formatting,
            full_destination: FormatSource::from(formatted_path.to_string()),
        }
    }

    /// Normalizes and combines the destination path with the given path name
    fn normalize_path(&self, path_name: &str) -> String {
        path_name.replace(":", "")
            .trim_start_matches('\\')
            .to_string()
    }

    /// Concatenation of destination with string parameter.
    pub fn get_filepath(&self, path_name: String) -> PathBuf {
        let normalized_path = self.normalize_path(&path_name);
        self.full_destination.clone().push(&normalized_path).to_path()
    }

    pub fn get_filepath_as_str(&self, path_name: String) -> String {
        self.get_filepath(path_name).to_str().unwrap().to_string()
    }

    /// Create a file in output directory.
    /// If string path as given, the entire path will be created.
    /// Output the file descriptor.
    pub async fn create_file(&self, path_name: String) -> File {
        let file_path = self.get_filepath(path_name.clone());
        self.create_folderpath(path_name).await;
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(file_path)
            .await
            .expect("Impossible to create output file")
    }

    /// Creates the entire directory path as given.
    pub async fn create_folderpath(&self, path_name: String) {
        let mut folder_path = self.get_filepath(path_name);
        folder_path.pop();
        if !folder_path.exists() {
            fs::create_dir_all(folder_path).await.expect("Failed to create directory");
        }
    }

    /// Zip the destination file by the given name.
    pub async fn zip(&mut self, zip_password: Option<String>) -> Result<()> {
        let hostname = System::host_name().unwrap();
        let zip_name = format!("Collector_{}.zip", hostname);
        let zip_path = self.base_destination.clone().push(&zip_name).to_string();

        // Create zip file
        let file = std::fs::File::create(&zip_path)?;
        let mut zip = ZipWriter::new(file);
        let mut options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o644);

        if let Some(ref pwd) = zip_password {
            options = options.with_aes_encryption(AesMode::Aes192, &pwd);
        }

        // Walk in output directory
        let mut buffer = vec![0; 4096]; // Pre-allocate buffer
        let walker = WalkDir::new(self.full_destination.clone().to_path()).into_iter();

        for entry in walker {
            let entry = entry?;
            let path = entry.path();
            let relative_path = path.strip_prefix(self.full_destination.clone().to_path())?;

            if path.is_file() {
                zip.start_file_from_path(relative_path, options.clone())?;
                let mut f = fs::File::open(path).await?;
                let bytes_read = f.read(&mut buffer).await?;
                zip.write_all(&buffer[..bytes_read])?;
            } else if !relative_path.as_os_str().is_empty() {
                zip.add_directory_from_path(relative_path, options.clone())?;
            }
        }

        zip.finish()?;
        fs::remove_dir_all(&self.full_destination.to_string()).await?;
        Ok(())
    }
}