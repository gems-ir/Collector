use tokio::io::AsyncWriteExt;
use tokio::fs::File;
use anyhow::{Result,anyhow};

use std::io::{Read,Seek,BufReader};

use ntfs::{Ntfs,NtfsFile,NtfsReadSeek};
use ntfs::indexes::NtfsFileNameIndex;

use crate::extract::sector_reader::SectorReader;

struct InfoFS<'n, T>
where
    T: Read + Seek,{
	current_directory: Vec<NtfsFile<'n>>,
	fs: T,
    ntfs: &'n Ntfs,
}

pub async fn extract_ntfs(device_name: String, artifact_name: String, output_file: &mut File) -> Result<String,>{
	// Create ntfs struct
	let f = std::fs::File::open(device_name.clone())?;
	let sr = SectorReader::new(f,4096)?;
	let mut fs = BufReader::new(sr);	
	let mut ntfs = Ntfs::new(&mut fs)?;
	ntfs.read_upcase_table(&mut fs)?;
	let current_directory = vec![ntfs.root_directory(&mut fs)?];
	let mut infofs = InfoFS{
		current_directory,
		fs,
		ntfs: &ntfs
	};
	
	// Find file or path
	let mut split_artifact: Vec<&str> = artifact_name.split('\\').collect();
	let binding = split_artifact.clone();
 	let get_filename = binding.last();
	let _ = &split_artifact.pop();
	if split_artifact.len() > 1 {
		let _ = move_to(&mut infofs, split_artifact);
		let file = from_to(&mut infofs, get_filename.unwrap().to_string());
		match file{
			Ok(_) => {
				let _ = write_out(&mut infofs,output_file,file?).await;
			}
			Err(e) => return Err(anyhow!(e)), 
		}
	}else{
		let file = from_to(&mut infofs, artifact_name.clone());
		match file{
			Ok(_) => {
				let _ = write_out(&mut infofs,output_file,file?).await;
			}
			Err(e) => return Err(anyhow!(e)), 
		}
	}
	// let _ = &device_name.push_str("\\");
	let full_artifact_path = device_name + "\\" + &artifact_name;
	Ok(format!("A file has been recover: {}",full_artifact_path))
}

fn move_to<T>(infofs: &mut InfoFS<T>,artifact_path: Vec<&str>) -> Result<()>
where
    T: Read + Seek,
{
	for path in artifact_path{
		let index = infofs.current_directory.last().unwrap().directory_index(&mut infofs.fs)?;
		let mut finder = index.finder();
		let entry = NtfsFileNameIndex::find(&mut finder, infofs.ntfs, &mut infofs.fs, path);
		if entry.is_none(){
			return Ok(())
		}
		let file = entry.unwrap()?.to_file(infofs.ntfs, &mut infofs.fs)?;
		infofs.current_directory.push(file);
	}
	Ok(())

}

fn from_to<'n,T>(infofs: &mut InfoFS<'n, T>,artifact_filename: String) -> Result<NtfsFile<'n>,>
where
	T: Read + Seek,
{
	let index = infofs.current_directory.last().unwrap().directory_index(&mut infofs.fs)?;
	let mut finder = index.finder();
	let entry = NtfsFileNameIndex::find(&mut finder, infofs.ntfs, &mut infofs.fs, &artifact_filename);
	let test_entry = match entry {
		Some(entry) => entry,
		None => return Err(anyhow!("Error on file : {}",artifact_filename)),
	};
	let file = test_entry.unwrap().to_file(infofs.ntfs, &mut infofs.fs)?;
	Ok(file)
}

async fn write_out<T>(infofs: &mut InfoFS<'_, T>,output_file: &mut File, file: NtfsFile<'_>) -> Result<()>
where
	T: Read + Seek,
{
	let data_item = file.data(&mut infofs.fs, "");
	let data_item = data_item.unwrap()?;
	let data_attribute = data_item.to_attribute()?;
	let mut data_value = data_attribute.value(&mut infofs.fs)?;
	let mut buf = [0u8; 32768];
	loop {
		let bytes_read = data_value.read(&mut infofs.fs, &mut buf)?;
		if bytes_read == 0 {
			break;
		}

		let _ = output_file.write_all(&buf[..bytes_read]).await?;
	}
	Ok(())
}