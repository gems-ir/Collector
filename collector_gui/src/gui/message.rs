use crate::com::Resource;
use std::path::PathBuf;

/// All messages/events in the application
#[derive(Debug, Clone)]
pub enum Message {
    // Theme
    ToggleTheme,

    // Input paths
    SelectSourceFolder,
    SelectDestinationFolder,
    SourceFolderSelected(Option<PathBuf>),
    DestinationFolderSelected(Option<PathBuf>),

    // VSS
    ToggleVss(bool),

    // Output options
    ToggleZip(bool),
    ToggleZipPassword(bool),
    ZipPasswordChanged(String),

    // Resources
    ResourcesLoaded(Vec<Resource>),
    SearchQueryChanged(String),
    CategorySelected(String),
    ToggleResource(String),
    ViewResource(Resource),
    CloseModal,
    ToggleShowSelectedOnly(bool),

    // Collection
    StartCollection,
    CollectionCompleted,
}
