use hashira::{hooks::use_server_context, web::RequestExt};
use yew::function_component;

#[function_component]
pub fn ThemeToggle() -> yew::HtmlResult {
    let ctx = use_server_context();
    let is_dark = yew::use_prepared_state!(
        |_| -> bool {
            // SAFETY: Is safe to unwrap here because this is only ran on the server
            let ctx = ctx.unwrap();
            ctx.request()
                .cookie("dark")
                .map(|c| c.value() == "true")
                .unwrap_or_default()
        },
        ()
    )?
    .expect("failed to get dark mode state");

    Ok(yew::html! {
        <form class="theme-toggle">
            if *is_dark {
                <button formaction="/api/change_theme">
                    {"☀️"}
                </button>
            } else {
                <button formaction="/api/change_theme">
                    {"🌙"}
                </button>
            }
        </form>
    })
}
