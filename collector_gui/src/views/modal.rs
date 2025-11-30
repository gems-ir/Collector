use iced::widget::{button, column, container, horizontal_rule, horizontal_space, row, scrollable, text};
use iced::{Alignment, Element, Length};

use crate::com::Resource;
use crate::style::theme::{badge_style, code_block_style, icon_button_style, modal_content_style, modal_overlay_style};
use crate::style::icons::{self, icon_button};
use crate::gui::message::Message;


/// View of the resource detail modal (overlay)
pub fn view_resource_modal(resource: &Resource, is_dark: bool) -> Element<'_, Message> {
    // Header with title and close button
    let header = container(
        row![
            text(&resource.name).size(18),
            horizontal_space(),
            button(icon_button(icons::X))
                .on_press(Message::CloseModal)
                .padding(8)
                .style(icon_button_style(is_dark)),
        ]
        .align_y(Alignment::Center),
    )
    .padding([15, 20])
    .width(Length::Fill);

    // category badge
    let category_badge = container(text(&resource.category).size(11))
        .padding([4, 10])
        .style(|_| badge_style());

    // Metadata
    let metadata = container(
        row![
            text("Category:").size(12),
            category_badge,
            horizontal_space().width(20),
            text("Description:").size(12),
            text(&resource.description).size(12),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .padding([10, 20]);

    // Scrollable content
    let content_scroll = scrollable(
        container(
            text(&resource.content)
                .size(11)
                .font(iced::Font::MONOSPACE),
        )
        .padding(15)
        .width(Length::Fill)
        .style(move |_| code_block_style(is_dark)),
    )
    .height(Length::Fixed(350.0));

    let content_container = container(content_scroll).padding(20);

    // Modal assembly
    let modal_content = column![
        header,
        horizontal_rule(1),
        metadata,
        horizontal_rule(1),
        content_container,
    ]
    .spacing(0)
    .width(Length::Fixed(750.0));

    let modal_box = container(modal_content).style(move |_| modal_content_style(is_dark));

    // Dark overlay + centered modal
    container(modal_box)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .style(|_| modal_overlay_style())
        .into()
}
