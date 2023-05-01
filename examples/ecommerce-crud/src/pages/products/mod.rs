use hashira::{app::AppNested, page_component};

use crate::App;

#[page_component]
pub fn ProductsPage() -> yew::Html {
    yew::html! {
        "Products Page"
    }
}

pub fn products() -> AppNested<App> {
    hashira::app::nested().page("/", |ctx| async move {
        let res = ctx.render::<ProductsPage, _>().await;
        Ok(res)
    })
}
