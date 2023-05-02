use std::str::FromStr;

use hashira::web::status::StatusCode;
use hashira::{app::RenderContext, error::ResponseError, page_component, web::Response};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use yew::Properties;

use crate::{database::get_todos, models::Todo, App};

async fn render(mut ctx: RenderContext) -> hashira::Result<Response> {
    ctx.title("Todo App | Edit");

    let id = ctx
        .params()
        .get("id")
        .and_then(|s| Uuid::from_str(s).ok())
        .ok_or_else(|| ResponseError::from(StatusCode::UNPROCESSABLE_ENTITY))?;

    let todos = get_todos().await;

    let todo = todos
        .iter()
        .find(|x| x.id == id)
        .cloned()
        .ok_or_else(|| ResponseError::from(StatusCode::NOT_FOUND))?;

    let res = ctx
        .render_with_props::<EditTodoPage, App>(EditTodoPageProps { todo })
        .await;
    Ok(res)
}

#[derive(Debug, PartialEq, Properties, Serialize, Deserialize)]
pub struct EditTodoPageProps {
    todo: Todo,
}

#[page_component("/edit/:id", loader = "render")]
pub fn EditTodoPage(props: &EditTodoPageProps) -> yew::Html {
    yew::html! {
        <div class="mt-10">
            <form class="border rounded p-4" method="POST" action="/api/todos">
                <div class="mb-4">
                    <label class="block text-gray-700 font-bold mb-2" for="id">
                    {"Id"}
                    </label>
                    <input class="appearance-none border rounded w-full py-2 px-3 text-gray-400 bg-gray-200
                        leading-tight focus:outline-none focus:shadow-outline"
                        id="id"
                        name="id"
                        type="text"
                        value={props.todo.id.to_string()}
                        readonly={true}/>
                </div>
                <div class="mb-4">
                    <label class="block text-gray-700 font-bold mb-2" for="title">
                    {"Title"}
                    </label>
                    <input class="appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
                        id="title"
                        name="title"
                        type="text"
                        value={props.todo.title.clone()}
                        placeholder="Enter title" />
                </div>
                <div class="mb-4">
                    <label class="block text-gray-700 font-bold mb-2" for="description">
                    {"Description"}
                    </label>
                    <textarea class="appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
                        id="description"
                        rows={4}
                        name="description"
                        value={props.todo.description.clone().unwrap_or_default()}
                        placeholder="Enter description">
                    </textarea>
                </div>
                <div class="flex flex-row gap-4 justify-end">
                    <button class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded focus:outline-none focus:shadow-outline"
                        type={"submit"}>
                        {"Update Todo"}
                    </button>
                    <a href="/" class="bg-red-500 hover:bg-red-700 text-white font-bold py-2 px-4 rounded focus:outline-none focus:shadow-outline"
                        type={"submit"}>
                        {"Cancel"}
                    </a>
                </div>
            </form>
        </div>
    }
}
