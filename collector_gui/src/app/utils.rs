use iced::font::Weight;
use iced::widget::{container, text, Text};
use iced::{Border, Center, Element, Fill, Font, Renderer, Theme};

pub(crate) fn containerized<'a, Message: 'a>(value: impl Into<Element<'a, Message, Theme, Renderer>>) -> Element<'a, Message> {
    container(value)
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

pub(crate) fn titled<'a>(value: &str) -> Text<'a, Theme, Renderer> {
    text(value.to_string()).size(18).font(Font {
        weight: Weight::Bold,
        ..Font::default()
    })
}