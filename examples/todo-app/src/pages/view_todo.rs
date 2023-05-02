use hashira::page_component;

#[page_component("/:id")]
pub fn ViewTodoPage() -> yew::Html {
    yew::html! {
        "View Todo Page"
    }
}
