use std::{
    cell::RefCell,
    collections::{BTreeMap, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque},
    hash::Hash,
    num::{
        NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroU128, NonZeroU16,
        NonZeroU32, NonZeroU64, NonZeroU8,
    },
    sync::{
        atomic::{
            AtomicBool, AtomicI16, AtomicI32, AtomicI64, AtomicI8, AtomicIsize, AtomicU16,
            AtomicU32, AtomicU64, AtomicU8, AtomicUsize,
        },
        Mutex, RwLock,
    },
};

use super::{Json, Response};
use crate::{app::ResponseError, error::Error};
use http::{header, response::Parts, HeaderMap, StatusCode};
use serde::{de::DeserializeOwned, Serialize};

/// Converts a type into a JSON response.
pub trait IntoJsonResponse {
    /// The json body.
    type Data: Serialize + DeserializeOwned;

    /// Converts this type info a json response.
    fn into_json_response(self) -> crate::Result<Response<Self::Data>>;
}

impl IntoJsonResponse for () {
    type Data = ();

    fn into_json_response(self) -> crate::Result<Response<Self::Data>> {
        Ok(make_json_response(()))
    }
}

/// This type is a workaround to allow to return `crate::Result<Response<T>>` as a 
/// type that implements `IntoJsonResponse`. When specialization get stabilized
/// we can just implement `IntoJsonResponse` directly.
pub struct ResultResponse<T>(pub crate::Result<Response<T>>);

impl<T> IntoJsonResponse for ResultResponse<T>
where
    T: Serialize + DeserializeOwned,
{
    type Data = T;

    fn into_json_response(self) -> crate::Result<Response<Self::Data>> {
        self.0
    }
}

impl<T> IntoJsonResponse for Response<T>
where
    T: Serialize + DeserializeOwned,
{
    type Data = T;

    fn into_json_response(self) -> crate::Result<Response<Self::Data>> {
        Ok(self)
    }
}

impl<T> IntoJsonResponse for Json<T>
where
    T: Serialize + DeserializeOwned,
{
    type Data = T;

    fn into_json_response(self) -> crate::Result<Response<Self::Data>> {
        Ok(make_json_response(self.into_inner()))
    }
}

impl<T> IntoJsonResponse for Option<T>
where
    T: Serialize + DeserializeOwned,
{
    type Data = T;

    fn into_json_response(self) -> crate::Result<Response<Self::Data>> {
        match self {
            Some(x) => Ok(make_json_response(x)),
            None => Err(ResponseError::from(StatusCode::NOT_FOUND).into()),
        }
    }
}

impl<T, E> IntoJsonResponse for Result<T, E>
where
    T: Serialize + DeserializeOwned,
    E: Into<Error>,
{
    type Data = T;

    fn into_json_response(self) -> crate::Result<Response<Self::Data>> {
        match self {
            Ok(x) => Ok(make_json_response(x)),
            Err(err) => Err(err.into()),
        }
    }
}

impl IntoJsonResponse for serde_json::Value {
    type Data = serde_json::Value;

    fn into_json_response(self) -> crate::Result<Response<Self::Data>> {
        Ok(make_json_response(self))
    }
}

macro_rules! impl_into_json_response {
    ($($ty:ty),*) => {
        $(
            impl IntoJsonResponse for $ty {
                type Data = $ty;

                fn into_json_response(self) -> crate::Result<Response<Self::Data>> {
                    Ok(make_json_response(self))
                }
            }
        )*
    };
}

impl_into_json_response! {
    char, bool, String,
    u8, u16, u32, u64, u128,
    i8, i16, i32, i64, i128,
    f32, f64,
    AtomicBool,
    AtomicI8, AtomicI16, AtomicI32, AtomicI64, AtomicIsize,
    AtomicU8, AtomicU16, AtomicU32, AtomicU64, AtomicUsize,
    NonZeroU8, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU128,
    NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI128
}

macro_rules! impl_generic_into_response {
    ($($ty:ty),*) => {
        $(
            impl<T> IntoJsonResponse for $ty
                where T: Serialize + DeserializeOwned
            {
                type Data = $ty;

                fn into_json_response(self) -> crate::Result<Response<Self::Data>> {
                    Ok(make_json_response(self))
                }
            }
        )*
    };
}

impl_generic_into_response! {
    Vec<T>, VecDeque<T>, LinkedList<T>,
    RefCell<T>, Mutex<T>, RwLock<T>
}

impl<T> IntoJsonResponse for HashSet<T>
where
    T: Serialize + DeserializeOwned,
    T: Eq + Hash,
{
    type Data = HashSet<T>;

    fn into_json_response(self) -> crate::Result<Response<Self::Data>> {
        Ok(make_json_response(self))
    }
}

impl<T> IntoJsonResponse for BinaryHeap<T>
where
    T: Serialize + DeserializeOwned + Ord,
{
    type Data = BinaryHeap<T>;

    fn into_json_response(self) -> crate::Result<Response<Self::Data>> {
        Ok(make_json_response(self))
    }
}

impl<K, V> IntoJsonResponse for HashMap<K, V>
where
    K: Serialize + DeserializeOwned + Eq + Hash,
    V: Serialize + DeserializeOwned,
{
    type Data = HashMap<K, V>;

    fn into_json_response(self) -> crate::Result<Response<Self::Data>> {
        Ok(make_json_response(self))
    }
}

impl<K, V> IntoJsonResponse for BTreeMap<K, V>
where
    K: Serialize + DeserializeOwned + Ord,
    V: Serialize + DeserializeOwned,
{
    type Data = BTreeMap<K, V>;

    fn into_json_response(self) -> crate::Result<Response<Self::Data>> {
        Ok(make_json_response(self))
    }
}

impl<T> IntoJsonResponse for (Parts, T)
where
    T: Serialize + DeserializeOwned,
{
    type Data = T;

    fn into_json_response(self) -> crate::Result<Response<Self::Data>> {
        let (mut parts, body) = self;

        // Ensure is application/json
        parts
            .headers
            .insert(
                header::CONTENT_TYPE,
                header::HeaderValue::from_str(mime::APPLICATION_JSON.essence_str()).unwrap(),
            )
            .unwrap();

        Ok(Response::from_parts(parts, body))
    }
}

impl<T> IntoJsonResponse for (Response<()>, T)
where
    T: Serialize + DeserializeOwned,
{
    type Data = T;

    fn into_json_response(self) -> crate::Result<Response<Self::Data>> {
        let (mut res, body) = self;

        // Ensure is application/json
        res.headers_mut().insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_str(mime::APPLICATION_JSON.essence_str()).unwrap(),
        );

        Ok(res.map(|_| body))
    }
}

impl<T> IntoJsonResponse for (HeaderMap, T)
where
    T: Serialize + DeserializeOwned,
{
    type Data = T;

    fn into_json_response(self) -> crate::Result<Response<Self::Data>> {
        let (mut headers, body) = self;

        // Ensure is application/json
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_str(mime::APPLICATION_JSON.essence_str()).unwrap(),
        );

        let mut res = Response::new(body);
        *res.headers_mut() = headers;
        Ok(res)
    }
}

#[inline]
fn make_json_response<T: Serialize + DeserializeOwned>(value: T) -> Response<T> {
    Response::builder()
        .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.essence_str())
        .body(value)
        .unwrap()
}
