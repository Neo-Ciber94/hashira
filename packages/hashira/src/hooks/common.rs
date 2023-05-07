use super::use_page_data;
use yew::hook;

/// Returns the current route `Uri`.
#[hook]
pub fn use_current_path() -> http::Uri {
    let page_data = use_page_data();
    page_data.uri.clone()
}

/// Returns the params of the route.
#[hook]
pub fn use_params() -> crate::routing::Params {
    let page_data = use_page_data();
    page_data.params.clone()
}
