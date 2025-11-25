use crate::config::{Config, AppData};

use dioxus::prelude::*;


#[component]
pub fn Output(config: Config) -> Element {
    rsx!{
        Zip{
            config
        }
    }
}

#[component]
pub fn Zip(config: Config) -> Element {
    let mut zip = use_signal(|| false);
    let mut zip_toggle = use_signal(|| false);
    if !(*zip.read()) {
        zip_toggle.set(false)
    }
    let mut zip_pass = use_signal(|| String::new());

    let mut app_data = use_context::<Signal<AppData>>();
    let mut is_initialized = use_signal(|| false);

    use_effect(move || {
        if !is_initialized(){
            if let Some(zip_config) = config.zip {
                zip.set(zip_config);
            }
            if let Some(zip_pass_config) = &config.zip_pass{
                zip_toggle.set(true);
                zip_pass.set(zip_pass_config.to_string())
            }
            is_initialized.set(true);
        }
        app_data.write().zip = zip();
        if zip_toggle(){
            app_data.write().zip_pass = Some(zip_pass());
        }else{
            app_data.write().zip_pass = None;
        }
    });


    rsx!{
        div{
            class: "grid grid-rows-2",
            div{
                class:"grid grid-cols-2",
                div{
                    class:"flex gap-2 justify-self-end",
                        label {
                            class: "",
                            r#for: "zip",
                            "Zip the destination folder"
                        }
                        input {
                            r#type: "checkbox",
                            name: "zip",
                            checked: zip(),
                            oninput: move |e| zip.set(e.checked()),
                        }
                }
                div{}
            }
            div {
                class: "grid grid-cols-2",
                div{
                    class: "flex gap-2 justify-self-end",
                    if *zip.read() {
                            label {
                                r#for: "zip_pass",
                                "Add zip password"
                            }
                            input {
                                r#type: "checkbox",
                                name: "zip_pass",
                                checked: zip_toggle(),
                                oninput: move |e| zip_toggle.set(e.checked()),
                            }
                        }
                }

                div {
                    class: "ml-3",
                    if *zip_toggle.read() {
                        input {
                            class: "border rounded-xl pl-2 h-6 focus:ring-1 focus:outline-none dark:focus:ring-green-800 dark:focus:border-green-800 focus:ring-green-500 focus:border-green-500",
                            placeholder: " zip password",
                            min: 3,
                            r#type: "text",
                            value: zip_pass(),
                            oninput: move |e| zip_pass.set(e.value()),
                            name: "zip_password",
                        }
                    }
                }
            }
        }
    }
}

