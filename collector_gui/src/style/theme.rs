use iced::widget::{button, container};
use iced::{Border, Color};

/// Colors of Collector theme 
pub mod colors {
    use iced::Color;

    // Mode clair
    pub const LIGHT_BACKGROUND: Color = Color::from_rgb(0.96, 0.96, 0.96);
    pub const LIGHT_SURFACE: Color = Color::from_rgb(1.0, 1.0, 1.0);
    pub const LIGHT_CARD: Color = Color::from_rgb(1.0, 1.0, 1.0);
    pub const LIGHT_BORDER: Color = Color::from_rgb(0.88, 0.88, 0.88);
    pub const LIGHT_TEXT: Color = Color::from_rgb(0.13, 0.13, 0.13);

    // Mode sombre
    pub const DARK_BACKGROUND: Color = Color::from_rgb(0.07, 0.07, 0.07);
    pub const DARK_SURFACE: Color = Color::from_rgb(0.12, 0.12, 0.12);
    pub const DARK_CARD: Color = Color::from_rgb(0.16, 0.16, 0.16);
    pub const DARK_BORDER: Color = Color::from_rgb(0.25, 0.25, 0.25);
    pub const DARK_TEXT: Color = Color::from_rgb(0.88, 0.88, 0.88);

    // Couleurs d'accent
    pub const PRIMARY: Color = Color::from_rgb(0.18, 0.49, 0.20);

    // Table
    pub const TABLE_HEADER: Color = Color::from_rgb(0.24, 0.32, 0.36);
    pub const TABLE_ROW_ALT: Color = Color::from_rgb(0.97, 0.97, 0.97);
    pub const TABLE_ROW_ALT_DARK: Color = Color::from_rgb(0.14, 0.14, 0.14);
}

/// Style for cards/containers with border
pub fn card_style(is_dark: bool) -> container::Style {
    container::Style {
        background: Some(if is_dark { colors::DARK_CARD } else { colors::LIGHT_CARD }.into()),
        border: Border {
            color: if is_dark { colors::DARK_BORDER } else { colors::LIGHT_BORDER },
            width: 1.0,
            radius: 8.0.into(),
        },
        text_color: Some(if is_dark { colors::DARK_TEXT } else { colors::LIGHT_TEXT }),
        ..Default::default()
    }
}

/// Style for section header
pub fn section_header_style() -> container::Style {
    container::Style {
        background: Some(colors::PRIMARY.into()),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 8.0.into(),
        },
        text_color: Some(Color::WHITE),
        ..Default::default()
    }
}

/// Style for table header 
pub fn table_header_style() -> container::Style {
    container::Style {
        background: Some(colors::TABLE_HEADER.into()),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 6.0.into(),
        },
        text_color: Some(Color::WHITE),
        ..Default::default()
    }
}

/// Style for alternating table rows
pub fn table_row_style(is_dark: bool, is_alternate: bool) -> container::Style {
    let background = if is_alternate {
        if is_dark { colors::TABLE_ROW_ALT_DARK } else { colors::TABLE_ROW_ALT }
    } else if is_dark { colors::DARK_SURFACE } else { colors::LIGHT_SURFACE };

    container::Style {
        background: Some(background.into()),
        ..Default::default()
    }
}

/// Style for the primary button (Start)
pub fn primary_button_style(_theme: &iced::Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(colors::PRIMARY.into()),
        text_color: Color::WHITE,
        border: Border {
            color: colors::PRIMARY,
            width: 0.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

/// Style for secondary buttons (icons)
pub fn icon_button_style(is_dark: bool) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |_theme: &iced::Theme, _status: button::Status| button::Style {
        background: Some(if is_dark { colors::DARK_CARD } else { colors::LIGHT_SURFACE }.into()),
        text_color: if is_dark { colors::DARK_TEXT } else { colors::LIGHT_TEXT },
        border: Border {
            color: if is_dark { colors::DARK_BORDER } else { colors::LIGHT_BORDER },
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

/// Style for the main background of the application
pub fn app_background_style(is_dark: bool) -> container::Style {
    container::Style {
        background: Some(if is_dark { colors::DARK_BACKGROUND } else { colors::LIGHT_BACKGROUND }.into()),
        text_color: Some(if is_dark { colors::DARK_TEXT } else { colors::LIGHT_TEXT }),
        ..Default::default()
    }
}

/// Style for the footer/status bar
pub fn status_bar_style(is_dark: bool) -> container::Style {
    container::Style {
        background: Some(if is_dark { colors::DARK_SURFACE } else { Color::from_rgb(0.93, 0.93, 0.93) }.into()),
        border: Border {
            color: if is_dark { colors::DARK_BORDER } else { colors::LIGHT_BORDER },
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

/// Style for the modal overlay
pub fn modal_overlay_style() -> container::Style {
    container::Style {
        background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.7).into()),
        ..Default::default()
    }
}

/// Style for modal content
pub fn modal_content_style(is_dark: bool) -> container::Style {
    container::Style {
        background: Some(if is_dark { colors::DARK_CARD } else { colors::LIGHT_CARD }.into()),
        border: Border {
            color: if is_dark { colors::DARK_BORDER } else { colors::LIGHT_BORDER },
            width: 1.0,
            radius: 12.0.into(),
        },
        text_color: Some(if is_dark { colors::DARK_TEXT } else { colors::LIGHT_TEXT }),
        ..Default::default()
    }
}

/// Style for badges
pub fn badge_style() -> container::Style {
    container::Style {
        background: Some(colors::PRIMARY.into()),
        text_color: Some(Color::WHITE),
        border: Border {
            radius: 4.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Style for codes block
pub fn code_block_style(is_dark: bool) -> container::Style {
    container::Style {
        background: Some(if is_dark { colors::DARK_BACKGROUND } else { Color::from_rgb(0.95, 0.95, 0.95) }.into()),
        border: Border {
            color: if is_dark { colors::DARK_BORDER } else { colors::LIGHT_BORDER },
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}
