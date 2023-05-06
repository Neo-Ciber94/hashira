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
    /// The type of the body of the action response.
    type Response: IntoJsonResponse + 'static;

    /// The path of the route.
    fn route() -> &'static str;

    /// Returns the methods this action can be called:
    fn method() -> HttpMethod {
        HttpMethod::GET
            | HttpMethod::POST
            | HttpMethod::PUT
            | HttpMethod::PATCH
            | HttpMethod::DELETE
    }

    /// Call this action and returns a response.
    fn call(ctx: RequestContext) -> BoxFuture<crate::Result<Self::Response>>;
}
