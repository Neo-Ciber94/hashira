use hashira::{app::AppNested, page_component};
use crate::App;


#[page_component]
pub fn AuthPage() -> yew::Html {
    yew::html! {
        "Auth Page"
    }
}

pub fn auth() -> AppNested<App> {
    hashira::app::nested().page("/", |ctx| async move {
        let res = ctx.render::<AuthPage, _>().await;
        Ok(res)
    })
}
