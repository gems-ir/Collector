use iced::widget::{button, checkbox, column, container, row, text, text_input, vertical_space};
use iced::{Alignment, Element, Length};

use crate::style::icons::{self, icon_button};
use crate::style::theme::{icon_button_style, card_style, section_header_style};
use crate::gui::message::Message;
use crate::gui::CollectorApp;

/// View of the entry section (source, destination, VSS)
pub fn view_input_section(app: &CollectorApp) -> Element<'_, Message> {
    let header = container(text("Input Configuration").size(14))
        .width(Length::Fill)
        .padding([8, 15])
        .style(|_| section_header_style());

    let source_label = text("Source folder:").size(13);
    let source_value = app
        .source_path
        .as_ref()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let source_input = text_input("Select source folder...", &source_value)
        .width(Length::Fill)
        .padding([6,8]);

    let source_btn = button(icon_button(icons::FOLDER_OPEN))
        .on_press(Message::SelectSourceFolder)
        .padding([6,8])
        .style(icon_button_style(app.is_dark));

    let source_row = row![source_label, source_input, source_btn]
        .spacing(10)
        .align_y(Alignment::Center);

    let dest_label = text("Destination folder:").size(13);
    let dest_value = app
        .destination_path
        .as_ref()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let dest_input = text_input("Select destination folder...", &dest_value)
        .width(Length::Fill)
        .padding([6,8]);

    let dest_btn = button(icon_button(icons::FOLDER_OPEN))
        .on_press(Message::SelectDestinationFolder)
        .padding([6,8])
        .style(icon_button_style(app.is_dark));

    let dest_row = row![dest_label, dest_input, dest_btn]
        .spacing(10)
        .align_y(Alignment::Center);

    let mut content = column![source_row, dest_row].spacing(15);

    #[cfg(target_os = "windows")]
    {
        let vss_row = row![
            icon_button(icons::HARD_DRIVE),
            checkbox(" Enable VSS extraction", app.vss_enabled)
                .on_toggle(Message::ToggleVss)
                .text_size(13),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        content = content.push(vertical_space().height(5));
        content = content.push(vss_row);
    }

    let card_content = column![header, container(content).padding(15)].spacing(0);

    container(card_content)
        .width(Length::FillPortion(1))
        .style(move |_| card_style(app.is_dark))
        .into()
}
