use std::fmt::Debug;

use http::{HeaderMap, HeaderName, HeaderValue};

/// Represent the default headers to send in a response.
#[derive(Default, Debug, Clone)]
pub struct DefaultHeaders(HeaderMap);

impl DefaultHeaders {
    /// Constructs an empty instance.
    pub fn new() -> Self {
        Default::default()
    }

    /// Add the given header key-value pair.
    /// 
    /// # Panics
    /// - If the key or value are invalid header name or value.
    pub fn add<K, V>(mut self, key: K, value: V) -> Self
    where
        HeaderName: TryFrom<K>,
        <HeaderName as TryFrom<K>>::Error: Into<http::Error>,
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<http::Error>,
    {
        let name = <HeaderName as TryFrom<K>>::try_from(key)
            .map_err(Into::into)
            .expect("invalid header name");
        let value = <HeaderValue as TryFrom<V>>::try_from(value)
            .map_err(Into::into)
            .expect("invalid header value");

        self.0.insert(name, value);
        self
    }

    /// Returns the inner header map.
    pub fn into_inner(self) -> HeaderMap {
        self.0
    }
}
