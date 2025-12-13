use crate::com::Resource;
use crate::com::collection::CollectionResult;
use std::path::PathBuf;

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
    // TO REMOVE
    // #[allow(dead_code)]
    // CollectionProgress { current: u64, total: u64, file: String },
    CollectionCompleted(CollectionResult),
    TickProgress,
}
