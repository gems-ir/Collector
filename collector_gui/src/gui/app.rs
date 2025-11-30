use crate::com::config::AppData;
use std::path::PathBuf;
use crate::com::Resource;
use dark_light::Mode;
use iced::widget::{column, container, row, vertical_space};
use iced::{Element, Length, Task, Theme};

use crate::gui::message::Message;
use crate::style::theme::app_background_style;
use crate::views::{view_footer, view_input_section, view_output_section, view_resource_modal, view_resources_section};
use crate::com::{filter_resources, get_categories, load_resources, collection::run_collection, config::Config};

#[derive(Debug, Clone, PartialEq)]
pub enum CollectionState {
    Ready,
    Collecting,
}

pub struct CollectorApp {
    // Config
    pub config: Config,
    pub app_data: AppData,

    // Theme
    pub is_dark: bool,

    // Paths
    pub source_path: Option<PathBuf>,
    pub destination_path: Option<PathBuf>,

    // VSS
    pub vss_enabled: bool,

    // Zip
    pub zip_enabled: bool,
    pub zip_password_enabled: bool,
    pub zip_password: String,

    // Resources
    pub resources: Vec<Resource>,
    pub filtered_resources: Vec<Resource>,
    pub categories: Vec<String>,
    pub search_query: String,
    pub selected_category: String,
    pub checked_resources: Vec<String>,
    pub show_selected_only: bool,

    // Modal
    pub viewing_resource: Option<Resource>,

    // Collection state
    pub collection_state: CollectionState,
    pub collection_message: String,
}

impl CollectorApp {
    pub fn new() -> (Self, Task<Message>) {
        let config = Config::parse_config_file();
        let is_dark = matches!(dark_light::detect(), Ok(Mode::Dark));

        let app = CollectorApp::create(config.clone(), is_dark);

        let load_resources_task = Task::perform(
            async move { load_resources(&config).await },
            Message::ResourcesLoaded,
        );

        (app, load_resources_task)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ToggleTheme => {
                self.is_dark = !self.is_dark;
                Task::none()
            }

            Message::SelectSourceFolder => Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .pick_folder()
                        .await
                        .map(|f| f.path().to_path_buf())
                },
                Message::SourceFolderSelected,
            ),

            Message::SelectDestinationFolder => Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .pick_folder()
                        .await
                        .map(|f| f.path().to_path_buf())
                },
                Message::DestinationFolderSelected,
            ),

            Message::SourceFolderSelected(path) => {
                if let Some(p) = path {
                    self.source_path = Some(p.clone());
                    self.app_data.source_path = p.to_string_lossy().to_string();
                }
                Task::none()
            }

            Message::DestinationFolderSelected(path) => {
                if let Some(p) = path {
                    self.destination_path = Some(p.clone());
                    self.app_data.destination_path = p.to_string_lossy().to_string();
                }
                Task::none()
            }

            Message::ToggleVss(enabled) => {
                self.vss_enabled = enabled;
                self.app_data.vss = enabled;
                Task::none()
            }

            Message::ToggleZip(enabled) => {
                self.zip_enabled = enabled;
                self.app_data.zip = enabled;
                if !enabled {
                    self.zip_password_enabled = false;
                }
                Task::none()
            }

            Message::ToggleZipPassword(enabled) => {
                self.zip_password_enabled = enabled;
                if !enabled {
                    self.zip_password.clear();
                    self.app_data.zip_pass = None;
                }
                Task::none()
            }

            Message::ZipPasswordChanged(password) => {
                self.zip_password = password.clone();
                self.app_data.zip_pass = Some(password);
                Task::none()
            }

            Message::ResourcesLoaded(resources) => {
                self.resources = resources.clone();

                let mut cats = get_categories(&resources);
                cats.insert(0, "All".to_string());
                self.categories = cats;

                let initial_checked: Vec<String> = resources
                    .iter()
                    .filter(|r| r.is_checked)
                    .map(|r| r.name.clone())
                    .collect();
                if !initial_checked.is_empty() {
                    self.checked_resources = initial_checked;
                }

                self.filtered_resources = filter_resources(
                    &self.resources,
                    &self.search_query,
                    &self.selected_category,
                    self.show_selected_only,
                    &self.checked_resources,
                );

                Task::none()
            }

            Message::SearchQueryChanged(query) => {
                self.search_query = query;
                self.filtered_resources = filter_resources(
                    &self.resources,
                    &self.search_query,
                    &self.selected_category,
                    self.show_selected_only,
                    &self.checked_resources,
                );
                Task::none()
            }

            Message::CategorySelected(category) => {
                self.selected_category = category;
                self.filtered_resources = filter_resources(
                    &self.resources,
                    &self.search_query,
                    &self.selected_category,
                    self.show_selected_only,
                    &self.checked_resources,
                );
                Task::none()
            }

            Message::ToggleShowSelectedOnly(enabled) => {
                self.show_selected_only = enabled;
                self.filtered_resources = filter_resources(
                    &self.resources,
                    &self.search_query,
                    &self.selected_category,
                    self.show_selected_only,
                    &self.checked_resources,
                );
                Task::none()
            }

            Message::ToggleResource(name) => {
                if self.checked_resources.contains(&name) {
                    self.checked_resources.retain(|n| n != &name);
                } else {
                    self.checked_resources.push(name);
                }
                self.app_data.resource_list = self.checked_resources.clone();

                if self.show_selected_only {
                    self.filtered_resources = filter_resources(
                        &self.resources,
                        &self.search_query,
                        &self.selected_category,
                        self.show_selected_only,
                        &self.checked_resources,
                    );
                }
                Task::none()
            }

            Message::ViewResource(resource) => {
                self.viewing_resource = Some(resource);
                Task::none()
            }

            Message::CloseModal => {
                self.viewing_resource = None;
                Task::none()
            }

            Message::StartCollection => {
                if self.collection_state != CollectionState::Collecting {
                    self.collection_state = CollectionState::Collecting;
                    self.collection_message = "Collection in progress, please wait...".to_string();

                    let source = self
                        .source_path
                        .clone()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_default();
                    let destination = self
                        .destination_path
                        .clone()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_default();
                    let resources = self.checked_resources.clone();
                    let resource_path = self.config.resource_path.clone().unwrap_or_default();
                    let vss_enabled = self.vss_enabled;
                    let zip_enabled = self.zip_enabled;
                    let zip_pass = if self.zip_password_enabled {
                        Some(self.zip_password.clone())
                    } else {
                        None
                    };

                    return Task::perform(
                        async move {
                            run_collection(
                                source,
                                destination,
                                resources,
                                resource_path,
                                vss_enabled,
                                zip_enabled,
                                zip_pass,
                            )
                            .await
                        },
                        |_| Message::CollectionCompleted,
                    );
                }
                Task::none()
            }

            Message::CollectionCompleted => {
                self.collection_state = CollectionState::Ready;
                self.collection_message = "Collection completed successfully!".to_string();
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        // If a modal is open, display only the modal
        if let Some(ref resource) = self.viewing_resource {
            return view_resource_modal(resource, self.is_dark);
        }

        let top_section = row![view_input_section(self), view_output_section(self)]
            .spacing(15)
            .width(Length::Fill);

        let content = column![
            top_section,
            vertical_space().height(10),
            view_resources_section(self),
            vertical_space().height(10),
            view_footer(self),
        ]
        .spacing(0)
        .padding(20);

        let is_dark = self.is_dark;
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| app_background_style(is_dark))
            .into()
    }

    pub fn theme(&self) -> Theme {
        if self.is_dark {
            Theme::Dark
        } else {
            Theme::Light
        }
    }

    pub fn title(&self) -> String {
        String::from("Collector GUI - Artifact Collection Tool")
    }

    pub fn create(config: Config, is_dark: bool) -> Self {
        let source_path = config.source_path.clone().map(PathBuf::from);
        let destination_path = config.destination_path.clone().map(PathBuf::from);
        let checked_resources = config.resource_list.clone().unwrap_or_default();
        let zip_enabled = config.zip.unwrap_or(false);
        let zip_password = config.zip_pass.clone().unwrap_or_default();
        let zip_password_enabled = !zip_password.is_empty();
        let vss_enabled = config.vss.unwrap_or(false);

        Self {
            config,
            app_data: AppData::default(),
            is_dark,
            source_path,
            destination_path,
            vss_enabled,
            zip_enabled,
            zip_password_enabled,
            zip_password,
            resources: Vec::new(),
            filtered_resources: Vec::new(),
            categories: vec!["All".to_string()],
            search_query: String::new(),
            selected_category: "All".to_string(),
            checked_resources,
            show_selected_only: false,
            viewing_resource: None,
            collection_state: CollectionState::Ready,
            collection_message: "Ready to collect".to_string(),
        }
    }
}
