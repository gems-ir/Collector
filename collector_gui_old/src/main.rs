#[cfg(target_os = "linux")]
mod values_linux;
#[cfg(target_os = "windows")]
mod values_windows;
mod config;
mod app;

use iced::widget::{column, container, row, text, Column, Container};
use iced::window::Position;
use iced::{color, Background, Border, Center, Fill, Renderer, Size, Subscription, Task, Theme};
// use iced::Theme;

use crate::app::information::{Infos, InfosMsg};
use crate::app::list::{ListResources, ListResourcesMsg};
use crate::app::output::{Output, OutputMsg};
use crate::app::start::{Start, StartMsg};
use crate::config::Config;

pub fn main() -> iced::Result {
    iced::application(CollectorGui::title, CollectorGui::update, CollectorGui::view)
        .window(iced::window::Settings {
            icon: Some(iced::window::icon::from_file("assets/logo.ico").unwrap()),
            min_size: Some(Size::new(1000.0, 450.0)),
            ..Default::default()
        })
        .position(Position::Specific(iced::Point::new(50.0, 50.0)))
        .window_size(Size::new(1300.0, 600.0))
        // .theme(CollectorGui::theme)
        .run()
}

#[derive(Debug)]
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
    fn new() -> (Self, Task<CollectorGuiMsg>) {
        (Self {
            ..Default::default()
        },
         Task::none()
        )
    }
    fn update(&mut self, action: CollectorGuiMsg) -> Task<CollectorGuiMsg> {
        println!("{:?}", self);
        // self.resources = ListResources::new();
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
    pub fn title(&self) -> String {
        "Collector GUI".to_string()
    }
    fn subscription(&self) -> Subscription<CollectorGuiMsg> {
        println!("hello subscription");
        Subscription::none()
    }
}

impl Default for CollectorGui {
    fn default() -> Self {
        Self {
            start: Start::default(),
            resources: ListResources::new(),
            information: Infos::default(),
            output: Output::default(),
            is_loading: false,
        }
    }
}