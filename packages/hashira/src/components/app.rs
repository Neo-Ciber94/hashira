use serde::{Deserialize, Serialize};
use yew::{function_component, Html, Properties};

pub struct RenderFn(Box<dyn Fn() -> Html + Send>);

impl Default for RenderFn {
    fn default() -> Self {
        RenderFn::new(|| Html::default())
    }
}

impl RenderFn {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn() -> Html + Send + 'static,
    {
        RenderFn(Box::new(f))
    }
}

impl PartialEq for RenderFn {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(&self.0, &other.0)
    }
}

#[derive(PartialEq, Default, Properties)]
pub struct AppPageProps {
    pub render: RenderFn,
}

#[function_component]
pub fn AppPage(props: &AppPageProps) -> Html {
    let render = &props.render;
    (render.0)()
}

/// Represents the data of the current page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageData {
    pub component_name: String,
    pub props: serde_json::Value,
}
