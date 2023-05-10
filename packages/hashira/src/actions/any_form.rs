use js_sys::Array;
use wasm_bindgen::JsCast;
use web_sys::FormData;

use crate::{
    error::{Error, JsError},
    web::Form,
};

use super::{
    into_request_params::{IntoRequestParameters, RequestParameters},
    RequestOptions,
};

/// A form that can be multipart or url-encoded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnyForm {
    /// A multipart form.
    Multipart(FormData),

    /// An url-encoded form.
    UrlEncoded(FormData),
}

impl IntoRequestParameters for AnyForm {
    fn into_request_init(self, options: &RequestOptions) -> Result<RequestParameters, Error> {
        match self {
            AnyForm::Multipart(form) => form.into_request_init(options),
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

                Form(params).into_request_init(options)
            }
        }
    }
}
