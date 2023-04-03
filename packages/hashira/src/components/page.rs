use std::sync::Arc;

use http::StatusCode;
use serde::{Deserialize, Serialize};
use yew::{function_component, html::ChildrenProps, BaseComponent, Html, Properties};

use crate::{
    app::{client_router::ClientRouter, error_router::ClientErrorRouter},
    components::error::{ErrorPage, NotFoundPage},
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
    pub error: Option<PageError>,
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

    if let Some(error) = &props.error {
        return match error_router.recognize_error(&error.status) {
            Some(comp) => {
                let props = props.props_json.clone();
                yew::html! {
                    {comp.render_with_props(props)}
                }
            }
            None => {
                log::warn!("fallback error page was not registered");
                yew::html! {
                    <ErrorPage status={error.status} message={error.message.clone()} />
                }
            }
        };
    }

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
                log::warn!("404 error page was not registered");
                yew::html! {
                    <NotFoundPage />
                }
            }
        },
    }
}

/// Represents an error that occurred on the server.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageError {
    /// The status code of the error.
    #[serde(with = "crate::web::serde::status_code")]
    pub status: StatusCode,

    /// A message of the error.
    pub message: Option<String>,
}

/// Represents the data of the current page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageData {
    /// The path of the component.
    pub path: String,

    /// An error that ocurred in the route.
    pub error: Option<PageError>,

    /// Component being rendered, (remove?)
    pub component_name: String,

    /// Properties of the current component.
    pub props: serde_json::Value,
}
