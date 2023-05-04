use crate::{components::Counter, App};
use hashira::{app::RenderContext, error::Error, page_component, server::Metadata, web::Response};
use serde::{Deserialize, Serialize};
use yew::Properties;

#[derive(PartialEq, Clone, Properties, Serialize, Deserialize)]
pub struct CounterPageProps {
    #[prop_or_default]
    pub counter_start: i32,
}

async fn render(mut ctx: RenderContext, uri: hashira::web::uri::Uri) -> Result<Response, Error> {
    println!("{}", uri);
    ctx.title("Hashira | Counter");
    ctx.metadata(Metadata::new().description("A Hashira sample counter"));
    let props = yew::props! { CounterPageProps {} };
    let res = ctx.render_with_props::<CounterPage, App>(props).await;
    Ok(res)
}

#[page_component("/counter", render = "render")]
pub fn CounterPage(props: &CounterPageProps) -> yew::Html {
    yew::html! {
        <div class="container">
            <Counter value={props.counter_start}/>
        </div>
    }
}
