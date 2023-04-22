use crate::error::Error;
use std::pin::Pin;
use futures::Stream;
//use std::future::Future;

// A boxed stream that return a result type.
pub type TryBoxStream<T> = Pin<Box<dyn Stream<Item = Result<T, Error>> + Send + Sync>>;

// A convenient boxed future.
pub type BoxFuture<T> = Pin<Box<dyn std::future::Future<Output = T> + Send>>;

//pub type BoxFuture<T> = futures::future::BoxFuture<'static, T>;

// A boxed future that is `Send + Sync`
//pub type BoxSyncFuture<T> = Pin<Box<dyn Future<Output = T> + Send + Sync>>;