mod com;
mod gui;
mod views;
mod utils;
mod style;

use iced::window::{Settings,icon};
use iced::Size;

use crate::gui::CollectorApp;

const ICO_COLLETOR: &str = "assets/logo.ico";

fn main() -> iced::Result {
    iced::application(CollectorApp::title, CollectorApp::update, CollectorApp::view)
        .theme(CollectorApp::theme)
        .window(
            Settings{
                size: Size::new(1050.0, 700.0),
                min_size: Some(Size::new(900.0,600.0)),
                icon: Some(icon::from_file(ICO_COLLETOR).unwrap()),
                ..Default::default()
            }
        )
        .run_with(CollectorApp::new)
}
