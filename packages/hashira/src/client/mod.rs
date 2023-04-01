use crate::components::{AppPageProps, PageData, RenderFn};
use serde::de::DeserializeOwned;
use yew::BaseComponent;
use yew::Renderer;

use crate::components::{AppPage, HASHIRA_PAGE_DATA, HASHIRA_ROOT};

pub fn hydrate<COMP>()
where
    COMP: BaseComponent,
    COMP::Properties: DeserializeOwned + Send + Clone,
{
    let page_data_element = find_element_by_id(HASHIRA_PAGE_DATA);
    let content = page_data_element
        .text_content()
        .expect("unable to get page data");
    let page_data =
        serde_json::from_str::<PageData>(&content).expect("failed to deserialize page data");
    let component_name = std::any::type_name::<COMP>().to_string();

    if component_name != page_data.component_name {
        return;
    }

    let props = serde_json::from_value::<COMP::Properties>(page_data.props)
        .expect("failed to deserialize props");
    let render = RenderFn::new(move || {
        let props = props.clone();
        yew::html! {
            <COMP ..props/>
        }
    });

    let root = find_element_by_id(HASHIRA_ROOT);
    let renderer = Renderer::<AppPage>::with_root_and_props(root, AppPageProps { render });
    renderer.hydrate();
}

fn find_element_by_id(id: &str) -> web_sys::Element {
    let window = web_sys::window().expect("unable to get `window`");
    let document = window.document().expect("unable to get `document`");

    let selector = format!("#{}", id);
    let element = document
        .query_selector(&selector)
        .expect("failed to select element")
        .unwrap_or_else(|| panic!("unable to find '{id}'"));

    element
}
