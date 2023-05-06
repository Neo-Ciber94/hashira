use hashira::{
    app::RenderContext,
    page_component,
    server::Metadata,
    web::{Inject, Response},
};
use serde::{Deserialize, Serialize};

use yew::{function_component, Properties};

use crate::{models::Todo, App};

#[cfg(feature = "client")]
async fn render(_ctx: RenderContext) -> hashira::Result<Response> {
    unreachable!()
}

#[cfg(not(feature = "client"))]
async fn render(
    mut ctx: RenderContext,
    Inject(pool): Inject<sqlx::SqlitePool>,
) -> hashira::Result<Response> {
    ctx.title("Todo App | List");
    ctx.metadata(Metadata::new().description("List of all the todos"));

    let todos = sqlx::query_as::<_, Todo>("SELECT * FROM todos ORDER BY id desc")
        .fetch_all(&pool)
        .await?;

    let res = ctx
        .render_with_props::<ListTodosPage, App>(ListTodosPageProps { todos })
        .await;

    Ok(res)
}

#[derive(Debug, Properties, PartialEq, Serialize, Deserialize)]
pub struct ListTodosPageProps {
    todos: Vec<Todo>,
}

#[page_component("/", render = "render")]
pub fn ListTodosPage(props: &ListTodosPageProps) -> yew::Html {
    yew::html! {
        <>
            <div class="flex flex-col gap-2 p-4 items-center w-full md:w-[60vw] mx-auto">
                <div class="flex flex-row justify-end w-full">
                    <a title="Add Todo" href="/todos/add" class="rounded text-white py-2 px-4 my-2 bg-blue-500">{"Add Todo"}</a>
                </div>

                <>
                    {for props.todos.iter().map(|todo| {
                        yew::html_nested! {
                            <TodoItem todo={todo.clone()}/>
                        }
                    })}
                </>
            </div>
        </>
    }
}

#[derive(Debug, PartialEq, Properties, Serialize, Deserialize)]
struct TodoItemProps {
    todo: Todo,
}

#[function_component]
fn TodoItem(props: &TodoItemProps) -> yew::Html {
    let delete = {
        let id = props.todo.id.clone();
        move |_| {
            log::info!("Delete: {id}");
        }
    };

    yew::html! {
        <div class="border rounded p-4 shadow w-full">
            <div class="flex items-center justify-between">
                <h2 class="text-lg font-medium">{format!("{}", props.todo.title)}</h2>
                <label class="flex items-center">
                <span class="text-gray-600 mr-2">{"Done?"}</span>
                <input type="checkbox" class="form-checkbox" />
                </label>
            </div>
            if let Some(description) = &props.todo.description {
                <p class="text-gray-600 mt-2">{format!("{description}")}</p>
            }

            <div class="flex flex-row gap-2 text-sm justify-start font-semibold monospace mt-2">
                <a class="text-green-500 hover:text-green-700 p-2 rounded-full hover:bg-gray-300/20 block"
                    href={format!("/todos/edit/{}", props.todo.id)}>{"Edit"}</a>
                <button class="text-red-500 hover:text-red-700 p-2 rounded-full hover:bg-gray-300/20 block"
                    onclick={delete}>{"Delete"}</button>
            </div>
        </div>
    }
}
