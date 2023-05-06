use super::Action;
use crate::{
    error::{Error, JsError},
    web::{Form, IntoJsonResponse, Json},
};
use http::{
    header::{self},
    HeaderMap, HeaderName, HeaderValue, Method,
};
use serde::Serialize;
use std::{fmt::Debug, marker::PhantomData};
use wasm_bindgen::JsValue;
use web_sys::{FormData, Headers, RequestInit};
use yew::{hook, use_state, UseStateHandle};

/// Converts a type info a javascript `RequestInit`.
pub trait IntoRequestInit {
    /// Returns a new `RequestInit` to create a request.
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
    result: UseStateHandle<Option<Result<<A::Data as IntoJsonResponse>::Data, Error>>>,
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
    pub fn data(&self) -> Option<&<A::Data as IntoJsonResponse>::Data> {
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

        let mut init = obj.into_request_init()?;
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

        let request =
            web_sys::Request::new_with_str_and_init(A::route(), &init).map_err(JsError::new)?;

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
            _marker: self._marker.clone(),
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
    <A::Data as IntoJsonResponse>::Data: Debug,
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
