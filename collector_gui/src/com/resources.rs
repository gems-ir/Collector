use std::collections::HashSet;
use collector_core::prelude::*;
use crate::com::config::Config;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Resource {
    pub name: String,
    pub category: String,
    pub description: String,
    pub content: String,
    pub is_checked: bool,
}

pub async fn load_resources(config: &Config) -> Vec<Resource> {
    let Some(ref resource_path) = config.resource_path else {
        return Vec::new();
    };

    let parser = match ResourcesParser::new(resource_path) {
        Ok(p) => p,
        Err(_) => return Vec::new(),
    };

    let artifacts = match parser.get_doc_struct().await {
        Ok(a) => a,
        Err(_) => return Vec::new(),
    };

    artifacts
        .iter()
        .filter(|a| a.artifact.path.is_some())
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

pub fn get_categories(resources: &[Resource]) -> Vec<String> {
    let cats: HashSet<String> = resources.iter().map(|r| r.category.clone()).collect();
    let mut sorted_cats: Vec<String> = cats.into_iter().collect();
    sorted_cats.sort();
    sorted_cats
}

pub fn filter_resources(
    resources: &[Resource],
    search_query: &str,
    selected_category: &str,
    show_selected_only: bool,
    checked_resources: &[String],
) -> Vec<Resource> {
    let query = search_query.to_lowercase();
    
    resources
        .iter()
        .filter(|resource| {
            let name_matches = query.is_empty() || resource.name.to_lowercase().contains(&query);
            let category_matches = selected_category == "All" || resource.category == selected_category;
            let selected_matches = !show_selected_only || checked_resources.contains(&resource.name);
            
            name_matches && category_matches && selected_matches
        })
        .cloned()
        .collect()
}
