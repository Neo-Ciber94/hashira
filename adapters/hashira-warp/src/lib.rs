pub mod core;

use hashira::adapter::Adapter;
use std::net::SocketAddr;

// A placeholder for an empty filter.
type Empty = warp::filters::BoxedFilter<(Box<dyn warp::Reply>,)>;

/// An adapter for `warp`.
pub struct HashiraWarp<F>(Option<F>);

impl HashiraWarp<Empty> {
    /// Constructs an adapter without any configuration.
    pub fn new() -> HashiraWarp<Empty> {
        HashiraWarp(None)
    }
}

impl Default for HashiraWarp<Empty> {
    fn default() -> Self {
        Self::new()
    }
}

#[hashira::async_trait]
impl<F> Adapter for HashiraWarp<F>
where
    F: warp::Filter<Error = warp::Rejection> + Send + Sync + Clone + 'static,
    F::Extract: warp::reply::Reply,
{
    async fn serve(mut self, app: hashira::app::AppService) -> Result<(), hashira::error::Error> {
        let host = hashira::env::get_host().unwrap_or_else(|| String::from("127.0.0.1"));
        let port = hashira::env::get_port().unwrap_or(5000);
        let addr: SocketAddr = format!("{host}:{port}").as_str().parse().unwrap();

        println!("Server started at: http://{addr}");

        let filter = crate::core::router(app);

        match self.0.take() {
            Some(this) => {
                let routes = this.or(filter);
                warp::serve(routes).run(addr).await;
            }
            None => {
                warp::serve(filter).run(addr).await;
            }
        }

        Ok(())
    }
}

impl<F> From<F> for HashiraWarp<F>
where
    F: warp::Filter<Error = warp::Rejection> + Send + Sync + Clone + 'static,
    F::Extract: warp::reply::Reply,
{
    fn from(value: F) -> Self {
        HashiraWarp(Some(value))
    }
}
