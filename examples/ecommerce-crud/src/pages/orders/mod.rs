use hashira::{app::AppNested, page_component};
use crate::components::Text;
use crate::App;

#[page_component]
pub fn OrdersPage() -> yew::Html {
    yew::html! {
        <Text>
            {"Orders Page"}
        </Text>
    }
}

pub fn orders() -> AppNested<App> {
    hashira::app::nested().page("/", |ctx| async move {
        let res = ctx.render::<OrdersPage, _>().await;
        Ok(res)
    })
}
