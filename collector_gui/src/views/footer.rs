use iced::widget::{Column, Space, button, container, progress_bar, row, text};
use iced::{Alignment, Element, Length};

use crate::gui::message::Message;
use crate::gui::{CollectionState, CollectorApp};
use crate::style::icons::{self, icon_button, icon_small};
use crate::style::theme::{
    icon_button_style, primary_button_style, progress_bar_style, status_bar_style,
};

pub fn view_footer(app: &CollectorApp) -> Element<'_, Message> {
    let status_icon: Element<'_, Message> = match app.collection_state {
        CollectionState::Collecting => icon_small(icons::LOADER),
        CollectionState::Completed => icon_small(icons::CHECK),
        CollectionState::Error => icon_small(icons::X),
        CollectionState::Ready => icon_small(icons::SHIELD_CHECK),
    };

    let status_text = text(&app.collection_message).size(13);

    let status_row = row![status_icon, status_text]
        .spacing(8)
        .align_y(Alignment::Center);

    let is_dark = app.is_dark;
    let status_container = container(status_row)
        .width(Length::Fill)
        .padding([12, 16])
        .style(move |_| status_bar_style(is_dark));

    let theme_btn = button(if app.is_dark {
        icon_button(icons::SUN)
    } else {
        icon_button(icons::MOON)
    })
    .on_press(Message::ToggleTheme)
    .padding(8)
    .style(icon_button_style(app.is_dark));

    // Progress bar (visible only when collecting)
    let progress_section: Element<'_, Message> =
        if app.collection_state == CollectionState::Collecting {
            let percentage = if app.progress_total > 0 {
                app.progress_current as f32 / app.progress_total as f32
            } else {
                0.0
            };

            let progress_text = format!("{}/{}", app.progress_current, app.progress_total);

            let bar = progress_bar(0.0..=1.0, percentage)
                .length(150)
                .style(progress_bar_style(is_dark));

            let progress_label = text(progress_text).size(11);

            Column::new()
                .push(bar)
                .push(progress_label)
                .spacing(4)
                .align_x(Alignment::Center)
                .into()
        } else {
            Space::new().width(Length::Fixed(0.0)).into()
        };

    let start_btn = if app.collection_state == CollectionState::Collecting {
        button(
            row![icon_button(icons::LOADER), text(" Collecting...").size(14)]
                .spacing(8)
                .align_y(Alignment::Center),
        )
        .padding([10, 20])
        .style(primary_button_style)
    } else {
        button(
            row![icon_button(icons::PLAY), text(" Start Collection").size(14)]
                .spacing(8)
                .align_y(Alignment::Center),
        )
        .on_press(Message::StartCollection)
        .padding([10, 20])
        .style(primary_button_style)
    };

    row![status_container, theme_btn, progress_section, start_btn]
        .spacing(8)
        .padding([8, 0])
        .align_y(Alignment::Center)
        .width(Length::Fill)
        .into()
}
