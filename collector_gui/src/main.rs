mod app;
mod config;
mod values_linux;
mod values_windows;

use dioxus::desktop::tao::window::Icon;
use dioxus::desktop::{Config, WindowBuilder};
use dioxus::prelude::*;
use dark_light::Mode;

use crate::app::{Start, Information, Output, ListResources};
use crate::config::{Config as ConfigCollector, AppData};

const ICON_BYTES: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../assets/logo.png"));


fn main() {
    let icon = image::load_from_memory(ICON_BYTES)
        .expect("Failed to load icon")
        .to_rgba8();
    let (width, height) = icon.dimensions();
    let icon = Icon::from_rgba(icon.into_raw(), width, height)
        .expect("Failed to create icon");
    LaunchBuilder::desktop().with_cfg(
        Config::new().with_window(
            WindowBuilder::new()
                .with_focused(true)
                .with_inner_size(dioxus::desktop::LogicalSize::new(1000, 650))
                .with_min_inner_size(dioxus::desktop::LogicalSize::new(720, 200))
                .with_title("Collector GUI")
                .with_window_icon(Some(icon))
        )
    )
        .launch(App)
}

#[component]
fn App() -> Element {
    use_context_provider(|| Signal::new(AppData::default()));

    let is_dark = use_signal(|| {
        match dark_light::detect() {
            Ok(Mode::Dark) => true,
            Ok(Mode::Light) => false,
            _  => false,
        }
    });
    rsx! {
		Stylesheet {
            href: asset!("assets/css/tailwind.css")
        }
        div{
            class: if is_dark() { "dark min-h-screen" } else { "min-h-screen" },
            MainFrame{is_dark}
        }
	}
}

#[component]
pub fn MainFrame(is_dark: Signal<bool>) -> Element {
    let config = ConfigCollector::parse_config_file();
    rsx!{
        div{
            class:"dark:bg-slate-800 p-3 h-dvh text-black dark:text-slate-300 text-sm overflow min-h-fit min-w-fit flex-col",
            div{
                // top div
                class:"grid grid-cols-2 border-b-1 p-1 divide-x",
                div{
                    // Input, left
                    class:"",
                    div{
                        class:"w-full flex justify-center pb-4",
                        h1 {
                            class: "border-b-1 text-xl dark:text-slate-300",
                           "Input information"
                        }
                    }
                    Information{config: config.clone()}
                }
                div{
                    // Output, right
                    class:"",
                    div{
                        class:"w-full flex justify-center pb-4",
                        h1 {
                            class: "border-b-1 text-xl dark:text-slate-300",
                           "Output information"
                        }
                    }
                    Output{config: config.clone()}
                }
            }
             
            div{
                class:"h-7/10 min-h-50 w-full border-b-1",
                div{
                    class:"h-full",
                    ListResources{config: config.clone()}
                }
            }
            div{
                // footer, bottom, start
                class:"pt-3",
                div{
                    Start{is_dark}
                }
            }
           
        }
    }
}