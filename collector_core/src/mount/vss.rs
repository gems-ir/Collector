//! VSS implementation for Windows.

use std::ffi::OsString;
use std::mem::zeroed;
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;
use std::ptr::null_mut;

use tokio::fs;
use widestring::{U16CStr, encode_utf16};
use winapi::shared::rpcdce::{RPC_C_AUTHN_LEVEL_PKT_PRIVACY, RPC_C_IMP_LEVEL_IMPERSONATE};
use winapi::shared::winerror::{E_ACCESSDENIED, E_INVALIDARG, S_OK};
use winapi::um::cguid::GUID_NULL;
use winapi::um::combaseapi::{COINITBASE_MULTITHREADED, CoInitializeEx, CoInitializeSecurity};
use winapi::um::fileapi::GetVolumeNameForVolumeMountPointW;
use winapi::um::objidl::EOAC_DYNAMIC_CLOAKING;
use winapi::um::vsbackup::{CreateVssBackupComponents, IVssBackupComponents};
use winapi::um::vss::{
    IVssEnumObject, VSS_BT_FULL, VSS_CTX_ALL, VSS_OBJECT_NONE, VSS_OBJECT_PROP,
    VSS_OBJECT_SNAPSHOT, VSS_SNAPSHOT_PROP,
};
use winapi::um::winnt::HRESULT;

use crate::error::{CollectorError, Result};

#[derive(Debug, Clone)]
pub struct VssSnapshot {
    pub original_volume_name: String,
    pub device_volume_name: String,
}

impl VssSnapshot {
    pub fn snapshot_id(&self) -> Option<&str> {
        self.device_volume_name.split('\\').last()
    }
}

#[derive(Debug, Clone)]
pub struct Vss {
    drive_letter: String,
}

impl Vss {
    pub fn new<S: Into<String>>(drive_letter: S) -> Self {
        Self {
            drive_letter: drive_letter.into(),
        }
    }

    pub fn drive_letter(&self) -> &str {
        &self.drive_letter
    }

    pub fn get_snapshots(&self) -> Result<Vec<VssSnapshot>> {
        let all_snapshots = list_all_snapshots()?;

        let volume_name = DriveLetter::new(&self.drive_letter)
            .to_volume()
            .ok_or_else(|| {
                CollectorError::VssOperation(format!(
                    "Failed to get volume for {}",
                    self.drive_letter
                ))
            })?;

        let filtered: Vec<VssSnapshot> = all_snapshots
            .into_iter()
            .filter(|s| s.original_volume_name == volume_name)
            .collect();

        if filtered.is_empty() {
            return Err(CollectorError::NoVssSnapshots(self.drive_letter.clone()));
        }

        Ok(filtered)
    }

    pub fn to_volume(&self) -> Option<String> {
        DriveLetter::new(&self.drive_letter).to_volume()
    }

    pub async fn mount_snapshot(snapshot: &VssSnapshot, dest_path: &PathBuf) -> Result<PathBuf> {
        let snapshot_name = snapshot.snapshot_id().unwrap_or("unknown");
        let mount_point = dest_path.join(snapshot_name);

        fs::symlink_dir(&snapshot.original_volume_name, &mount_point)
            .await
            .map_err(|e| CollectorError::VssMountFailed(format!("Symlink error: {}", e)))?;

        Ok(mount_point)
    }
}

#[derive(Debug)]
pub struct DriveLetter {
    letter: String,
}

impl DriveLetter {
    pub fn new<S: Into<String>>(letter: S) -> Self {
        Self {
            letter: letter.into(),
        }
    }

    pub fn to_volume(&self) -> Option<String> {
        const VOLUME_MAX_LEN: usize = 50;

        let mut drive_path: Vec<u16> = encode_utf16(&mut self.letter.chars()).collect();
        drive_path.push(0);

        let mut buffer = [0u16; VOLUME_MAX_LEN];

        let result = unsafe {
            GetVolumeNameForVolumeMountPointW(
                drive_path.as_ptr(),
                buffer.as_mut_ptr(),
                VOLUME_MAX_LEN as u32,
            )
        };

        if result == 0 {
            return None;
        }

        let volume = OsString::from_wide(&buffer)
            .to_string_lossy()
            .trim_end_matches('\0')
            .to_string();

        Some(volume)
    }
}

fn list_all_snapshots() -> Result<Vec<VssSnapshot>> {
    let mut snapshots = Vec::new();

    unsafe {
        let mut backup_components: *mut IVssBackupComponents = null_mut();
        let mut enum_object: *mut IVssEnumObject = null_mut();
        let mut prop: VSS_OBJECT_PROP = zeroed();

        let hr = CoInitializeEx(null_mut(), COINITBASE_MULTITHREADED);
        check_hresult(hr, "CoInitializeEx")?;

        let hr = CoInitializeSecurity(
            null_mut(),
            -1,
            null_mut(),
            null_mut(),
            RPC_C_AUTHN_LEVEL_PKT_PRIVACY,
            RPC_C_IMP_LEVEL_IMPERSONATE,
            null_mut(),
            EOAC_DYNAMIC_CLOAKING,
            null_mut(),
        );
        check_hresult(hr, "CoInitializeSecurity")?;

        let hr = CreateVssBackupComponents(&mut backup_components);
        if hr == E_ACCESSDENIED {
            return Err(CollectorError::InsufficientPrivileges);
        }
        check_hresult(hr, "CreateVssBackupComponents")?;

        let backup = backup_components.as_ref().unwrap();

        check_hresult(
            backup.InitializeForBackup(null_mut()),
            "InitializeForBackup",
        )?;
        check_hresult(backup.SetContext(VSS_CTX_ALL as i32), "SetContext")?;
        check_hresult(
            backup.SetBackupState(true, true, VSS_BT_FULL, false),
            "SetBackupState",
        )?;
        check_hresult(
            backup.Query(
                GUID_NULL,
                VSS_OBJECT_NONE,
                VSS_OBJECT_SNAPSHOT,
                &mut enum_object,
            ),
            "Query",
        )?;

        let enum_obj = enum_object.as_ref().unwrap();
        let mut fetched: u32 = 0;

        loop {
            let hr = enum_obj.Next(1, &mut prop, &mut fetched);

            if hr != S_OK || fetched == 0 {
                break;
            }

            let snap: &VSS_SNAPSHOT_PROP = prop.Obj.Snap();
            let mut snap_props: VSS_SNAPSHOT_PROP = zeroed();

            if IVssBackupComponents::GetSnapshotProperties(
                &*backup_components,
                snap.m_SnapshotId,
                &mut snap_props,
            ) == S_OK
            {
                let original =
                    U16CStr::from_ptr_str(snap_props.m_pwszOriginalVolumeName).to_string_lossy();
                let device =
                    U16CStr::from_ptr_str(snap_props.m_pwszSnapshotDeviceObject).to_string_lossy();

                snapshots.push(VssSnapshot {
                    original_volume_name: original,
                    device_volume_name: device,
                });
            }
        }
    }

    Ok(snapshots)
}

fn check_hresult(hr: HRESULT, operation: &str) -> Result<()> {
    match hr {
        S_OK => Ok(()),
        E_ACCESSDENIED => Err(CollectorError::InsufficientPrivileges),
        E_INVALIDARG => Err(CollectorError::VssOperation(format!(
            "{}: Invalid argument",
            operation
        ))),
        _ => Err(CollectorError::VssComInit(format!(
            "{} failed: 0x{:08X}",
            operation, hr
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vss_new() {
        let vss = Vss::new("C:\\");
        assert_eq!(vss.drive_letter(), "C:\\");
    }

    #[test]
    fn test_drive_letter() {
        let dl = DriveLetter::new("C:\\");
        // Note: This test will only pass if C: exists
        // We just test that it doesn't panic
        let _ = dl.to_volume();
    }

    #[test]
    fn test_vss_snapshot_id() {
        let snapshot = VssSnapshot {
            original_volume_name: "\\\\?\\Volume{guid}\\".to_string(),
            device_volume_name: "\\\\?\\GLOBALROOT\\Device\\HarddiskVolumeShadowCopy1".to_string(),
        };

        assert_eq!(snapshot.snapshot_id(), Some("HarddiskVolumeShadowCopy1"));
    }
}
