use hashira::{app::AppNested, page_component};

use crate::App;

#[page_component]
pub fn UsersPage() -> yew::Html {
    yew::html! {
        "Users Page"
    }
}

pub fn users() -> AppNested<App> {
    hashira::app::nested().page("/", |ctx| async move {
        let res = ctx.render::<UsersPage, _>().await;
        Ok(res)
    })
}
