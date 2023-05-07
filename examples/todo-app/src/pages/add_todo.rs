use crate::{
    models::{CreateTodo, Todo},
    App,
};
use hashira::{
    action,
    actions::use_action_with_callback,
    app::RenderContext,
    components::Form,
    page_component,
    utils::{redirect_to, show_alert},
    web::Inject,
    web::{Json, Response},
};
use yew::classes;

#[action("/api/todos/create")]
#[cfg(feature = "client")]
#[allow(dead_code)]
pub async fn CreateTodoAction() -> hashira::Result<Json<Todo>> {
    unreachable!()
}

#[action("/api/todos/create")]
#[cfg(not(feature = "client"))]
pub async fn CreateTodoAction(
    form: hashira::web::Form<CreateTodo>,
    Inject(pool): Inject<sqlx::SqlitePool>,
) -> hashira::Result<Json<Todo>> {
    let CreateTodo { title, description } = form.into_inner();
    let mut conn = pool.acquire().await?;

    let inserted_id = sqlx::query!(
        "INSERT INTO todos(title, description) VALUES (?1, ?2)",
        title,
        description
    )
    .execute(&mut conn)
    .await?
    .last_insert_rowid();

    let inserted = sqlx::query_as::<_, Todo>("SELECT * FROM todos WHERE id = ?1")
        .bind(inserted_id)
        .fetch_one(&mut conn)
        .await?;

    Ok(Json(inserted))
}

async fn render(mut ctx: RenderContext) -> hashira::Result<Response> {
    ctx.title("Todo App | Add");
    let res = ctx.render::<AddTodoPage, App>().await;
    Ok(res)
}

#[page_component("/add", render = "render")]
pub fn AddTodoPage() -> yew::Html {
    let action = use_action_with_callback(|ret| match &*ret {
        Ok(_) => redirect_to("/"),
        Err(err) => show_alert(format!("failed to add: {err}")),
    });

    let loading_class = if action.is_loading() {
        "animate-pulse"
    } else {
        ""
    };

    yew::html! {
        <div class="mt-10 w-11/12 md:w-2/3 lg:w-[700px] mx-auto">
            <Form<CreateTodoAction> action={action.clone()} class={classes!("border", "rounded", "p-4", loading_class)}>
                <div class="mb-4">
                    <label class="block text-gray-700 font-bold mb-2" for="title">
                    {"Title"}
                    <span class="text-red-500">{"*"}</span>
                    </label>
                    <input class="appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
                        id="title"
                        name="title"
                        type="text"
                        required={true}
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
                        placeholder="Enter description">
                    </textarea>
                </div>
                <div class="flex flex-row gap-4 justify-end">
                    <button disabled={action.is_loading()} class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded focus:outline-none focus:shadow-outline"
                        type={"submit"}
                        >
                        {"Create"}
                    </button>
                    <a href="/" class="bg-red-500 hover:bg-red-700 text-white font-bold py-2 px-4 rounded focus:outline-none focus:shadow-outline"
                        type={"submit"}>
                        {"Cancel"}
                    </a>
                </div>
            </Form<CreateTodoAction>>
        </div>
    }
}
