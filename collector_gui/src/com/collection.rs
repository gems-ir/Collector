use collector_core::prelude::*;
use std::sync::Arc;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct CollectionProgress {
    pub current: u64,
    pub total: u64,
    pub current_file: String,
}

#[derive(Debug, Clone)]
pub struct CollectionResult {
    pub success: bool,
    pub message: String,

    // TO REMOVE
    // stats: Option<CollectionStats>,
}

pub async fn run_collection(
    source: String,
    destination: String,
    resources: Vec<String>,
    resource_path: String,
    vss_enabled: bool,
    zip_enabled: bool,
    zip_pass: Option<String>,
    progress_sender: mpsc::UnboundedSender<CollectionProgress>,
) -> CollectionResult {
    // Parse resources
    let mut parser = match ResourcesParser::new(&resource_path) {
        Ok(p) => p,
        Err(e) => return CollectionResult {
            success: false,
            message: format!("Failed to parse resources: {}", e),
            // stats: None,
        },
    };

    let artifacts = match parser.get_doc_struct().await {
        Ok(a) => a,
        Err(e) => return CollectionResult {
            success: false,
            message: format!("Failed to load artifacts: {}", e),
            // stats: None,
        },
    };

    let patterns = match parser.select_artifact(resources, &artifacts) {
        Ok(p) => p,
        Err(e) => return CollectionResult {
            success: false,
            message: format!("Failed to select artifacts: {}", e),
            // stats: None,
        },
    };

    // Create collector
    let mut collector = match ArtifactCollector::new(&source, &destination, patterns.clone()).await {
        Ok(c) => c,
        Err(e) => return CollectionResult {
            success: false,
            message: format!("Failed to create collector: {}", e),
            // stats: None,
        },
    };

    // Collect with progress
    let sender = Arc::new(progress_sender);
    let stats = {
        let sender = sender.clone();
        match collector.collect_with_progress(move |current, total, path| {
            let _ = sender.send(CollectionProgress {
                current,
                total,
                current_file: path.to_string(),
            });
        }).await {
            Ok(s) => s,
            Err(e) => return CollectionResult {
                success: false,
                message: format!("Collection failed: {}", e),
                // stats: None,
            },
        }
    };

    // VSS collection (Windows only)
    #[cfg(target_os = "windows")]
    if vss_enabled {
        let mut vss_collector = VssCollector::new(&source, &destination, patterns);
        if let Err(e) = vss_collector.collect_from_snapshots().await {
            return CollectionResult {
                success: false,
                message: format!("VSS collection failed: {}", e),
                // stats: None,
            };
        }
    }

    // Suppress unused variable warning on non-Windows
    #[cfg(not(target_os = "windows"))]
    let _ = vss_enabled;

    // Zip if enabled
    if zip_enabled {
        if let Err(e) = collector.create_archive(zip_pass).await {
            return CollectionResult {
                success: false,
                message: format!("Failed to create archive: {}", e),
                // stats: Some(stats),
            };
        }
    }

    CollectionResult {
        success: true,
        message: format!(
            "Collection completed: {} files ({})",
            stats.files_collected,
            format_bytes(stats.bytes_collected)
        ),
        // stats: Some(stats),
    }
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
