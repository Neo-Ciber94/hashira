mod create_todo;
mod list_todos;
mod update_todo;
mod view_todo;

use hashira::app::AppNested;

use crate::App;

pub use self::{create_todo::*, list_todos::*, update_todo::*, view_todo::*};

pub fn todos() -> AppNested<App> {
    hashira::app::nested()
        .page("/", |ctx| async {
            let res = ctx.render::<ListTodosPage, _>().await;
            Ok(res)
        })
        .page("/add", |ctx| async {
            let res = ctx.render::<ListTodosPage, _>().await;
            Ok(res)
        })
        .page("/edit/:id", |ctx| async {
            let res = ctx.render::<UpdateTodoPage, _>().await;
            Ok(res)
        })
        .page("/:id", |ctx| async {
            let res = ctx.render::<ViewTodoPage, _>().await;
            Ok(res)
        })
}
