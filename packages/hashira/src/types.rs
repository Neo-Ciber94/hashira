use crate::error::Error;
use futures::Stream;
use std::pin::Pin;

// A boxed stream that return a result type.
pub type TryBoxStream<T> = Pin<Box<dyn Stream<Item = Result<T, Error>> + Send + Sync>>;

// A convenient boxed future.
pub type BoxFuture<T> = Pin<Box<dyn std::future::Future<Output = T> + Send>>;

// A convenient async boxed future.
pub type BoxSyncFuture<T> = Pin<Box<dyn std::future::Future<Output = T> + Send + Sync>>;
