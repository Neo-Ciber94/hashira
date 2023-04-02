use hashira::{app::{RenderContext, AppService, Metadata, App as HashiraApp}, web::{Response, ResponseExt}};
use serde::{Deserialize, Serialize};
use yew::{html::ChildrenProps, use_state, Properties, BaseComponent};

#[yew::function_component]
pub(crate) fn App(props: &ChildrenProps) -> yew::Html {
    yew::html! {
       <>
        <header>
            <nav>
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
        <h1>{format!("Hello {}!", props.name)}</h1>
    }
}


// Setup all the components
pub fn hashira<C>() -> AppService<C>
    where
        C: BaseComponent<Properties = ChildrenProps>,
    {
        HashiraApp::<C>::new()
            //.app_data(...)
            .page("/", |mut ctx: RenderContext<HomePage, C>| async {
                ctx.add_metadata(
                    Metadata::new()
                        .viewport("width=device-width, initial-scale=1.0")
                        .title("Hashira Sample App | Counter")
                        .description("A counter made with hashira actix-web"),
                );
    
                let html = ctx.render().await;
                Response::html(html)
            })
            .page(
                "/hello/:name",
                |mut ctx: RenderContext<HelloPage, C>| async {
                    let name = ctx.params().find("name").unwrap().to_owned();
                    ctx.add_metadata(
                        Metadata::new()
                            .viewport("width=device-width, initial-scale=1.0")
                            .title("Hashira Sample App | Hello")
                            .description("A hashira greeter"),
                    );
    
                    let html = ctx.render_with_props(HelloPageProps { name }).await;
    
                    Response::html(html)
                },
            )
            .build()
    }
    