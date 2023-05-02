use hashira::{hooks::use_server_context, web::RequestExt};
use web_sys::window;
use yew::{function_component, html::onsubmit::Event, use_state};

#[function_component]
pub fn ThemeToggle() -> yew::HtmlResult {
    let ctx = use_server_context();
    let initial_state = yew::use_prepared_state!(
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

    let is_dark = use_state(|| *initial_state);

    let on_dark_mode_toggle = {
        let is_dark = is_dark.clone();
        move |e: Event| {
            // This will prevent the form from reload while we notify the server
            // to change the theme. If javascript if off, this still works
            e.prevent_default();

            // Add `dark` to the body
            let window = window().unwrap();
            let body = window.document().unwrap().body().unwrap();
            body.class_list().toggle("dark").unwrap();
            is_dark.set(!*is_dark);

            // Notify the server
            let _ = window.fetch_with_str("/api/change_theme");
        }
    };

    Ok(yew::html! {
        <form class="theme-toggle" onsubmit={on_dark_mode_toggle}>
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
