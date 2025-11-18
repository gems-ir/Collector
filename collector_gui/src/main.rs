#[cfg(target_os = "linux")]
mod values_linux;
#[cfg(target_os = "windows")]
mod values_windows;
mod config;
mod app;

use iced::widget::{column, container, row, text, Column, Container};
use iced::{color, Background, Border, Center, Fill, Renderer, Task, Theme};
// use iced::Theme;

use crate::app::information::{Infos, InfosMsg};
use crate::app::list::{ListResources, ListResourcesMsg};
use crate::app::output::{Output, OutputMsg};
use crate::app::start::{Start, StartMsg};
use crate::config::Config;

pub fn main() -> iced::Result {
    iced::application("Collector GUI", CollectorGui::update, CollectorGui::view)
        .window(iced::window::Settings {
            icon: Some(iced::window::icon::from_file("assets/logo.ico").unwrap()),
            size: iced::Size::new(1300.0, 600.0),
            min_size: Some(iced::Size::new(1000.0, 450.0)),
            position: iced::window::Position::Specific(iced::Point::new(50.0, 50.0)),
            ..Default::default()
        })
        // .theme(CollectorGui::theme)
        .run()
}

#[derive(Default)]
struct CollectorGui {
    start: Start,
    information: Infos,
    resources: ListResources,
    output: Output,
    is_loading: bool,
}

#[derive(Debug, Clone)]
enum CollectorGuiMsg {
    StartBox(StartMsg),
    OutputBox(OutputMsg),
    ListBox(ListResourcesMsg),
    InfoBox(InfosMsg),
}


impl CollectorGui {
    fn update(&mut self, action: CollectorGuiMsg) -> Task<CollectorGuiMsg> {
        // println!("receiving action: {:?} ", action);
        match action {
            CollectorGuiMsg::StartBox(_start_msg) => {
                Task::none()
            }
            CollectorGuiMsg::OutputBox(output_msg) => {
                self.output.update(output_msg).map(CollectorGuiMsg::OutputBox)
            }
            CollectorGuiMsg::InfoBox(infos_msg) => {
                match infos_msg {
                    InfosMsg::OpenFolder(_) => self.is_loading = true,
                    InfosMsg::FolderOpened(_, _) => self.is_loading = false,
                    _ => {}
                }
                self.information.update(infos_msg).map(CollectorGuiMsg::InfoBox)
            }
            CollectorGuiMsg::ListBox(_list_resources_msg) => {
                Task::none()
            }
        }
    }
    fn view(&self) -> Column<'_, CollectorGuiMsg> {
        let config = if cfg!(target_os = "windows") {
            Config::parse_config_file("collector_config_windows.toml").unwrap()
        } else {
            Config::parse_config_file("collector_config_linux.toml").unwrap()
        };

        column![
            // text("Collector GUI ").size(32),
            self.title_main(),
            row![
                column![
                    self.information.view(&config).map(CollectorGuiMsg::InfoBox),
                    self.output.view(&config).map(CollectorGuiMsg::OutputBox),
                ]
                .spacing(10),
                self.resources.view(&config).map(CollectorGuiMsg::ListBox),
            ]
            .spacing(10),
            self.start.view().map(CollectorGuiMsg::StartBox),
        ]
            .padding(20)
            .spacing(10)
            .align_x(Center)
    }

    fn title_main<'a, Message: 'a>(&self) -> Container<'a, Message, Theme, Renderer> {
        container(text("Collector GUI").size(32))
            .style(|_theme| {
                container::Style {
                    border: Border {
                        // color: Color::from_rgb(0.5, 0.5, 0.5),
                        // width: 2.0,
                        radius: 5.0.into(),
                        ..Default::default()
                    },
                    background: Some(
                        Background::Color(
                            // Color::from_rgba(0.2, 0.3, 0.7, 0.5)
                            color!(0x067c8b)
                        )
                    ),
                    ..Default::default()
                }
            })
            .width(Fill)
            .align_x(Center)
    }

    // fn theme(&self) -> Theme {
    //     Theme::SolarizedDark
    // }
}