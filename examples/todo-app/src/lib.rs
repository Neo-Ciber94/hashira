mod components;
pub mod models;
mod pages;

use hashira::{
    app::{App as HashiraApp, AppService, LayoutContext, RenderContext},
    page_component,
    server::{LinkTag, PageLinks},
};
use yew::html::ChildrenProps;

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

#[page_component]
pub fn App(props: &ChildrenProps) -> yew::Html {
    yew::html! {
       <>
            <header class="w-full p-4 bg-slate-800">
                <nav class="flex flex-row gap-5 text-white text-lg">
                    <a href="/todos">{"Todos"}</a>
                    <a href="/todos/c502da4f-42c4-4b35-a47a-0f1c11ee632e">{"View Todo"}</a>
                    <a href="/todos/add">{"Create Todo"}</a>
                    <a href="/todos/edit/c502da4f-42c4-4b35-a47a-0f1c11ee632e">{"Update Todo"}</a>
                    <a href="/todos/delete/c502da4f-42c4-4b35-a47a-0f1c11ee632e">{"Delete Todo"}</a>
                </nav>
            </header>

        {for props.children.iter()}
       </>
    }
}

#[page_component]
pub fn HomePage() -> yew::Html {
    yew::html! {
        <div class="container">
            {"Todo App!"}
        </div>
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
        .nest("/todos", crate::pages::todos())
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
