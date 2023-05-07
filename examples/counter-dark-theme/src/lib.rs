mod components;
mod pages;

use crate::components::ThemeToggle;
use hashira::{
    app::{App as HashiraApp, AppService, LayoutContext},
    server::{LinkTag, PageLinks},
    web::RequestExt,
};
pub use pages::{CounterPage, CounterPageProps, HomePage};
use yew::{html::ChildrenProps, Html, Suspense};

pub async fn root_layout(mut ctx: LayoutContext) -> Html {
    use hashira::components::*;

    ctx.title("Hashira");
    ctx.links(PageLinks::new().insert(LinkTag::stylesheet("/static/global.css")));

    let mut dark_class = None;
    if ctx
        .request()
        .cookie("dark")
        .map(|c| c.value() == "true")
        .unwrap_or_default()
    {
        dark_class = Some("dark");
    }

    yew::html! {
        <html lang="en">
            <head>
                <Title/>
                <Meta/>
                <Links/>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1.0" />
            </head>
            <body class={yew::classes!(dark_class)}>
                <Main>
                    <Content/>
                </Main>
                <Scripts/>
                <LiveReload/>
            </body>
        </html>
    }
}

#[yew::function_component]
pub fn App(props: &ChildrenProps) -> yew::Html {
    yew::html! {
       <>
        <header>
            <nav>
                <a href="/">{"Home"}</a>
                <a href="/counter">{"Counter"}</a>
                <div class="theme-toggle">
                    <Suspense>
                        <ThemeToggle/>
                    </Suspense>
                </div>
            </nav>
        </header>
        <>{for props.children.iter()}</>
       </>
    }
}

// Setup all the components
pub fn hashira() -> AppService {
    HashiraApp::<App>::new()
        .use_default_error_pages()
        .layout(root_layout)
        .page::<HomePage>()
        .page::<CounterPage>()
        .build()
}

#[cfg(feature = "client")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    wasm_logger::init(wasm_logger::Config::default());
    log::debug!("Hydrating app...");
    let service = hashira();
    hashira::client::mount::<App>(service);
}
