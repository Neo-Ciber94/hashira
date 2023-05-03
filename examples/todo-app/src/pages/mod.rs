mod add_todo;
mod list_todos;
mod edit_todo;

use hashira::app::AppNested;

use crate::App;

pub use self::{add_todo::*, list_todos::*, edit_todo::*};

pub fn todos() -> AppNested<App> {
    hashira::app::nested()
        .page::<AddTodoPage>()
        .page::<EditTodoPage>()
        .page::<ListTodosPage>()
}
