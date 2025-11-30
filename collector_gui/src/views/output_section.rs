use iced::widget::{checkbox, column, container, row, text, text_input, vertical_space};
use iced::{Alignment, Element, Length};

use crate::style::icons::{self, icon_button};
use crate::style::theme::{card_style, section_header_style};
use crate::gui::message::Message;
use crate::gui::CollectorApp;

/// View of the checkout section (zip, password)
pub fn view_output_section(app: &CollectorApp) -> Element<'_, Message> {
    let header = container(text("Output Configuration").size(14))
        .width(Length::Fill)
        .padding([8, 15])
        .style(|_| section_header_style());

    let zip_row = row![
        icon_button(icons::FILE_ARCHIVE),
        checkbox(" Compress output to ZIP", app.zip_enabled)
            .on_toggle(Message::ToggleZip)
            .text_size(13),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let mut content = column![zip_row].spacing(12);

    if app.zip_enabled {
        let mut pass_toggle_row = row![
            icon_button(icons::LOCK),
            checkbox(" Encrypt with password", app.zip_password_enabled)
                .on_toggle(Message::ToggleZipPassword)
                .text_size(13),
        ]
        .spacing(8)
        .align_y(Alignment::Center);


        if app.zip_password_enabled {
            let pass_input = text_input("Enter ZIP password...", &app.zip_password)
                .on_input(Message::ZipPasswordChanged)
                .width(Length::Fill)
                .padding([6,8]);

            let pass_row = row![text("").width(Length::Fixed(30.0)), pass_input,].spacing(8);

            pass_toggle_row = pass_toggle_row.push(pass_row);
            content = content.push(pass_toggle_row);
            
        }else{
            content = content.push(pass_toggle_row);

        }
    }

    content = content.push(vertical_space().height(10));

    let card_content = column![header, container(content).padding(15)].spacing(0);

    container(card_content)
        .width(Length::FillPortion(1))
        .style(move |_| card_style(app.is_dark))
        .into()
}
