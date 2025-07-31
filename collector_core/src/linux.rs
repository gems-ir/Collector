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
    pub src: String,
    pub dst: String,
    pub artifacts_glob: Vec<String>,
    writer: Writer,
    csv_copy: CsvLogFile,
}

impl Collect {
    pub async fn new(src: String, dst: String, artifacts_glob: Vec<String>) -> Collect {
        let create_writer: Writer = Writer::new(dst.clone());
        let csv_filename = create_writer.get_filepath_as_str("Collector_copy.csv".into());
        let _create_csv = create_writer.create_file("Collector_copy.csv".into()).await;
        Collect {
            src,
            dst,
            artifacts_glob,
            writer: create_writer,
            csv_copy: CsvLogFile::new(csv_filename).await,
        }
    }

    pub async fn start(&mut self) {
        if !is_admin() {
            panic!("You need to run as Administrator!");
        }
        for artifact in self.artifacts_glob.clone() {
            let mut artifact_element = artifact.to_string();
            if artifact_element.starts_with("\\") {
                artifact_element.remove(0);
            }

            let src_path: PathBuf = FormatSource::from(&self.src).to_path().join(artifact_element);
            let source_with_artifact_out: &str = src_path.to_str().expect("Invalid path for artifact");

            for entry in Self::fetch_entries(&source_with_artifact_out) {
                if let Err(e) = self.process_entry(entry).await {
                    eprintln!("Error for entry : {:?}", e);
                }
            }
        }
    }


    /// Process a single entry
    async fn process_entry(&mut self, mut entry: PathBuf) -> Result<()> {
        let mut output_file: File = self.writer.create_file(entry.clone().to_string_lossy().to_string()).await;

        // Filesystem approach
        if self.process_filesystem(&mut entry.clone(), &mut output_file, entry.clone().to_string_lossy().to_string())
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
        to_entry: &mut PathBuf,
        output_file: &mut File,
        mod_entry: String,
    ) -> Result<(), ()> {
        if try_filesystem(to_entry.clone(), output_file).await.is_ok() {
            let filepath_art = self.writer.get_filepath_as_str(mod_entry.clone());
            self.write_csv_row(mod_entry, filepath_art.to_string(), false).await;
            return Ok(());
        }
        Err(())
    }

    /// Get all entries matching the glob
    fn fetch_entries(pattern: &str) -> Vec<PathBuf> {
        glob(pattern)
            .expect("Error to parse pattern")
            .filter_map(Result::ok)
            .filter(|p| p.is_file()) // Filter only files
            .collect()
    }

    pub async fn zip(&self, zip_password: Option<String>) -> Result<()> {
        let zipping = self.writer.zip(zip_password);
        zipping.await
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


        let mut get_file = File::open(destination_artifact).await.unwrap();
        let mut hasher = Sha1::new();
        let mut contents = [0u8; 4092];
        loop {
            let reader = get_file.read(&mut contents).await;
            if reader.unwrap() == 0 {
                break;
            }
            hasher.update(contents);
        }
        log_item.hasfile_sha256 = hex::encode(hasher.finalize());

        let _ = self.csv_copy.add_row_struct(log_item).await;
    }
}