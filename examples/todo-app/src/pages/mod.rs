mod create_todo;
mod list_todos;
mod update_todo;
mod view_todo;

use hashira::app::AppNested;

use crate::App;

pub use self::{create_todo::*, list_todos::*, update_todo::*, view_todo::*};

pub fn todos() -> AppNested<App> {
    hashira::app::nested()
        .page::<ListTodosPage>()
        .page::<ListTodosPage>()
        .page::<UpdateTodoPage>()
        .page::<ViewTodoPage>()
}
