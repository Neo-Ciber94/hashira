use std::sync::Arc;

use http::StatusCode;
use serde::{Deserialize, Serialize};
use yew::{function_component, html::ChildrenProps, BaseComponent, Html, Properties};

use crate::{
    app::{client_router::ClientRouter, error_router::ClientErrorRouter},
    components::error::NotFoundPage,
};

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
    pub client_error_router: Arc<ClientErrorRouter>,
}

#[function_component]
pub fn Page<ROOT>(props: &PageProps) -> Html
where
    ROOT: BaseComponent<Properties = ChildrenProps>,
{
    let path = props.path.as_str();
    let router = &props.client_router;
    let error_router = &props.client_error_router;

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
        Err(_) => match error_router.recognize_error(&StatusCode::NOT_FOUND) {
            Some(comp) => {
                let props = props.props_json.clone();
                yew::html! {
                    {comp.render_with_props(props)}
                }
            }
            None => {
                log::error!("not error page was registered for 404 errors");
                yew::html! {
                    <NotFoundPage />
                }
            }
        },
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
