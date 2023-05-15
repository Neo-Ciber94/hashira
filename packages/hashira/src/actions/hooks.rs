use super::{into_request_params::IntoRequestParameters, Action};
use crate::{error::BoxError, web::IntoJsonResponse};
use http::{HeaderMap, HeaderName, HeaderValue, Method};
use std::{fmt::Debug, marker::PhantomData, ops::Deref, rc::Rc};
use web_sys::AbortSignal;
use yew::{hook, use_state, Callback, UseStateHandle};

/// Additional options to set to a client request.
#[derive(Debug, Clone)]
pub struct RequestOptions {
    pub headers: HeaderMap,
    pub method: Method,
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
        <HeaderName as TryFrom<K>>::Error: Into<BoxError>,
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<BoxError>,
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
    pub fn try_header<K, V>(mut self, key: K, value: V) -> Result<Self, BoxError>
    where
        HeaderName: TryFrom<K>,
        <HeaderName as TryFrom<K>>::Error: Into<BoxError>,
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<BoxError>,
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

#[allow(type_alias_bounds)]
type ActionData<A: Action> = <A::Response as IntoJsonResponse>::Data;

#[allow(type_alias_bounds)]
type ActionResult<A: Action> = crate::Result<ActionData<A>>;

/// Allow to read the value of a action result.
pub struct UseActionRef<A: Action>(Rc<ActionResult<A>>);

impl<A: Action> Deref for UseActionRef<A> {
    type Target = ActionResult<A>;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

pub struct UseActionOptions<A: Action> {
    on_complete: Option<Callback<UseActionRef<A>>>,
    signal: Option<AbortSignal>,
}

impl<A: Action> UseActionOptions<A> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn on_complete<F>(mut self, f: F) -> Self
    where
        F: Fn(UseActionRef<A>) + 'static,
    {
        self.on_complete = Some(Callback::from(f));
        self
    }

    pub fn signal(mut self, signal: AbortSignal) -> Self {
        self.signal = Some(signal);
        self
    }
}

impl<A: Action> Debug for UseActionOptions<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UseActionOptions")
            .field("on_complete", &self.on_complete)
            .field("signal", &self.signal)
            .finish()
    }
}

impl<A: Action> Clone for UseActionOptions<A> {
    fn clone(&self) -> Self {
        Self {
            on_complete: self.on_complete.clone(),
            signal: self.signal.clone(),
        }
    }
}

impl<A: Action> Default for UseActionOptions<A> {
    fn default() -> Self {
        Self {
            on_complete: Default::default(),
            signal: Default::default(),
        }
    }
}

/// A handle for a server action.
pub struct UseActionHandle<A, T>
where
    A: Action,
{
    loading: UseStateHandle<bool>,
    result: UseStateHandle<Option<Rc<ActionResult<A>>>>,
    options: UseActionOptions<A>,
    _marker: PhantomData<T>,
}

impl<A, T> UseActionHandle<A, T>
where
    A: Action,
    T: IntoRequestParameters,
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
        self.result.as_deref().and_then(|x| x.as_ref().ok())
    }

    /// Returns the response error if any.
    pub fn error(&self) -> Option<&BoxError> {
        self.result.as_deref().and_then(|x| x.as_ref().err())
    }

    /// Sends a request to the server.
    #[cfg(not(target_arch = "wasm32"))]
    #[allow(unused_variables)]
    pub fn send(&self, obj: T) -> Result<(), BoxError> {
        unreachable!("client only function")
    }

    /// Sends a request to the server using the given method.
    #[cfg(target_arch = "wasm32")]
    #[allow(unused_variables)]
    pub fn send(&self, obj: T) -> Result<(), BoxError> {
        self.send_with_options(obj, RequestOptions::new())
    }

    /// Sends a request to the server using the given options.
    #[cfg(not(target_arch = "wasm32"))]
    #[allow(unused_variables)]
    pub fn send_with_options(&self, obj: T, options: RequestOptions) -> Result<(), BoxError> {
        unreachable!("client only function")
    }

    #[cfg(target_arch = "wasm32")]
    pub fn send_with_options(&self, obj: T, options: RequestOptions) -> Result<(), BoxError> {
        use crate::actions::into_request_params::RequestParameters;
        use crate::client::fetch_json;
        use crate::error::JsError;
        use wasm_bindgen::{JsCast, JsValue};
        use web_sys::{Headers, RequestInit};

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

        let request_parameters = obj.into_request_init(&options)?;
        let RequestParameters {
            init,
            search_params,
        } = request_parameters;

        let mut init = init.unwrap_or_else(|| RequestInit::new());
        let headers = match js_sys::Reflect::get(&init, &JsValue::from("headers")) {
            Ok(x) => {
                // If is falsy means it can be null or undefined, so we just create an instance
                if x.is_falsy() {
                    Headers::new().map_err(JsError::new)?
                } else {
                    // Otherwise we try to convert and return a new header if fail
                    match x.dyn_into::<Headers>() {
                        Ok(headers) => headers,
                        Err(err) => {
                            log::debug!("failed to cast property `headers` to Headers type: {err:?}");
                            Headers::new().map_err(JsError::new)?
                        },
                    }
                }
            }
            Err(err) => {
                log::debug!("failed to get `RequestInit::headers`: {err:?}");
                Headers::new().map_err(JsError::new)?
            }
        };

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
        init.signal(self.options.signal.as_ref());

        let mut url = A::route().to_owned();

        if let Some(search_params) = search_params {
            url.push_str(&format!("?{search}", search = search_params.to_string()));
        }

        let request = web_sys::Request::new_with_str_and_init(&url, &init).map_err(JsError::new)?;
        let on_complete = self.options.on_complete.clone();

        wasm_bindgen_futures::spawn_local(async move {
            let _guard = _guard;
            let ret = Rc::new(fetch_json(request).await);

            if let Some(on_complete) = on_complete {
                on_complete.emit(UseActionRef(ret.clone()));
            }

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
            options: self.options.clone(),
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
            .field("options", &self.options)
            .finish()
    }
}

/// Returns a handle to execute an action on the server.
#[hook]
pub fn use_action<A, T>() -> UseActionHandle<A, T>
where
    A: Action,
    T: IntoRequestParameters,
{
    use_action_with_options(Default::default())
}

/// Returns a handle to execute an action on the server and takes a callback that is called when an action completes.
#[hook]
pub fn use_action_with_callback<'a, A, T, F>(on_complete: F) -> UseActionHandle<A, T>
where
    A: Action,
    T: IntoRequestParameters,
    F: Fn(&crate::Result<<A::Response as IntoJsonResponse>::Data>) + 'static,
{
    let options = UseActionOptions::new().on_complete(move |ret| {
        on_complete(&*ret);
    });
    use_action_with_options(options)
}

/// Returns a handle to execute an action on the server using the given options.
#[hook]
pub fn use_action_with_options<A, T>(options: UseActionOptions<A>) -> UseActionHandle<A, T>
where
    A: Action,
    T: IntoRequestParameters,
{
    let result = use_state(|| None);
    let loading = use_state(|| false);

    UseActionHandle {
        result,
        loading,
        options,
        _marker: PhantomData,
    }
}
