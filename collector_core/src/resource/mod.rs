mod file_struct;
mod parser;

pub use file_struct::{Artifact, Metadata, Target, YamlArtifact};
pub use parser::ResourcesParser;

// Alias for backward compatibility
pub type YamlParser = ResourcesParser;
