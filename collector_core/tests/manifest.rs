//! End-to-end check of the collection output: CSV, manifest and archive.
//!
//! A second `ArtifactCollector` on the same destination simulates what the VSS collection
//! does (one new collector per snapshot).

use std::io::Read;

use chrono::Utc;
use collector_core::ArtifactCollector;
use collector_core::manifest::{
    CollectionInfo, ExtractionMethod, FileOrigin, MANIFEST_FILE, MANIFEST_RECORDS_FILE, Manifest,
    ManifestHeader,
};
use collector_core::utils::is_admin;
use sha2::{Digest, Sha256};

fn sha256_hex(data: &[u8]) -> String {
    hex::encode(Sha256::digest(data))
}

#[tokio::test]
async fn collection_produces_csv_manifest_and_archive() {
    if !is_admin() {
        eprintln!("skipped: the collector requires administrator privileges");
        return;
    }

    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let destination = temp.path().join("out");
    std::fs::create_dir_all(source.join("logs")).unwrap();
    std::fs::create_dir_all(&destination).unwrap();

    std::fs::write(source.join("logs/a.log"), b"first artifact").unwrap();
    std::fs::write(source.join("logs/b.log"), b"second artifact").unwrap();
    std::fs::write(source.join("c.dat"), b"third artifact").unwrap();

    let started_at = Utc::now();

    // Live collection.
    let mut live = ArtifactCollector::new(&source, &destination, vec!["logs/*.log".into()])
        .await
        .unwrap();
    let mut stats = live.collect().await.unwrap();
    assert_eq!(stats.files_collected, 2);

    // Second collector on the same destination (what VSS does): must append, not erase.
    let mut second = ArtifactCollector::new(&source, &destination, vec!["*.dat".into()])
        .await
        .unwrap();
    stats.merge(&second.collect().await.unwrap());
    drop(second);

    let header = ManifestHeader::new(
        CollectionInfo::new(
            started_at,
            source.to_string_lossy(),
            vec!["Test".into()],
            false,
            true,
            false,
        ),
        stats.clone(),
    );
    let listed = live.writer().write_manifest(header).await.unwrap();
    assert_eq!(listed, 3);

    let hostname = live.writer().hostname().to_string();
    live.create_archive(None).await.unwrap();

    // Read the archive back.
    let zip_path = destination.join(format!("Collector_{}.zip", hostname));
    let mut archive = zip::ZipArchive::new(std::fs::File::open(&zip_path).unwrap()).unwrap();
    let names: Vec<String> = archive.file_names().map(str::to_string).collect();
    assert!(
        !names.iter().any(|n| n.ends_with(MANIFEST_RECORDS_FILE)),
        "temporary records must not be archived: {names:?}"
    );

    let mut read_entry = |name: &str| {
        let mut content = Vec::new();
        archive
            .by_name(name)
            .unwrap_or_else(|_| panic!("missing entry {name} in {names:?}"))
            .read_to_end(&mut content)
            .unwrap();
        content
    };

    // CSV: header + one row per file, none lost.
    let csv = String::from_utf8(read_entry("Collector_copy.csv")).unwrap();
    assert_eq!(csv.lines().count(), 4, "CSV content:\n{csv}");

    // Manifest.
    let manifest: Manifest = serde_json::from_slice(&read_entry(MANIFEST_FILE)).unwrap();
    assert_eq!(manifest.header.manifest_version, 1);
    assert_eq!(manifest.files_count, 3);
    assert_eq!(manifest.header.stats, stats);
    assert!(!manifest.header.host.hostname.is_empty());

    for file in &manifest.files {
        assert_eq!(file.origin, FileOrigin::Live);
        assert_eq!(file.extraction, ExtractionMethod::Filesystem);
        assert!(!file.archive_path.contains('\\'));

        // The archive entry exists and matches the recorded SHA-256 and size.
        let content = read_entry(&file.archive_path);
        assert_eq!(file.size, content.len() as u64);
        assert_eq!(file.sha256, sha256_hex(&content));

        // Source timestamps are those of the source file, not of the copy.
        let source_mtime = std::fs::metadata(&file.source_path)
            .unwrap()
            .modified()
            .unwrap();
        let recorded = file
            .source_times
            .as_ref()
            .and_then(|t| t.modified.as_deref())
            .expect("source modification time");
        assert_eq!(
            chrono::DateTime::parse_from_rfc3339(recorded).unwrap(),
            chrono::DateTime::<Utc>::from(source_mtime)
        );
    }
}
