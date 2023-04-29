use hashira::{
    app::{App as HashiraApp, AppService, RenderContext},
    page_component,
    server::{LinkTag, Metadata, PageLinks},
};
use yew::{html::ChildrenProps, BaseComponent};

#[page_component]
pub fn App(props: &ChildrenProps) -> yew::Html {
    yew::html! {
        <>{for props.children.iter()}</>
    }
}

#[page_component]
pub fn HomePage() -> yew::Html {
    yew::html! {
        <div class="container">
            <div class="space-y-4">
                <div class="w-96 bg-white shadow rounded">
                    {"w-96"}
                </div>
                <div class="w-80 bg-white shadow rounded">
                    {"w-80"}
                </div>
                <div class="w-72 bg-white shadow rounded">
                    {"w-72"}
                </div>
                <div class="w-64 bg-white shadow rounded">
                    {"w-64"}
                </div>
                <div class="w-60 bg-white shadow rounded">
                    {"w-60"}
                </div>
                <div class="w-56 bg-white shadow rounded">
                    {"w-56"}
                </div>
                <div class="w-52 bg-white shadow rounded">
                    {"w-52"}
                </div>
                <div class="w-48 bg-white shadow rounded">
                    {"w-48"}
                </div>
            </div>
        </div>
    }
}

// Setup all the components
pub fn hashira<BASE>() -> AppService
where
    BASE: BaseComponent<Properties = ChildrenProps>,
{
    HashiraApp::<BASE>::new()
        .use_default_error_pages()
        .page("/", |mut ctx: RenderContext| async {
            ctx.metadata(Metadata::new().description("An Hashira x TailwindCSS example"));
            ctx.links(PageLinks::new().insert(LinkTag::stylesheet("static/global.css")));

            let res = ctx.render::<HomePage, BASE>().await;
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
