use hashira::page_component;

#[page_component("/add")]
pub fn CreateTodoPage() -> yew::Html {
    yew::html! {
        "Create Todo Page"
    }
}
