use hashira::{app::RenderContext, error::BoxError, page_component, server::Metadata, web::Response};
use yew::{function_component, html::ChildrenProps};

#[function_component]
pub fn App(props: &ChildrenProps) -> yew::Html {
    yew::html! {
        <>{for props.children.iter()}</>
    }
}

async fn render(mut ctx: RenderContext) -> Result<Response, BoxError> {
    ctx.metadata(Metadata::new().description("Hashira | Hello"));
    let res = ctx.render::<HelloPage, App>().await;
    Ok(res)
}

#[page_component("/", render = "render")]
pub fn HelloPage() -> yew::Html {
    yew::html! {
        <h1>{"Hello World!"}</h1>
    }
}