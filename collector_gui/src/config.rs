#[cfg(target_os = "linux")]
use crate::values_linux::*;
#[cfg(target_os = "windows")]
use crate::values_windows::*;
use serde::Deserialize;
use std::fs;


#[derive(PartialEq, Deserialize, Clone, Debug, Default)]
pub(crate) struct Config {
    pub(crate) source_path: Option<String>,
    pub(crate) destination_path: Option<String>,
    pub(crate) resource_path: Option<String>,
    pub(crate) resource_list: Option<Vec<String>>,
    pub(crate) verbose: Option<bool>,
    pub(crate) zip: Option<bool>,
    pub(crate) zip_pass: Option<String>,
    pub(crate) vss: Option<bool>,
    pub(crate) log: Option<bool>,
}

impl Config {
    pub(crate) fn parse_config_file() -> Self {
        let filename = if cfg!(target_os = "windows") {
            "collector_config_windows.toml"
        } else if cfg!(target_os = "linux") {
            "collector_config_linux.toml"
        } else {
            ""
        };

        let read_buf = fs::read(filename).unwrap_or_default();
        let mut config: Config = toml::from_slice(&read_buf).unwrap_or_default();
        config.merge_args();

        config
    }

    fn merge_args(&mut self) {
        if self.source_path.is_none() {
            self.source_path = Some(SOURCE_PATH.to_string());
        }

        if self.destination_path.is_none() {
            self.destination_path = Some(DESTINATION_PATH.to_string());
        }

        let default_resources: Vec<String> = RESOURCES_EXAMPLE
            .into_iter()
            .map(String::from)
            .collect();
        if self.resource_list.is_none() {
            self.resource_list = Some(default_resources);
        }

        if self.resource_path.is_none() {
            self.resource_path = Some(PATH_RESOURCE.to_string());
        }

        if self.zip.is_none() {
            self.zip = Some(false);
        }

        if self.zip_pass.is_none() || self.zip_pass == Some("".to_string()) {
            self.zip_pass = None;
        }

        #[cfg(target_os = "windows")]
        if self.vss.is_none() {
            self.vss = Some(false);
        }

        if self.log.is_none() {
            self.log = Some(false);
        }

        if self.verbose.is_none() {
            self.verbose = Some(false);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub(crate) struct AppData {
    pub(crate) source_path: String,
    pub(crate) destination_path: String,
    pub(crate) resource_path: String,
    pub(crate) resource_list: Vec<String>,
    pub(crate) verbose: Option<bool>,
    pub(crate) zip: bool,
    pub(crate) zip_pass: Option<String>,
    pub(crate) vss: bool,
    pub(crate) log: Option<bool>,
}