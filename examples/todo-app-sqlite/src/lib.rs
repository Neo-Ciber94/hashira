mod components;
mod models;
mod pages;

use crate::components::NavBar;
use hashira::web::status::StatusCode;
use hashira::{
    app::{redirection, App as HashiraApp, AppService, LayoutContext},
    server::{LinkTag, PageLinks},
};
use yew::{function_component, html::ChildrenProps};

// Setup all the components
#[allow(unused_mut)]
pub async fn hashira() -> hashira::Result<AppService> {
    let mut app = HashiraApp::<App>::new()
        .use_default_error_pages()
        .layout(root_layout)
        .route(redirection("/", "/todos", StatusCode::TEMPORARY_REDIRECT))
        .nest("/todos", crate::pages::todos());

    #[cfg(not(target_arch = "wasm32"))]
    {
        // Setup the database
        use sqlx::SqlitePool;
        let database_url = std::env::var("DATABASE_URL").expect("database is not set");
        let pool = SqlitePool::connect(&database_url).await?;
        app = app.app_data(pool);
    }

    Ok(app.build())
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
                <WasmLoading />
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

    wasm_bindgen_futures::spawn_local(async move {
        let service = hashira().await.expect("failed to init hashira");
        hashira::client::mount::<App>(service);
    });
}
