//! Resource file parser (YAML).

use glob::glob;
use serde::Deserialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tokio::fs;

use crate::error::{CollectorError, Result};
use crate::resource::file_struct::*;
use crate::utils::FormatSource;

/// Parser for YAML artifact resources.
#[derive(Clone, Debug)]
pub struct ResourcesParser {
    resource_path: FormatSource,
    artifact_patterns: Vec<String>,
    processed_artifacts: HashSet<String>,
}

impl ResourcesParser {
    pub fn new<P: AsRef<Path>>(resource_path: P) -> Result<Self> {
        let path = resource_path.as_ref();

        if !path.exists() {
            return Err(CollectorError::ResourcesDirectoryNotFound(
                path.to_path_buf(),
            ));
        }

        let mut format_path = FormatSource::new(path);
        format_path.push("**/*.yaml");

        Ok(Self {
            resource_path: format_path,
            artifact_patterns: Vec::new(),
            processed_artifacts: HashSet::new(),
        })
    }

    pub fn with_default_path() -> Result<Self> {
        Self::new("./Resources/")
    }

    pub fn resource_path(&self) -> &FormatSource {
        &self.resource_path
    }

    pub fn get_yaml_files(&self) -> Vec<PathBuf> {
        let pattern = self.resource_path.to_string_lossy();
        glob(&pattern)
            .map(|entries| entries.filter_map(|e| e.ok()).collect())
            .unwrap_or_default()
    }

    pub async fn get_doc_struct(&self) -> Result<Vec<YamlArtifact>> {
        let mut artifacts = Vec::new();

        for file_path in self.get_yaml_files() {
            match self.parse_yaml_file(&file_path).await {
                Ok(file_artifacts) => artifacts.extend(file_artifacts),
                Err(e) => log::warn!("Failed to parse {}: {}", file_path.display(), e),
            }
        }

        Ok(artifacts)
    }

    async fn parse_yaml_file(&self, path: &Path) -> Result<Vec<YamlArtifact>> {
        let content = fs::read_to_string(path)
            .await
            .map_err(|e| CollectorError::FileRead {
                path: path.to_path_buf(),
                source: e,
            })?;

        let mut artifacts = Vec::new();

        for document in serde_yml::Deserializer::from_str(&content) {
            match YamlArtifact::deserialize(document) {
                Ok(artifact) => {
                    if !artifact.metadata.target.matches_current_os() {
                        continue;
                    }

                    if let Err(e) = validate_artifact(&artifact) {
                        log::warn!("Invalid artifact '{}': {}", artifact.metadata.name, e);
                        continue;
                    }

                    artifacts.push(artifact);
                }
                Err(e) => {
                    return Err(CollectorError::ResourceParse {
                        path: path.to_path_buf(),
                        reason: e.to_string(),
                    });
                }
            }
        }

        Ok(artifacts)
    }

    pub fn parse_from_raw(&self, filenames: &[String], contents: &[String]) -> Vec<YamlArtifact> {
        let mut artifacts = Vec::new();

        for (idx, content) in contents.iter().enumerate() {
            let filename = filenames.get(idx).map(|s| s.as_str()).unwrap_or("unknown");

            for document in serde_yml::Deserializer::from_str(content) {
                match YamlArtifact::deserialize(document) {
                    Ok(artifact) => {
                        if !artifact.metadata.target.matches_current_os() {
                            continue;
                        }
                        if validate_artifact(&artifact).is_err() {
                            continue;
                        }
                        artifacts.push(artifact);
                    }
                    Err(e) => log::error!("Failed to parse {}: {}", filename, e),
                }
            }
        }

        artifacts
    }

    pub fn select_artifact(
        &mut self,
        artifact_names: Vec<String>,
        all_artifacts: &[YamlArtifact],
    ) -> Result<Vec<String>> {
        self.artifact_patterns.clear();
        self.processed_artifacts.clear();

        self.resolve_artifacts_recursive(&artifact_names, all_artifacts)?;

        Ok(self.artifact_patterns.clone())
    }

    fn resolve_artifacts_recursive(
        &mut self,
        names: &[String],
        all_artifacts: &[YamlArtifact],
    ) -> Result<()> {
        for name in names {
            if self.processed_artifacts.contains(name) {
                continue;
            }

            let artifact = all_artifacts
                .iter()
                .find(|a| a.metadata.name == *name)
                .ok_or_else(|| CollectorError::ResourceNotFound(name.clone()))?;

            self.processed_artifacts.insert(name.clone());

            if let Some(group_refs) = &artifact.artifact.group {
                self.resolve_artifacts_recursive(group_refs, all_artifacts)?;
            }

            if let Some(paths) = &artifact.artifact.path {
                for path in paths {
                    let normalized = normalize_artifact_path(path);
                    if !self.artifact_patterns.contains(&normalized) {
                        self.artifact_patterns.push(normalized);
                    }
                }
            }
        }

        Ok(())
    }

    pub fn get_artifact_names(artifacts: &[YamlArtifact]) -> Vec<String> {
        artifacts.iter().map(|a| a.metadata.name.clone()).collect()
    }

    pub fn get_by_category(
        artifacts: &[YamlArtifact],
    ) -> std::collections::HashMap<String, Vec<String>> {
        let mut categories = std::collections::HashMap::new();

        for artifact in artifacts {
            let category = artifact.metadata.category_or_default().to_string();
            categories
                .entry(category)
                .or_insert_with(Vec::new)
                .push(artifact.metadata.name.clone());
        }

        categories
    }

    pub fn get_targets(artifacts: &[YamlArtifact]) -> Vec<&YamlArtifact> {
        artifacts.iter().filter(|a| a.has_paths()).collect()
    }

    pub fn get_groups(artifacts: &[YamlArtifact]) -> Vec<&YamlArtifact> {
        artifacts.iter().filter(|a| a.is_group()).collect()
    }
}

fn validate_artifact(artifact: &YamlArtifact) -> Result<()> {
    if !artifact.artifact.is_valid() {
        let reason = match (&artifact.artifact.path, &artifact.artifact.group) {
            (None, None) => "neither 'path' nor 'group' defined",
            (Some(_), Some(_)) => "both 'path' and 'group' defined",
            _ => "unknown error",
        };
        return Err(CollectorError::InvalidResource {
            name: artifact.metadata.name.clone(),
            reason: reason.to_string(),
        });
    }
    Ok(())
}

fn normalize_artifact_path(path: &str) -> String {
    let mut normalized = path.to_string();
    while normalized.starts_with('\\') || normalized.starts_with('/') {
        normalized.remove(0);
    }
    normalized
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;

    fn create_test_artifact(
        name: &str,
        paths: Option<Vec<&str>>,
        group: Option<Vec<&str>>,
    ) -> YamlArtifact {
        YamlArtifact {
            metadata: Metadata {
                name: name.to_string(),
                description: format!("Test artifact {}", name),
                date: None,
                category: Some("Test".to_string()),
                target: Target::current(),
                source: None,
            },
            artifact: Artifact {
                path: paths.map(|p| p.iter().map(|s| s.to_string()).collect()),
                group: group.map(|g| g.iter().map(|s| s.to_string()).collect()),
            },
        }
    }

    #[test]
    fn test_normalize_artifact_path() {
        assert_eq!(
            normalize_artifact_path("\\Windows\\System32"),
            "Windows\\System32"
        );
        assert_eq!(normalize_artifact_path("/home/user"), "home/user");
        assert_eq!(normalize_artifact_path("\\\\path"), "path");
        assert_eq!(normalize_artifact_path("normal/path"), "normal/path");
    }

    #[test]
    fn test_validate_artifact_valid_path() {
        let artifact = create_test_artifact("Test", Some(vec!["path1"]), None);
        assert!(validate_artifact(&artifact).is_ok());
    }

    #[test]
    fn test_validate_artifact_valid_group() {
        let artifact = create_test_artifact("Test", None, Some(vec!["Other"]));
        assert!(validate_artifact(&artifact).is_ok());
    }

    #[test]
    fn test_validate_artifact_invalid_both() {
        let artifact = create_test_artifact("Test", Some(vec!["path"]), Some(vec!["group"]));
        assert!(validate_artifact(&artifact).is_err());
    }

    #[test]
    fn test_validate_artifact_invalid_none() {
        let artifact = create_test_artifact("Test", None, None);
        assert!(validate_artifact(&artifact).is_err());
    }

    #[test]
    fn test_select_artifact_simple() {
        let artifacts = vec![
            create_test_artifact("MFT", Some(vec!["\\$MFT"]), None),
            create_test_artifact("Prefetch", Some(vec!["\\Windows\\Prefetch\\*"]), None),
        ];

        let mut parser = YamlParser {
            resource_path: FormatSource::new("."),
            artifact_patterns: Vec::new(),
            processed_artifacts: HashSet::new(),
        };

        let result = parser.select_artifact(vec!["MFT".to_string()], &artifacts);
        assert!(result.is_ok());

        let patterns = result.unwrap();
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0], "$MFT");
    }

    #[test]
    fn test_select_artifact_group() {
        let artifacts = vec![
            create_test_artifact("MFT", Some(vec!["\\$MFT"]), None),
            create_test_artifact("USN", Some(vec!["\\$Extend\\$UsnJrnl"]), None),
            create_test_artifact("NTFS", None, Some(vec!["MFT", "USN"])),
        ];

        let mut parser = YamlParser {
            resource_path: FormatSource::new("."),
            artifact_patterns: Vec::new(),
            processed_artifacts: HashSet::new(),
        };

        let result = parser.select_artifact(vec!["NTFS".to_string()], &artifacts);
        assert!(result.is_ok());

        let patterns = result.unwrap();
        assert_eq!(patterns.len(), 2);
        assert!(patterns.contains(&"$MFT".to_string()));
        assert!(patterns.contains(&"$Extend\\$UsnJrnl".to_string()));
    }

    #[test]
    fn test_select_artifact_not_found() {
        let artifacts = vec![create_test_artifact("MFT", Some(vec!["\\$MFT"]), None)];

        let mut parser = YamlParser {
            resource_path: FormatSource::new("."),
            artifact_patterns: Vec::new(),
            processed_artifacts: HashSet::new(),
        };

        let result = parser.select_artifact(vec!["NonExistent".to_string()], &artifacts);
        assert!(result.is_err());

        match result.unwrap_err() {
            CollectorError::ResourceNotFound(name) => assert_eq!(name, "NonExistent"),
            _ => panic!("Expected ResourceNotFound error"),
        }
    }

    #[test]
    fn test_get_artifact_names() {
        let artifacts = vec![
            create_test_artifact("A", Some(vec!["a"]), None),
            create_test_artifact("B", Some(vec!["b"]), None),
        ];

        let names = YamlParser::get_artifact_names(&artifacts);
        assert_eq!(names, vec!["A", "B"]);
    }

    #[test]
    fn test_get_by_category() {
        let mut artifacts = vec![
            create_test_artifact("A", Some(vec!["a"]), None),
            create_test_artifact("B", Some(vec!["b"]), None),
        ];
        artifacts[0].metadata.category = Some("Cat1".to_string());
        artifacts[1].metadata.category = Some("Cat2".to_string());

        let categories = YamlParser::get_by_category(&artifacts);
        assert!(categories.contains_key("Cat1"));
        assert!(categories.contains_key("Cat2"));
    }
}
