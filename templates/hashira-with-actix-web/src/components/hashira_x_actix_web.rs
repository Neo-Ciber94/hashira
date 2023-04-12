use yew::function_component;

#[function_component]
pub fn HashiraActixWeb() -> yew::Html {
    yew::html! {
        <div class="logo-container">
            <span class="hashira" title="Hashira">{"Hashira"}</span>
            <span class="divider">{'\u{00D7}'}</span>
            <a href="https://actix.rs/" target="_blank" rel="noopener">
                <img title="Actix Web" alt="Actix Web" src="https://actix.rs/img/logo.png"/>
            </a>
        </div>
    }
}
