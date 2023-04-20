use crate::app::AppService;
use crate::components::{PageData, PageProps};
use crate::context::ServerContext;
use yew::html::ChildrenProps;
use yew::BaseComponent;
use yew::Renderer;

use crate::components::{Page, HASHIRA_PAGE_DATA, HASHIRA_ROOT};

// TODO: Add custom panic hook

pub fn mount<C>(service: AppService)
where
    C: BaseComponent<Properties = ChildrenProps>,
{
    let page_data_element = find_element_by_id(HASHIRA_PAGE_DATA);
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

    let root = find_element_by_id(HASHIRA_ROOT);
    let renderer = Renderer::<Page<C>>::with_root_and_props(root, props);
    renderer.hydrate();
}

fn find_element_by_id(id: &str) -> web_sys::Element {
    let window = web_sys::window().expect("unable to get `window`");
    let document = window.document().expect("unable to get `document`");

    let selector = format!("#{}", id);
    document
        .query_selector(&selector)
        .expect("failed to select element")
        .unwrap_or_else(|| panic!("unable to find '{id}'"))
}
