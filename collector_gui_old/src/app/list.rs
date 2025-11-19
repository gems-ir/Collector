use crate::app::utils::{containerized, titled};
use crate::config::Config;
use collector_core::resource::YamlArtifact;
use iced::widget::{column, keyed_column, row, text, text_input};
use iced::{Center, Element};
use uuid::Uuid;

#[derive(Default, Debug)]
pub(crate) struct ListResources {
    search_input: String,
    resource_list: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) enum ListResourcesMsg {}

impl ListResources {
    pub(crate) fn new() -> Self {
        ListResources {
            search_input: String::new(),
            // resource_list: vec!["one ca marches".to_string(), "deux c'est cool".to_string()],
            resource_list: Vec::new(),
        }
    }

    pub(crate) fn view(&self, _config: &Config) -> Element<'_, ListResourcesMsg> {
        let listing: Element<_> = keyed_column(
            self.resource_list
                .iter()
                .enumerate()
                .map(|(i, s)| (i, text(s).into())
                ))
            .into();
        let lines = column![
            titled("Resources"),
            text_input("Search in resources", &self.search_input),
            listing,
        ]
            .spacing(10)
            .align_x(Center);
        containerized(lines)
    }
}

#[derive(Debug)]
pub(crate) struct Resource {
    id: Uuid,
    name: String,
    category: String,
    content: String,
}

#[derive(Debug)]
pub(crate) enum ResourceMsg {}

impl Resource {
    pub(crate) fn new(yaml_art: YamlArtifact) -> Self {
        let raw_artifact: String = serde_yml::to_string(&yaml_art).unwrap();
        let category = yaml_art.metadata.category.clone().unwrap_or("Other".to_string());
        Self {
            id: Uuid::new_v4(),
            name: yaml_art.metadata.name,
            category,
            content: raw_artifact,
        }
    }
    pub(crate) fn view(&self) -> Element<'_, ResourceMsg> {
        row![
            text(self.name.clone()),
        ]
            .into()
    }
}