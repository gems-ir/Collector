mod app;
mod config;
mod values_linux;
mod values_windows;

use dioxus::desktop::tao::window::Icon;
use dioxus::desktop::{Config, WindowBuilder};
use dioxus::prelude::*;
use dark_light::Mode;

use app::{Start, Information, Output, ListResources};

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
                // .with_always_on_bottom(false)
                .with_title("Collector Gui")
                .with_window_icon(Some(icon))
        )
    )
        .launch(App)
}

#[component]
fn App() -> Element {
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
    rsx!{
        div{
            class:"dark:bg-slate-800 p-3 h-dvh text-black dark:text-slate-300 text-sm",
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
                    Information{}
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
                    Output{}
                }
            }
            div{
                // resources, middle
                class:"h-7/10",
                div{
                    class:"h-full grid",
                    ListResources{}
                }
            }
            div{
                // footer, bottom, start
                class:"",
                div{
                    class:"flex justify-center w-full  ",
                    Start{is_dark}
                }
            }
        }
    }
}