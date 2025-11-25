use std::path::PathBuf;
use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons;
use dioxus_free_icons::Icon;
use crate::config::{Config, AppData};

#[component]
pub fn Information(config: Config) -> Element {

    let mut source_folder = use_signal(|| None::<PathBuf>);
    let mut destination_folder = use_signal(|| None::<PathBuf>);
    let mut vss_toggle = use_signal(|| false);
    let mut app_data = use_context::<Signal<AppData>>();
    let mut is_initialized = use_signal(|| false);

    use_effect(move || {
            if !is_initialized() {
                if let Some(src_path_config) = &config.source_path {
                    source_folder.set(Some(PathBuf::from(src_path_config)));
                }
                if let Some(dst_path_config) = &config.destination_path {
                    destination_folder.set(Some(PathBuf::from(dst_path_config)));
                }
                if let Some(vss_toggle_config) = config.vss {
                    vss_toggle.set(vss_toggle_config);
                }
                is_initialized.set(true);
           }

            if let Some(entry_path) = source_folder(){
                app_data.write().source_path = entry_path.into_os_string().into_string().unwrap();
            }else{
                app_data.write().source_path = String::new();
            }
            if let Some(dest_path) = destination_folder(){
                app_data.write().destination_path = dest_path.into_os_string().into_string().unwrap();
            }else{
                app_data.write().destination_path = String::new();
            }
            app_data.write().vss = vss_toggle();
        }
    );


    rsx! {
        div {
            class:" flex flex-col gap-2 items-center",
            OpenFolder {
                string_input: "Select source folder: ".to_string(),
                name_input: "in".to_string(),
                selected_folder: source_folder
            }
            OpenFolder {
                string_input: "Select destination folder: ".to_string(),
                name_input: "out".to_string(),
                selected_folder: destination_folder
            }
            if cfg!(target_os = "windows") {
                div {
                    class: "flex gap-1",
                    input {
                        checked: vss_toggle,
                        type: "checkbox"
                    }
                    p{"Allow VSS extracting"}
                }
            }
        }
    }
}

#[component]
pub fn OpenFolder(string_input: String, name_input: String, selected_folder: Signal<Option<PathBuf>>) -> Element {
    // let mut selected_folder = use_signal(|| None::<PathBuf>);

    let select_folder = move |_: MouseEvent| {
        spawn(async move {
            if let Some(folder) = rfd::AsyncFileDialog::new().pick_folder().await {
                selected_folder.set(Some(folder.path().to_path_buf()));
            }
        });
    };
    
    rsx! {
        div{
            class: "flex items-center gap-2",
            p{
                "{string_input}"
            }
            input{
                placeholder:"Choose your file...",
                readonly: true,
                class: "border rounded-xl pl-2 focus:ring-1 focus:outline-none dark:focus:ring-green-800 dark:focus:border-green-800 focus:ring-green-500 focus:border-green-500",
                name:"{name_input}",
                value: if let Some(path) = selected_folder() {
                    path.display().to_string()
                } else {
                    ""
                }
            }
            button{
                class:"hover:text-green-500 dark:hover:text-green-700 hover:cursor-pointer",
                onclick: select_folder,
                Icon{
                    width:20,
                    height:20,
                    icon: ld_icons::LdFolderOpen
                }
            }
        }
    }
}