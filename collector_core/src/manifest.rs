//! Collection manifest.
//!
//! Every collected file is recorded (one JSON line, appended as soon as the file is copied)
//! in a temporary records file inside the output folder. At the end of the collection,
//! [`Writer::write_manifest`](crate::writer::Writer::write_manifest) turns these records and
//! the collection metadata into `Collector_manifest.json`, then removes the records file so
//! that it never ends up in the archive.
//!
//! Unlike `Collector_copy.csv` (kept unchanged for compatibility), the manifest carries the
//! SHA-256 of each file and the timestamps of the **source** file, read before the copy.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use sysinfo::System;
use tokio::fs::{File, OpenOptions};
use tokio::io::AsyncWriteExt;

use crate::error::{CollectorError, Result};
use crate::platform::CollectionStats;

/// File name of the manifest, at the root of the collection folder.
pub const MANIFEST_FILE: &str = "Collector_manifest.json";
/// Temporary per-file records (JSON Lines). Removed when the manifest is written.
pub const MANIFEST_RECORDS_FILE: &str = ".collector_manifest_records.jsonl";
/// Version of the manifest format. Incremented on incompatible changes.
pub const MANIFEST_VERSION: u32 = 1;

/// Formats a timestamp as RFC 3339 UTC with the full sub-second precision available.
pub fn to_rfc3339(time: DateTime<Utc>) -> String {
    time.to_rfc3339_opts(SecondsFormat::AutoSi, true)
}

fn system_time_to_rfc3339(time: std::io::Result<SystemTime>) -> Option<String> {
    time.ok().map(|t| to_rfc3339(DateTime::<Utc>::from(t)))
}

/// Where a collected file comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileOrigin {
    /// The live file system.
    Live,
    /// A Volume Shadow Copy snapshot.
    Vss,
}

/// How the file content was read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtractionMethod {
    /// Regular file system API.
    Filesystem,
    /// Raw NTFS parsing (locked files).
    Ntfs,
}

/// Timestamps of the source file, read on the collected host before copying it.
///
/// Values are `None` when the platform or file system does not provide them, or when the
/// file metadata cannot be read (for example NTFS metafiles such as `$MFT`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceTimes {
    pub created: Option<String>,
    pub modified: Option<String>,
    pub accessed: Option<String>,
}

impl SourceTimes {
    /// Reads the timestamps of `path` from the file system. Returns `None` if the metadata
    /// cannot be read at all.
    pub fn from_path(path: &Path) -> Option<Self> {
        let metadata = std::fs::metadata(path).ok()?;
        Some(Self {
            created: system_time_to_rfc3339(metadata.created()),
            modified: system_time_to_rfc3339(metadata.modified()),
            accessed: system_time_to_rfc3339(metadata.accessed()),
        })
    }
}

/// One collected file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileRecord {
    /// Path of the file on the collected host (or on the mounted snapshot for VSS).
    pub source_path: String,
    /// Path of the entry inside the collection folder / archive, `/`-separated.
    pub archive_path: String,
    pub origin: FileOrigin,
    /// Snapshot identifier (e.g. `HarddiskVolumeShadowCopy3`) when `origin` is `vss`.
    pub vss_snapshot: Option<String>,
    pub extraction: ExtractionMethod,
    pub size: u64,
    pub sha1: String,
    pub sha256: String,
    /// When the copy finished (UTC).
    pub collected_at: String,
    /// Timestamps of the source file, read before the copy.
    pub source_times: Option<SourceTimes>,
}

/// Appends [`FileRecord`]s to the temporary records file.
///
/// The file is opened in append mode so that several collectors (live volume, then each
/// VSS snapshot) writing to the same destination add to the same list.
pub struct ManifestRecorder {
    path: PathBuf,
    file: File,
}

impl ManifestRecorder {
    pub async fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .await
            .map_err(|e| CollectorError::FileWrite {
                path: path.clone(),
                source: e,
            })?;
        Ok(Self { path, file })
    }

    /// Appends one record and flushes it, so that a crash never loses a recorded file.
    pub async fn record(&mut self, record: &FileRecord) -> Result<()> {
        let mut line = serde_json::to_string(record)
            .map_err(|e| CollectorError::Manifest(format!("Failed to serialize record: {}", e)))?;
        line.push('\n');

        let write_error = |e| CollectorError::FileWrite {
            path: self.path.clone(),
            source: e,
        };
        self.file
            .write_all(line.as_bytes())
            .await
            .map_err(write_error)?;
        self.file.flush().await.map_err(write_error)
    }
}

/// Information about the collecting tool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub version: String,
}

impl Default for ToolInfo {
    fn default() -> Self {
        Self {
            name: "gems-ir/Collector".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// Information about the collected host.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostInfo {
    pub hostname: String,
    pub os_name: Option<String>,
    pub os_version: Option<String>,
    pub long_os_version: Option<String>,
    pub kernel_version: Option<String>,
    pub cpu_arch: String,
    pub boot_time: Option<String>,
}

impl HostInfo {
    /// Reads host information from the running system.
    pub fn current() -> Self {
        let boot_time = DateTime::<Utc>::from_timestamp(System::boot_time() as i64, 0)
            .filter(|_| System::boot_time() > 0)
            .map(to_rfc3339);

        Self {
            hostname: System::host_name().unwrap_or_else(|| "unknown".to_string()),
            os_name: System::name(),
            os_version: System::os_version(),
            long_os_version: System::long_os_version(),
            kernel_version: System::kernel_version(),
            cpu_arch: System::cpu_arch(),
            boot_time,
        }
    }
}

/// Collection options and timing, provided by the caller (CLI or GUI).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CollectionInfo {
    pub started_at: String,
    pub finished_at: String,
    pub source: String,
    pub resources: Vec<String>,
    pub vss: bool,
    pub zip: bool,
    /// Whether the archive is password protected. The password itself is never recorded.
    pub encrypted: bool,
}

impl CollectionInfo {
    /// Builds the collection information; `finished_at` is set to now.
    pub fn new(
        started_at: DateTime<Utc>,
        source: impl Into<String>,
        resources: Vec<String>,
        vss: bool,
        zip: bool,
        encrypted: bool,
    ) -> Self {
        Self {
            started_at: to_rfc3339(started_at),
            finished_at: to_rfc3339(Utc::now()),
            source: source.into(),
            resources,
            vss,
            zip,
            encrypted,
        }
    }
}

/// Everything in the manifest except the file list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ManifestHeader {
    pub manifest_version: u32,
    pub tool: ToolInfo,
    pub host: HostInfo,
    pub collection: CollectionInfo,
    pub stats: CollectionStats,
}

impl ManifestHeader {
    pub fn new(collection: CollectionInfo, stats: CollectionStats) -> Self {
        Self {
            manifest_version: MANIFEST_VERSION,
            tool: ToolInfo::default(),
            host: HostInfo::current(),
            collection,
            stats,
        }
    }
}

/// Full manifest, as read back by consumers (e.g. Vanadium) and tests.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Manifest {
    #[serde(flatten)]
    pub header: ManifestHeader,
    pub files_count: u64,
    pub files: Vec<FileRecord>,
}

/// Writes the manifest to `manifest_path`, streaming the records from `records_path` so
/// that memory use does not depend on the number of collected files. The records file is
/// removed afterwards. Returns the number of files written.
pub(crate) fn write_manifest_file(
    header: &ManifestHeader,
    records_path: &Path,
    manifest_path: &Path,
) -> Result<u64> {
    use std::io::{BufRead, BufReader, BufWriter, Write};

    let manifest_error = |e: std::io::Error| CollectorError::FileWrite {
        path: manifest_path.to_path_buf(),
        source: e,
    };

    // Serialize the header as an object, then reopen it to append the file list.
    let header_json = serde_json::to_string_pretty(header)
        .map_err(|e| CollectorError::Manifest(format!("Failed to serialize manifest: {}", e)))?;
    let header_body = header_json
        .strip_suffix('}')
        .map(str::trim_end)
        .ok_or_else(|| CollectorError::Manifest("Unexpected manifest header".to_string()))?;

    let output = std::fs::File::create(manifest_path).map_err(manifest_error)?;
    let mut out = BufWriter::new(output);
    write!(out, "{},\n  \"files\": [", header_body).map_err(manifest_error)?;

    let mut count = 0u64;
    if records_path.exists() {
        let input = std::fs::File::open(records_path).map_err(|e| CollectorError::FileRead {
            path: records_path.to_path_buf(),
            source: e,
        })?;

        for line in BufReader::new(input).lines() {
            let line = line.map_err(|e| CollectorError::FileRead {
                path: records_path.to_path_buf(),
                source: e,
            })?;
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            // Validate every record before publishing it.
            serde_json::from_str::<FileRecord>(line)
                .map_err(|e| CollectorError::Manifest(format!("Invalid manifest record: {}", e)))?;

            let separator = if count == 0 { "\n    " } else { ",\n    " };
            write!(out, "{}{}", separator, line).map_err(manifest_error)?;
            count += 1;
        }
    }

    let closing = if count == 0 { "]" } else { "\n  ]" };
    write!(out, "{},\n  \"files_count\": {}\n}}\n", closing, count).map_err(manifest_error)?;
    out.flush().map_err(manifest_error)?;

    if records_path.exists() {
        std::fs::remove_file(records_path).map_err(|e| CollectorError::FileWrite {
            path: records_path.to_path_buf(),
            source: e,
        })?;
    }

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(name: &str) -> FileRecord {
        FileRecord {
            source_path: format!("C:\\Windows\\Prefetch\\{}", name),
            archive_path: format!("C/Windows/Prefetch/{}", name),
            origin: FileOrigin::Live,
            vss_snapshot: None,
            extraction: ExtractionMethod::Filesystem,
            size: 12,
            sha1: "a".repeat(40),
            sha256: "b".repeat(64),
            collected_at: "2026-10-15T10:00:00Z".to_string(),
            source_times: Some(SourceTimes {
                created: Some("2026-01-01T00:00:00Z".to_string()),
                modified: None,
                accessed: None,
            }),
        }
    }

    fn header() -> ManifestHeader {
        ManifestHeader::new(
            CollectionInfo::new(
                Utc::now(),
                "C:\\",
                vec!["Prefetch".into()],
                false,
                true,
                true,
            ),
            CollectionStats::default(),
        )
    }

    #[tokio::test]
    async fn test_manifest_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let records = dir.path().join(MANIFEST_RECORDS_FILE);
        let manifest = dir.path().join(MANIFEST_FILE);

        let mut recorder = ManifestRecorder::open(&records).await.unwrap();
        recorder.record(&record("A.pf")).await.unwrap();
        drop(recorder);
        // A second recorder (e.g. a VSS collector) appends to the same file.
        let mut recorder = ManifestRecorder::open(&records).await.unwrap();
        recorder.record(&record("B.pf")).await.unwrap();

        let count = write_manifest_file(&header(), &records, &manifest).unwrap();
        assert_eq!(count, 2);
        assert!(!records.exists());

        let parsed: Manifest =
            serde_json::from_str(&std::fs::read_to_string(&manifest).unwrap()).unwrap();
        assert_eq!(parsed.header.manifest_version, MANIFEST_VERSION);
        assert_eq!(parsed.files_count, 2);
        assert_eq!(parsed.files, vec![record("A.pf"), record("B.pf")]);
        assert!(parsed.header.collection.encrypted);
    }

    #[test]
    fn test_manifest_without_records() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = dir.path().join(MANIFEST_FILE);

        let count =
            write_manifest_file(&header(), &dir.path().join("missing.jsonl"), &manifest).unwrap();
        assert_eq!(count, 0);

        let parsed: Manifest =
            serde_json::from_str(&std::fs::read_to_string(&manifest).unwrap()).unwrap();
        assert!(parsed.files.is_empty());
    }

    #[test]
    fn test_invalid_record_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let records = dir.path().join(MANIFEST_RECORDS_FILE);
        std::fs::write(&records, "{not json}\n").unwrap();

        let result = write_manifest_file(&header(), &records, &dir.path().join(MANIFEST_FILE));
        assert!(result.is_err());
    }

    #[test]
    fn test_source_times_from_path() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("file.txt");
        std::fs::write(&path, b"x").unwrap();

        let times = SourceTimes::from_path(&path).unwrap();
        assert!(times.modified.is_some());
        assert!(SourceTimes::from_path(&dir.path().join("missing")).is_none());
    }
}
