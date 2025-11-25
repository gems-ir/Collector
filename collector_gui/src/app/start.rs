use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons;
use dioxus_free_icons::Icon;

use crate::config::AppData;

#[component]
pub fn Start(is_dark: Signal<bool>) -> Element {
    let app_data = use_context::<Signal<AppData>>();
    let execute_collector = move || {
        println!("{:?}",app_data());
    };

    rsx!{
        div{
            class:"w-full flex gap-3 justify-between",
            div{
                class:"bg-green-200 dark:bg-green-800 w-full",
                "loading bar"
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
	                    class:"bg-slate-200 dark:bg-slate-500 dark:border-slate-400 dark:text-white hover:text-green-500 dark:hover:text-green-500 hover:cursor-pointer border-2 rounded-md hover:border-green-500 dark:hover:border-green-600 px-8 py-2 transition",
                        onclick: move |_| {
                            execute_collector()
                        }, 
	                    "Start"
	                }
                }
            }
        }
    }
}