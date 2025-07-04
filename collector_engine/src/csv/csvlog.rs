
use std::path::Path;
use tokio::fs::{File,OpenOptions};
use serde::Serialize;
use csv_async::{AsyncWriterBuilder,AsyncSerializer,AsyncReader};
use chrono::Utc;

#[derive(Debug, Serialize, Clone)]
pub struct CsvLogItem{
	collect_time: String,
	pub source_file : String,
	pub destination_file : String,
	pub hasfile_sha256: String,
	pub from_ntfs: bool,
	pub modfified_time: String,
	pub access_time: String,
}

impl Default for CsvLogItem {
	fn default() -> Self {
		CsvLogItem{
			collect_time: Utc::now().to_rfc3339(),
			source_file : String::from(""),
			destination_file : String::from(""),
			hasfile_sha256: String::from(""),
			from_ntfs: false,
			modfified_time: Utc::now().to_rfc3339(),
			access_time: Utc::now().to_rfc3339(),
		}
	}
}

pub struct CsvLog{
	csv_writer: AsyncSerializer<File>,
}

impl CsvLog{
	pub async fn new(dst_file: String) -> Self {
		let csv_file = OpenOptions::new()
			.create(true)
			.write(true)
			.append(true)
			.open(&dst_file)
			.await
			.unwrap();
		if Self::check_has_headers(&dst_file).await{
			let mut try_build = AsyncWriterBuilder::new();
			let _try_has_header = try_build.has_headers(false);
			CsvLog{
				csv_writer: try_build.create_serializer(csv_file),
			}
		}else{
			CsvLog{
				csv_writer: AsyncWriterBuilder::new().create_serializer(csv_file),
			}
		}
	}
	
	pub async fn add_row_struct(&mut self, csv_log_item: CsvLogItem){
		let _ = self.csv_writer.serialize(csv_log_item).await;
	}

	async fn check_has_headers(dst: &str) -> bool{
		let get_file: &Path = Path::new(dst);
		if get_file.exists(){
			let csv_file = OpenOptions::new()
			.read(true)
			.open(dst)
			.await
			.expect("error to read csv");
			let mut csv_reader = AsyncReader::from_reader(csv_file);
			if csv_reader.headers().await.unwrap().is_empty(){
				return false
			}else{
				return true
			}
		}
		return false
	}
}

