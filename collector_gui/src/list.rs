use iced::widget::{column, container, text};
use iced::{Border, Center, Element, Fill};

#[derive(Default)]
pub(crate) struct ListResources {}

#[derive(Debug, Clone)]
pub(crate) enum ListResourcesMsg {}

impl ListResources {
    pub(crate) fn view(&self) -> Element<'_, ListResourcesMsg> {
        let lines = column![
            text("Select your resource:".to_string()),
        ]
            .spacing(10);
        container(lines)
            .padding(15)
            .width(Fill)
            .align_x(Center)
            .align_y(Center)
            .style(|_theme| {
                container::Style {
                    border: Border {
                        color: iced::Color::from_rgb(0.5, 0.5, 0.5),
                        width: 2.0,
                        radius: 5.0.into(),
                    },
                    ..Default::default()
                }
            })
            .into()
    }
}