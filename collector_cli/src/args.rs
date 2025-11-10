#[cfg(target_os = "linux")]
use crate::values_linux::*;
#[cfg(target_os = "windows")]
use crate::values_windows::*;
use clap::{Args, Parser, Subcommand};

/// This is the best and fast artifact collector.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct ArgsCollector {
    #[command(subcommand)]
    pub command: Option<ResourcesCommand>,

    /// The source path of collecting artifact.
    #[arg(short,long, default_value=SOURCE_PATH)]
    pub source: String,

    /// The destination path of collecting artifact.
    #[arg(short,long, default_value=DESTINATION_PATH)]
    pub destination: String,

    /// Resources selection.
    /// You can list with "resources" command.
    /// Example: MFT,Prefetch,EVTX
    #[arg(short,long, default_value=RESOURCES_EXAMPLE,value_delimiter = ',')]
    pub resources: Vec<String>,

    /// Path to artifact resources.
    #[arg(short,long,default_value=PATH_RESOURCE)]
    pub path_resources: String,

    /// Zip the output directory.
    #[arg(long)]
    pub zip: bool,

    /// Set zip password.
    #[arg(long)]
    pub pass: Option<String>,

    /// Collect from vss. (Take more time)
    #[cfg(target_os = "windows")]
    #[arg(long)]
    pub vss: bool,

    /// Use config file
    #[arg(long, short)]
    pub config: Option<String>,

    /// Print log output in terminal. (longer)
    #[arg(long)]
    pub log: bool,

    /// Verbose log
    #[arg(short, long)]
    pub verbose: bool,

}

#[derive(Subcommand, Debug)]
pub enum ResourcesCommand {
    /// Resource list options
    Resources(ResourcesArgs),
}

#[derive(Debug, Args)]
pub struct ResourcesArgs {
    #[command(subcommand)]
    pub command: ListResources,
}


#[derive(Debug, Subcommand)]
pub enum ListResources {
    /// List all target names
    Targets,
    /// List all group name
    Groups,
    /// List all categories and his corresponding resource name
    Categories,
}