use hashira::hooks::use_query_params;
use serde::{Deserialize, Serialize};
use yew::function_component;

#[derive(Debug, Serialize, Deserialize)]
struct Theme {
    dark: bool,
}

#[function_component]
pub fn ThemeToggle() -> yew::Html {
    let theme = use_query_params::<Theme>();

    let is_dark = use_query_params::<Theme>()
        .map(|q| q.dark)
        .unwrap_or(false);

    yew::html! {
        <form class="theme-toggle">
            if is_dark {
                <button name="dark" value="false">
                    {"☀️"}
                </button>
            } else {
                <button name="dark" value="true">
                    {"🌙"}
                </button>
            }
        </form>
    }
}
