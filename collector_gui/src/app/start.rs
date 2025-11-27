use tokio::time::{sleep, Duration};
use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons;
use dioxus_free_icons::Icon;

use crate::config::{AppData, Config};
use collector_core::windows_vss::CollectVss;
use collector_core::{Collect,resource::YamlParser};

#[component]
pub fn Start(config: Config, is_dark: Signal<bool>) -> Element {
    let mut is_collecting = use_signal(|| false);  
    let mut collect_msg = use_signal(|| String::from("Ready to collect"));

    let class_start_button = if is_collecting() {
        "bg-red-300 dark:bg-red-600 border-2 border-slate-600 dark:border-slate-400 dark:text-slate-200 rounded-md px-8 py-2 cursor-not-allowed opacity-100 dark:opacity-60 transition"
    } else {
        "bg-slate-200 dark:bg-slate-500 dark:border-slate-400 dark:text-white hover:text-green-500 dark:hover:text-green-500 hover:cursor-pointer border-2 rounded-md hover:border-green-500 dark:hover:border-green-600 px-8 py-2 transition"
    };

    let app_data = use_context::<Signal<AppData>>();

    let start_collection = move |_| {
        if !is_collecting() {
            collect_msg.set("Collection in progress, please wait...".to_string());
            is_collecting.set(true);

            let config_c = config.clone();
            spawn(async move {
                
                let mut parser_obj: YamlParser = YamlParser::new(config_c.resource_path.unwrap());
                let doc_artifacts = parser_obj.get_doc_struct().await;
                let list_artifacts: Vec<String> = parser_obj.select_artifact(app_data().resource_list.clone(), doc_artifacts);
                
                let mut collector_obj = Collect::new(app_data().source_path.clone(), app_data().destination_path.clone(), list_artifacts.clone()).await;
                collector_obj.start().await;

                collect_msg.set("Collection completed!".to_string());
                if app_data().vss  {
                    sleep(Duration::from_secs(3)).await;
                    
                    collect_msg.set("Collection from VSS is in progress, please wait...".to_string());
                    let vss_obj = CollectVss::new(app_data().source_path.clone(), app_data().destination_path.clone(), list_artifacts);
                    vss_obj.collect().await;

                    collect_msg.set("VSS collection completed!".to_string());
                }
                
                sleep(Duration::from_secs(3)).await;
                collect_msg.set("Ready to collect".to_string());
                is_collecting.set(false);
            });
        }
    };

    rsx!{
        div{
            class:"w-full flex gap-3 justify-between",
            div{
                class:"bg-stone-300 dark:bg-stone-800 w-full flex justify-end items-center text-md gap-3 pr-8 rounded-md border ",
                p {
                    class:  if is_collecting() { 
                        "text-green-600 dark:text-green-400 font-bold" 
                    } else { 
                        "text-gray-600 dark:text-gray-400 italic" 
                    },
                    "{collect_msg}"
                }
                if is_collecting(){
                    div { 
                        class:"text-green-500",
                        role: "status",
                        svg {
                            class: "inline w-8 h-8 text-neutral-tertiary animate-spin fill-success",
                            view_box: "0 0 100 101",
                            path {
                                d: "M100 50.5908C100 78.2051 77.6142 100.591 50 100.591C22.3858 100.591 0 78.2051 0 50.5908C0 22.9766 22.3858 0.59082 50 0.59082C77.6142 0.59082 100 22.9766 100 50.5908ZM9.08144 50.5908C9.08144 73.1895 27.4013 91.5094 50 91.5094C72.5987 91.5094 90.9186 73.1895 90.9186 50.5908C90.9186 27.9921 72.5987 9.67226 50 9.67226C27.4013 9.67226 9.08144 27.9921 9.08144 50.5908Z",
                                fill: "currentColor",
                            }
                            path {
                                d: "M93.9676 39.0409C96.393 38.4038 97.8624 35.9116 97.0079 33.5539C95.2932 28.8227 92.871 24.3692 89.8167 20.348C85.8452 15.1192 80.8826 10.7238 75.2124 7.41289C69.5422 4.10194 63.2754 1.94025 56.7698 1.05124C51.7666 0.367541 46.6976 0.446843 41.7345 1.27873C39.2613 1.69328 37.813 4.19778 38.4501 6.62326C39.0873 9.04874 41.5694 10.4717 44.0505 10.1071C47.8511 9.54855 51.7191 9.52689 55.5402 10.0491C60.8642 10.7766 65.9928 12.5457 70.6331 15.2552C75.2735 17.9648 79.3347 21.5619 82.5849 25.841C84.9175 28.9121 86.7997 32.2913 88.1811 35.8758C89.083 38.2158 91.5421 39.6781 93.9676 39.0409Z",
                                fill: "currentFill",
                            }
                        }
                        span { class: "sr-only", "Loading..." }
                    }
                }
            }
            div{
                class:"flex px-3 gap-4",
                button {
                    class:"dark:hover:text-yellow-200 hover:text-yellow-600 hover:cursor-pointer transition",
                    onclick: move |_| {
                        is_dark.set(!is_dark());
                    },
                    if is_dark() {
                        Icon{
                            width:20,
                            height:20,
                            icon: ld_icons::LdSun
                        }
                    } else {
                        Icon{
                            width:20,
                            height:20,
                            icon: ld_icons::LdMoon
                        }
                    }
                }
                div{
                	class:"",
	                button {
                        class:"{class_start_button}",
                        onclick: start_collection, 
                        "Start"
	                }
                }
            }
        }
    }
}