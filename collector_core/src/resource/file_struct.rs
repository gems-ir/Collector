//! YAML artifact resource structures.

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct YamlArtifact {
    pub metadata: Metadata,
    pub artifact: Artifact,
}

impl YamlArtifact {
    pub fn is_group(&self) -> bool {
        self.artifact.group.is_some()
    }

    pub fn has_paths(&self) -> bool {
        self.artifact.path.is_some()
    }

    pub fn paths(&self) -> Option<&Vec<String>> {
        self.artifact.path.as_ref()
    }

    pub fn groups(&self) -> Option<&Vec<String>> {
        self.artifact.group.as_ref()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Target {
    Linux,
    Windows,
}

impl Target {
    pub fn matches_current_os(&self) -> bool {
        match self {
            Target::Linux => cfg!(target_os = "linux"),
            Target::Windows => cfg!(target_os = "windows"),
        }
    }

    pub fn current() -> Self {
        if cfg!(target_os = "windows") {
            Target::Windows
        } else {
            Target::Linux
        }
    }
}

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Target::Linux => write!(f, "Linux"),
            Target::Windows => write!(f, "Windows"),
        }
    }
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

impl Metadata {
    pub fn category_or_default(&self) -> &str {
        self.category.as_deref().unwrap_or("Other")
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct Artifact {
    pub path: Option<Vec<String>>,
    pub group: Option<Vec<String>>,
}

impl Artifact {
    pub fn with_paths(paths: Vec<String>) -> Self {
        Self { path: Some(paths), group: None }
    }

    pub fn with_group(group: Vec<String>) -> Self {
        Self { path: None, group: Some(group) }
    }

    pub fn is_valid(&self) -> bool {
        matches!((&self.path, &self.group), (Some(_), None) | (None, Some(_)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_matches_current_os() {
        let current = Target::current();
        assert!(current.matches_current_os());
    }

    #[test]
    fn test_target_display() {
        assert_eq!(format!("{}", Target::Linux), "Linux");
        assert_eq!(format!("{}", Target::Windows), "Windows");
    }

    #[test]
    fn test_metadata_category_or_default() {
        let with_category = Metadata {
            name: "Test".to_string(),
            description: "Test".to_string(),
            date: None,
            category: Some("FileSystem".to_string()),
            target: Target::Windows,
            source: None,
        };
        assert_eq!(with_category.category_or_default(), "FileSystem");

        let without_category = Metadata {
            name: "Test".to_string(),
            description: "Test".to_string(),
            date: None,
            category: None,
            target: Target::Windows,
            source: None,
        };
        assert_eq!(without_category.category_or_default(), "Other");
    }

    #[test]
    fn test_artifact_is_valid() {
        let path_artifact = Artifact::with_paths(vec!["test".to_string()]);
        assert!(path_artifact.is_valid());

        let group_artifact = Artifact::with_group(vec!["MFT".to_string()]);
        assert!(group_artifact.is_valid());

        let invalid_both = Artifact {
            path: Some(vec!["test".to_string()]),
            group: Some(vec!["MFT".to_string()]),
        };
        assert!(!invalid_both.is_valid());

        let invalid_none = Artifact::default();
        assert!(!invalid_none.is_valid());
    }

    #[test]
    fn test_yaml_artifact_helpers() {
        let path_artifact = YamlArtifact {
            metadata: Metadata {
                name: "Test".to_string(),
                description: "Test".to_string(),
                date: None,
                category: None,
                target: Target::Windows,
                source: None,
            },
            artifact: Artifact::with_paths(vec!["path1".to_string()]),
        };

        assert!(path_artifact.has_paths());
        assert!(!path_artifact.is_group());
        assert_eq!(path_artifact.paths().unwrap().len(), 1);
    }
}
