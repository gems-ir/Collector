use serde::{Deserialize, Serialize};

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