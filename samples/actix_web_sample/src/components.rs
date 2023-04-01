use serde::{Deserialize, Serialize};
use yew::{html::ChildrenProps, use_state, Properties};

#[yew::function_component]
pub(crate) fn App(props: &ChildrenProps) -> yew::Html {
    yew::html! {
       <>
        <header>
            <nav>
                <a href="/">{"Home"}</a>
                <a href="/hello/freddy">{"Hello"}</a>
            </nav>
        </header>
        <>{for props.children.iter()}</>
       </>
    }
}

#[yew::function_component]
pub fn HomePage() -> yew::Html {
    let counter = use_state(|| 0);
    let increment = {
        let counter = counter.clone();
        move |_| {
            let value = *counter + 1;
            counter.set(value);
        }
    };

    let decrement = {
        let counter = counter.clone();
        move |_| {
            let value = *counter - 1;
            counter.set(value);
        }
    };

    yew::html! {
        <div>
            <button onclick={decrement}>{ "-1" }</button>
            <p>{ *counter }</p>
            <button onclick={increment}>{ "+1" }</button>
        </div>
    }
}

#[derive(PartialEq, Serialize, Deserialize, Properties, Clone)]
pub struct HelloPageProps {
    pub name: String,
}

#[yew::function_component]
pub fn HelloPage(props: &HelloPageProps) -> yew::Html {
    yew::html! {
        <h1>{format!("Hello {}!", props.name)}</h1>
    }
}
