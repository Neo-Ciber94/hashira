use crate::error::BoxError;
use futures::Stream;
use std::pin::Pin;

// A boxed stream that return a result type.
pub type TryBoxStream<T> = Pin<Box<dyn Stream<Item = Result<T, BoxError>> + Send + Sync>>;

// A convenient boxed future.
pub type BoxFuture<T> = Pin<Box<dyn std::future::Future<Output = T> + Send>>;
