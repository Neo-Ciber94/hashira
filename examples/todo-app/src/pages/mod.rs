mod add_todo;
mod edit_todo;
mod list_todos;

use crate::App;
use hashira::app::AppNested;

pub use self::{add_todo::*, edit_todo::*, list_todos::*};

pub fn todos() -> AppNested<App> {
    hashira::app::nested()
        .action::<CreateTodoAction>()
        .action::<EditTodoAction>()
        .action::<DeleteTodoAction>()
        .action::<ToggleDoneAction>()
        .page::<AddTodoPage>()
        .page::<EditTodoPage>()
        .page::<ListTodosPage>()
}
