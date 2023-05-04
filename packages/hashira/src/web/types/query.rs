use crate::web::FromRequest;
use serde::de::DeserializeOwned;
use std::future::{ready, Ready};

/// Represents an url search params.
pub struct Query<Q>(Q);

impl<Q> Query<Q> {
    /// Returns the inner query.
    pub fn into_inner(self) -> Q {
        self.0
    }
}

impl<Q: DeserializeOwned> FromRequest for Query<Q> {
    type Error = serde_qs::Error;
    type Fut = Ready<Result<Query<Q>, Self::Error>>;

    fn from_request(ctx: &crate::app::RequestContext) -> Self::Fut {
        match ctx.request().uri().query() {
            Some(s) => ready(serde_qs::from_str(s).map(Query)),
            None => ready(Err(serde_qs::Error::Custom(
                "url does not contain a query string".to_owned(),
            ))),
        }
    }
}
