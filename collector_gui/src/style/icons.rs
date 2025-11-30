use iced::widget::svg;
use iced::{ContentFit, Element, Length};

macro_rules! include_icon {
    ($name:expr) => {
        include_bytes!(concat!("../../assets/icons/", $name, ".svg"))
    };
}

pub const FOLDER_OPEN: &[u8] = include_icon!("folder-open");
pub const SUN: &[u8] = include_icon!("sun");
pub const MOON: &[u8] = include_icon!("moon");
pub const EYE: &[u8] = include_icon!("eye");
pub const X: &[u8] = include_icon!("x");
pub const PLAY: &[u8] = include_icon!("play");
pub const SEARCH: &[u8] = include_icon!("search");
pub const CHECK: &[u8] = include_icon!("check");
pub const LOADER: &[u8] = include_icon!("loader");
pub const FILE_ARCHIVE: &[u8] = include_icon!("file-archive");
pub const LOCK: &[u8] = include_icon!("lock");
pub const SHIELD_CHECK: &[u8] = include_icon!("shield-check");
pub const HARD_DRIVE: &[u8] = include_icon!("hard-drive");

/// SVG widget for icon
pub fn icon<'a, Message: 'a>(icon_bytes: &'static [u8], size: u16) -> Element<'a, Message> {
    svg(svg::Handle::from_memory(icon_bytes))
        .width(Length::Fixed(size as f32))
        .height(Length::Fixed(size as f32))
        .content_fit(ContentFit::Contain)
        .into()
}

pub fn icon_button<'a, Message: 'a>(icon_bytes: &'static [u8]) -> Element<'a, Message> {
    icon(icon_bytes, 16)
}

pub fn icon_small<'a, Message: 'a>(icon_bytes: &'static [u8]) -> Element<'a, Message> {
    icon(icon_bytes, 14)
}
