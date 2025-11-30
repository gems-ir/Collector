use collector_core::resource::YamlParser;
use collector_core::Collect;

/// Execute the collection in async mode
pub async fn run_collection(
    source: String,
    destination: String,
    resources: Vec<String>,
    resource_path: String,
    _vss_enabled: bool,
    _zip_enabled: bool,
    _zip_pass: Option<String>,
) {
    // Parse resources
    let mut parser_obj = YamlParser::new(resource_path);
    let doc_artifacts = parser_obj.get_doc_struct().await;
    let list_artifacts: Vec<String> = parser_obj.select_artifact(resources, doc_artifacts);

    // Run collection
    let mut collector_obj = Collect::new(
        source.clone(),
        destination.clone(),
        list_artifacts.clone(),
    )
    .await;
    collector_obj.start().await;

    // VSS collection (Windows only)
    #[cfg(target_os = "windows")]
    if _vss_enabled {
        use collector_core::windows_vss::CollectVss;
        let vss_obj = CollectVss::new(source, destination.clone(), list_artifacts);
        vss_obj.collect().await;
    }

    // Zip if enabled
    if _zip_enabled {
        let _ = collector_obj.zip(_zip_pass).await;
    }
}
