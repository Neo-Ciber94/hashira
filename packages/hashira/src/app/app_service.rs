use super::{router::PageRouterWrapper, AppData, RequestContext};
use crate::{
    error::ResponseError,
    routing::{
        ErrorRouter, HandlerKind, Params, ServerErrorRouter, ServerRouter, ServerRouterMatchError,
    },
    web::{IntoResponse, Request, Response},
};
use http::{HeaderMap, StatusCode};
use std::sync::Arc;

pub(crate) struct AppServiceInner {
    pub(crate) server_router: ServerRouter,
    pub(crate) client_router: PageRouterWrapper,
    pub(crate) server_error_router: ServerErrorRouter,
    pub(crate) client_error_router: Arc<ErrorRouter>,
    pub(crate) default_headers: HeaderMap,
    pub(crate) app_data: Arc<AppData>,

    #[cfg(feature = "hooks")]
    pub(crate) hooks: Arc<crate::events::Hooks>,
}

enum ErrorSource {
    Response(Response),
    Error(ResponseError),
}

/// The root service used for handling the `hashira` application.
pub struct AppService(Arc<AppServiceInner>);

impl AppService {
    pub(crate) fn new(inner: Arc<AppServiceInner>) -> Self {
        Self(inner)
    }

    /// Create a context to be used in the request.
    pub fn create_context(
        &self,
        request: Arc<Request>,
        params: Params,
        error: Option<ResponseError>,
    ) -> RequestContext {
        let client_router = self.0.client_router.clone();
        let error_router = self.0.client_error_router.clone();
        let app_data = self.0.app_data.clone();

        RequestContext::new(
            request,
            app_data,
            client_router,
            error_router,
            error,
            params,
        )
    }

    /// Returns the app data of the app.
    pub fn app_data(&self) -> &AppData {
        self.0.app_data.as_ref()
    }

    /// Returns the page router.
    #[cfg_attr(not(feature = "client"), allow(dead_code))]
    pub(crate) fn page_router(&self) -> &PageRouterWrapper {
        &self.0.client_router
    }

    /// Returns the router for handling error pages on the client.
    #[cfg_attr(not(feature = "client"), allow(dead_code))]
    pub(crate) fn error_router(&self) -> &Arc<ErrorRouter> {
        &self.0.client_error_router
    }

    /// Process the incoming request and return the response.
    pub async fn handle(&self, req: Request) -> Response {
        let mut res = self._handle(req).await;

        // Merge the response headers with the default headers
        if !self.0.default_headers.is_empty() {
            let mut headers = self.0.default_headers.clone();
            headers.extend(res.headers().clone());
            *res.headers_mut() = headers;
        }

        res
    }

    async fn _handle(&self, req: Request) -> Response {
        let req = Arc::new(req);

        // Handle the request normally
        #[cfg(not(feature = "hooks"))]
        {
            self.handle_request(req).await
        }

        #[cfg(feature = "hooks")]
        {
            use crate::{events::Next, types::BoxFuture};

            let hooks = &self.0.hooks.on_handle_hooks;

            // Handle the request normally to avoid any extra allocations
            if hooks.is_empty() {
                return self.handle_request(req).await;
            }

            let this = self.clone();
            let next = Box::new(move |req| {
                Box::pin(async move { this.handle_request(req).await }) as BoxFuture<Response>
            }) as Next;

            // We execute the hooks in the order they were added
            let handler = hooks.iter().rev().fold(next, move |cur, next_handler| {
                let next_handler = next_handler.clone_handler();
                Box::new(move |req| Box::pin(async move { next_handler.call(req, cur).await }))
            }) as Next;

            // Handle the request
            handler(req).await
        }
    }

    async fn handle_request(&self, req: Arc<Request>) -> Response {
        // We remove the trailing slash from the path,
        // when adding a path we ensure it cannot end with a slash
        // and should start with a slash

        let mut path = req.uri().path().trim();

        // We trim the trailing slash or should we redirect?
        if path.len() > 1 && path.ends_with('/') {
            path = path.trim_end_matches('/');
        }

        let method = req.method().into();

        match self.0.server_router.at(path, method) {
            Ok(mtch) => {
                let route = mtch.value;
                let params = mtch.params;
                let ctx = self.create_context(req.clone(), params, None);

                let res = route.handler().call(ctx).await;
                let status = res.status();

                // Only component pages render error by default
                let should_render = route
                    .extensions()
                    .get::<HandlerKind>()
                    .map(|kind| kind == &HandlerKind::Page)
                    .unwrap_or_default();

                if status.is_client_error() || status.is_server_error() {
                    return self
                        .handle_error(req, ErrorSource::Response(res), should_render)
                        .await;
                }

                res
            }
            Err(ServerRouterMatchError::MethodMismatch) => {
                let src =
                    ErrorSource::Error(ResponseError::from_status(StatusCode::METHOD_NOT_ALLOWED));
                self.handle_error(req, src, true).await
            }
            Err(_) => {
                // we treat any other error as 404
                let src = ErrorSource::Error(ResponseError::from_status(StatusCode::NOT_FOUND));
                self.handle_error(req, src, true).await
            }
        }
    }

    async fn handle_error(
        &self,
        req: Arc<Request>,
        src: ErrorSource,
        should_render: bool,
    ) -> Response {
        // If the response is marked as not render, skip any error handler and return the response
        if !should_render {
            return match src {
                ErrorSource::Response(res) => res,
                ErrorSource::Error(err) => err.into_response(),
            };
        }

        let err = match src {
            ErrorSource::Response(res) => {
                let status = res.status();

                // We get the message from the error which may be attached to the response
                let message = res
                    .extensions()
                    .get::<ResponseError>()
                    .and_then(|e| e.message())
                    .map(|s| s.to_owned());
                ResponseError::from((status, message))
            }
            ErrorSource::Error(res) => res,
        };

        let status = err.status();
        let mut response = match self.0.server_error_router.find(&status) {
            Some(error_handler) => {
                let params = Params::default();
                let ctx = self.create_context(req, params, Some(err.clone()));

                match error_handler.call(ctx, status).await {
                    Ok(res) => res,
                    Err(err) => match err.downcast::<ResponseError>() {
                        Ok(err) => (*err).into_response(),
                        Err(err) => {
                            (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()
                        }
                    },
                }
            }
            None => err.clone().into_response(),
        };

        // Ensure the status code of the response
        *response.status_mut() = status;

        // Append the error to the response
        response.extensions_mut().insert(err);

        #[cfg(feature = "hooks")]
        {
            let hooks = &self.0.hooks;

            for on_error in hooks.on_server_error_hooks.iter() {
                response = on_error.call(response);
            }
        }

        // Returns the error response
        response
    }
}

impl Clone for AppService {
    fn clone(&self) -> Self {
        AppService(self.0.clone())
    }
}

#[cfg(test)]
mod tests {
    #![allow(dead_code, unused_imports)]
    use std::{str::FromStr, sync::Arc};

    use bytes::Bytes;
    use http::{Method, StatusCode};
    use yew::{function_component, html::ChildrenProps};

    use crate::{
        app::App,
        routing::Route,
        web::{Body, Request, Response},
    };

    use super::AppService;

    #[test]
    #[should_panic]
    fn invalid_route_test_1() {
        let _ = App::<Base>::new().route(Route::get("", noop));
    }

    #[test]
    #[should_panic]
    fn invalid_route_test_2() {
        let _ = App::<Base>::new().route(Route::get("/path/", noop));
    }

    #[tokio::test]
    async fn router_test() {
        let service = App::<Base>::new()
            .route(Route::get("/a", noop))
            .route(Route::post("/b", noop))
            .route(Route::delete("/c", noop))
            .build();

        let res1 = service.handle_request(create_req("/a", Method::GET)).await;
        assert_eq!(res1.status(), StatusCode::OK);

        let res2 = service.handle_request(create_req("/b", Method::POST)).await;
        assert_eq!(res2.status(), StatusCode::OK);

        let res3 = service
            .handle_request(create_req("/c", Method::DELETE))
            .await;
        assert_eq!(res3.status(), StatusCode::OK);

        let res4 = service.handle_request(create_req("/d", Method::GET)).await;
        assert_eq!(res4.status(), StatusCode::NOT_FOUND);

        let res5 = service.handle_request(create_req("/a", Method::POST)).await;
        assert_eq!(res5.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    #[cfg(not(feature = "client"))]
    async fn nested_route_test() {
        use crate::app::nested;

        let service = App::<Base>::new()
            .nest(
                "/vowels",
                nested()
                    .route(Route::post("/a", noop))
                    .route(Route::post("/b", noop))
                    .route(Route::post("/c", noop)),
            )
            .nest(
                "/xyz",
                nested()
                    .route(Route::post("/x", noop))
                    .route(Route::post("/y", noop))
                    .route(Route::post("/z", noop)),
            )
            .build();

        // Test requests to nested routes
        let res1 = service
            .handle_request(create_req("/vowels/a", Method::POST))
            .await;
        assert_eq!(res1.status(), StatusCode::OK);

        let res2 = service
            .handle_request(create_req("/xyz/z", Method::POST))
            .await;
        assert_eq!(res2.status(), StatusCode::OK);

        // Test requests to non-existent nested routes
        let res3 = service
            .handle_request(create_req("/vowels/d", Method::POST))
            .await;
        assert_eq!(res3.status(), StatusCode::NOT_FOUND);

        let res4 = service
            .handle_request(create_req("/xyz/w", Method::POST))
            .await;
        assert_eq!(res4.status(), StatusCode::NOT_FOUND);

        // Test requests to nested routes with invalid methods
        let res5 = service
            .handle_request(create_req("/vowels/b", Method::GET))
            .await;
        assert_eq!(res5.status(), StatusCode::METHOD_NOT_ALLOWED);

        let res6 = service
            .handle_request(create_req("/xyz/x", Method::DELETE))
            .await;
        assert_eq!(res6.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    #[cfg(not(feature = "client"))]
    async fn page_route_test() {
        #[function_component]
        fn CompA() -> yew::Html {
            yew::html! {
                "test - component (a)"
            }
        }

        #[function_component]
        fn CompB() -> yew::Html {
            yew::html! {
                "test - component (b)"
            }
        }

        crate::impl_page_component!(CompA, "/a");
        crate::impl_page_component!(CompB, "/b");

        let service = App::<Base>::new().page::<CompA>().page::<CompB>().build();

        let res1 = send_request_get_text(&service, "/a", "").await;
        assert_eq!(res1.status(), StatusCode::OK);
        assert!(
            res1.body().contains("test - component (a)"),
            "body: {}",
            res1.body()
        );

        let res2 = send_request_get_text(&service, "/b", "").await;
        assert_eq!(res2.status(), StatusCode::OK);
        assert!(
            res2.body().contains("test - component (b)"),
            "body: {}",
            res2.body()
        );
    }

    #[tokio::test]
    #[cfg(not(feature = "client"))]
    async fn error_route_test() {
        use crate::routing::HandlerKind;

        #[function_component]
        fn NotFoundTest() -> yew::Html {
            yew::html! {
                "test - not found"
            }
        }

        #[function_component]
        fn NotAllowedTest() -> yew::Html {
            yew::html! {
                "test - not allowed"
            }
        }

        #[function_component]
        fn ErrorFallbackTest() -> yew::Html {
            yew::html! {
                "test - oh oh"
            }
        }

        #[function_component]
        fn CompA() -> yew::Html {
            yew::html! {
                "test - component (a)"
            }
        }

        crate::impl_page_component!(NotFoundTest);
        crate::impl_page_component!(NotAllowedTest);
        crate::impl_page_component!(ErrorFallbackTest);

        let mut route = Route::get("/throw_error", |bytes: Bytes| async move {
            let status_str = String::from_utf8(bytes.to_vec()).unwrap();
            let status = StatusCode::from_str(&status_str).unwrap();
            status
        });

        // Only pages route return error pages
        route.extensions_mut().insert(HandlerKind::Page);

        let service = App::<Base>::new()
            .error_page::<NotFoundTest>(StatusCode::NOT_FOUND)
            .error_page::<NotAllowedTest>(StatusCode::METHOD_NOT_ALLOWED)
            .error_page_fallback::<ErrorFallbackTest>()
            .route(route)
            .build();

        let res1 = send_request_get_text(&service, "/throw_error", "404").await;
        assert_eq!(res1.status(), StatusCode::NOT_FOUND);
        assert!(
            res1.body().contains("test - not found"),
            "body: {}",
            res1.body()
        );

        let res2 = send_request_get_text(&service, "/throw_error", "405").await;
        assert_eq!(res2.status(), StatusCode::METHOD_NOT_ALLOWED);
        assert!(
            res2.body().contains("test - not allowed"),
            "body: {}",
            res2.body()
        );

        let res3 = send_request_get_text(&service, "/throw_error", "403").await;
        assert_eq!(res3.status(), StatusCode::FORBIDDEN);
        println!("{}", res3.body().len());
        assert!(
            res3.body().contains("test - oh oh"),
            "body: {}",
            res3.body()
        );
    }

    #[test]
    #[should_panic]
    fn invalid_page_route_test_1() {
        #[function_component]
        fn CompA() -> yew::Html {
            yew::html! {
                "test - component (a)"
            }
        }

        crate::impl_page_component!(CompA, "");
        let _ = App::<Base>::new().page::<CompA>().build();
    }

    #[test]
    #[should_panic]
    fn invalid_page_route_test_2() {
        #[function_component]
        fn CompA() -> yew::Html {
            yew::html! {
                "test - component (a)"
            }
        }

        crate::impl_page_component!(CompA, "/a/");
        let _ = App::<Base>::new().page::<CompA>().build();
    }

    #[test]
    #[should_panic]
    fn duplicated_error_handler_test() {
        #[function_component]
        fn ErrorPage1() -> yew::Html {
            yew::html! {
                "test - not found"
            }
        }

        #[function_component]
        fn ErrorPage2() -> yew::Html {
            yew::html! {
                "test - not found"
            }
        }

        crate::impl_page_component!(ErrorPage1);
        crate::impl_page_component!(ErrorPage2);

        let _ = App::<Base>::new()
            .error_page::<ErrorPage1>(StatusCode::NOT_FOUND)
            .error_page::<ErrorPage2>(StatusCode::NOT_FOUND)
            .build();
    }

    #[test]
    fn app_data_test() {
        let service = App::<Base>::new()
            .app_data(String::from("hello world!"))
            .app_data(Arc::new(69420_i32))
            .build();

        assert!(service.app_data().get::<String>().is_some());
        assert!(service.app_data().get::<Arc<i32>>().is_some());
    }

    // Helpers

    #[function_component]
    fn Base(props: &ChildrenProps) -> yew::Html {
        yew::html! {
            {for props.children.iter()}
        }
    }

    async fn noop() {}

    fn create_req(path: &str, method: Method) -> Arc<Request> {
        Request::builder()
            .method(method)
            .uri(path)
            .body(Body::empty())
            .unwrap()
            .into()
    }

    async fn send_request_get_text(
        service: &AppService,
        path: &str,
        body: &str,
    ) -> Response<String> {
        let req = Request::builder()
            .uri(path)
            .body(Body::from(body.to_owned()))
            .unwrap()
            .into();

        let res = service.handle_request(req).await;
        let (parts, body) = res.into_parts();
        let bytes = body.into_bytes().await.unwrap();
        let body = String::from_utf8(bytes.to_vec()).unwrap();
        Response::from_parts(parts, body)
    }
}
