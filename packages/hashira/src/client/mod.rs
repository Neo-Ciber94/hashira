use yew::Renderer;

use crate::components::{AppPage, HASHIRA_ROOT};

pub fn hydrate() {
    let window = web_sys::window().expect("unable to get `window`");
    let document = window.document().expect("unable to get `document`");

    let selector = format!("#{}", HASHIRA_ROOT);
    let root = document
        .query_selector(&selector)
        .expect("failed to select element")
        .unwrap_or_else(|| panic!("unable to find 'HASHIRA_ROOT'"));

    let renderer = Renderer::<AppPage>::with_root(root);
    renderer.hydrate();
}
