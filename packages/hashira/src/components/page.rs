use serde::{Deserialize, Serialize};
use yew::{function_component, html::ChildrenProps, BaseComponent, Html, Properties};

use crate::server::client_router::ClientRouter;

pub struct RenderFn(Box<dyn Fn() -> Html + Send + Sync>);

impl RenderFn {
    pub fn render(&self) -> Html {
        (self.0)()
    }
}

impl Default for RenderFn {
    fn default() -> Self {
        RenderFn::new(|| Html::default())
    }
}

impl RenderFn {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn() -> Html + Send + Sync + 'static,
    {
        RenderFn(Box::new(f))
    }
}

impl PartialEq for RenderFn {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(&self.0, &other.0)
    }
}

#[derive(PartialEq, Properties)]
pub struct PageProps {
    pub path: String,
    pub props_json: serde_json::Value,
    pub client_router: ClientRouter,
}

#[function_component]
pub fn Page<ROOT>(props: &PageProps) -> Html
where
    ROOT: BaseComponent<Properties = ChildrenProps>,
{
    let path = props.path.as_str();
    let router = &props.client_router;
    match router.recognize(path) {
        Ok(mtch) => {
            let route = mtch.handler();
            let props = props.props_json.clone();

            yew::html! {
                <ROOT>
                    {route.render(props)}
                </ROOT>
            }
        }
        // TODO: Add custom error pages
        Err(path) => {
            yew::html! {
              <>
                <h1>{"404 | Not Found"}</h1>
                <p>{format!("Unable to find: {path}")}</p>
              </>
            }
        }
    }
}

/// Represents the data of the current page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageData {
    // The path of the component.
    pub path: String,

    // Component being rendered, (remove?)
    pub component_name: String,

    // Properties of the current component.
    pub props: serde_json::Value,
}
