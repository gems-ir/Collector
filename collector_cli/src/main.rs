mod args;
mod config;

#[cfg(target_os = "linux")]
mod values_linux;
#[cfg(target_os = "windows")]
mod values_windows;

use args::{ArgsCollector, ListResources, ResourcesCommand};
use clap::Parser;
use collector_core::prelude::*;
use config::Config;
use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, ConfigBuilder, TermLogger, TerminalMode, WriteLogger};
use std::fs::File;
use std::time::Instant;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn setup_logging(args: &ArgsCollector, log_filename: &str) -> Result<()> {
    let config = if args.verbose {
        ConfigBuilder::new().set_time_format_rfc3339().build()
    } else {
        ConfigBuilder::new()
            .set_time_format_rfc3339()
            .add_filter_ignore_str("collector_core")
            .build()
    };

    let file = File::create(log_filename)
        .map_err(|e| CollectorError::FileWrite { 
            path: log_filename.into(), 
            source: e 
        })?;

    let loggers: Vec<Box<dyn simplelog::SharedLogger>> = if args.log {
        vec![
            TermLogger::new(LevelFilter::Info, config.clone(), TerminalMode::Mixed, ColorChoice::Auto),
            WriteLogger::new(LevelFilter::Info, config, file),
        ]
    } else {
        vec![WriteLogger::new(LevelFilter::Info, config, file)]
    };

    CombinedLogger::init(loggers)
        .map_err(|e| CollectorError::Config(e.to_string()))?;

    Ok(())
}

fn print_header() {
    println!();
    println!("╔══════════════════════════════════════════════════╗");
    println!("║            COLLECTOR CLI v{}                 ║", VERSION);
    println!("║       Fast Artifact Collector for DFIR           ║");
    println!("╚══════════════════════════════════════════════════╝");
    println!();
}

fn print_separator() {
    println!("{}", "─".repeat(52));
}

async fn handle_resources_command(args: &ArgsCollector, cmd: &ListResources) -> Result<()> {
    let parser = ResourcesParser::new(&args.path_resources)?;
    let artifacts = parser.get_doc_struct().await?;

    match cmd {
        ListResources::Targets => {
            println!("\n┌─ Available Targets ─────────────────────────────┐");
            let targets: Vec<_> = artifacts.iter()
                .filter(|a| a.artifact.path.is_some())
                .collect();
            
            for artifact in targets {
                let category = artifact.metadata.category.as_deref().unwrap_or("Other");
                println!("│  {:<30} [{}]", artifact.metadata.name, category);
            }
            println!("└──────────────────────────────────────────────────┘");
        }
        ListResources::Groups => {
            println!("\n┌─ Available Groups ──────────────────────────────┐");
            for artifact in artifacts.iter().filter(|a| a.artifact.group.is_some()) {
                println!("│  {}", artifact.metadata.name);
                if let Some(groups) = &artifact.artifact.group {
                    for (i, g) in groups.iter().enumerate() {
                        let prefix = if i == groups.len() - 1 { "└──" } else { "├──" };
                        println!("│     {} {}", prefix, g);
                    }
                }
            }
            println!("└──────────────────────────────────────────────────┘");
        }
        ListResources::Categories => {
            println!("\n┌─ Categories ────────────────────────────────────┐");
            let categories = ResourcesParser::get_by_category(&artifacts);
            let mut sorted_categories: Vec<_> = categories.iter().collect();
            sorted_categories.sort_by_key(|(k, _)| *k);
            
            for (category, names) in sorted_categories {
                println!("│");
                println!("│  [{}]", category);
                for name in names {
                    println!("│     • {}", name);
                }
            }
            println!("└──────────────────────────────────────────────────┘");
        }
    }

    Ok(())
}

async fn run_collection(args: ArgsCollector) -> Result<()> {
    let hostname = sysinfo::System::host_name().unwrap_or_else(|| "unknown".into());
    let timestamp = chrono::Utc::now().timestamp();
    let log_filename = format!("collector_{}_{}.log", hostname, timestamp);
    let verbose = args.verbose;

    setup_logging(&args, &log_filename)?;

    print_header();

    log::info!("{}", "=".repeat(50));
    log::info!("Source: {}", args.source);
    log::info!("Destination: {}", args.destination);
    log::info!("Resources: {:?}", args.resources);
    log::info!("Resources path: {}", args.path_resources);
    log::info!("Log file: {}", log_filename);
    log::info!("{}", "=".repeat(50));

    println!("  Source:       {}", args.source);
    println!("  Destination:  {}", args.destination);
    println!("  Resources:    {:?}", args.resources);
    if verbose {
        println!("  Resources path: {}", args.path_resources);
    }
    println!("  Log file:     {}", log_filename);
    print_separator();

    // Parse resources
    println!("\n[1/4] Parsing resource files...");
    log::info!("Parsing resource files");
    
    let mut parser = ResourcesParser::new(&args.path_resources)?;
    let artifacts = parser.get_doc_struct().await?;
    let patterns = parser.select_artifact(args.resources.clone(), &artifacts)?;
    
    println!("      Found {} artifact patterns", patterns.len());
    log::info!("Found {} artifact patterns", patterns.len());

    if verbose {
        println!("      Patterns:");
        for pattern in &patterns {
            println!("        • {}", pattern);
        }
    }

    // Create collector
    println!("\n[2/4] Initializing collector...");
    log::info!("Initializing collector");
    
    let mut collector = ArtifactCollector::new(
        &args.source,
        &args.destination,
        patterns.clone(),
    ).await?;

    let total_files = collector.count_files();
    println!("      Found {} files to collect", total_files);
    log::info!("Found {} files to collect", total_files);

    // Collect
    println!("\n[3/4] Collecting artifacts...");
    log::info!("Starting collection");
    
    let timer = Instant::now();
    let stats = collector.collect().await?;
    let elapsed = timer.elapsed();

    println!("      Collected {} files ({})", stats.files_collected, format_bytes(stats.bytes_collected));
    if verbose {
        println!("      Filesystem extractions: {}", stats.filesystem_extractions);
        println!("      NTFS extractions: {}", stats.ntfs_extractions);
        println!("      Failed extractions: {}", stats.failed_extractions);
    } else {
        println!("      Filesystem: {} | NTFS: {} | Failed: {}", 
            stats.filesystem_extractions, 
            stats.ntfs_extractions, 
            stats.failed_extractions
        );
    }
    log::info!("Collection complete: {} files", stats.files_collected);

    // VSS collection (Windows only)
    #[cfg(target_os = "windows")]
    if args.vss {
        println!("\n[3b/4] Collecting from VSS snapshots...");
        log::info!("Starting VSS collection");
        
        let mut vss_collector = VssCollector::new(
            &args.source,
            &args.destination,
            patterns,
        );
        
        match vss_collector.collect_from_snapshots().await {
            Ok(vss_stats) => {
                println!("      VSS: {} files collected ({})", 
                    vss_stats.files_collected,
                    format_bytes(vss_stats.bytes_collected)
                );
                if verbose {
                    println!("      VSS Filesystem: {}", vss_stats.filesystem_extractions);
                    println!("      VSS NTFS: {}", vss_stats.ntfs_extractions);
                    println!("      VSS Failed: {}", vss_stats.failed_extractions);
                }
                log::info!("VSS collection complete: {} files", vss_stats.files_collected);
            }
            Err(e) => {
                println!("      VSS collection failed: {}", e);
                log::error!("VSS collection failed: {}", e);
            }
        }
    }

    // ZIP
    if args.zip {
        println!("\n[4/4] Creating ZIP archive...");
        log::info!("Creating ZIP archive");
        
        collector.create_archive(args.pass).await?;
        println!("      Archive created successfully");
        log::info!("Archive created");
    } else {
        println!("\n[4/4] Skipping ZIP (not requested)");
    }

    // Summary
    print_separator();
    println!("\n  Collection completed in {:.2}s", elapsed.as_secs_f64());
    println!("  Total files: {}", stats.files_collected);
    println!("  Total size:  {}", format_bytes(stats.bytes_collected));
    println!();

    log::info!("Execution took {:.2}s", elapsed.as_secs_f64());

    Ok(())
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

#[tokio::main]
async fn main() {
    let mut args = ArgsCollector::parse();

    // Load config file if specified
    if let Some(ref config_path) = args.config.clone() {
        if let Err(e) = Config::parse_config_file(config_path.clone(), &mut args) {
            eprintln!("Error loading config: {}", e);
            std::process::exit(0);
        }
    }

    // Handle subcommands
    if let Some(ResourcesCommand::Resources(ref listing)) = args.command {
        if let Err(e) = handle_resources_command(&args, &listing.command).await {
            eprintln!("Error: {}", e);
            std::process::exit(0);
        }
        return;
    }

    // Run collection
    if let Err(e) = run_collection(args).await {
        eprintln!("\nError: {}", e);
        log::error!("Collection failed: {}", e);
        std::process::exit(0);
    }
}
