mod list_parse;
mod args;
#[cfg(target_os = "linux")]
mod values_linux;
#[cfg(target_os = "windows")]
mod values_windows;
mod config;

use args::*;
use chrono::Utc;
use clap::Parser;
use collector_core::resource::{YamlArtifact, YamlParser};
#[cfg(target_os = "windows")]
use collector_core::windows_vss::CollectVss;
use collector_core::Collect;
use config::Config;
use list_parse::ArtifactListing;
use log::*;
use simplelog::*;
use std::fs::File;
use std::panic;
use std::time;
use sysinfo::System;

fn custom_panic_hook() {
    panic::set_hook(Box::new(|info| {
        // Check if the panic has a message
        if let Some(s) = info.payload().downcast_ref::<&str>() {
            eprintln!("Error : {}", s);
        } else {
            eprintln!("Error : An unexpected error occurred.");
        }
    }));
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    custom_panic_hook();

    // Argument parser
    let mut args = ArgsCollector::parse();

    let config_path = args.config.clone();
    if let Some(path) = config_path {
        Config::parse_config_file(path, &mut args)?;
    }

    if let Some(command) = args.command {
        match command {
            ResourcesCommand::Resources(listing) => {
                let mut parser_obj = YamlParser::new(args.path_resources.clone());
                let doc_artifacts = parser_obj.get_doc_struct().await;
                let load_art_list = ArtifactListing::load(doc_artifacts);

                match listing.command {
                    ListResources::Targets => {
                        for name in load_art_list.names_pa() {
                            println!("{}", name);
                        }
                    }
                    ListResources::Groups => {
                        for name in load_art_list.names_gr() {
                            println!("{}", name);
                        }
                    }
                    ListResources::Categories => {
                        for name in load_art_list.list_categories() {
                            println!("{}", name);
                        }
                    }
                }
            }
        }
        return Ok(());
    }


    let mut log_config = ConfigBuilder::new()
        .set_time_format_rfc3339()
        .add_filter_ignore_str("collector_engine::collect")
        .build();
    if args.verbose {
        log_config = ConfigBuilder::new()
            .set_time_format_rfc3339()
            .build();
    }
    let get_time = Utc::now().timestamp().to_string();
    let get_hostname = System::host_name().unwrap();
    let name_log_file = format!("collector_{}_{}.log", get_hostname, get_time);
    // logger
    if args.log {
        CombinedLogger::init(vec![
            TermLogger::new(
                LevelFilter::Info,
                log_config.clone(),
                TerminalMode::Mixed,
                ColorChoice::Auto,
            ),
            WriteLogger::new(
                LevelFilter::Info,
                log_config.clone(),
                File::create(&name_log_file).unwrap(),
            ),
        ]).unwrap();
    } else {
        CombinedLogger::init(vec![
            WriteLogger::new(
                LevelFilter::Info,
                log_config.clone(),
                File::create(&name_log_file).unwrap(),
            ),
        ]).unwrap();
    }

    let now = time::Instant::now();

    info!("{}","=".repeat(50));
    info!("Source of artifact: \"{}\"",args.source);
    info!("Destination of artifact: \"{}\"",args.destination);
    info!("List of resources collected: {:?}",args.resources);
    info!("Access path to resource files: \"{}\"",args.path_resources);
    info!("Output file log: \"{}\"",&name_log_file);
    info!("{}","=".repeat(50));


    // Parse yaml files in resources folder
    info!("Start of yaml resource files analysis ");
    let arg_resources = args.resources;
    let mut parser_obj: YamlParser = YamlParser::new(args.path_resources);
    let doc_artifacts: Vec<YamlArtifact> = parser_obj.get_doc_struct().await;
    let list_artifacts: Vec<String> = parser_obj.select_artifact(arg_resources, doc_artifacts);
    info!("End of yaml resource file analysis");


    // Start collect
    info!("Start to collect artifact");
    let mut collector_obj = Collect::new(args.source.clone(), args.destination.clone(), list_artifacts.clone()).await;
    let _collector_obj_start = collector_obj.start().await;
    info!("End to collect artifact");

    // Start collect vss
    #[cfg(target_os = "windows")]
    if args.vss {
        info!("Start to collect artifact from VSS");
        let vss_obj = CollectVss::new(args.source.clone(), args.destination.clone(), list_artifacts.clone());
        vss_obj.collect().await;
        info!("End to collect artifact from VSS");
    }

    // zip if it's need
    if args.zip {
        info!("Start to zip output directory");
        let _result = collector_obj.zip(args.pass).await;
        info!("End to zip output directory");
    }

    let elapsed_time = now.elapsed();
    info!("The execution took {} seconds.", elapsed_time.as_secs());
    Ok(())
}