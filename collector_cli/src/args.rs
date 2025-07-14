use clap::{Parser, Subcommand, Args};

/// This is a best and fast artifact collector.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct ArgsCollector{

    #[command(subcommand)]
    pub command: Option<ResourcesCommand>,

    /// The source path of collecting artifact.
    #[arg(short,long, default_value="C:\\")]
    pub source: String,

    /// The destination path of collecting artifact.
    #[arg(short,long, default_value=".\\out\\")]
    pub destination: String,

    /// Resources selection.
    /// You can list with "resources" command.
    /// Exemple: MFT,Prefetch,EVTX
    #[arg(short,long, default_value="All",value_delimiter = ',')]
    pub resources: Vec<String>,

    /// Path to artifact resources.
    #[arg(short,long,default_value=".\\Resources\\")]
    pub path_resources: String,

    /// Zip the output directory.
    #[arg(long)]
    pub zip: bool,

    /// Set zip password.
    #[arg(long)]
    pub pass: Option<String>,

    /// Collect from vss. (longer)
    #[arg(long)]
    pub vss: bool,

    /// Print log output in terminal. (longer)
    #[arg(long)]
    pub log: bool,

    /// Verbose log
    #[arg(short,long)]
    pub verbose: bool,

}

#[derive(Subcommand,Debug)]
pub enum ResourcesCommand{
    /// Resource list options
    Resources(ResourcesArgs),
}

#[derive(Debug, Args)]
pub struct ResourcesArgs{
    #[command(subcommand)]
    pub command: ListResources,
}


#[derive(Debug, Subcommand)]
pub enum ListResources{
    /// List all target names
    Targets,
    /// List all group name
    Groups,
    /// List all categories and his corresponding resource name
    Categories,
}
