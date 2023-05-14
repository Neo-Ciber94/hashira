pub mod core;
use hashira::{adapter::Adapter, app::AppService};
use std::net::SocketAddr;

/// An adapter for `tide`.
#[derive(Default)]
pub struct HashiraTide<S = ()>(Option<tide::Server<S>>);

#[hashira::async_trait]
impl<S> Adapter for HashiraTide<S>
where
    S: Clone + Send + Sync + 'static,
{
    /// Starts the server.
    async fn serve(mut self, app: AppService) -> Result<(), hashira::error::BoxError> {
        let host = hashira::env::get_host().unwrap_or_else(|| String::from("127.0.0.1"));
        let port = hashira::env::get_port().unwrap_or(5000);
        let addr: SocketAddr = format!("{host}:{port}").as_str().parse().unwrap();

        println!("Server started at: http://{addr}");

        match self.0.take() {
            Some(router) => {
                let tide = crate::core::with_router(router, app);
                tide.listen(addr).await?;
            }
            None => {
                let tide = crate::core::router(app);
                tide.listen(addr).await?;
            }
        }

        Ok(())
    }
}

impl<S> HashiraTide<S> {
    /// Constructs a default hashira adapter.
    pub fn new() -> Self {
        HashiraTide(None)
    }
}

impl<S> From<tide::Server<S>> for HashiraTide<S> {
    fn from(value: tide::Server<S>) -> Self {
        HashiraTide(Some(value))
    }
}
