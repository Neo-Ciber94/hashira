use crate::{
    models::{DeleteTodo, Todo, ToggleTodo},
    App,
};
use hashira::{
    action,
    actions::{use_action, use_action_with_callback},
    app::RenderContext,
    components::Form,
    page_component,
    server::Metadata,
    utils::show_alert,
    web::{Inject, Response},
};
use serde::{Deserialize, Serialize};
use yew::{function_component, use_state, Callback, Properties};

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
    let todos = use_state(|| props.todos.clone());

    let delete_action = use_action_with_callback::<DeleteTodoAction, _, _>(|ret| {
        if let Err(err) = &*ret {
            show_alert(format!("failed to add: {err}"));
        }
    });

    let on_delete = {
        let todos = todos.clone();
        let delete_action = delete_action.clone();
        Callback::from(move |id: i64| {
            delete_action
                .send(hashira::web::Form(DeleteTodo { id }))
                .unwrap();
            let new_todos = todos.iter().filter(|x| x.id != id).cloned().collect();
            todos.set(new_todos);
        })
    };

    yew::html! {
        <>
            <div class="flex flex-col gap-2 p-4 items-center w-full md:w-[60vw] mx-auto">
                <div class="flex flex-row justify-end w-full">
                    <a title="Add Todo" href="/todos/add" class="rounded text-white py-2 px-4 my-2 bg-blue-500">{"Add Todo"}</a>
                </div>

                <>
                    {for todos.iter().map(|todo| {
                        let on_delete = on_delete.clone();
                        let todo = todo.clone();
                        yew::html_nested! {
                            <TodoItem todo={todo.clone()} on_delete={move |_| on_delete.emit(todo.id)} />
                        }
                    })}
                </>
            </div>
        </>
    }
}

#[action("/api/todos/toggle")]
#[cfg(feature = "client")]
#[allow(dead_code)]
pub async fn ToggleDoneAction() -> hashira::Result<()> {
    unreachable!()
}

#[action("/api/todos/toggle")]
#[cfg(not(feature = "client"))]
pub async fn ToggleDoneAction(
    form: hashira::web::Form<ToggleTodo>,
    Inject(pool): Inject<sqlx::SqlitePool>,
) -> hashira::Result<()> {
    use axum::http::StatusCode;
    use hashira::error::ResponseError;

    let ToggleTodo { id } = form.into_inner();
    let mut conn = pool.acquire().await?;

    let todo = sqlx::query_as::<_, Todo>("SELECT * FROM todos WHERE id = ?1")
        .bind(id)
        .fetch_optional(&mut conn)
        .await?;

    let Some(todo) = todo else {
        return Err(ResponseError::from(StatusCode::NOT_FOUND).into());
    };

    let done = !todo.done;
    sqlx::query!(
        "UPDATE todos
        SET done = ?1 
        WHERE id = ?2",
        done,
        id
    )
    .execute(&mut conn)
    .await?;

    Ok(())
}

#[action("/api/todos/delete")]
#[cfg(feature = "client")]
pub async fn DeleteTodoAction() -> hashira::Result<()> {
    unreachable!()
}

#[action("/api/todos/delete")]
#[cfg(not(feature = "client"))]
pub async fn DeleteTodoAction(
    form: hashira::web::Form<DeleteTodo>,
    Inject(pool): Inject<sqlx::SqlitePool>,
) -> hashira::Result<()> {
    let DeleteTodo { id } = form.into_inner();
    sqlx::query!("DELETE FROM todos WHERE id = ?1", id)
        .execute(&pool)
        .await?;

    Ok(())
}

#[derive(Debug, PartialEq, Properties)]
struct TodoItemProps {
    todo: Todo,
    on_delete: Callback<()>,
}

#[function_component]
fn TodoItem(props: &TodoItemProps) -> yew::Html {
    let toggle_action = use_action::<ToggleDoneAction, _>();
    let checked = use_state(|| props.todo.done);
    let id = props.todo.id;
    let on_delete = props.on_delete.clone();
    let toggle = {
        let toggle_action = toggle_action.clone();
        let checked = checked.clone();
        move |_| {
            checked.set(!*checked);

            toggle_action
                .send(hashira::web::Form(ToggleTodo { id }))
                .unwrap();
        }
    };

    yew::html! {
        <div class="border rounded p-4 shadow w-full">
            <div class="flex items-center justify-between">
                <h2 class="text-lg font-medium">{format!("{}", props.todo.title)}</h2>
                <label class="flex items-center">
                <span class="text-gray-600 mr-2">{"Done?"}</span>
                <input type="checkbox" class="form-checkbox" onchange={toggle} checked={*checked}/>
                </label>
            </div>
            if let Some(description) = &props.todo.description {
                <p class="text-gray-600 mt-2">{format!("{description}")}</p>
            }

            <div class="flex flex-row gap-2 text-sm justify-start font-semibold monospace mt-2">
                <a class="text-green-500 hover:text-green-700 p-2 rounded-full hover:bg-gray-300/20 block"
                    href={format!("/todos/edit/{}", props.todo.id)}>{"Edit"}
                </a>
                <button class="text-red-500 hover:text-red-700 p-2 rounded-full hover:bg-gray-300/20 block"
                    onclick={move |_| on_delete.emit(())}
                >{"Delete"}
                </button>
            </div>
        </div>
    }
}
