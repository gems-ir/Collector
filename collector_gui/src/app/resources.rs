use std::collections::HashSet;
use crate::config::Config;
use collector_core::resource::YamlParser;
use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons;
use dioxus_free_icons::Icon;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Resource {
	name: String,
	category: String,
	description: String,
	content: String,
	is_checked: bool,
}

#[derive(Clone, PartialEq)]
pub(crate) struct EmptyTable{
	class: String,
	text: String,
}


#[component]
pub fn ListResources() -> Element {
	let config = Config::parse_config_file();

	let resources_list = use_resource(move || {
		let value = config.clone();
		async move {
			get_resources(value.clone()).await
		}
	});

	let mut selected_resource = use_signal(|| None::<Resource>);
	let search_query = use_signal(|| String::new());
	let selected_category = use_signal(|| String::from("All"));

	let mut checked_resources = use_signal(|| Vec::<String>::new()); // output resources 
	let mut is_initialized = use_signal(|| false);

	use_effect(move || {
		if !is_initialized() {
			if let Some(resources) = resources_list.read().as_ref() {
				let initial_checked: Vec<String> = resources
					.iter()
					.filter(|r| r.is_checked)
					.map(|r| r.name.clone())
					.collect();
				
				if !initial_checked.is_empty() {
					checked_resources.set(initial_checked);
					is_initialized.set(true);
				}
			}
		}
	});

	rsx! {
		SearchResources{
			search_query: search_query,
			selected_category: selected_category,
			resources: resources_list.read_unchecked().clone()
		}
		match &*resources_list.read_unchecked() {
			Some(resources) => {
				let et = if resources.is_empty(){
					EmptyTable{
						class: "bg-red-500 h-10 flex justify-center items-center p-4".to_string(),
						text: "Resources path not found !".to_string(),
					}
				}else{
					EmptyTable{
						class: "h-10 flex justify-center items-center p-4".to_string(),
						text: "No resource found !".to_string(),
					}
				};

				let filtered_resources: Vec<Resource> = resources
						.iter()
						.filter(|resource| {
							let query = search_query().to_lowercase();
							let category = selected_category();
							
							let name_matches = if query.is_empty() {
								true
							} else {
								resource.name.to_lowercase().contains(&query)
							};
							
							let category_matches = if category == "All" {
								true
							} else {
								resource.category == category
							};
							
							name_matches && category_matches
						})
						.cloned()
						.collect();
				rsx! {
					TableResources {
						resource_list: filtered_resources,
						checked_resources,
						on_view: move |resource: Resource| {
							selected_resource.set(Some(resource));
						},
						et
					}
				}
			},
			None => rsx! {
				div { class: "p-4 text-center",
					"Resources loading..."
				}
			}
		}
		if let Some(resource) = selected_resource() {
			ResourceModal {
				resource: resource.clone(),
				on_close: move |_| selected_resource.set(None)
			}
		}
		
		div { class: "px-4 flex gap-2",
			h3 { 
				class: "font-bold mb-2 text-stone-900", 
				"Resources selected:" 
			}
			if checked_resources().is_empty() {
				p { "Empty selected resources." }
			} else {
				div {
					class: "flex gap-2",
					for name in checked_resources() {
						div{
							class: "flex",
							p { "{name}, " }
						}
					}
				}
			}
		}
		 
	}
}

#[component]
pub fn TableResources(
		resource_list: Vec<Resource>, 
		checked_resources: Signal<Vec<String>>, 
		on_view: EventHandler<Resource>, 
		et: EmptyTable
	) -> Element {
	 let mut toggle_checkbox = move |name: String| {
        let mut checked = checked_resources();
        if checked.contains(&name) {
            checked.retain(|n| n != &name);
        } else {
            checked.push(name);
        }
        checked_resources.set(checked);
    };
	rsx! {
		div{
			class:"overflow-auto border rounded-md h-full",
			table{
				class:"w-full",
				thead{
					class:"sticky top-0",                    
					tr{
						class:"border-b dark:border-gray-200 bg-gray-400 dark:bg-zinc-900 h-8",
						th{
							class:"dark:text-gray-100 text-sm w-20",
							""
						}
						th{
							class:"dark:text-gray-100 text-sm w-sm",
							"Name"
						}
						th{
							class:"dark:text-gray-100 text-sm w-md",
							"Category"
						}
						th{
							class:"dark:text-gray-100 text-sm w-md",
							"Description"
						}
						th{
							class:"dark:text-gray-100 text-sm w-md",
							"View"
						}
					}
				}
				tbody{
					if resource_list.is_empty() {
						tr {
							class:"",
							td {
								colspan: "5",
								div{
									class:"{et.class}",
									"{et.text}"
								}
							}
						}
					}else{
						for resource in resource_list {
						    {
						        let resource_name = resource.name.clone();
						        
						        rsx! {
						            tr{
						                class:"border-b dark:border-gray-200 even:bg-slate-200 even:dark:bg-slate-600 h-8 hover:bg-slate-300 dark:hover:bg-slate-500",
						                td{
						                    class:"text-center w-20",
						                    input{
						                        r#type:"checkbox",
						                        checked: checked_resources().contains(&resource_name),
						                        onchange: move |_| {
						                            toggle_checkbox(resource_name.clone())
						                        }
						                    }
						                }
						                td{
						                    class:"dark:text-gray-100 text-center w-sm max-w-3xs truncate",
						                    "{resource.name}"
						                }
						                td{
						                    class:"dark:text-gray-100 text-center w-md max-w-3xs truncate",
						                    "{resource.category}"
						                }
						                td{
						                    class:"dark:text-gray-100 text-center w-md max-w-3xs truncate",
						                    "{resource.description}"
						                }
						                td{
						                    class:"dark:text-gray-100 w-md",
						                    div{
						                        class:"w-full flex justify-center",
						                        button{
						                            class:"hover:text-green-500 dark:hover:text-green-700 cursor-pointer",
						                            onclick: move |_| {
						                                on_view.call(resource.clone());
						                            },
						                            Icon{
						                                height: 20,
						                                width: 20,
						                                icon: ld_icons::LdEye
						                            }
						                        }
						                    }
						                }
						            }
						        }
						    }
						}
					}
				}
			}
		}
	}
}



#[component]
pub fn SearchResources(
		search_query: Signal<String>, 
		selected_category: Signal<String>,
		resources: Option<Vec<Resource>>
	) -> Element {

	let categories: Vec<String> = if let Some(res) = resources {
		let cats: HashSet<String> = res.iter()
			.map(|r| r.category.clone())
			.collect();
		
		let mut sorted_cats: Vec<String> = cats.into_iter().collect();
		sorted_cats.sort();
		sorted_cats
	} else {
		Vec::new()
	};

	rsx! {
		div {
			class: "flex p-2 gap-4 items-center",
			// Search div
			div{
				class:"flex p-2 gap-2",
				label { 
					class: "",
					"Search :" 
				}
				input{
					class:"border rounded-xl pl-2 w-3xs dark:border-gray-600 bg-white dark:bg-slate-600 dark:text-gray-200 focus:outline-none focus:ring-2 focus:ring-green-500 dark:focus:ring-green-800",
					placeholder:"Search by name ...",
					name:"search_resource",
					oninput: move |evt| search_query.set(evt.value())
				}
			}
			// categoryes div
			div { 
				class: "flex gap-2 items-center",
				label { 
					class: "",
					"Category : " 
				}
				select {
					class: "border rounded-xl px-1 dark:border-gray-600 bg-white dark:bg-slate-600 focus:outline-none focus:ring-2 focus:ring-green-500 dark:focus:ring-green-800 cursor-pointer",
					value: "{selected_category}",
					onchange: move |evt| selected_category.set(evt.value()),
					option { value: "All", "All Categories" }
					
					for category in categories {
						option { 
							value: "{category}",
							"{category}"
						}
					}
				}
				
			}
		}
	}
}

#[component]
pub fn ResourceModal(
		resource: Resource, 
		on_close: EventHandler<MouseEvent>
	) -> Element {
	
	rsx! {
		div {
			class: "fixed inset-0 bg-black/80 flex items-center justify-center z-50",
			onclick: move |evt| on_close.call(evt),
			
			// Modal content
			div {
				class: "bg-white dark:bg-slate-800 rounded-lg shadow-xl max-w-4xl w-full mx-4 max-h-[90vh] flex flex-col",
				onclick: move |evt| evt.stop_propagation(), 
				
				// Header avec titre et bouton close
				div {
					class: "flex items-center justify-between p-4 border-b dark:border-gray-600",
					h2 {
						class: "text-2xl font-bold text-gray-800 dark:text-gray-100",
						"{resource.name}"
					}
					button {
						class: "text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 text-2xl font-bold w-8 h-8 flex items-center justify-center rounded hover:bg-gray-200 dark:hover:bg-gray-700",
						onclick: move |evt| on_close.call(evt),
						"×"
					}
				}
				
				// Metadata
				div {
					class: "px-4 py-2 bg-gray-50 dark:bg-slate-700 border-b dark:border-gray-600",
					div { class: "flex gap-4 text-sm",
						span {
							class: "text-gray-600 dark:text-gray-300",
							strong { "Category: " }
							"{resource.category}"
						}
						span {
							class: "text-gray-600 dark:text-gray-300",
							strong { "Description: " }
							"{resource.description}"
						}
					}
				}
				
				div {
					class: "flex-1 overflow-auto p-4",
					pre {
						class: "bg-gray-100 dark:bg-slate-900 p-4 rounded text-sm text-gray-800 dark:text-gray-200 font-mono whitespace-pre-wrap",
						"{resource.content}"
					}
				}
			}
		}
	}
}

async fn get_resources(config: Config) -> Vec<Resource> {    
	let Some(resource_path) = config.resource_path else {
		return Vec::new();
	};
	
	let mut parser_obj = YamlParser::new(resource_path);
	let doc_artifacts = parser_obj.get_doc_struct().await;
	


	doc_artifacts
		.iter()
		.map(|artifact| {
			let name = artifact.metadata.name.clone();
			let is_checked = config.resource_list
				.as_ref()
				.map(|names| names.contains(&name))
				.unwrap_or(false);

			Resource {
				name,
				category: artifact.metadata.category.clone().unwrap_or_else(|| "Other".to_string()),
				description: artifact.metadata.description.clone(),
				content: toml::to_string(artifact).unwrap_or_default(),
				is_checked
			}
		})
		.collect()
}
