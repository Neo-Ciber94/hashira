use yew::{html::ChildrenProps, AttrValue, Children, Properties};

pub const HASHIRA_TITLE_MARKER: &str = "<!--hashira_title-->";
pub const HASHIRA_META_MARKER: &str = "<!--hashira_meta-->";
pub const HASHIRA_CONTENT_MARKER: &str = "<!--hashira_content-->";
pub const HASHIRA_LINKS_MARKER: &str = "<!--hashira_links-->";
pub const HASHIRA_SCRIPTS_MARKER: &str = "<!--hashira_scripts-->";
pub const HASHIRA_ROOT: &str = "__hashira__root__";
pub const HASHIRA_PAGE_DATA: &str = "__hashira__page_data__";
pub const HASHIRA_WASM_LOADER: &str = "__hashira_wasm_loader";

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct MainProps {
    #[prop_or_default]
    pub class: Option<String>,

    #[prop_or_default]
    pub style: Option<String>,

    #[prop_or_default]
    pub children: Children,
}

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
pub fn Main(props: &MainProps) -> yew::Html {
    yew::html! {
        <main id={HASHIRA_ROOT} class={props.class.clone()} style={props.style.clone()}>
            {for props.children.iter()}
        </main>
    }
}

/// A component to handle the live reload on the client.
#[cfg(debug_assertions)]
#[yew::function_component]
pub fn LiveReload() -> yew::Html {
    // Include the script for reloading
    const LIVE_RELOAD_SCRIPT: &str = include_str!("./livereload.js");

    use crate::env::{HASHIRA_LIVE_RELOAD_HOST, HASHIRA_LIVE_RELOAD_PORT};

    // Not live reload
    if !crate::env::is_live_reload() {
        return yew::Html::default();
    }

    let host = std::env::var(HASHIRA_LIVE_RELOAD_HOST)
        .map(|env| format!("'{}'", env))
        .unwrap_or_else(|_| String::from("undefined"));

    let port = std::env::var(HASHIRA_LIVE_RELOAD_PORT).unwrap_or_else(|_| String::from("0"));

    yew::Html::from_html_unchecked(AttrValue::from(format!(
        r#"
        <script>
            window.{HASHIRA_LIVE_RELOAD_HOST} = {host};
            window.{HASHIRA_LIVE_RELOAD_PORT} = {port};

            {LIVE_RELOAD_SCRIPT}
        </script>"#
    )))
}

/// A component to handle the live reload on the client.
#[cfg(not(debug_assertions))]
#[yew::function_component]
pub fn LiveReload() -> yew::Html {
    yew::Html::default()
}

/// WASM could take time to load specially if is no optimized,
/// This is the base component used to notify when the wasm is loaded,
/// the hydration script update this component after is loaded.
#[yew::function_component]
pub fn WasmLoadingBase(props: &ChildrenProps) -> yew::Html {
    // If the wasm bundle is not available we don't need to render this component.
    if crate::consts::IS_SERVER {
        if crate::env::get_client_name().is_none() {
            return yew::Html::default();
        }
    }

    // The `data-wasm-loaded` should be set different on the client:
    // https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/dataset#name_conversion
    yew::html! {
        <div id={HASHIRA_WASM_LOADER} data-wasm-loaded="false">
            {for props.children.iter()}
        </div>
    }
}

/// A loading screen while the wasm is loading.
///
/// FIXME: Remove this in production? in production the wasm load really fast,
/// maybe we should remove this for production.
#[yew::function_component]
pub fn WasmLoading() -> yew::Html {
    yew::html! {
        <WasmLoadingBase>
            <style>
                {yew::Html::from_html_unchecked(AttrValue::from(format!(r#"
                    #{HASHIRA_WASM_LOADER}[data-wasm-loaded="false"] {{
                        position: absolute;
                        z-index: 9999;
                        width: 100%;
                        height: 100%;
                        background-color: white;
                        opacity: 0;
                        cursor: wait;
                    }}
                "#)))}
            </style>
        </WasmLoadingBase>
    }
}
