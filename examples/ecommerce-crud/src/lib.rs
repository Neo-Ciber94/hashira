mod components;
mod models;
mod pages;

use crate::components::{root_layout, NavBar};
use hashira::{
    app::{App as HashiraApp, AppService, RenderContext},
    page_component,
};
use yew::html::ChildrenProps;

#[page_component]
pub fn App(props: &ChildrenProps) -> yew::Html {
    yew::html! {
       <>
        <NavBar/>
        <>{for props.children.iter()}</>
       </>
    }
}

#[page_component]
pub fn HomePage() -> yew::Html {
    yew::html! {
        <div>{"Home Page"}</div>
    }
}

// Setup all the components
pub fn hashira() -> AppService {
    HashiraApp::<App>::new()
        .use_default_error_pages()
        .layout(root_layout)
        .page("/", |ctx: RenderContext| async {
            let res = ctx.render::<HomePage, _>().await;
            Ok(res)
        })
        .nest("/products", crate::pages::products())
        .nest("/orders", crate::pages::orders())
        .nest("/users", crate::pages::users())
        .nest("/auth", crate::pages::auth())
        .build()

        

}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    wasm_logger::init(wasm_logger::Config::default());
    log::debug!("Hydrating app...");
    let service = hashira();
    hashira::client::mount::<App>(service);
}
