use crate::components::{PageData, PageProps, RenderFn};
use serde::de::DeserializeOwned;
use yew::html::ChildrenProps;
use yew::BaseComponent;
use yew::Renderer;

use crate::components::{Page, HASHIRA_PAGE_DATA, HASHIRA_ROOT};

pub fn mount<COMP>()
where
    COMP: BaseComponent,
    COMP::Properties: DeserializeOwned + Send + Clone,
{
    mount_to::<COMP, crate::components::app::App>();
}

pub fn mount_to<COMP, ROOT>()
where
    ROOT: BaseComponent<Properties = ChildrenProps>,
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
            <ROOT>
                <COMP ..props/>
            </ROOT>
        }
    });

    let root = find_element_by_id(HASHIRA_ROOT);
    let renderer = Renderer::<Page>::with_root_and_props(root, PageProps { render });
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
