use super::Action;
use crate::{
    error::{Error, JsError},
    web::{Form, IntoJsonResponse, Json},
};
use http::{
    header::{self},
    HeaderMap, HeaderName, HeaderValue, Method,
};
use js_sys::Array;
use serde::Serialize;
use std::{fmt::Debug, marker::PhantomData};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{FormData, Headers, RequestInit, UrlSearchParams};
use yew::{hook, use_state, UseStateHandle};

/// Initialization params for a request.
#[derive(Default, Debug, Clone)]
pub struct RequestParameters {
    /// The initial request state.
    pub init: Option<RequestInit>,

    /// Search params for the url.
    pub search_params: Option<UrlSearchParams>,
}

/// Creates an object to initialize a request.
pub trait IntoRequestInit {
    /// Returns an object used to initialize a request.
    fn into_request_init(self) -> Result<RequestParameters, Error>;
}

impl IntoRequestInit for RequestInit {
    fn into_request_init(self) -> Result<RequestParameters, Error> {
        Ok(RequestParameters {
            init: Some(self),
            search_params: None,
        })
    }
}

impl IntoRequestInit for UrlSearchParams {
    fn into_request_init(self) -> Result<RequestParameters, Error> {
        Ok(RequestParameters {
            init: None,
            search_params: Some(self),
        })
    }
}

impl IntoRequestInit for String {
    fn into_request_init(self) -> Result<RequestParameters, Error> {
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

        Ok(RequestParameters {
            init: Some(init),
            search_params: None,
        })
    }
}

impl IntoRequestInit for &'static str {
    fn into_request_init(self) -> Result<RequestParameters, Error> {
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

        Ok(RequestParameters {
            init: Some(init),
            search_params: None,
        })
    }
}

impl<T: Serialize> IntoRequestInit for Json<T> {
    fn into_request_init(self) -> Result<RequestParameters, Error> {
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

        Ok(RequestParameters {
            init: Some(init),
            search_params: None,
        })
    }
}

impl<T: Serialize> IntoRequestInit for Form<T> {
    fn into_request_init(self) -> Result<RequestParameters, Error> {
        let mut init = RequestInit::new();
        let headers = Headers::new().map_err(JsError::new)?;
        headers
            .set(
                header::CONTENT_TYPE.as_str(),
                mime::APPLICATION_WWW_FORM_URLENCODED.essence_str(),
            )
            .map_err(JsError::new)?;

        init.headers(&headers);

        let params = serde_urlencoded::to_string(&self.0)?;
        let search_params = UrlSearchParams::new_with_str(&params).map_err(JsError::new)?;

        Ok(RequestParameters {
            init: Some(init),
            search_params: Some(search_params),
        })
    }
}

impl IntoRequestInit for FormData {
    fn into_request_init(self) -> Result<RequestParameters, Error> {
        let mut init = RequestInit::new();
        init.body(Some(&self));
        Ok(RequestParameters {
            init: Some(init),
            search_params: None,
        })
    }
}

/// A form that can be multipart or url-encoded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnyForm {
    /// A multipart form.
    Multipart(FormData),

    /// An url-encoded form.
    UrlEncoded(FormData),
}

impl IntoRequestInit for AnyForm {
    fn into_request_init(self) -> Result<RequestParameters, Error> {
        match self {
            AnyForm::Multipart(form) => form.into_request_init(),
            AnyForm::UrlEncoded(form) => {
                let iter = js_sys::try_iter(&form).map_err(JsError::new)?;
                let mut params = Vec::new();
                if let Some(iter) = iter {
                    for x in iter {
                        let x = x.map_err(JsError::new)?;
                        let arr = x.dyn_into::<Array>().map_err(JsError::new)?;
                        let key = arr.at(0).as_string().unwrap();
                        let Some(value) = arr.at(1).as_string() else {
                            return Err("FormData value was not a string, use a multipart form instead".to_owned().into());
                        };

                        params.push((key, value));
                    }
                }

                Form(params).into_request_init()
            }
        }
    }
}

/// Additional options to set to a client request.
#[derive(Debug, Clone)]
pub struct RequestOptions {
    headers: HeaderMap,
    method: Method,
}

impl RequestOptions {
    /// Constructs a default instance.
    pub fn new() -> Self {
        RequestOptions {
            headers: HeaderMap::new(),
            method: Method::POST,
        }
    }

    /// Append a header.
    ///
    /// # Panic
    /// If the name or value are invalid.
    pub fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        HeaderName: TryFrom<K>,
        <HeaderName as TryFrom<K>>::Error: Into<Error>,
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<Error>,
    {
        let name = <HeaderName as TryFrom<K>>::try_from(key)
            .map_err(Into::into)
            .expect("invalid header name");
        let value = <HeaderValue as TryFrom<V>>::try_from(value)
            .map_err(Into::into)
            .expect("invalid header value");
        self.headers.insert(name, value);
        self
    }

    /// Attempts to append a header.
    ///
    /// # Returns
    /// An error if the header name or value are invalid.
    pub fn try_header<K, V>(mut self, key: K, value: V) -> Result<Self, Error>
    where
        HeaderName: TryFrom<K>,
        <HeaderName as TryFrom<K>>::Error: Into<Error>,
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<Error>,
    {
        let name = <HeaderName as TryFrom<K>>::try_from(key).map_err(Into::into)?;
        let value = <HeaderValue as TryFrom<V>>::try_from(value).map_err(Into::into)?;
        self.headers.insert(name, value);
        Ok(self)
    }

    /// Changes the method used to send the request.
    pub fn method(mut self, method: Method) -> Self {
        self.method = method;
        self
    }
}

impl Default for RequestOptions {
    fn default() -> Self {
        RequestOptions::new()
    }
}

/// A handle for a server action.
pub struct UseActionHandle<A, T>
where
    A: Action,
{
    loading: UseStateHandle<bool>,
    result: UseStateHandle<Option<Result<<A::Response as IntoJsonResponse>::Data, Error>>>,
    _marker: PhantomData<T>,
}

impl<A, T> UseActionHandle<A, T>
where
    A: Action,
    T: IntoRequestInit,
{
    /// Returns `true` if the action is processing.
    pub fn is_loading(&self) -> bool {
        *self.loading
    }

    /// Returns `true` if the action returned an error.
    pub fn is_error(&self) -> bool {
        match &*self.result {
            Some(x) => x.is_err(),
            None => false,
        }
    }

    /// Returns the response value if any.
    pub fn data(&self) -> Option<&<A::Response as IntoJsonResponse>::Data> {
        self.result.as_ref().and_then(|x| x.as_ref().ok())
    }

    /// Returns the response error if any.
    pub fn error(&self) -> Option<&Error> {
        self.result.as_ref().and_then(|x| x.as_ref().err())
    }

    /// Sends a request to the server.
    #[cfg(not(target_arch = "wasm32"))]
    #[allow(unused_variables)]
    pub fn send(&self, obj: T) -> Result<(), Error> {
        unreachable!("client only function")
    }

    /// Sends a request to the server using the given method.
    #[cfg(target_arch = "wasm32")]
    #[allow(unused_variables)]
    pub fn send(&self, obj: T) -> Result<(), Error> {
        self.send_with_options(obj, RequestOptions::new().method(Method::POST))
    }

    /// Sends a request to the server using the given options.
    #[cfg(not(target_arch = "wasm32"))]
    #[allow(unused_variables)]
    pub fn send_with_options(&self, obj: T, options: RequestOptions) -> Result<(), Error> {
        unreachable!("client only function")
    }

    #[cfg(target_arch = "wasm32")]
    pub fn send_with_options(&self, obj: T, options: RequestOptions) -> Result<(), Error> {
        use crate::client::fetch_json;

        struct OnDrop<F: FnOnce()>(Option<F>);
        impl<F: FnOnce()> Drop for OnDrop<F> {
            fn drop(&mut self) {
                if let Some(f) = self.0.take() {
                    f();
                }
            }
        }

        let loading = self.loading.clone();
        loading.set(true);

        let _guard = OnDrop(Some(move || loading.set(false)));
        let result = self.result.clone();

        let request_parameters = obj.into_request_init()?;
        let RequestParameters {
            init,
            search_params,
        } = request_parameters;

        let mut init = init.unwrap_or_else(|| RequestInit::new());
        let headers = Headers::new().map_err(JsError::new)?;

        let mut last_name = None;
        for (name, value) in options.headers {
            if let Some(name) = name {
                last_name = Some(name);
            }

            let key = last_name.as_ref().unwrap();
            headers
                .append(key.as_str(), value.to_str()?)
                .map_err(JsError::new)?;
        }

        init.headers(&headers);
        init.method(options.method.as_str());

        let mut url = A::route().to_owned();

        if let Some(search_params) = search_params {
            url.push_str(&format!("?{search}", search = search_params.to_string()));
        }

        let request = web_sys::Request::new_with_str_and_init(&url, &init).map_err(JsError::new)?;

        wasm_bindgen_futures::spawn_local(async move {
            let _guard = _guard;
            let ret = fetch_json(request).await;
            result.set(Some(ret));
        });

        Ok(())
    }
}

impl<A, T> Clone for UseActionHandle<A, T>
where
    A: Action,
{
    fn clone(&self) -> Self {
        Self {
            loading: self.loading.clone(),
            result: self.result.clone(),
            _marker: self._marker,
        }
    }
}

impl<A, T> PartialEq for UseActionHandle<A, T>
where
    A: Action,
{
    fn eq(&self, other: &Self) -> bool {
        // TODO: Add proper equality implementation
        self.loading == other.loading && std::ptr::eq(&*self.result, &*other.result)
    }
}

impl<A, T> Debug for UseActionHandle<A, T>
where
    A: Action,
    <A::Response as IntoJsonResponse>::Data: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UseActionHandle")
            .field("loading", &self.loading)
            .field("result", &self.result)
            .finish()
    }
}

/// Returns a handle to execute an action on ths server.
#[hook]
pub fn use_action<A, T>() -> UseActionHandle<A, T>
where
    A: Action,
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
