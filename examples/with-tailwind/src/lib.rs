use hashira::{
    app::{App as HashiraApp, AppService, LayoutContext, RenderContext},
    page_component,
    server::{LinkTag, Metadata, PageLinks},
    web::{IntoResponse, Response},
};
use yew::{function_component, html::ChildrenProps, BaseComponent};

pub async fn root_layout(_: LayoutContext) -> yew::Html {
    use hashira::components::*;

    yew::html! {
        <html lang="en">
            <head>
                <Title/>
                <Meta/>
                <Links/>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1.0" />
            </head>
            <body class="h-screen bg-gradient-to-r from-indigo-500 via-purple-500 to-pink-500">
                <Main class="flex justify-center items-center h-full">
                    <Content/>
                </Main>
                <Scripts/>
                <LiveReload/>
            </body>
        </html>
    }
}

#[function_component]
pub fn App(props: &ChildrenProps) -> yew::Html {
    yew::html! {
        <>{for props.children.iter()}</>
    }
}

async fn hello_page_loader(mut ctx: RenderContext) -> Result<Response, hashira::error::Error> {
    ctx.title("Hashira x TailwindCSS");
    ctx.metadata(Metadata::new().description("A Hashira x TailwindCSS example"));
    ctx.links(PageLinks::new().insert(LinkTag::stylesheet("/static/global.css")));

    let res = ctx.render::<HomePage, App>().await;
    Ok(res.into_response())
}

#[page_component("/", loader = "hello_page_loader")]
pub fn HomePage() -> yew::Html {
    yew::html! {
        <div class="bg-gray-900 font-bold rounded-lg shadow-lg w-11/12 h-[400px]
        flex flex-row justify-center items-center">
            <h1 class="text-center text-white text-4xl flex md:flex-row items-center flex-col md:gap-0 gap-2">
                <span>{"Hashira"}</span>
                <span class="mx-5">{'\u{00D7}'}</span>
                <span class="h-auto w-[250px]">
                    <img alt="TailwindCSS" src="/static/tailwindcss-logo.svg"/>

                </span>
            </h1>
        </div>
    }
}

// Setup all the components
pub fn hashira<BASE>() -> AppService
where
    BASE: BaseComponent<Properties = ChildrenProps>,
{
    HashiraApp::<BASE>::new()
        .layout(root_layout)
        .use_default_error_pages()
        .page::<HomePage>()
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
