use hashira::page_component;

// pub fn GetText(ctx: RenderContext) -> Vec<String> {
//     todo!()
// }

#[page_component]
//#[hashira::loader(GetText)]
pub fn ListTodosPage() -> yew::Html {
    yew::html! {
        "List Todos Page"
    }
}
