use iced::widget::{button, checkbox, column, container, row, scrollable, text, Column};
use iced::{Alignment, Element, Length};

use crate::com::Resource;
use crate::style::icons::{self, icon_button};
use crate::style::theme::{icon_button_style, table_header_style, table_row_style};
use crate::gui::message::Message;
use crate::gui::CollectorApp;

// View of the resources table
pub fn view_resources_table(app: &CollectorApp) -> Element<'_, Message> {
    let header = row![
        text("").width(Length::Fixed(40.0)),
        text("Name")
            .width(Length::FillPortion(2))
            .align_x(Alignment::Center)
            .size(13),
        text("Category")
            .width(Length::FillPortion(2))
            .align_x(Alignment::Center)
            .size(13),
        text("Description")
            .width(Length::FillPortion(3))
            .align_x(Alignment::Center)
            .size(13),
        text("View")
            .width(Length::Fixed(60.0))
            .align_x(Alignment::Center)
            .size(13),
    ]
    .spacing(5)
    .padding([8, 10]);

    let header_container = container(header)
        .width(Length::Fill)
        .style(|_| table_header_style());

    let rows: Vec<Element<'_, Message>> = if app.filtered_resources.is_empty() {
        let empty_msg = if app.resources.is_empty() {
            "Loading resources..."
        } else {
            "No resources match your search"
        };
        vec![container(text(empty_msg).align_x(Alignment::Center).size(14))
            .width(Length::Fill)
            .padding(30)
            .into()]
    } else {
        app.filtered_resources
            .iter()
            .enumerate()
            .map(|(idx, resource)| create_resource_row(resource, &app.checked_resources, app.is_dark, idx))
            .collect()
    };

    let body = Column::with_children(rows).spacing(0);
    let scrollable_body = scrollable(body).height(Length::Fill);

    column![header_container, scrollable_body]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

// Create a line for a resource
fn create_resource_row(
    resource: &Resource,
    checked_resources: &[String],
    is_dark: bool,
    index: usize,
) -> Element<'static, Message> {
    let is_checked = checked_resources.contains(&resource.name);
    let name = resource.name.clone();
    let resource_clone = resource.clone();
    let is_alternate = index % 2 == 1;

    let row_content = row![
        checkbox("", is_checked)
            .on_toggle(move |_| Message::ToggleResource(name.clone()))
            .width(Length::Fixed(40.0)),
        text(resource.name.clone())
            .width(Length::FillPortion(2))
            .align_x(Alignment::Center)
            .size(12),
        container(text(resource.category.clone()).size(11))
            .width(Length::FillPortion(2))
            .align_x(Alignment::Center),
        text(truncate_text(&resource.description, 40))
            .width(Length::FillPortion(3))
            .align_x(Alignment::Center)
            .size(12),
        button(icon_button(icons::EYE))
            .on_press(Message::ViewResource(resource_clone))
            .width(Length::Fixed(50.0))
            .padding(6)
            .style(icon_button_style(is_dark)),
    ]
    .spacing(5)
    .padding([6, 10])
    .align_y(Alignment::Center);

    container(row_content)
        .width(Length::Fill)
        .style(move |_| table_row_style(is_dark, is_alternate))
        .into()
}

fn truncate_text(text: &str, max_len: usize) -> String {
    if text.len() > max_len {
        format!("{}...", &text[..max_len])
    } else {
        text.to_string()
    }
}
