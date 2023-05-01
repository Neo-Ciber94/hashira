use yew::{function_component, html::ChildrenProps};

#[function_component]
pub fn Text(props: &ChildrenProps) -> yew::Html {
    yew::html! {
        <div class="text-white">
            {for props.children.iter()}
        </div>
    }
}