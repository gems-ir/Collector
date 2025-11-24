use std::path::PathBuf;
use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons;
use dioxus_free_icons::Icon;

#[component]
pub fn Information() -> Element {
    rsx! {
        div {
            class:" flex flex-col gap-2 items-center",
            OpenFolder {
                string_input: "Select source folder: ".to_string(),
                name_input: "in".to_string()
            }
            OpenFolder {
                string_input: "Select destination folder: ".to_string(),
                name_input: "out".to_string()
            }
            if cfg!(target_os = "windows") {
                div {
                    class: "flex gap-1",
                    input {
                        type: "checkbox"
                    }
                    p{"Allow VSS extracting"}
                }
            }
        }
    }
}

#[component]
pub fn OpenFolder(string_input: String, name_input: String) -> Element {
    let mut selected_folder = use_signal(|| None::<PathBuf>);

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
                class: "border rounded-xl pl-2 focus:ring-1 focus:outline-none focus:ring-green-800 focus:border-green-800 focus:ring-green-500 focus:border-green-500",
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