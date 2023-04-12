mod components;

use crate::components::Counter;
use hashira::{
    app::{App as HashiraApp, AppService, RenderContext},
    server::{LinkTag, Metadata, PageLinks},
};
use serde::{Deserialize, Serialize};
use yew::{html::ChildrenProps, BaseComponent, Properties};

#[yew::function_component]
pub fn App(props: &ChildrenProps) -> yew::Html {
    yew::html! {
       <>
        <header>
            <nav>
                <a href="/counter">{"Counter"}</a>
            </nav>
        </header>
        <>{for props.children.iter()}</>
       </>
    }
}

#[derive(PartialEq, Clone, Properties, Serialize, Deserialize)]
pub struct HomePageProps {
    #[prop_or_default]
    counter_start: i32,
}

#[yew::function_component]
pub fn HomePage(props: &HomePageProps) -> yew::Html {
    yew::html! {
        <Counter value={props.counter_start}/>
    }
}

// Setup all the components
pub fn hashira<C>() -> AppService
where
    C: BaseComponent<Properties = ChildrenProps>,
{
    HashiraApp::<C>::new()
        .use_default_error_pages()
        .page("/", |mut ctx: RenderContext<HomePage, C>| async {
            ctx.add_title("Hashira | Counter");
            ctx.add_links(PageLinks::new().add(LinkTag::stylesheet("/static/global.css")));
            ctx.add_metadata(Metadata::new().description("A counter made with hashira actix-web"));

            let props = yew::props! { HomePageProps {} };
            let res = ctx.render_with_props(props).await;
            Ok(res)
        })
        .build()
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    wasm_logger::init(wasm_logger::Config::default());
    log::debug!("Hydrating app...");
    let service = hashira::<App>();
    hashira::client::mount::<App>(service);
}
