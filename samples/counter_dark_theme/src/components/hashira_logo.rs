use yew::function_component;

#[function_component]
pub fn HashiraLogo() -> yew::Html {
    yew::html! {
        <div class="logo-container">
            <span class="hashira" title="Hashira">{"Hashira"}</span>
        </div>
    }
}
