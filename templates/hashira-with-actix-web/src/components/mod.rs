use yew::{function_component, use_state, Properties};

#[derive(Properties, PartialEq)]
pub struct CounterProps {
    #[prop_or_default]
    pub value: i32,
}

#[function_component]
pub fn Counter(props: &CounterProps) -> yew::Html {
    let counter = use_state(|| props.value);
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
        <div class="counter">
            <button onclick={decrement}>{"-"}</button>
            <span>{ *counter }</span>
            <button onclick={increment}>{"+"}</button>
        </div>
    }
}
