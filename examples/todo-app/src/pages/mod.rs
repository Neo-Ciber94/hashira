mod add_todo;
mod edit_todo;
mod list_todos;

use hashira::app::AppNested;
use crate::App;

pub use self::{add_todo::*, edit_todo::*, list_todos::*};

pub fn todos() -> AppNested<App> {
    hashira::app::nested()
        .action::<CreateTodoAction>()
        .page::<AddTodoPage>()
        .page::<EditTodoPage>()
        .page::<ListTodosPage>()
}
