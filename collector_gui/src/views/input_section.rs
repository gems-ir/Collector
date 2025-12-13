use iced::widget::{button, checkbox, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Element, Length};

use crate::gui::CollectorApp;
use crate::gui::message::Message;
use crate::style::icons::{self, icon_small};
use crate::style::theme::{card_style, icon_button_style, section_header_style};

/// View of the entry section (source, destination, VSS)
pub fn view_input_section(app: &CollectorApp) -> Element<'_, Message> {
    // Header
    let header = container(text("Input Configuration").size(14))
        .width(Length::Fill)
        .padding([6, 15])
        .style(|_| section_header_style());

    // Source label
    let source_label = text("Source folder:").size(13);
    let source_value = app
        .source_path
        .as_ref()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let source_input = text_input("Select source folder...", &source_value)
        .width(Length::Fill)
        .padding([2, 8]);

    let source_btn = button(icon_small(icons::FOLDER_OPEN))
        .on_press(Message::SelectSourceFolder)
        .padding([6, 10])
        .style(icon_button_style(app.is_dark));

    let source_row = row![source_label, source_input, source_btn]
        .spacing(10)
        // .height(Length::Fixed(25.0))
        .align_y(Alignment::Center);

    // Destination label
    let dest_label = text("Destination folder:").size(13);
    let dest_value = app
        .destination_path
        .as_ref()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let dest_input = text_input("Select destination folder...", &dest_value)
        // .size(14)
        .width(Length::FillPortion(5))
        .padding([2, 8]);

    let dest_btn = button(icon_small(icons::FOLDER_OPEN))
        .on_press(Message::SelectDestinationFolder)
        .padding([6, 10])
        .width(Length::Shrink)
        .style(icon_button_style(app.is_dark));

    let dest_row = row![dest_label, dest_input, dest_btn]
        .spacing(10)
        // .height(Length::Fixed(25.0))
        .align_y(Alignment::Center);

    let mut content = column![source_row, dest_row].spacing(15);

    // VSS label
    #[cfg(target_os = "windows")]
    {
        let vss_row = row![
            icon_small(icons::HARD_DRIVE),
            checkbox(app.vss_enabled)
                .label(" Enable VSS extraction")
                .on_toggle(Message::ToggleVss)
                .text_size(13),
        ]
        .spacing(8)
        .height(Length::Shrink)
        .align_y(Alignment::Center);

        // content = content.push(Space::new().height(Length::Fixed(5.0)));
        content = content.push(vss_row);
    }

    let card_content = scrollable(column![header, container(content).padding(15)].spacing(0));

    container(card_content)
        // .height(Length::Fill)
        .width(Length::FillPortion(1))
        .style(move |_| card_style(app.is_dark))
        .into()
}
