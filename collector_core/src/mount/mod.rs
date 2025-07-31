pub mod vss_info;

use crate::mount::vss_info::{VSSObj,DriveLetter};
use std::path::PathBuf;
use tokio::fs;
use anyhow::{Result,anyhow};

#[derive(Debug, Clone)]
pub struct Vss {
    pub drive_letter: String, 
}

impl Vss {
    pub fn new(drive_letter: String) -> Self{
        Vss {
            drive_letter: drive_letter,
        }
    }

    /// Try to find all VSS backup from the driver letter of this object.
    pub  fn get_list(&self) -> Result<Vec<VSSObj>,>{
        let get_vss_list = match VSSObj::get_list() {
            Ok(obj) => obj,
            Err(err) => return Err(err) 
        };
        let get_original_volume = match self.convert(){
            Some(origine_volume) => origine_volume,
            None => return Err(anyhow!("[VSS] Original volume can't be found"))
        };
        let get_filter_list: Vec<VSSObj> = get_vss_list.iter().filter(|o| o.original_volume_name == get_original_volume).cloned().collect();
        if get_filter_list.len() == 0{
            Err(anyhow!("[VSS] No VSS found with this drive letter"))
        }else{
            Ok(get_filter_list)
        }
    }

    /// Get the full volume from the drive letter is stored.
    pub fn convert(&self) -> Option<String>{
        let dl = &self.drive_letter;
        let dlv: Option<String> = DriveLetter::from(dl.to_string()).to_volume();
        dlv
    }

    /// Get the VSSObj (=vss snapshot inside windows) and the destination path to mount the VSS.
    /// Output the destination path where the VSS is mounted.
    pub async fn mount_vss(vss_item: VSSObj ,dest_path: PathBuf) -> PathBuf {
        let get_vss_name = vss_item.device_volume_name.split("\\").last().unwrap();
        let concat_dst_name = dest_path.join(get_vss_name);
        let _ = fs::symlink_dir(vss_item.original_volume_name, &concat_dst_name).await;
        concat_dst_name
    }

}



#[cfg(test)]
mod tests {

}
