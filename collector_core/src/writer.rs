use crate::utils;

use anyhow::Result;
use std::io::Write;
use std::path::{Path, PathBuf};
use sysinfo::System;
use tokio::{fs::{self, File, OpenOptions, }, io::AsyncReadExt};
use walkdir::WalkDir;
use zip::{write::SimpleFileOptions, AesMode, ZipWriter};

#[derive(Debug,Clone)]
pub struct Writer {
    base_dst: PathBuf,
    full_dst: PathBuf,
}


impl Writer {
    pub fn new<P : AsRef<Path> + AsRef<str>>(destination_path: P) -> Self{
        let base_destination_formating: PathBuf = utils::FormatSource::from(destination_path).to_path();
        let get_hostname = System::host_name().unwrap();
        let binding = base_destination_formating.join(format!("Collector_{}",get_hostname));
        let format_middle_path = binding.to_str().unwrap();
        let full_destination: PathBuf = utils::FormatSource::from(format_middle_path).to_path();
        Writer {
            base_dst: base_destination_formating,
            full_dst: full_destination,
        }
    }

    /// Concatenation of destination with string parameter.
    pub fn get_filepath(&self, path_name: String) -> PathBuf{
        let mut format_file: String = path_name.replace(":","");
        if format_file.starts_with("\\"){
            format_file.remove(0);
        }
        let create_pathbuf: PathBuf = self.full_dst.join(format_file);
        create_pathbuf
    }

    pub fn get_filepath_as_str(&self, path_name: String) -> String {
        self.get_filepath(path_name).to_str().unwrap().to_string()
    }

    /// Create a file in output directory.
    /// If string path as given, the entire path will be created.
    /// Output the file descriptor.
    pub async fn create_file(&self, path_name: String) -> File {
        let get_filepath = self.get_filepath(path_name.clone());
        let _create_folder = self.create_folderpath(path_name.clone()).await;
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(get_filepath)
            .await
            .expect("Impossible to create output file")
    }

    /// create the entire path as given.
    pub async fn create_folderpath(&self, path_name: String){
        let mut get_filepath: PathBuf = self.get_filepath(path_name);
        let _pop_filename = get_filepath.pop();
        if !get_filepath.exists(){
            let _ = fs::create_dir_all(get_filepath).await;
        }
    }


    /// Zip the destination file by the given name.
    pub async fn zip(&self, zip_password: Option<String>) -> Result<()>{
        let get_hostname = System::host_name().unwrap();
        let format_zip_name = format!("Collector_{}.zip",get_hostname);

        // Create zip file
        let path_zip = Path::new(&format_zip_name);
        let path_out = Path::new(&self.base_dst).join(path_zip);

        let file = std::fs::File::create(path_out)?;
        let mut zip = ZipWriter::new(file);
        let mut options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o644);

        if zip_password.is_some(){
            let password_extract = &zip_password.as_ref().unwrap();
            options = SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated)
                .unix_permissions(0o644)
                .with_aes_encryption(AesMode::Aes192 ,password_extract);
        }


        // Walk in output directory
        let mut buffer = Vec::new();
        let walker = WalkDir::new(&self.full_dst).into_iter();
        for entry in walker{
            let unwraper = entry?; // DirEntry(".\\out\\Windows")
            let path = unwraper.path(); // ".\\out\\Windows"
            let name = path.strip_prefix(self.full_dst.clone())?;
            if path.is_file() {
                zip.start_file_from_path(name, options)?;
                let mut f = fs::File::open(path).await?;
                f.read_to_end(&mut buffer).await?;
                zip.write_all(&buffer)?;
                buffer.clear();
            }else if !name.as_os_str().is_empty() {
                zip.add_directory_from_path(name, options)?;

            }
        }
        zip.finish()?;
        fs::remove_dir_all(self.full_dst.clone()).await?;
        Ok(())
    }
}