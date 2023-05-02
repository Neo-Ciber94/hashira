mod components;
mod database;
mod models;
mod pages;

use hashira::web::status::StatusCode;
use hashira::{
    app::{redirect, App as HashiraApp, AppService, LayoutContext},
    server::{LinkTag, PageLinks},
};
use yew::{function_component, html::ChildrenProps};

use crate::components::NavBar;

// Setup all the components
pub fn hashira() -> AppService {
    HashiraApp::<App>::new()
        .use_default_error_pages()
        .layout(root_layout)
        .route(redirect("/", "/todos", StatusCode::PERMANENT_REDIRECT))
        .nest("/todos", crate::pages::todos())
        .build()
}

#[function_component]
pub fn App(props: &ChildrenProps) -> yew::Html {
    yew::html! {
       <>
        <NavBar />
        <div class="container mx-auto">
            {for props.children.iter()}
        </div>
       </>
    }
}

async fn root_layout(mut ctx: LayoutContext) -> yew::Html {
    use hashira::components::*;

    ctx.title("Todo App");
    ctx.links(PageLinks::new().insert(LinkTag::stylesheet("/static/global.css")));

    yew::html! {
        <html lang="en">
            <head>
                <Title/>
                <Meta/>
                <Links/>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1.0" />
            </head>
            <body>
                <Main>
                    <Content/>
                </Main>
                <Scripts/>
                <LiveReload/>
            </body>
        </html>
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    wasm_logger::init(wasm_logger::Config::default());
    log::debug!("Hydrating app...");
    let service = hashira();
    hashira::client::mount::<App>(service);
}
