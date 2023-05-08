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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use serde::Deserialize;

    use crate::{
        app::{
            router::{PageRouter, PageRouterWrapper},
            AppData, RequestContext,
        },
        routing::{ErrorRouter, Params},
        web::{Body, FromRequest, Query, Request},
    };

    #[tokio::test]
    async fn query_from_request_test() {
        #[derive(Deserialize)]
        struct MyStruct {
            text: String,
            number: i64,
        }

        let req = Request::builder()
            .uri("/path/to/route?text=hello-world&number=999")
            .body(Body::empty())
            .unwrap();

        let ctx = create_request_context(req);
        let query = Query::<MyStruct>::from_request(&ctx)
            .await
            .unwrap()
            .into_inner();

        assert_eq!(query.text, "hello-world");
        assert_eq!(query.number, 999);
    }

    fn create_request_context(req: Request) -> RequestContext {
        RequestContext::new(
            Arc::new(req),
            Arc::new(AppData::default()),
            PageRouterWrapper::from(PageRouter::new()),
            Arc::new(ErrorRouter::new()),
            None,
            Params::default(),
        )
    }
}
