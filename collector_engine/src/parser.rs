use crate::helper::FormatSource;
use std::path::{PathBuf, Path};
use tokio::fs;
use glob::glob;
use serde::{Serialize, Deserialize};

type GlobString = String;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct YamlArtifact {
	pub metadata: Metadata,
	pub artifact: Artifact
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Target {
	Linux,
	Windows,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Metadata {
	pub name: String,
	pub description: String,
	pub date: Option<String>,
	pub category: Option<String>,
	pub target: Target,
	pub source: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Artifact {
	pub path: Option<Vec<String>>,
	pub group: Option<Vec<String>>
}

// This structure parse yaml file from resources.
#[derive(Clone)]
pub struct YamlParser {
	pub resource_path: String,
	artifact_element_glob: Vec<GlobString>,
}

impl YamlParser {
	// Initialize  YamlParser structure with default element
	pub fn init() -> Self {
		YamlParser {
			resource_path: String::new(),
			artifact_element_glob: Vec::new(),
		}
	}

	// Create structure with parameter.
	pub fn new(resource_path: String) -> Self {
		let mut format_resource_path = FormatSource::from(resource_path).to_string();
		if !Path::new(&format_resource_path).exists() {
			panic!("Resources path doesn't exists");
		}
		format_resource_path.push_str("**/*.yaml");
		YamlParser {
			resource_path: format_resource_path,
			artifact_element_glob: Vec::new(),
		}
	}

	pub fn get_yaml_file(&self) -> Vec<PathBuf> {
		let mut list_yaml_file = Vec::new();
		for entry in glob(&self.resource_path).expect("Failed to read glob pattern") {
			let path_to_string = entry.unwrap();
			list_yaml_file.push(path_to_string.to_path_buf());
		}
		list_yaml_file
	}

	pub async fn get_doc_struct(&self) -> Vec<YamlArtifact> {
		let mut parse_file = Vec::new();
		for file in &self.get_yaml_file() {
			let reader = fs::read_to_string(file.clone()).await;

			for document in serde_yml::Deserializer::from_str(&reader.unwrap()) {
				let value = YamlArtifact::deserialize(document);
				if let Err(e) = value {
					eprintln!("Error of file {:?}: {:?}", &file, e.to_string());
					continue; // Skip this document and continue with the next
				}
				let value = value.unwrap();
				if should_skip_artifact(&value.metadata.target) {
					continue; // Skip this artifact if it's not for the current OS
				}
				if validate_artifact(&value.artifact).is_err() {
					eprintln!("Error of file {:?}: artifact.group and artifact.path have not been found!", &file);
					continue; // Skip this artifact if validation fails
				}
				parse_file.push(value);
			}
		}
		parse_file
	}

	pub fn get_struct_from_raw(&self, list_filename: Vec<String>, list_raw: Vec<String>) -> Vec<YamlArtifact> {
		let mut parse_file = Vec::new();
		for (num_raw_data, raw_data) in list_raw.iter().enumerate() {
			for document in serde_yml::Deserializer::from_str(raw_data) {
				let value = YamlArtifact::deserialize(document);
				if let Err(e) = value {
					eprintln!("Error of file {}: {:?}", list_filename[num_raw_data], e.to_string());
					continue; // Skip this document and continue with the next
				}
				let value = value.unwrap();
				if should_skip_artifact(&value.metadata.target) {
					continue; // Skip this artifact if it's not for the current OS
				}
				if validate_artifact(&value.artifact).is_err() {
					eprintln!("Error of file {}: artifact.group and artifact.path have not been found!", list_filename[num_raw_data]);
					continue; // Skip this artifact if validation fails
				}
				parse_file.push(value);
			}
		}
		parse_file
	}

	// Recursive function to extract all glob path from yaml and selecting artifact.
	pub fn select_artifact(&mut self, artifacts_name: Vec<GlobString>, doc_artifact: Vec<YamlArtifact>) -> Vec<GlobString> {
		let get_doc_artifact = doc_artifact;
		for artifact_want in artifacts_name {
			let get = &get_doc_artifact.iter().find(|e| e.metadata.name == artifact_want);
			match get {
				Some(struct_element) => {
					match &struct_element.artifact.group {
						Some(name_artifact_file) => self.select_artifact(name_artifact_file.to_vec(), get_doc_artifact.clone()),
						None => Vec::new()
					};
					match &struct_element.artifact.path {
						Some(name_artifact_elements) => name_artifact_elements.iter().for_each(|e| {
							if !self.artifact_element_glob.contains(e) {
								self.artifact_element_glob.push(e.to_string())
							}
						}),
						None => {}
					};
				},
				None => panic!("Error of artifact argument : \"{}\" name not found in file resources", &artifact_want),
			}
		}
		self.artifact_element_glob.clone()
	}
}

// Helper function to determine if an artifact should be skipped based on the target OS
fn should_skip_artifact(target: &Target) -> bool {
	match target {
		Target::Linux => cfg!(target_os = "windows"),
		Target::Windows => cfg!(target_os = "linux"),
		// _ => false,
	}
}

// Helper function to validate the artifact structure
fn validate_artifact(artifact: &Artifact) -> Result<(), &'static str> {
	match (&artifact.path, &artifact.group) {
		(None, None) => Err("Both artifact.group and artifact.path are missing"),
		(Some(_), Some(_)) => Err("Both artifact.group and artifact.path are present, please select one"),
		_ => Ok(()),
	}
}