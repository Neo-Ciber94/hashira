use yew::{function_component, html::ChildrenProps};


// Default app component.
#[function_component]
pub fn App(props: &ChildrenProps) -> yew::Html {
    yew::html! {
        <>{for props.children.iter()}</>
    }
}