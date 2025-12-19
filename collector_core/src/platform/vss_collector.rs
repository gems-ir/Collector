#[cfg(target_os = "windows")]
use std::path::PathBuf;

#[cfg(target_os = "windows")]
use tokio::fs;
#[cfg(target_os = "windows")]
use uuid::Uuid;

#[cfg(target_os = "windows")]
use crate::error::Result;
#[cfg(target_os = "windows")]
use crate::mount::{Vss, VssSnapshot};
#[cfg(target_os = "windows")]
use crate::platform::{ArtifactCollector, CollectionStats};
#[cfg(target_os = "windows")]
use crate::utils::require_admin;

#[cfg(target_os = "windows")]
pub struct VssCollector {
    drive_letter: String,
    destination: PathBuf,
    patterns: Vec<String>,
    temp_dir: Option<PathBuf>,
}

#[cfg(target_os = "windows")]
impl VssCollector {
    pub fn new<S, D>(drive_letter: S, destination: D, patterns: Vec<String>) -> Self
    where
        S: Into<String>,
        D: Into<PathBuf>,
    {
        Self {
            drive_letter: drive_letter.into(),
            destination: destination.into(),
            patterns,
            temp_dir: None,
        }
    }

    pub async fn collect_from_snapshots(&mut self) -> Result<CollectionStats> {
        require_admin()?;

        let vss = Vss::new(&self.drive_letter);
        let snapshots = vss.get_snapshots()?;

        log::info!(
            "Found {} VSS snapshots for {}",
            snapshots.len(),
            self.drive_letter
        );

        let temp_base = std::env::temp_dir();
        let temp_dir = temp_base.join(Uuid::new_v4().to_string());
        fs::create_dir_all(&temp_dir).await?;
        self.temp_dir = Some(temp_dir.clone());

        let mut combined_stats = CollectionStats::default();

        for snapshot in snapshots {
            log::info!(
                "Processing snapshot: {}",
                snapshot.snapshot_id().unwrap_or("unknown")
            );

            match self.collect_from_snapshot(&snapshot, &temp_dir).await {
                Ok(stats) => {
                    combined_stats.files_collected += stats.files_collected;
                    combined_stats.bytes_collected += stats.bytes_collected;
                    combined_stats.filesystem_extractions += stats.filesystem_extractions;
                    combined_stats.ntfs_extractions += stats.ntfs_extractions;
                    combined_stats.failed_extractions += stats.failed_extractions;
                }
                Err(e) => log::error!("Failed to collect from snapshot: {}", e),
            }
        }

        if let Err(e) = fs::remove_dir_all(&temp_dir).await {
            log::warn!("Failed to cleanup temp dir: {}", e);
        }

        Ok(combined_stats)
    }

    async fn collect_from_snapshot(
        &self,
        snapshot: &VssSnapshot,
        temp_dir: &PathBuf,
    ) -> Result<CollectionStats> {
        let mount_point = Vss::mount_snapshot(snapshot, temp_dir).await?;

        if mount_point.is_symlink() {
            let mut collector =
                ArtifactCollector::new(&mount_point, &self.destination, self.patterns.clone())
                    .await?
                    .with_vss_snapshot(snapshot.clone());

            collector.collect().await
        } else {
            log::warn!("Failed to mount snapshot");
            Ok(CollectionStats::default())
        }
    }
}

#[cfg(target_os = "windows")]
impl Drop for VssCollector {
    fn drop(&mut self) {
        if let Some(ref temp_dir) = self.temp_dir {
            let _ = std::fs::remove_dir_all(temp_dir);
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub struct VssCollector;

#[cfg(not(target_os = "windows"))]
impl VssCollector {
    pub fn new<S, D>(_drive_letter: S, _destination: D, _patterns: Vec<String>) -> Self
    where
        S: Into<String>,
        D: Into<std::path::PathBuf>,
    {
        Self
    }
}
