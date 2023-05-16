use crate::{
    error::{BoxError, JsError},
    web::{Form, Json},
};
use http::{
    header::{self},
    Method,
};
use serde::Serialize;
use std::fmt::Debug;
use wasm_bindgen::JsValue;
use web_sys::{FormData, Headers, RequestInit, UrlSearchParams};

use super::RequestOptions;

/// Initialization params for a request.
#[derive(Default, Debug, Clone)]
pub struct RequestInitConfig {
    /// The initial request state.
    pub init: Option<RequestInit>,

    /// Search params for the url.
    pub search_params: Option<UrlSearchParams>,
}

/// Creates an object to initialize a request.
pub trait IntoRequestConfig {
    /// Returns an object used to initialize a request.
    fn into_request_config(self, _options: &RequestOptions) -> Result<RequestInitConfig, BoxError>;
}

impl IntoRequestConfig for RequestInit {
    fn into_request_config(self, _options: &RequestOptions) -> Result<RequestInitConfig, BoxError> {
        Ok(RequestInitConfig {
            init: Some(self),
            search_params: None,
        })
    }
}

impl IntoRequestConfig for UrlSearchParams {
    fn into_request_config(self, _options: &RequestOptions) -> Result<RequestInitConfig, BoxError> {
        Ok(RequestInitConfig {
            init: None,
            search_params: Some(self),
        })
    }
}

impl IntoRequestConfig for String {
    fn into_request_config(self, _options: &RequestOptions) -> Result<RequestInitConfig, BoxError> {
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

        Ok(RequestInitConfig {
            init: Some(init),
            search_params: None,
        })
    }
}

impl IntoRequestConfig for &'static str {
    fn into_request_config(self, _options: &RequestOptions) -> Result<RequestInitConfig, BoxError> {
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

        Ok(RequestInitConfig {
            init: Some(init),
            search_params: None,
        })
    }
}

impl<T: Serialize> IntoRequestConfig for Json<T> {
    fn into_request_config(self, _options: &RequestOptions) -> Result<RequestInitConfig, BoxError> {
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

        Ok(RequestInitConfig {
            init: Some(init),
            search_params: None,
        })
    }
}

impl<T: Serialize> IntoRequestConfig for Form<T> {
    fn into_request_config(self, options: &RequestOptions) -> Result<RequestInitConfig, BoxError> {
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

        if options.method == Method::GET || options.method == Method::HEAD {
            let search_params = UrlSearchParams::new_with_str(&params).map_err(JsError::new)?;

            Ok(RequestInitConfig {
                init: Some(init),
                search_params: Some(search_params),
            })
        } else {
            init.body(Some(&JsValue::from_str(&params)));

            Ok(RequestInitConfig {
                init: Some(init),
                search_params: None,
            })
        }
    }
}

impl IntoRequestConfig for FormData {
    fn into_request_config(self, _options: &RequestOptions) -> Result<RequestInitConfig, BoxError> {
        let mut init = RequestInit::new();
        init.body(Some(&self));
        Ok(RequestInitConfig {
            init: Some(init),
            search_params: None,
        })
    }
}
