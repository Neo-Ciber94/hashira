use yew::hook;
use super::use_page_data;

/// Returns the path of the current route
#[hook]
pub fn use_path_name() -> String {
    let page_data = use_page_data();
    page_data.uri.path().to_string()
}