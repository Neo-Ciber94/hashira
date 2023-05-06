use std::time::Duration;

use hashira::{
    actions::use_action, app::RenderContext, components::Form, page_component, web::Response,
};

use crate::App;

#[hashira::action("/api/todo/create")]
pub async fn CreateTodoAction() -> String {
    #[cfg(not(target_arch = "wasm32"))]
    {
        tokio::time::sleep(Duration::from_secs(2)).await;
        //let res = Response::new("Hello World!".to_owned());
        return String::from("hello world!");
        //return Ok(res);
    }

    #[cfg(target_arch = "wasm32")]
    unreachable!()
}

async fn render(mut ctx: RenderContext) -> hashira::Result<Response> {
    ctx.title("Todo App | Add");
    let res = ctx.render::<AddTodoPage, App>().await;
    Ok(res)
}

#[page_component("/add", render = "render")]
pub fn AddTodoPage() -> yew::Html {
    let action = use_action();

    yew::html! {
        <div class="mt-10">
            if action.is_loading() {
                <div>{"Loading..."}</div>
            }
            <Form<CreateTodoAction> action={action.clone()} class="border rounded p-4">
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
            </Form<CreateTodoAction>>
        </div>
    }
}
