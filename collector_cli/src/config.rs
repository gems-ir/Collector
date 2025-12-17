use crate::args::ArgsCollector;
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Deserialize, Clone)]
pub(crate) struct Config {
    source_path: String,
    destination_path: String,
    path_resources: String,
    list_resources: Vec<String>,
    verbose: Option<bool>,
    zip: Option<bool>,
    zip_pass: Option<String>,
    #[cfg(target_os = "windows")]
    vss: Option<bool>,
    log: Option<bool>,
}

impl Config {
    pub(crate) fn parse_config_file<P: AsRef<Path>>(
        path: P,
        args: &mut ArgsCollector,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path_ref = path.as_ref();

        if !path_ref.exists() {
            return Err(format!("Config file does not exist: {}", path_ref.display()).into());
        }

        let config_reader = fs::read(path_ref)?;
        let config: Config = toml::from_slice(&config_reader)?;
        config.merge_args(args);

        Ok(())
    }

    fn merge_args(self, args: &mut ArgsCollector) {
        // let default_resources: Vec<String> = RESOURCES_EXAMPLE.split(',').map(|s| s.to_string()).collect();

        args.source = self.source_path;
        args.destination = self.destination_path;
        args.path_resources = self.path_resources;
        args.resources = self.list_resources;

        if !args.zip {
            args.zip = self.zip.unwrap_or(false);
        }

        if args.pass.is_none() {
            args.pass = self.zip_pass;
        }

        #[cfg(target_os = "windows")]
        if !args.vss {
            args.vss = self.vss.unwrap_or(false);
        }

        if !args.log {
            args.log = self.log.unwrap_or(false);
        }

        if !args.verbose {
            args.verbose = self.verbose.unwrap_or(false);
        }
    }
}
