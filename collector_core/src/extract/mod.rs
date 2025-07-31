mod lowfs;
mod sector_reader;

#[cfg(target_os = "windows")]
use crate::mount::vss_info::VSSObj;

use anyhow::Result;
use log::*;
use regex::Regex;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};


pub async fn try_filesystem(source: PathBuf, dest_file: &mut File) -> Result<()> {
    let get_source_string: &str = source.to_str().unwrap();
    let mut file = File::open(get_source_string).await?;
    let mut contents = vec![];
    file.read_to_end(&mut contents).await?;
    dest_file.write_all(&contents).await?;
    match dest_file.write_all(&contents).await {
        Ok(_inf) => {
            info!("A file has been recover: {}",get_source_string);
            Ok(())
        }
        Err(err) => {
            error!("{}",err);
            Err(err.into())
        }
    }
    // Ok(())
}

#[cfg(target_os = "windows")]
pub async fn try_ntfs(source: PathBuf, dest_file: &mut File, vss_item: Option<VSSObj>) -> Result<(), > {
    let drive_letter: String = get_drive_letter(source.clone()).unwrap_or_else(|| String::new());
    let mut volume_entry: String = drive_letter.clone();
    if volume_entry.ends_with("\\") {
        let _ = &volume_entry.pop();
    }
    let mut build_source: String = String::from("\\\\?\\") + &volume_entry;

    if vss_item.is_some() {
        build_source = vss_item.unwrap().device_volume_name;
    }

    // Create output file
    let output_path: String = source.clone().to_str().unwrap().to_string();
    let available_artefact = output_path.replace(&drive_letter, "");
    let out_info = lowfs::extract_ntfs(build_source, available_artefact, dest_file).await;
    match out_info {
        Ok(res) => {
            info!("{}",res);
            Ok(())
        }
        Err(err) => {
            error!("Impossible to extract file: {} ",&source.display());
            Err(err)
        }
    }
}


fn get_drive_letter(path: PathBuf) -> Option<String> {
    let format_path: &str = path.to_str().unwrap();
    let drive_letter_regex = Regex::new(r"(^[A-Za-z]:\\)").expect("Failed to parse regex");
    let caps = drive_letter_regex.captures(format_path);
    if caps.is_some() {
        let drive_letter = caps.unwrap().get(0).map_or("", |m| m.as_str());
        Some(drive_letter.to_string())
    } else {
        None
    }
}