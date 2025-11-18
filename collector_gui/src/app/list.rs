use crate::app::utils::{containerized, titled};
use crate::config::Config;
use iced::widget::column;
use iced::Element;

#[derive(Default)]
pub(crate) struct ListResources {}

#[derive(Debug, Clone)]
pub(crate) enum ListResourcesMsg {}

impl ListResources {
    pub(crate) fn view(&self, _config: &Config) -> Element<'_, ListResourcesMsg> {
        let lines = column![
            titled("Resources")
        ]
            .spacing(10);
        containerized(lines)
    }
}