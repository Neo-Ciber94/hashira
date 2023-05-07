use hashira::{app::RenderContext, error::Error, page_component, server::Metadata, web::Response};
use yew::{function_component, html::ChildrenProps, use_state};

#[function_component]
pub fn App(props: &ChildrenProps) -> yew::Html {
    yew::html! {
        <>{for props.children.iter()}</>
    }
}

async fn render(mut ctx: RenderContext) -> Result<Response, Error> {
    ctx.metadata(Metadata::new().description("Hashira x Actix Web example"));
    let res = ctx.render::<HomePage, App>().await;
    Ok(res)
}

#[page_component("/", render = "render")]
pub fn HomePage() -> yew::Html {
    yew::html! {
        <div class="container">
            <div class="logo-container">
                <span class="hashira" title="Hashira">{"Hashira"}</span>
                <span class="divider">{'\u{00D7}'}</span>
                <a href="https://actix.rs/" target="_blank" rel="noopener">
                    <img title="Actix Web" alt="Actix Web" src="https://actix.rs/img/logo.png"/>
                </a>
            </div>

            <div class="counter-container">
                <Counter />
            </div>
        </div>
    }
}

#[function_component]
pub fn Counter() -> yew::Html {
    let counter = use_state(|| 0);
    let increment = {
        let counter = counter.clone();
        move |_| {
            let value = *counter + 1;
            counter.set(value);
        }
    };

    let decrement = {
        let counter = counter.clone();
        move |_| {
            let value = *counter - 1;
            counter.set(value);
        }
    };

    yew::html! {
        <div class="counter">
            <button onclick={decrement}>{'\u{2013}'}</button>
            <span>{ *counter }</span>
            <button onclick={increment}>{"+"}</button>
        </div>
    }
}
