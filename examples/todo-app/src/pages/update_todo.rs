use hashira::page_component;

#[page_component("/edit/:id")]
pub fn UpdateTodoPage() -> yew::Html {
    yew::html! {
        "Update Todo Page"
    }
}
