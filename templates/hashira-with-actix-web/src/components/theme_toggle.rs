use hashira::hooks::use_query_params;
use serde::{Deserialize, Serialize};
use yew::function_component;

#[derive(Serialize, Deserialize)]
struct Theme {
    dark: bool,
}

#[function_component]
pub fn ThemeToggle() -> yew::Html {
    // let is_dark = use_query_params::<Theme>()
    //     .map(|q| !q.dark)
    //     .unwrap_or(false);

    yew::html! {
        <form>
            <input type="submit" name="dark" value={"true"}/>
        </form>
    }
}
