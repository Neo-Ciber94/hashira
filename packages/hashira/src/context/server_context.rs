use crate::app::RequestContext;
use std::ops::Deref;
use yew::{function_component, hook, use_context, Children, ContextProvider, Properties};

#[derive(Clone)]
struct ServerContextInner {
    ctx: RequestContext,
}

impl Deref for ServerContextInner {
    type Target = RequestContext;

    fn deref(&self) -> &Self::Target {
        &self.ctx
    }
}

impl PartialEq for ServerContextInner {
    fn eq(&self, other: &Self) -> bool {
        self.ctx == other.ctx
    }
}

/// Contains data about the server, this data is only available on the server.
#[derive(Clone, PartialEq)]
pub struct ServerContext {
    inner: Option<ServerContextInner>,
}

impl ServerContext {
    pub(crate) fn new(ctx: Option<RequestContext>) -> Self {
        ServerContext {
            inner: ctx.map(|ctx| ServerContextInner { ctx }),
        }
    }
}

impl Deref for ServerContext {
    type Target = RequestContext;

    fn deref(&self) -> &Self::Target {
        let Some(ctx) = self.inner.as_ref() else {
            unreachable!("`ServerContext` is only available on the server")
        };

        ctx
    }
}

#[doc(hidden)]
#[derive(PartialEq, Properties)]
pub struct ServerContextProps {
    pub server_context: ServerContext,
    pub children: Children,
}

/// Provides the `ServerContext` to the children components.
#[doc(hidden)]
#[function_component]
pub fn ServerContextProvider(props: &ServerContextProps) -> yew::Html {
    yew::html! {
        <ContextProvider<ServerContext> context={props.server_context.clone()}>
            {for props.children.iter()}
        </ContextProvider<ServerContext>>
    }
}

/// Returns handle containing server data.
/// 
/// # Returns
/// - `Some(ServerContext)` when ran on the server.
/// - `None` when ran on the client.
#[hook]
pub fn use_server_context() -> Option<ServerContext> {
    // The server context is only available on the server side.
    if !crate::consts::IS_SERVER {
        return None;
    }

    // Returns the value
    use_context::<ServerContext>()
}
