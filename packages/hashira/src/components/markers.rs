// TODO: append an uuid to ensure the value is unique?

use yew::AttrValue;

pub const HASHIRA_META_MARKER: &str = "<!--hashira_meta-->";
pub const HASHIRA_CONTENT_MARKER: &str = "<!--hashira_content-->";
pub const HASHIRA_LINKS_MARKER: &str = "<!--hashira_links-->";
pub const HASHIRA_SCRIPTS_MARKER: &str = "<!--hashira_scripts-->";
pub const HASHIRA_ROOT: &str = "__hashira__root__";
pub const HASHIRA_PAGE_DATA: &str = "__hashira__page_data__";

#[yew::function_component]
pub fn Meta() -> yew::Html {
    yew::Html::from_html_unchecked(AttrValue::from(HASHIRA_META_MARKER))
}

#[yew::function_component]
pub fn Content() -> yew::Html {
    yew::Html::from_html_unchecked(AttrValue::from(HASHIRA_CONTENT_MARKER))
}

#[yew::function_component]
pub fn Links() -> yew::Html {
    yew::Html::from_html_unchecked(AttrValue::from(HASHIRA_LINKS_MARKER))
}

#[yew::function_component]
pub fn Scripts() -> yew::Html {
    yew::Html::from_html_unchecked(AttrValue::from(HASHIRA_SCRIPTS_MARKER))
}
