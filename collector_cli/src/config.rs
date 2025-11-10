use crate::args::ArgsCollector;
#[cfg(target_os = "linux")]
use crate::values_linux::*;
#[cfg(target_os = "windows")]
use crate::values_windows::*;
use serde::Deserialize;
use std::fs;
use std::path::Path;


#[derive(Deserialize, Clone)]
pub(crate) struct Config {
    source_path: Option<String>,
    destination_path: Option<String>,
    resource_path: Option<String>,
    resource_list: Option<Vec<String>>,
    verbose: Option<bool>,
    zip: Option<bool>,
    zip_pass: Option<String>,
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
        if args.source == SOURCE_PATH && let Some(source) = self.source_path {
            args.source = source;
        }

        if args.destination == DESTINATION_PATH && let Some(destination) = self.destination_path {
            args.destination = destination;
        }

        let default_resources: Vec<String> = RESOURCES_EXAMPLE
            .split(',')
            .map(|s| s.to_string())
            .collect();
        if args.resources == default_resources && let Some(resources) = self.resource_list {
            args.resources = resources;
        }

        if args.path_resources == PATH_RESOURCE && let Some(path) = self.resource_path {
            args.path_resources = path;
        }

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