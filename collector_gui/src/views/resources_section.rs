use iced::widget::{checkbox, column, container, pick_list, row, text, text_input};
use iced::{Alignment, Element, Length};

use crate::gui::CollectorApp;
use crate::gui::message::Message;
use crate::style::icons::{self, icon_small};
use crate::style::theme::card_style;
use crate::views::view_resources_table;

/// View of the complete resources section
pub fn view_resources_section(app: &CollectorApp) -> Element<'_, Message> {
    let filter_row = view_filter_bar(app);
    let table = view_resources_table(app);
    let selected_row = view_selected_resources(app);

    let content = column![filter_row, table, selected_row].spacing(10);

    container(content)
        .width(Length::Fill)
        .height(Length::FillPortion(3))
        .padding(5)
        .style(move |_| card_style(app.is_dark))
        .into()
}

/// Search bar and filter by category
fn view_filter_bar(app: &CollectorApp) -> Element<'_, Message> {
    let search_icon = icon_small(icons::SEARCH);

    let search_input = text_input("Search resources...", &app.search_query)
        .on_input(Message::SearchQueryChanged)
        .width(Length::Fixed(220.0))
        .padding([4, 8]);

    let category_picker = pick_list(
        app.categories.clone(),
        Some(app.selected_category.clone()),
        Message::CategorySelected,
    )
    .placeholder("All Categories")
    .padding([4, 8]);

    let show_selected_checkbox = checkbox(app.show_selected_only)
        .label("Selected only")
        .on_toggle(Message::ToggleShowSelectedOnly)
        .text_size(12);

    row![
        row![search_icon, search_input]
            .spacing(5)
            .align_y(Alignment::Center),
        row![text("Category:").size(13), category_picker]
            .spacing(5)
            .align_y(Alignment::Center),
        show_selected_checkbox,
    ]
    .align_y(Alignment::Center)
    .padding([10, 15])
    .spacing(20)
    .into()
}

/// Displaying selected resources
fn view_selected_resources(app: &CollectorApp) -> Element<'_, Message> {
    let check_icon = icon_small(icons::CHECK);

    let selected_text = if app.checked_resources.is_empty() {
        text("No resources selected").size(12)
    } else {
        text(format!(
            "{} selected: {}",
            app.checked_resources.len(),
            app.checked_resources.join(", ")
        ))
        .size(12)
    };

    container(
        row![check_icon, selected_text]
            .spacing(8)
            .align_y(Alignment::Center),
    )
    .padding([8, 15])
    .into()
}
