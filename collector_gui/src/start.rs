use iced::widget::{button, container, row};
use iced::{Border, Center, Element, Fill};

#[derive(Default)]
pub(crate) struct Start {}

#[derive(Debug, Clone)]
pub(crate) enum StartMsg {
    SettingClicked,
    StartClicked,
}

impl Start {
    pub(crate) fn view(&self) -> Element<'_, StartMsg> {
        let buttons = row![
            button("Settings").on_press(StartMsg::SettingClicked),
            button("Start").on_press(StartMsg::StartClicked),
        ]
            .spacing(10);
        container(buttons)
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