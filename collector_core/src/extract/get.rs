
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::extract::lowfs;
use crate::error::{CollectorError, Result};

#[cfg(target_os = "windows")]
use crate::mount::VssSnapshot;
#[cfg(target_os = "windows")]
use regex::Regex;

pub async fn extract_via_filesystem(source: &PathBuf, dest_file: &mut File) -> Result<u64> {
    let mut source_file = File::open(source).await.map_err(|e| CollectorError::FileRead {
        path: source.clone(),
        source: e,
    })?;

    let mut contents = Vec::new();
    source_file.read_to_end(&mut contents).await.map_err(|e| {
        CollectorError::FileRead { path: source.clone(), source: e }
    })?;

    let bytes_written = contents.len() as u64;
    
    dest_file.write_all(&contents).await.map_err(|e| {
        CollectorError::FileWrite { path: source.clone(), source: e }
    })?;

    log::info!("Extracted via filesystem: {}", source.display());
    Ok(bytes_written)
}

#[cfg(target_os = "windows")]
pub async fn extract_via_ntfs(
    source: &PathBuf,
    dest_file: &mut File,
    vss_snapshot: Option<&VssSnapshot>,
) -> Result<u64> {
    let drive_letter = get_drive_letter(source)?;
    
    let mut volume_entry = drive_letter.clone();
    if volume_entry.ends_with('\\') {
        volume_entry.pop();
    }

    let build_source = if let Some(vss) = vss_snapshot {
        vss.device_volume_name.clone()
    } else {
        format!("\\\\?\\{}", volume_entry)
    };

    let relative_path = source.to_string_lossy().replace(&drive_letter, "");
    let bytes = lowfs::extract_ntfs(build_source, relative_path, dest_file).await?;
    
    log::info!("Extracted via NTFS: {}", source.display());
    Ok(bytes)
}

#[cfg(target_os = "windows")]
pub async fn extract_file(
    source: &PathBuf,
    dest_file: &mut File,
    vss_snapshot: Option<&VssSnapshot>,
) -> Result<(u64, bool)> {
    match extract_via_filesystem(source, dest_file).await {
        Ok(bytes) => return Ok((bytes, false)),
        Err(e) => log::debug!("Filesystem failed, trying NTFS: {}", e),
    }

    let bytes = extract_via_ntfs(source, dest_file, vss_snapshot).await?;
    Ok((bytes, true))
}

#[cfg(not(target_os = "windows"))]
pub async fn extract_file(
    source: &PathBuf,
    dest_file: &mut File,
    _vss_snapshot: Option<()>,
) -> Result<(u64, bool)> {
    let bytes = extract_via_filesystem(source, dest_file).await?;
    Ok((bytes, false))
}

#[cfg(target_os = "windows")]
fn get_drive_letter(path: &PathBuf) -> Result<String> {
    let path_str = path.to_string_lossy();
    let re = Regex::new(r"^([A-Za-z]:\\)").expect("Invalid regex");

    re.captures(&path_str)
        .and_then(|caps| caps.get(0))
        .map(|m| m.as_str().to_string())
        .ok_or_else(|| CollectorError::InvalidDriveLetter(path_str.to_string()))
}


#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn test_extract_via_filesystem() {
        let temp_dir = tempdir().unwrap();
        
        // Create source file
        let source_path = temp_dir.path().join("source.txt");
        let mut source_file = File::create(&source_path).await.unwrap();
        source_file.write_all(b"test content").await.unwrap();
        drop(source_file);

        // Create destination file
        let dest_path = temp_dir.path().join("dest.txt");
        let mut dest_file = File::create(&dest_path).await.unwrap();

        // Extract
        let result = extract_via_filesystem(&source_path, &mut dest_file).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 12); // "test content" = 12 bytes
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_get_drive_letter() {
        let path = PathBuf::from("C:\\Windows\\System32");
        let result = get_drive_letter(&path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "C:\\");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_get_drive_letter_invalid() {
        let path = PathBuf::from("/home/user");
        let result = get_drive_letter(&path);
        assert!(result.is_err());
    }
}
