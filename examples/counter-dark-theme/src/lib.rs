mod components;
mod pages;

use crate::components::{root_layout, ThemeToggle};
use hashira::{
    app::{App as HashiraApp, AppService, RenderContext},
    server::Metadata,
};
pub use pages::{CounterPage, CounterPageProps, HomePage};
use yew::{html::ChildrenProps, BaseComponent, Suspense};

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
pub fn hashira<BASE>() -> AppService
where
    BASE: BaseComponent<Properties = ChildrenProps>,
{
    HashiraApp::<BASE>::new()
        .use_default_error_pages()
        .layout(root_layout)
        .page("/", |mut ctx: RenderContext| async {
            ctx.metadata(Metadata::new().description("A Hashira sample app"));
            let res = ctx.render::<HomePage, BASE>().await;
            Ok(res)
        })
        .page("/counter", |mut ctx: RenderContext| async {
            ctx.title("Hashira | Counter");
            ctx.metadata(Metadata::new().description("A Hashira sample counter"));
            let props = yew::props! { CounterPageProps {} };
            let res = ctx.render_with_props::<CounterPage, BASE>(props).await;
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
