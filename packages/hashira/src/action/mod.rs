mod handler;
mod hooks;

pub use handler::*;
pub use hooks::*;

use crate::{
    app::{HttpMethod, RequestContext},
    types::BoxFuture,
    web::IntoJsonResponse,
};

/// An action that can be execute on the server.
pub trait Action {
    /// The output of the action response.
    type Res: IntoJsonResponse + 'static;

    /// The path of the route.
    fn route() -> &'static str;

    /// Returns the methods this action can be called:
    ///
    /// # Examples
    /// ```no_run
    /// fn method() -> HttpMethod {
    ///     HttpMethod::POST | HttpMethod::PUT
    /// }
    /// ```
    fn method() -> HttpMethod {
        HttpMethod::GET
            | HttpMethod::POST
            | HttpMethod::PUT
            | HttpMethod::PATCH
            | HttpMethod::DELETE
    }

    /// Call this action and returns a response.
    fn call(ctx: RequestContext) -> BoxFuture<Self::Res>;
}
