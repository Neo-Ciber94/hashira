use std::ops::Deref;

use crate::{
    models::{Todo, UpdateTodo},
    App,
};
use hashira::web::Inject;
use hashira::{
    action,
    actions::use_action_with_callback,
    components::Form,
    utils::{redirect_to, show_alert},
    web::Json,
};
use hashira::{app::RenderContext, page_component, web::Response};
use serde::{Deserialize, Serialize};
use yew::{classes, use_state, Properties, TargetCast};
use web_sys::{HtmlInputElement, HtmlTextAreaElement};

#[action("/api/todos/update")]
#[cfg(feature = "client")]
#[allow(dead_code)]
pub async fn EditTodoAction() -> hashira::Result<Json<Todo>> {
    unreachable!()
}

#[action("/api/todos/update")]
#[cfg(not(feature = "client"))]
pub async fn EditTodoAction(
    form: hashira::web::Form<UpdateTodo>,
    Inject(pool): Inject<sqlx::SqlitePool>,
) -> hashira::Result<Json<Todo>> {
    let UpdateTodo {
        id,
        title,
        description,
    } = form.into_inner();
    let mut conn = pool.acquire().await?;

    sqlx::query!(
        "UPDATE todos
        SET title = ?1, description = ?2 
        WHERE id = ?3",
        title,
        description,
        id
    )
    .execute(&mut conn)
    .await?;

    let todo = sqlx::query_as::<_, Todo>("SELECT * FROM todos WHERE id = ?1")
        .bind(id)
        .fetch_one(&mut conn)
        .await?;

    Ok(Json(todo))
}

#[cfg(feature = "client")]
async fn render(mut _ctx: RenderContext) -> hashira::Result<Response> {
    unreachable!()
}

#[cfg(not(feature = "client"))]
async fn render(
    mut ctx: RenderContext,
    Inject(pool): Inject<sqlx::SqlitePool>,
) -> hashira::Result<Response> {
    use hashira::{error::ResponseError, web::status::StatusCode};
    use std::str::FromStr;

    ctx.title("Todo App | Edit");

    let id = ctx
        .params()
        .get("id")
        .and_then(|s| i64::from_str(s).ok())
        .ok_or_else(|| ResponseError::from(StatusCode::UNPROCESSABLE_ENTITY))?;

    let todo = sqlx::query_as::<_, Todo>("SELECT * FROM todos WHERE id = ?1")
        .bind(id)
        .fetch_optional(&pool)
        .await?;

    let Some(todo) = todo else {
        return ctx.not_found();
    };

    let res = ctx
        .render_with_props::<EditTodoPage, App>(EditTodoPageProps { todo })
        .await;
    Ok(res)
}

#[derive(Debug, PartialEq, Properties, Serialize, Deserialize)]
pub struct EditTodoPageProps {
    todo: Todo,
}

#[page_component("/edit/:id", render = "render")]
pub fn EditTodoPage(props: &EditTodoPageProps) -> yew::Html {
    let action = use_action_with_callback(|ret| match &*ret {
        Ok(_) => {
            redirect_to("/");
        }
        Err(err) => show_alert(format!("failed to update: {err}")),
    });

    let loading_class = if action.is_loading() {
        "animation-pulse"
    } else {
        ""
    };

    let title = use_state(|| props.todo.title.clone());
    let description = use_state(|| props.todo.description.clone());

    yew::html! {
        <div class="mt-10 w-11/12 md:w-2/3 lg:w-[700px] mx-auto">
            <Form<EditTodoAction> action={action.clone()} class={classes!("border", "rounded", "p-4", loading_class)}>
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
                        placeholder="Enter title"
                        value={title.deref().clone()}
                        onchange={{
                            let title = title.clone();
                            move |e: yew::html::onchange::Event| {
                                let el : HtmlInputElement  = e.target_dyn_into().unwrap();
                                title.set(el.value());
                            }
                        }}
                         />
                </div>
                <div class="mb-4">
                    <label class="block text-gray-700 font-bold mb-2" for="description">
                    {"Description"}
                    </label>
                    <textarea class="appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
                        id="description"
                        rows={4}
                        name="description"
                        placeholder="Enter description"
                        value={description.deref().clone()}
                        onchange={{
                            let description = description.clone();
                            move |e: yew::html::onchange::Event| {
                                let el : HtmlTextAreaElement  = e.target_dyn_into().unwrap();
                                description.set(Some(el.value()));
                            }
                        }}
                        >
                    </textarea>
                </div>
                <div class="flex flex-row gap-4 justify-end">
                    <button disabled={action.is_loading()} class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded focus:outline-none focus:shadow-outline"
                        type={"submit"}>
                        {"Update"}
                    </button>
                    <a href="/" class="bg-red-500 hover:bg-red-700 text-white font-bold py-2 px-4 rounded focus:outline-none focus:shadow-outline"
                        type={"submit"}>
                        {"Cancel"}
                    </a>
                </div>
            </Form<EditTodoAction>>
        </div>
    }
}
