mod app;

use app::{App, CreateMessageAction, MessagesPage};
use hashira::{
    app::{App as HashiraApp, AppService, LayoutContext},
    server::{LinkTag, PageLinks},
};
use std::sync::Arc;
use tokio::sync::RwLock;

// Queue of messages.
pub type Messages = Arc<RwLock<Vec<String>>>;

// Setup all the components
pub fn hashira() -> AppService {
    HashiraApp::<App>::new()
        .use_default_error_pages()
        .server_data(Messages::default())
        .layout(root_layout)
        .action::<CreateMessageAction>()
        .page::<MessagesPage>()
        .build()
}

pub async fn root_layout(mut ctx: LayoutContext) -> yew::Html {
    use hashira::components::*;

    ctx.title("Hashira");
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

#[cfg(feature = "client")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    wasm_logger::init(wasm_logger::Config::default());
    log::debug!("Hydrating app...");
    let service = hashira();
    hashira::client::mount::<App>(service);
}
