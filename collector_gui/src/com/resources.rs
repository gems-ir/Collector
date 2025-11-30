use std::collections::HashSet;

use collector_core::resource::YamlParser;

use crate::com::config::Config;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Resource {
    pub name: String,
    pub category: String,
    pub description: String,
    pub content: String,
    pub is_checked: bool,
}

/// Loads resources in async mode 
pub async fn load_resources(config: &Config) -> Vec<Resource> {
    let Some(ref resource_path) = config.resource_path else {
        return Vec::new();
    };

    let mut parser_obj = YamlParser::new(resource_path.clone());
    let doc_artifacts = parser_obj.get_doc_struct().await;

    doc_artifacts
        .iter()
        .map(|artifact| {
            let name = artifact.metadata.name.clone();
            let is_checked = config
                .resource_list
                .as_ref()
                .map(|names| names.contains(&name))
                .unwrap_or(false);

            Resource {
                name,
                category: artifact
                    .metadata
                    .category
                    .clone()
                    .unwrap_or_else(|| "Other".to_string()),
                description: artifact.metadata.description.clone(),
                content: toml::to_string(artifact).unwrap_or_default(),
                is_checked,
            }
        })
        .collect()
}

/// Extract unique categories of resources
pub fn get_categories(resources: &[Resource]) -> Vec<String> {
    let cats: HashSet<String> = resources.iter().map(|r| r.category.clone()).collect();
    let mut sorted_cats: Vec<String> = cats.into_iter().collect();
    sorted_cats.sort();
    sorted_cats
}

/// Filter resource by search, category and selection
pub fn filter_resources(
    resources: &[Resource],
    search_query: &str,
    selected_category: &str,
    show_selected_only: bool,
    checked_resources: &[String],
) -> Vec<Resource> {
    resources
        .iter()
        .filter(|resource| {
            let query = search_query.to_lowercase();
            let name_matches = if query.is_empty() {
                true
            } else {
                resource.name.to_lowercase().contains(&query)
            };

            let category_matches = if selected_category == "All" {
                true
            } else {
                resource.category == selected_category
            };

            let selected_matches = if show_selected_only {
                checked_resources.contains(&resource.name)
            } else {
                true
            };

            name_matches && category_matches && selected_matches
        })
        .cloned()
        .collect()
}
