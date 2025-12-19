use std::io::{BufReader, Read, Seek};

use ntfs::indexes::NtfsFileNameIndex;
use ntfs::{Ntfs, NtfsFile, NtfsReadSeek};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::error::{CollectorError, Result};
use crate::extract::sector_reader::SectorReader;
use crate::utils::NTFS_READ_BUFFER_SIZE;

struct NtfsContext<'n, T: Read + Seek> {
    current_directory: Vec<NtfsFile<'n>>,
    fs: T,
    ntfs: &'n Ntfs,
}

pub async fn extract_ntfs(
    device_name: String,
    artifact_path: String,
    output_file: &mut File,
) -> Result<u64> {
    let file = std::fs::File::open(&device_name).map_err(|e| {
        CollectorError::NtfsError(format!("Failed to open volume {}: {}", device_name, e))
    })?;

    let sector_reader = SectorReader::new(file, 4096)
        .map_err(|e| CollectorError::SectorReaderError(e.to_string()))?;

    let mut fs = BufReader::new(sector_reader);

    let mut ntfs = Ntfs::new(&mut fs)
        .map_err(|e| CollectorError::NtfsError(format!("Failed to init NTFS: {}", e)))?;

    ntfs.read_upcase_table(&mut fs)
        .map_err(|e| CollectorError::NtfsError(format!("Failed to read upcase table: {}", e)))?;

    let root_dir = ntfs
        .root_directory(&mut fs)
        .map_err(|e| CollectorError::NtfsError(format!("Failed to get root: {}", e)))?;

    let mut context = NtfsContext {
        current_directory: vec![root_dir],
        fs,
        ntfs: &ntfs,
    };

    let path_components: Vec<&str> = artifact_path
        .split('\\')
        .filter(|s| !s.is_empty())
        .collect();

    if path_components.is_empty() {
        return Err(CollectorError::NtfsExtraction {
            path: artifact_path.into(),
            reason: "Empty path".to_string(),
        });
    }

    let (parent_path, filename) = path_components.split_at(path_components.len() - 1);
    let filename = filename[0];

    if !parent_path.is_empty() {
        navigate_to_directory(&mut context, parent_path)?;
    }

    let file = find_file(&mut context, filename)?;
    let bytes = write_file_contents(&mut context, &file, output_file).await?;

    Ok(bytes)
}

fn navigate_to_directory<T: Read + Seek>(
    context: &mut NtfsContext<T>,
    path: &[&str],
) -> Result<()> {
    for component in path {
        let current_dir = context
            .current_directory
            .last()
            .ok_or_else(|| CollectorError::NtfsError("No current directory".to_string()))?;

        let index = current_dir
            .directory_index(&mut context.fs)
            .map_err(|e| CollectorError::NtfsError(format!("Directory index error: {}", e)))?;

        let mut finder = index.finder();
        let entry = NtfsFileNameIndex::find(&mut finder, context.ntfs, &mut context.fs, component);

        let entry = entry
            .ok_or_else(|| CollectorError::NtfsExtraction {
                path: component.to_string().into(),
                reason: "Directory not found".to_string(),
            })?
            .map_err(|e| CollectorError::NtfsError(format!("Entry error: {}", e)))?;

        let file = entry
            .to_file(context.ntfs, &mut context.fs)
            .map_err(|e| CollectorError::NtfsError(format!("File conversion error: {}", e)))?;

        context.current_directory.push(file);
    }

    Ok(())
}

fn find_file<'n, T: Read + Seek>(
    context: &mut NtfsContext<'n, T>,
    filename: &str,
) -> Result<NtfsFile<'n>> {
    let current_dir = context
        .current_directory
        .last()
        .ok_or_else(|| CollectorError::NtfsError("No current directory".to_string()))?;

    let index = current_dir
        .directory_index(&mut context.fs)
        .map_err(|e| CollectorError::NtfsError(format!("Directory index error: {}", e)))?;

    let mut finder = index.finder();
    let entry = NtfsFileNameIndex::find(&mut finder, context.ntfs, &mut context.fs, filename);

    let entry = entry
        .ok_or_else(|| CollectorError::NtfsExtraction {
            path: filename.to_string().into(),
            reason: "File not found".to_string(),
        })?
        .map_err(|e| CollectorError::NtfsError(format!("Entry error: {}", e)))?;

    entry
        .to_file(context.ntfs, &mut context.fs)
        .map_err(|e| CollectorError::NtfsError(format!("File conversion error: {}", e)))
}

async fn write_file_contents<T: Read + Seek>(
    context: &mut NtfsContext<'_, T>,
    file: &NtfsFile<'_>,
    output: &mut File,
) -> Result<u64> {
    let data_item = file
        .data(&mut context.fs, "")
        .ok_or_else(|| CollectorError::NtfsError("No data attribute".to_string()))?
        .map_err(|e| CollectorError::NtfsError(format!("Data error: {}", e)))?;

    let data_attribute = data_item
        .to_attribute()
        .map_err(|e| CollectorError::NtfsError(format!("Attribute error: {}", e)))?;

    let mut data_value = data_attribute
        .value(&mut context.fs)
        .map_err(|e| CollectorError::NtfsError(format!("Value error: {}", e)))?;

    let mut total_bytes = 0u64;
    let mut buffer = [0u8; NTFS_READ_BUFFER_SIZE];

    loop {
        let bytes_read = data_value
            .read(&mut context.fs, &mut buffer)
            .map_err(|e| CollectorError::NtfsError(format!("Read error: {}", e)))?;

        if bytes_read == 0 {
            break;
        }

        output
            .write_all(&buffer[..bytes_read])
            .await
            .map_err(|e| CollectorError::FileWrite {
                path: "output".into(),
                source: e,
            })?;

        total_bytes += bytes_read as u64;
    }

    Ok(total_bytes)
}
