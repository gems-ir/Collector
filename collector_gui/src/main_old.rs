mod source;
mod start;
mod list;

use iced::widget::{button, column, row, text, Column};
use iced::Center;

pub fn main() -> iced::Result {
    iced::application("Collector GUI", CollectorGui::update, CollectorGui::view)
        .run()
}

#[derive(Default)]
struct CollectorGui {
    value: u64,
}

#[derive(Debug, Clone)]
enum CollectorGuiMsg {
    ClickOn,
}

impl CollectorGui {
    fn update(&mut self, action: CollectorGuiMsg) {
        match action {
            CollectorGuiMsg::ClickOn => {
                self.value += 1;
            }
        }
    }
    fn view(&self) -> Column<'_, CollectorGuiMsg> {
        let events = row![
            button("Settings"),
            button("Start")
        ];
        column![
            text(self.value.to_string()),
            button("Click me!").on_press(CollectorGuiMsg::ClickOn),
            events
        ]
            .align_x(Center)
    }
}