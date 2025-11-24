use dioxus::prelude::*;


#[component]
pub fn Output() -> Element {
    rsx!{
        Zip{}
    }
}

#[component]
pub fn Zip() -> Element {
    let mut zip = use_signal(|| false);
    let mut zip_pass = use_signal(|| false);
    if !(*zip.read()) {
        zip_pass.set(false)
    }
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
                                oninput: move |e| zip_pass.set(e.checked()),
                            }
                        }
                }

                div {
                    class: "ml-3",
                    if *zip_pass.read() {
                        input {
                            class: "border rounded-xl pl-2 h-6",
                            placeholder: " zip password",
                            min: 3,
                            r#type: "text",
                            // oninput: move |e| zip_pass.set(e.checked()),
                            name: "zip_password",
                        }
                    }
                }
            }
        }
    }
}

