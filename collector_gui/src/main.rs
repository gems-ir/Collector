mod information;
mod start;
mod list;

use iced::widget::{column, row, Column};
use iced::{Center, Task};
// use iced::Theme;

use information::{Infos, InfosMsg};
use list::{ListResources, ListResourcesMsg};
use start::{Start, StartMsg};

pub fn main() -> iced::Result {
    iced::application("Collector GUI", CollectorGui::update, CollectorGui::view)
        .window(iced::window::Settings {
            icon: Some(iced::window::icon::from_file("assets/logo.png").unwrap()),
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
    is_loading: bool,
}

#[derive(Debug, Clone)]
enum CollectorGuiMsg {
    StartBox(StartMsg),
    InfoBox(InfosMsg),
    ListBox(ListResourcesMsg),
}

impl CollectorGui {
    fn update(&mut self, action: CollectorGuiMsg) -> Task<CollectorGuiMsg> {
        println!("receiving action: {:?} ", action);
        let task = match action {
            CollectorGuiMsg::StartBox(_start_msg) => {
                Task::none()
            }
            CollectorGuiMsg::InfoBox(infos_msg) => {
                match infos_msg {
                    InfosMsg::OpenFolder(_) => self.is_loading = true,
                    InfosMsg::FolderOpened(_, _) => self.is_loading = false,
                }
                self.information.update(infos_msg).map(CollectorGuiMsg::InfoBox)
            }
            CollectorGuiMsg::ListBox(_list_resources_msg) => {
                Task::none()
            }
        };
        println!("{:?}", self.information);
        task
    }
    fn view(&self) -> Column<'_, CollectorGuiMsg> {
        column![
            row![
                self.information.view().map(CollectorGuiMsg::InfoBox),
                self.resources.view().map(CollectorGuiMsg::ListBox),
            ]
            .spacing(10),
            self.start.view().map(CollectorGuiMsg::StartBox),
        ]
            .padding(20)
            .spacing(10)
            .align_x(Center)
    }

    // fn theme(&self) -> Theme {
    //     Theme::SolarizedDark
    // }
}