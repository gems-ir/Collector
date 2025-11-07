use iced::widget::{button, column, container, row, rule, text};
use iced::{Border, Center, Element, Fill, Padding, Task};
use std::path::PathBuf;

#[derive(Default, Debug)]
pub(crate) struct Infos {
    source_file: Option<PathBuf>,
    destination_file: Option<PathBuf>,
    is_loading: bool,
}

#[derive(Debug, Clone)]
pub(crate) enum InfosMsg {
    OpenFolder(FolderType),
    FolderOpened(FolderType, Result<PathBuf, Error>),
}

#[derive(Debug, Clone, Copy)]
pub enum FolderType {
    Source,
    Destination,
}

impl Infos {
    pub(crate) fn update(&mut self, action: InfosMsg) -> Task<InfosMsg> {
        match action {
            InfosMsg::OpenFolder(folder_type) => {
                if self.is_loading {
                    Task::none()
                } else {
                    self.is_loading = true;
                    Task::perform(
                        open_folder(),
                        move |result| InfosMsg::FolderOpened(folder_type, result),
                    )
                }
            }
            InfosMsg::FolderOpened(folder_type, result) => {
                self.is_loading = false;

                if let Ok(path) = result {
                    match folder_type {
                        FolderType::Source => {
                            println!("Source folder \"{}\" opened successfully!", path.display());
                            self.source_file = Some(path);
                        }
                        FolderType::Destination => {
                            println!("Destination folder \"{}\" opened successfully!", path.display());
                            self.destination_file = Some(path);
                        }
                    }
                }

                Task::none()
            }
        }
    }

    pub(crate) fn view(&self) -> Element<'_, InfosMsg> {
        let lines = column![
            container(
                column![
                    text("Select your information:"),
                    rule::Rule::horizontal(1)
                        .style(|_theme| rule::Style {
                            color: iced::Color::WHITE,
                            width: 2,
                            radius: 0.0.into(),
                            fill_mode: rule::FillMode::Percent(30.0),
                        })
                ]
                    .align_x(Center)
                    .spacing(2)
            )
            .padding(Padding::new(10.0).bottom(20)),
            self.folder_row("Select your target source folder:".to_string(), &self.source_file, FolderType::Source),
            self.folder_row("Select your destination folder:".to_string(), &self.destination_file, FolderType::Destination),
        ]
            .width(Fill)
            .align_x(Center)
            .spacing(10);

        container(lines)
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

    fn folder_row(&self, label: String, path: &Option<PathBuf>, folder_type: FolderType) -> Element<'_, InfosMsg> {
        let path_text = if let Some(path) = path {
            format!("{}", path.display())
        } else {
            "No folder selected".to_string()
        };

        let row = row![
            text(label),
            button("...")
                .padding(3)
                .height(25)
                .width(20)
                .on_press_maybe(
                    if self.is_loading { None } else { Some(InfosMsg::OpenFolder(folder_type)) }
                ),
            text(path_text)
        ]
            .spacing(10)
            .align_y(Center);
        row.into()
        // .into()
    }
}

#[derive(Debug, Clone)]
pub enum Error {
    DialogClosed,
}

async fn open_folder() -> Result<PathBuf, Error> {
    let picked_folder = rfd::AsyncFileDialog::new()
        .set_title("Select a folder...")
        .pick_folder()
        .await
        .ok_or(Error::DialogClosed)?;

    Ok(picked_folder.path().to_path_buf())
}