use collector_core::resource::{YamlArtifact, YamlParser};
use collector_core::windows_vss::CollectVss;
use collector_core::Collect;

use chrono::Utc;
use log::*;
use rust_embed::Embed;
use serde::{Deserialize, Serialize};
use simplelog::*;
use std::fs::File;
use std::include_str;
use std::str::from_utf8;
use std::time;
use sysinfo::System;

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    source_folder: String,
    destination_folder: String,
    resources_list_artefact: Vec<String>,
    vss: bool,
    zip: bool,
    zip_pass: String,
    verbose: bool,
}

#[derive(Embed)]
#[folder = "../../Resources"]
#[include = "**/*.yaml"]
struct Asset;

#[tokio::main]
async fn main() {
    let config_file = include_str!("../../collector_packer_config.json");
    let conf: Config = serde_json::from_str(config_file).unwrap();


    // LOG Config
    let mut config = ConfigBuilder::new()
        .set_time_format_rfc3339()
        .add_filter_ignore_str("collector_engine")
        .build();
    if conf.verbose {
        config = ConfigBuilder::new()
            .set_time_format_rfc3339()
            .build();
    }
    let get_time = Utc::now().timestamp().to_string();
    let get_hostname = System::host_name().unwrap();
    let name_log_file = format!("collector_{}_{}.log", get_hostname, get_time);
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            config.clone(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Info,
            config.clone(),
            File::create(&name_log_file).unwrap(),
        ),
    ]).unwrap();

    // start elapsed time
    let now = time::Instant::now();

    info!("{}","=".repeat(50));
    info!("Source of artifact: \"{}\"",conf.source_folder);
    info!("Destination of artifact: \"{}\"",conf.destination_folder);
    info!("List of resources collect: {:?}",conf.resources_list_artefact);
    info!("Output file log: \"{}\"",&name_log_file);
    info!("{}","=".repeat(50));


    info!("Start of yaml resource files analysis");
    if conf.resources_list_artefact.is_empty() {
        panic!("\"resources_list_artefact\" is empyty");
    }
    let list_yaml_file: Vec<String> = Asset::iter().map(|e| e.into_owned()).collect();
    let get_raw_yaml: Vec<String> = list_yaml_file.iter().map(|e| from_utf8(&Asset::get(e).unwrap().data.into_owned()).unwrap().to_string()).collect();
    let mut parser_obj: YamlParser = YamlParser::init();
    let doc_artifacts: Vec<YamlArtifact> = parser_obj.get_struct_from_raw(list_yaml_file, get_raw_yaml);
    let list_artifacts: Vec<String> = parser_obj.select_artifact(conf.resources_list_artefact, doc_artifacts);
    info!("End of yaml resource file analysis");

    // Start collect
    info!("Start to collect artifact");
    let mut collector_obj = Collect::new(conf.source_folder.clone(), conf.destination_folder.clone(), list_artifacts.clone()).await;
    let _collector_obj_start = collector_obj.start().await;
    info!("End to collect artifact");

    // Start collect vss
    if conf.vss {
        info!("Start to collect artifact from VSS");
        let vss_obj = CollectVss::new(conf.source_folder, conf.destination_folder, list_artifacts.clone());
        vss_obj.collect().await;
        info!("End to collect artifact from VSS");
    }

    // zip
    if conf.zip {
        info!("Start to zip output directory");
        let mut zip_pass_option: Option<String> = None;
        if conf.zip_pass.len() != 0 {
            zip_pass_option = Some(conf.zip_pass);
        }
        let _result = collector_obj.zip(zip_pass_option).await;
        info!("End to zip output directory");
    }

    let elapsed_time = now.elapsed();
    info!("The execution took {} seconds.", elapsed_time.as_secs());
}
