use crate::app::utils::containerized;
use iced::widget::{button, row};
use iced::Element;

#[derive(Default, Debug)]
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
        containerized(buttons)
    }
}