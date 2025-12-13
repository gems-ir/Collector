mod com;
mod gui;
mod style;
mod utils;
mod views;

use iced::Size;
use iced::window::{Settings, icon};

use crate::gui::CollectorApp;

fn load_icon() -> Option<icon::Icon> {
    let ico_bytes = include_bytes!("../../assets/logo.ico");
    icon::from_file_data(ico_bytes, None).ok()
}

fn main() -> iced::Result {
    iced::application(CollectorApp::new, CollectorApp::update, CollectorApp::view)
        .title(CollectorApp::title)
        .theme(CollectorApp::theme)
        .subscription(CollectorApp::subscription)
        .window(Settings {
            size: Size::new(1050.0, 700.0),
            min_size: Some(Size::new(800.0, 500.0)),
            icon: load_icon(),
            ..Default::default()
        })
        .run()
}
