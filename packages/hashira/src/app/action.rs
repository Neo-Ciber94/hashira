use super::{HttpMethod, RequestContext};
use crate::{types::BoxFuture, web::Response};
use serde::{de::DeserializeOwned, Serialize};

/// An action that can be execute on the server.
pub trait Action {
    /// The output of the action response.
    type Output: DeserializeOwned + Serialize;

    /// The path of the route.
    fn route() -> &'static str;

    /// Returns the methods this action can be called:
    ///
    /// # Examples
    /// ```no_run
    /// fn method() -> HttpMethod {
    ///     HttpMethod::POST | HttpMethod::PUT
    /// }
    /// ```
    fn method() -> HttpMethod {
        HttpMethod::GET
            | HttpMethod::POST
            | HttpMethod::PUT
            | HttpMethod::PATCH
            | HttpMethod::DELETE
    }

    /// Call this action and returns a response.
    fn call(ctx: RequestContext) -> BoxFuture<crate::Result<Response<Self::Output>>>;
}

pub mod hooks {
    use super::Action;
    use crate::{
        error::Error,
        web::{Form, Json},
    };
    use http::header;
    use serde::Serialize;
    use std::marker::PhantomData;
    use thiserror::Error;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{FormData, Headers, RequestInit};
    use yew::{hook, use_state, UseStateHandle};

    #[derive(Debug, Error)]
    #[error("Javascript error: {0}")]
    pub struct JsError(String);
    impl JsError {
        pub fn new(err: JsValue) -> Self {
            use std::fmt::Write;

            if let Some(s) = err.as_string() {
                return JsError(s);
            }

            match err.dyn_into::<js_sys::Error>() {
                Ok(err) => JsError(err.to_string().into()),
                Err(err) => {
                    let mut buf = String::new();
                    write!(buf, "{err:?}").expect("failed to format javascript error");
                    JsError(buf)
                }
            }
        }
    }

    /// Converts a type info a javascript `RequestInit`.
    pub trait IntoRequestInit {
        fn into_request_init(self) -> Result<RequestInit, Error>;
    }

    impl IntoRequestInit for RequestInit {
        fn into_request_init(self) -> Result<RequestInit, Error> {
            Ok(self)
        }
    }

    impl<T: Serialize> IntoRequestInit for Json<T> {
        fn into_request_init(self) -> Result<RequestInit, Error> {
            let mut init = RequestInit::new();
            let headers = Headers::new().map_err(JsError::new)?;
            headers
                .set(
                    header::CONTENT_TYPE.as_str(),
                    mime::APPLICATION_JSON.essence_str(),
                )
                .map_err(JsError::new)?;

            let json = serde_json::to_string(&self.0)?;

            init.headers(&headers);
            init.body(Some(&JsValue::from_str(&json)));

            Ok(init)
        }
    }

    impl<T: Serialize> IntoRequestInit for Form<T> {
        fn into_request_init(self) -> Result<RequestInit, Error> {
            let mut init = RequestInit::new();
            let headers = Headers::new().map_err(JsError::new)?;
            headers
                .set(
                    header::CONTENT_TYPE.as_str(),
                    mime::APPLICATION_WWW_FORM_URLENCODED.essence_str(),
                )
                .map_err(JsError::new)?;

            let json = serde_urlencoded::to_string(&self.0)?;

            init.headers(&headers);
            init.body(Some(&JsValue::from_str(&json)));

            Ok(init)
        }
    }

    impl IntoRequestInit for String {
        fn into_request_init(self) -> Result<RequestInit, Error> {
            let mut init = RequestInit::new();
            let headers = Headers::new().map_err(JsError::new)?;
            headers
                .set(
                    header::CONTENT_TYPE.as_str(),
                    mime::TEXT_PLAIN_UTF_8.essence_str(),
                )
                .map_err(JsError::new)?;

            init.headers(&headers);
            init.body(Some(&JsValue::from_str(&self)));

            Ok(init)
        }
    }

    impl IntoRequestInit for &'static str {
        fn into_request_init(self) -> Result<RequestInit, Error> {
            let mut init = RequestInit::new();
            let headers = Headers::new().map_err(JsError::new)?;
            headers
                .set(
                    header::CONTENT_TYPE.as_str(),
                    mime::TEXT_PLAIN_UTF_8.essence_str(),
                )
                .map_err(JsError::new)?;

            init.headers(&headers);
            init.body(Some(&JsValue::from_str(self)));

            Ok(init)
        }
    }

    impl IntoRequestInit for FormData {
        fn into_request_init(self) -> Result<RequestInit, Error> {
            let mut init = RequestInit::new();
            init.body(Some(&self));
            Ok(init)
        }
    }

    pub struct UseActionHandle<A: Action, T> {
        loading: UseStateHandle<bool>,
        result: UseStateHandle<Option<Result<A::Output, Error>>>,
        _marker: PhantomData<T>,
    }

    impl<A, T> UseActionHandle<A, T>
    where
        A: Action,
        A::Output: 'static,
        T: IntoRequestInit,
    {
        pub fn is_loading(&self) -> bool {
            *self.loading
        }

        pub fn is_error(&self) -> bool {
            match &*self.result {
                Some(x) => x.is_err(),
                None => false,
            }
        }

        pub fn data(&self) -> Option<&A::Output> {
            self.result.as_ref().and_then(|x| x.as_ref().ok())
        }

        pub fn error(&self) -> Option<&Error> {
            self.result.as_ref().and_then(|x| x.as_ref().err())
        }

        #[cfg(not(target_arch = "wasm32"))]
        #[allow(unused_variables)]
        pub fn send(&self, obj: T) -> Result<(), Error> {
            unreachable!("client only function")
        }

        #[cfg(target_arch = "wasm32")]
        pub fn send(&self, obj: T) -> Result<(), Error> {
            use serde::de::DeserializeOwned;
            use wasm_bindgen_futures::JsFuture;

            struct OnDrop<F: FnOnce()>(Option<F>);
            impl<F: FnOnce()> Drop for OnDrop<F> {
                fn drop(&mut self) {
                    if let Some(f) = self.0.take() {
                        f();
                    }
                }
            }

            async fn fetch<S: DeserializeOwned>(request: web_sys::Request) -> Result<S, Error> {
                let window = web_sys::window().unwrap();
                let resp_value = JsFuture::from(window.fetch_with_request(&request))
                    .await
                    .map_err(JsError::new)?;

                let resp: web_sys::Response = resp_value.dyn_into().unwrap();
                let json = resp.json().map_err(JsError::new)?;

                // Convert this other `Promise` into a rust `Future`.
                let json = JsFuture::from(json).await.map_err(JsError::new)?;

                match serde_wasm_bindgen::from_value(json) {
                    Ok(x) => Ok(x),
                    Err(err) => {
                        let s = err.to_string();
                        Err(s.into())
                    }
                }
            }

            let loading = self.loading.clone();
            loading.set(true);

            let _guard = OnDrop(Some(move || loading.set(false)));
            let result = self.result.clone();

            let mut init = obj.into_request_init()?;
            init.method("POST");

            let request =
                web_sys::Request::new_with_str_and_init(A::route(), &init).map_err(JsError::new)?;

            wasm_bindgen_futures::spawn_local(async move {
                let _guard = _guard;
                let ret = fetch(request).await;
                result.set(Some(ret));
            });

            Ok(())
        }
    }

    #[hook]
    pub fn use_action<A, T>() -> UseActionHandle<A, T>
    where
        A: Action,
        A::Output: 'static,
        T: IntoRequestInit,
    {
        let result = use_state(|| None);
        let loading = use_state(|| false);

        UseActionHandle {
            result,
            loading,
            _marker: PhantomData,
        }
    }
}

// A handler for server actions.
pub mod handler {
    use serde::{de::DeserializeOwned, Serialize};

    use crate::{
        app::{Handler, RequestContext},
        web::{FromRequest, Response},
    };

    /// Calls an action handler.
    pub async fn call_action<T, H, Args>(
        ctx: RequestContext,
        handler: H,
    ) -> crate::Result<Response<T>>
    where
        Args: FromRequest,
        H: Handler<Args, Output = crate::Result<Response<T>>>,
        T: Serialize + DeserializeOwned,
    {
        let args = match Args::from_request(&ctx).await {
            Ok(x) => x,
            Err(err) => return Err(err.into()),
        };

        let res = handler.call(args).await?;
        Ok(res)
    }
}
