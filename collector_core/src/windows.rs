use crate::csv::{CsvLogFile, CsvLogItem};
use crate::extract::{try_filesystem, try_ntfs};
use crate::mount::vss_info::VSSObj;
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
    pub source_directory: String,
    pub destination_directory: String,
    pub artifact_patterns: Vec<String>,
    writer: Writer,
    vss_snapshot: Option<VSSObj>,
    csv_logger: CsvLogFile,
}

impl Collect {
    pub async fn new(src: String, dst: String, patterns: Vec<String>) -> Collect {
        let writer = Writer::new(dst.clone());
        let csv_filename = writer.get_filepath_as_str("Collector_copy.csv".into());
        let _ = writer.create_file("Collector_copy.csv".into()).await;
        Collect {
            source_directory: src,
            destination_directory: dst,
            artifact_patterns: patterns,
            writer,
            vss_snapshot: None,
            csv_logger: CsvLogFile::new(csv_filename).await,
        }
    }

    pub async fn start(&mut self) {
        if !is_admin() {
            panic!("You need to run as Administrator!");
        }
        for pattern in self.artifact_patterns.clone() {
            let mut normalized_pattern = pattern.to_string();
            if normalized_pattern.starts_with("\\") {
                normalized_pattern.remove(0);
            }

            let source_path: PathBuf = FormatSource::from(&self.source_directory).to_path().join(normalized_pattern);
            let source_with_artifact_out: &str = source_path.to_str().expect("Invalid path for artifact");

            for entry in Self::fetch_entries(&source_with_artifact_out) {
                if let Err(e) = self.process_entry(entry).await {
                    eprintln!("Error for entry : {:?}", e);
                }
            }
        }
    }


    /// Process a single entry
    async fn process_entry(&mut self, mut entry: PathBuf) -> Result<()> {
        let modified_entry = self.modify_entry_path(entry.to_str().unwrap().to_string());
        let mut output_file: File = self.writer.create_file(modified_entry.clone()).await;

        // Filesystem approach
        if self.process_filesystem(&mut entry, &mut output_file, modified_entry.clone())
            .await
            .is_ok()
        {
            return Ok(());
        }

        // NTFS approach
        self.process_ntfs(&entry.clone(), &mut output_file, modified_entry.clone()).await
    }

    /// Process file using filesystem
    async fn process_filesystem(
        &mut self,
        to_entry: &mut PathBuf,
        output_file: &mut File,
        modified_entry: String,
    ) -> Result<(), ()> {
        if try_filesystem(to_entry.clone(), output_file).await.is_ok() {
            let destination_path = self.writer.get_filepath_as_str(modified_entry.clone());
            self.write_csv_row(modified_entry, destination_path.to_string(), false).await;
            return Ok(());
        }
        Err(())
    }

    /// Process file using NTFS
    async fn process_ntfs(
        &mut self,
        to_entry: &PathBuf,
        output_file: &mut File,
        modified_entry: String,
    ) -> Result<(), anyhow::Error> {
        let snapshot = self.vss_snapshot.clone();
        let path = if let Some(_) = snapshot {
            Self::adjust_path_for_ntfs(to_entry, &self.source_directory)
        } else {
            to_entry.to_path_buf()
        };

        if try_ntfs(path, output_file, snapshot).await.is_ok() {
            let metadata = std::fs::metadata(to_entry)
                .expect("Failed to extract metadata");
            let mtime = FileTime::from_last_modification_time(&metadata);
            let atime = FileTime::from_last_access_time(&metadata);
            let resolver = self.writer.get_filepath(modified_entry.clone());

            filetime::set_file_times(resolver, atime, mtime)
                .expect("Failed to set time");

            let destination_path = self.writer.get_filepath_as_str(modified_entry.clone());
            self.write_csv_row(modified_entry, destination_path.to_string(), true).await;
        }
        Ok(())
    }

    /// Adjust the path for NTFS snapshots
    fn adjust_path_for_ntfs(to_entry: &PathBuf, source: &str) -> PathBuf {
        let entry_str = to_entry.to_str().unwrap();
        let add_backslash = format!("{}\\", source);
        PathBuf::from(entry_str.replace(&add_backslash, ""))
    }

    /// Modify entry path for VSS-specific cases
    fn modify_entry_path(&mut self, path: String) -> String {
        if let Some(ref vss_snapshot) = self.vss_snapshot {
            let vss_as_path = PathBuf::from(&vss_snapshot.device_volume_name);
            if let Some(vss_name) = vss_as_path.file_name() {
                return path.replace(&self.source_directory, vss_name.to_str().unwrap());
            }
        }
        path
    }

    /// Get all entries matching the glob
    fn fetch_entries(pattern: &str) -> Vec<PathBuf> {
        glob(pattern)
            .expect("Error to parse pattern")
            .filter_map(Result::ok)
            .filter(|p| p.is_file()) // Filter only files
            .collect()
    }

    pub async fn zip(&mut self, zip_password: Option<String>) -> Result<()> {
        let zipping = self.writer.zip(zip_password);
        zipping.await
    }

    pub(crate) fn vss(&mut self, vss_item: VSSObj) {
        self.vss_snapshot = Some(vss_item);
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