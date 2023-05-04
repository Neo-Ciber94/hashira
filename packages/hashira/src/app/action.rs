use crate::{
    types::BoxFuture,
    web::{FromRequest, IntoResponse, Response},
};

use super::{HttpMethod, RequestContext};

/// An action that can be execute on the server.
pub trait Action {
    /// The input of the action.
    type Req: FromRequest;
    type Res: IntoResponse;

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

    /// Call this action with the given data and context and returns a response.
    fn call(ctx: RequestContext) -> BoxFuture<Self::Res>;
}

struct EchoAction;
impl Action for EchoAction {
    type Req = String;
    type Res = String;

    fn route() -> &'static str {
        todo!()
    }

    fn call(ctx: RequestContext) -> BoxFuture<Self::Res> {
        todo!()
    }
}

pub fn use_action<A>() {
    todo!()
}
