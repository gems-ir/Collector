use crate::app::utils::{containerized, titled};
use crate::config::Config;
use iced::widget::{button, checkbox, column, container, row, text};
use iced::{Center, Element, Fill, Length, Task};
use std::path::PathBuf;

#[derive(Default, Debug)]
pub(crate) struct Infos {
    source_file: Option<PathBuf>,
    destination_file: Option<PathBuf>,
    is_loading: bool,
    vss_checked: bool,
}

#[derive(Debug, Clone)]
pub(crate) enum InfosMsg {
    OpenFolder(FolderType),
    FolderOpened(FolderType, Result<PathBuf, Error>),
    VssToggle(bool),
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
            InfosMsg::VssToggle(is_checked) => {
                self.vss_checked = is_checked;
                Task::none()
            }
        }
    }

    pub(crate) fn view(&self, config: &Config) -> Element<'_, InfosMsg> {
        
        
        let lines = column![
            titled("Information"),
            self.folder_row("Select your target source folder:".to_string(), &self.source_file, FolderType::Source, config),
            // Show vss checkbox only if for Windows OS execution.
            container(
                checkbox("Allow VSS extracting", self.vss_checked).on_toggle(InfosMsg::VssToggle)
            )
            .height(if cfg!(target_os = "windows") { Length::Shrink } else { Length::Fixed(0.0) }),
            self.folder_row("Select your destination folder:".to_string(), &self.destination_file, FolderType::Destination, config),
        ]
            .width(Fill)
            .align_x(Center)
            .spacing(10);

        containerized(lines)
    }

    fn folder_row(&self, label: String, path: &Option<PathBuf>, folder_type: FolderType, config: &Config) -> Element<'_, InfosMsg> {
        let path_text = if let Some(path) = path {
            format!("{}", path.display())
        } else {
            match folder_type {
                FolderType::Source => config.clone().source_path.unwrap_or("No folder selected".to_string()),
                FolderType::Destination => config.clone().destination_path.unwrap_or("No folder selected".to_string())
            }
        };

        row![
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
            .align_y(Center)
            .into()
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