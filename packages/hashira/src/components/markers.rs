use yew::{html::ChildrenProps, AttrValue};

pub const HASHIRA_TITLE_MARKER: &str = "<!--hashira_title-->";
pub const HASHIRA_META_MARKER: &str = "<!--hashira_meta-->";
pub const HASHIRA_CONTENT_MARKER: &str = "<!--hashira_content-->";
pub const HASHIRA_LINKS_MARKER: &str = "<!--hashira_links-->";
pub const HASHIRA_SCRIPTS_MARKER: &str = "<!--hashira_scripts-->";
pub const HASHIRA_ROOT: &str = "__hashira__root__";
pub const HASHIRA_PAGE_DATA: &str = "__hashira__page_data__";

/// A marker for insert the page `<title>` element.
#[yew::function_component]
pub fn Title() -> yew::Html {
    yew::Html::from_html_unchecked(AttrValue::from(HASHIRA_TITLE_MARKER))
}

/// A marker for insert the page `<meta>` elements.
#[yew::function_component]
pub fn Meta() -> yew::Html {
    yew::Html::from_html_unchecked(AttrValue::from(HASHIRA_META_MARKER))
}

/// A marker for insert the page component.
#[yew::function_component]
pub fn Content() -> yew::Html {
    yew::Html::from_html_unchecked(AttrValue::from(HASHIRA_CONTENT_MARKER))
}

/// A marker for insert the page `<link>` elements.
#[yew::function_component]
pub fn Links() -> yew::Html {
    yew::Html::from_html_unchecked(AttrValue::from(HASHIRA_LINKS_MARKER))
}

/// A marker for insert the page `<script>` elements.
#[yew::function_component]
pub fn Scripts() -> yew::Html {
    yew::Html::from_html_unchecked(AttrValue::from(HASHIRA_SCRIPTS_MARKER))
}

/// A components that insert a `<main>` element where the page will be rendered and hydrated.
#[yew::function_component]
pub fn Main(props: &ChildrenProps) -> yew::Html {
    yew::html! {
        <main id={HASHIRA_ROOT}>
            {for props.children.iter()}
        </main>
    }
}
