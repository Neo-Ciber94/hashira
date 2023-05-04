use hashira::{app::RenderContext, error::Error, page_component, server::Metadata, web::Response};

use crate::App;

async fn render(mut ctx: RenderContext) -> Result<Response, Error> {
    ctx.metadata(Metadata::new().description("A Hashira sample app"));
    let res = ctx.render::<HomePage, App>().await;
    Ok(res)
}

#[page_component("/", render = "render")]
pub fn HomePage() -> yew::Html {
    yew::html! {
        <div class="container">
            <div class="logo-container">
                <span class="hashira" title="Hashira">{"Hashira"}</span>
            </div>
        </div>
    }
}
