mod fetch_json;
pub use fetch_json::*;

use wasm_bindgen::JsCast;
use crate::app::AppService;
use crate::components::{PageData, PageProps, HASHIRA_WASM_LOADER};
use crate::context::ServerContext;

use yew::html::ChildrenProps;
use yew::BaseComponent;
use yew::Renderer;

use crate::components::{Page, HASHIRA_PAGE_DATA, HASHIRA_ROOT};

pub fn mount<BASE>(service: AppService)
where
    BASE: BaseComponent<Properties = ChildrenProps>,
{
    let page_data_element = find_element_by_id(HASHIRA_PAGE_DATA).unwrap();
    let content = page_data_element
        .text_content()
        .expect("unable to get page data");
    let page_data =
        serde_json::from_str::<PageData>(&content).expect("failed to deserialize page data");

    let router = service.page_router().clone();
    let error_router = service.error_router().clone();
    let props = PageProps {
        page_data,
        error_router,
        router,
        server_context: ServerContext::new(None),
    };

    // Find the element to hydrate the page
    let root = find_element_by_id(HASHIRA_ROOT).unwrap();
    let renderer = Renderer::<Page<BASE>>::with_root_and_props(root, props);
    renderer.hydrate();

    // Notify the wasm is loaded
    if let Ok(wasm_loader) = find_element_by_id(HASHIRA_WASM_LOADER) {
        // The name is different: https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/dataset#name_conversion
        let wasm_loader: web_sys::HtmlElement = wasm_loader.dyn_into().unwrap();
        wasm_loader.dataset().set("wasmLoaded", "true").unwrap();
    }

    // Initialize
    #[cfg(feature = "hooks")]
    {
        use crate::events::Hooks;
        use std::sync::Arc;

        let hooks = service
            .app_data()
            .get::<Arc<Hooks>>()
            .expect("hooks were not set");

        // FIXME: We only use the initialize hooks once, so must be dropped somehow after being called
        for init in hooks.on_client_initialize_hooks.iter() {
            init.call(service.clone());
        }
    }
}

// TODO: during development show a modal with the error,
// this way the error is not just hidden by the console
#[allow(dead_code, unused_variables)]
fn set_panic_hook(service: &AppService) {
    #[cfg(feature = "hooks")]
    {
        use crate::events::Hooks;
        use std::sync::Arc;

        let service = service.clone();

        yew::set_custom_panic_hook(Box::new(move |info| {
            let hooks = service
                .app_data()
                .get::<Arc<Hooks>>()
                .expect("hooks were not set");

            for on_error in hooks.on_client_error_hooks.iter() {
                on_error.call(info);
            }

            // Send the error to the console
            console_error_panic_hook::hook(info);
        }));
    }
}

fn find_element_by_id(id: &str) -> Result<web_sys::Element, String> {
    let window = web_sys::window().expect("unable to get `window`");
    let document = window.document().expect("unable to get `document`");

    let selector = format!("#{}", id);
    document
        .query_selector(&selector)
        .expect("failed to select element")
        .ok_or_else(|| format!("unable to find '{id}'"))
}
