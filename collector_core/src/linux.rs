use crate::csv::{CsvLogFile, CsvLogItem};
use crate::extract::try_filesystem;
use crate::utils::{is_admin, FormatSource};
use crate::writer::Writer;

use anyhow::Result;
use filetime::FileTime;
use glob::glob;
use sha1::{Digest, Sha1};
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncReadExt;


pub struct Collect {
    pub source_directory: FormatSource,
    pub destination_directory: FormatSource,
    pub artifact_patterns: Vec<String>,
    writer: Writer,
    csv_logger: CsvLogFile,
}

impl Collect {
    pub async fn new(source: String, destination: String, patterns: Vec<String>) -> Collect {
        let writer = Writer::new(destination.clone());
        let csv_filename = writer.get_filepath_as_str("Collector_copy.csv".into());
        let _ = writer.create_file("Collector_copy.csv".into()).await;
        Collect {
            source_directory: FormatSource::from(source),
            destination_directory: FormatSource::from(destination),
            artifact_patterns: patterns,
            writer,
            csv_logger: CsvLogFile::new(csv_filename).await,
        }
    }

    pub async fn start(&mut self) {
        if !is_admin() {
            panic!("You need to run as Administrator!");
        }
        for pattern in self.artifact_patterns.clone() {
            let mut normalized_pattern = pattern;
            if normalized_pattern.starts_with("/") {
                normalized_pattern.remove(0);
            }
            
            let source_path = self.source_directory.clone().push(normalized_pattern.as_str()).to_string();
            for entry in Self::fetch_entries(source_path) {
                if let Err(e) = self.process_entry(entry).await {
                    eprintln!("Error for entry : {:?}", e);
                }
            }
        }
    }


    /// Process a single entry
    async fn process_entry(&mut self, entry: PathBuf) -> Result<()> {
        let mut normalized_entry = entry.clone();
        if normalized_entry.starts_with("/") {
            normalized_entry = normalized_entry.strip_prefix("/")?.to_path_buf();
        }

        let mut output_file: File = self.writer.create_file(normalized_entry.clone().to_string_lossy().to_string()).await;
        // Filesystem approach
        if self.process_filesystem(&mut normalized_entry.clone(), &mut output_file, normalized_entry.clone().to_string_lossy().to_string())
            .await
            .is_ok()
        {
            return Ok(());
        }
        Ok(())
    }

    /// Process file using filesystem
    async fn process_filesystem(
        &mut self,
        entry_path: &mut PathBuf,
        output_file: &mut File,
        entry_name: String,
    ) -> Result<(), ()> {
        if try_filesystem(entry_path.clone(), output_file).await.is_ok() {
            let destination_path = self.writer.get_filepath_as_str(entry_name.clone());
            self.write_csv_row(entry_name, destination_path.to_string(), false).await;
            return Ok(());
        }
        Err(())
    }

    /// Get all entries matching the glob
    fn fetch_entries(pattern: String) -> Vec<PathBuf> {
        glob(&pattern)
            .expect("Error to parse pattern")
            .filter_map(Result::ok)
            .filter(|p| p.is_file()) // Filter only files
            .collect()
    }

    pub async fn zip(&mut self, zip_password: Option<String>) -> Result<()> {
        self.writer.zip(zip_password).await
    }

    async fn write_csv_row(&mut self, source_artifact: String, destination_artifact: String, from_ntfs: bool) {
        let mut log_item: CsvLogItem = Default::default();
        log_item.source_file = source_artifact.clone();
        log_item.destination_file = destination_artifact.clone();

        let metadata = std::fs::metadata(&destination_artifact).expect("Failed to extract metadata");
        let mtime = FileTime::from_last_modification_time(&metadata);
        let atime = FileTime::from_last_access_time(&metadata);

        log_item.modfied_time = mtime.to_string();
        log_item.access_time = atime.to_string();
        log_item.from_ntfs = from_ntfs;


        let mut file = File::open(destination_artifact).await.unwrap();
        let mut hasher = Sha1::new();
        let mut buffer = [0u8; 4092];
        loop {
            let bytes_read = file.read(&mut buffer).await;
            if bytes_read.unwrap() == 0 {
                break;
            }
            hasher.update(buffer);
        }
        log_item.hasfile_sha256 = hex::encode(hasher.finalize());

        let _ = self.csv_logger.add_row_struct(log_item).await;
    }
}