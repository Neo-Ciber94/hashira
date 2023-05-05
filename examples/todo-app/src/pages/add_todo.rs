use std::future::ready;

use hashira::{
    app::{hooks::use_action, Action, RenderContext},
    page_component,
    web::{Json, Response},
};

use crate::App;

pub struct CreateTodoAction;
impl Action for CreateTodoAction {
    type Output = String;

    fn route() -> &'static str {
        "/api/todo/create"
    }

    fn call(
        ctx: hashira::app::RequestContext,
    ) -> hashira::types::BoxFuture<hashira::Result<Response<Self::Output>>> {
        let fut = hashira::app::call_action(ctx, create_todo_action);
        Box::pin(fut)
    }
}

async fn create_todo_action() -> hashira::Result<Response<String>> {
    let res = Response::new("Hello World!".to_owned());
    Ok(res)
}

async fn render(mut ctx: RenderContext) -> hashira::Result<Response> {
    ctx.title("Todo App | Add");
    let res = ctx.render::<AddTodoPage, App>().await;
    Ok(res)
}

#[page_component("/add", render = "render")]
pub fn AddTodoPage() -> yew::Html {
    let action = use_action::<CreateTodoAction, _>();

    let on_submit = move |e: yew::html::onsubmit::Event| {
        e.prevent_default();
        action.send(Json(String::from("hello"))).unwrap();
    };

    yew::html! {
        <div class="mt-10">
            <form class="border rounded p-4" method="POST" action={CreateTodoAction::route()} onsubmit={on_submit}>
                <div class="mb-4">
                    <label class="block text-gray-700 font-bold mb-2" for="title">
                    {"Title"}
                    </label>
                    <input class="appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
                        id="title"
                        name="title"
                        type="text"
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
                    <button class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded focus:outline-none focus:shadow-outline"
                        type={"submit"}
                        >
                        {"Create Todo"}
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
