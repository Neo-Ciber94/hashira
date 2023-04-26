use crate::components::Counter;
use hashira::page_component;
use serde::{Deserialize, Serialize};
use yew::Properties;

#[derive(PartialEq, Clone, Properties, Serialize, Deserialize)]
pub struct CounterPageProps {
    #[prop_or_default]
    pub counter_start: i32,
}

#[page_component]
pub fn CounterPage(props: &CounterPageProps) -> yew::Html {
    yew::html! {
        <div class="container">
            <Counter value={props.counter_start}/>
        </div>
    }
}
