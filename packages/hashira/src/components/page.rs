use crate::context::{ServerContext, ServerContextProvider};
use crate::routing::Params;
use crate::{
    app::{error_router::ErrorRouter, router::PageRouterWrapper},
    components::error::{ErrorPage, NotFoundPage},
};
use http::{StatusCode, Uri};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use yew::Suspense;
use yew::{function_component, html::ChildrenProps, BaseComponent, Html, Properties};
use super::id::PageId;

/// The props for the current page.
#[derive(Clone, PartialEq, Properties)]
pub struct PageProps {
    /// The id of the current page component.
    pub id: PageId,

    /// The path of the request
    pub path: String,

    /// An error that occurred while processing the request
    pub error: Option<PageError>,

    /// The props of the current page as JSON
    pub props_json: serde_json::Value,

    /// The router to render the page
    pub router: PageRouterWrapper,

    /// The router to render the error pages
    pub error_router: Arc<ErrorRouter>,

    /// Provides info about the current request
    pub server_context: ServerContext,
}

#[function_component]
pub fn Page<ROOT>(props: &PageProps) -> Html
where
    ROOT: BaseComponent<Properties = ChildrenProps>,
{
    let props = props.clone();

    yew::html! {
        <ServerContextProvider server_context={props.server_context.clone()}>
            <PageRouter<ROOT> ..props/>
        </ServerContextProvider>
    }
}

#[function_component]
pub fn PageRouter<ROOT>(props: &PageProps) -> Html
where
    ROOT: BaseComponent<Properties = ChildrenProps>,
{
    let router = &props.router;
    let error_router = &props.error_router;

    if let Some(error) = &props.error {
        return match error_router.find_match(&error.status) {
            Some(comp) => {
                let props = props.props_json.clone();
                yew::html! {
                    <Suspense>
                        {comp.render_with_props(props)}
                    </Suspense>
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

    match router.find_by_id(&props.id) {
        Some(route) => {
            let props = props.props_json.clone();

            yew::html! {
                <ROOT>
                    <Suspense>
                        {route.render(props)}
                    </Suspense>
                </ROOT>
            }
        }
        None => match error_router.find_match(&StatusCode::NOT_FOUND) {
            Some(comp) => {
                let props = props.props_json.clone();
                yew::html! {
                    <Suspense>
                        {comp.render_with_props(props)}
                    </Suspense>
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
    /// The id of the component of this page.
    pub id: PageId,

    /// The uri of the current page.
    #[serde(with = "crate::web::serde::uri")]
    pub uri: Uri,

    /// An error that ocurred in the route.
    pub error: Option<PageError>,

    /// Properties of the current page.
    pub props: serde_json::Value,

    /// Params of the page, if any.
    pub params: Params,
}
