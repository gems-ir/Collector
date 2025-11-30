use iced::widget::{button, container, horizontal_space, row, text};
use iced::{Alignment, Element, Length};

use crate::style::icons::{self, icon_button, icon_small};
use crate::style::theme::{icon_button_style, primary_button_style, status_bar_style};
use crate::gui::message::Message;
use crate::gui::{CollectionState, CollectorApp};

/// View of the footer with status and buttons
pub fn view_footer(app: &CollectorApp) -> Element<'_, Message> {
    let status_icon: Element<'_, Message> = if app.collection_state == CollectionState::Collecting {
        icon_small(icons::LOADER)
    } else if app.collection_message.contains("completed") {
        icon_small(icons::CHECK)
    } else {
        icon_small(icons::SHIELD_CHECK)
    };

    let status_text = text(&app.collection_message).size(13);

    let status_row = row![status_icon, status_text]
        .spacing(10)
        .align_y(Alignment::Center);

    let is_dark = app.is_dark;
    let status_container = container(status_row)
        .width(Length::Fill)
        .padding([12, 15])
        .style(move |_| status_bar_style(is_dark));

    let theme_btn = button(if app.is_dark {
        icon_button(icons::SUN)
    } else {
        icon_button(icons::MOON)
    })
    .on_press(Message::ToggleTheme)
    .padding(10)
    .style(icon_button_style(app.is_dark));

    let start_btn = if app.collection_state == CollectionState::Collecting {
        button(
            row![icon_button(icons::LOADER), text(" Collecting...").size(14)]
                .spacing(8)
                .align_y(Alignment::Center),
        )
        .padding([12, 24])
        .style(primary_button_style)
    } else {
        button(
            row![icon_button(icons::PLAY), text(" Start Collection").size(14)]
                .spacing(8)
                .align_y(Alignment::Center),
        )
        .on_press(Message::StartCollection)
        .padding([12, 24])
        .style(primary_button_style)
    };

    row![status_container, horizontal_space().width(15), theme_btn, start_btn]
        .spacing(10)
        .padding([10, 0])
        .align_y(Alignment::Center)
        .into()
}
