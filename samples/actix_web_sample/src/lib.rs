use hashira::{
    app::{App as HashiraApp, AppService, RenderContext},
    server::Metadata,
};
use serde::{Deserialize, Serialize};
use yew::{html::ChildrenProps, use_state, BaseComponent, Properties};

#[yew::function_component]
pub fn App(props: &ChildrenProps) -> yew::Html {
    yew::html! {
       <>
        <header>
            <nav style="display: flex; flex-direction: row; gap: 2px;">
                <a href="/">{"Home"}</a>
                <a href="/hello/freddy">{"Hello"}</a>
            </nav>
        </header>
        <>{for props.children.iter()}</>
       </>
    }
}

#[yew::function_component]
pub fn HomePage() -> yew::Html {
    let counter = use_state(|| 0);
    let increment = {
        let counter = counter.clone();
        move |_| {
            let value = *counter + 1;
            counter.set(value);
        }
    };

    let decrement = {
        let counter = counter.clone();
        move |_| {
            let value = *counter - 1;
            counter.set(value);
        }
    };

    yew::html! {
        <div>
            <button onclick={decrement}>{ "-1" }</button>
            <p>{ *counter }</p>
            <button onclick={increment}>{ "+1" }</button>
        </div>
    }
}

#[derive(PartialEq, Serialize, Deserialize, Properties, Clone)]
pub struct HelloPageProps {
    pub name: String,
}

#[yew::function_component]
pub fn HelloPage(props: &HelloPageProps) -> yew::Html {
    yew::html! {
        <h1>{format!("Hello {}?", props.name)}</h1>
    }
}

// Setup all the components
pub fn hashira<C>() -> AppService
where
    C: BaseComponent<Properties = ChildrenProps>,
{
    HashiraApp::<C>::new()
        //.app_data(...)
        .use_default_error_pages()
        .page("/", |mut ctx: RenderContext<HomePage, C>| async {
            ctx.add_title("Hashira | Counter");
            ctx.add_metadata(Metadata::new().description("A counter made with hashira actix-web"));

            let res = ctx.render().await;
            Ok(res)
        })
        .page(
            "/hello/:name",
            |mut ctx: RenderContext<HelloPage, C>| async {
                ctx.add_title("Hashira | Hello");
                ctx.add_metadata(Metadata::new().description("A hashira greeter"));

                let name = ctx.params().find("name").unwrap().to_owned();

                if !name.starts_with("f") {
                    return ctx.not_found();
                }

                let res = ctx.render_with_props(HelloPageProps { name }).await;
                Ok(res)
            },
        )
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