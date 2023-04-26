use hashira::page_component;

#[page_component]
pub fn HomePage() -> yew::Html {
    yew::html! {
        <div class="container">
            <div class="logo-container">
                <span class="hashira" title="Hashira">{"Hashira"}</span>
            </div>
        </div>
    }
}
