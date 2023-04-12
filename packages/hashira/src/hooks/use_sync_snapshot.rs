use yew::hook;

use crate::{app::RequestContext, context::use_server_context};

/// Expose two methods for synchronize data on the client and server side.
pub struct SyncSnapshot<T, S, C>
where
    S: FnOnce(&RequestContext) -> T,
    C: FnOnce() -> T,
{
    /// A closure that is called on the server to get the data snapshot.
    pub server: S,

    /// A closure that is called on the client to get the data snapshot.
    pub client: C,
}

/// Synchronize data between the server and the client,
/// this is done by providing two closures one which are called on the server and client,
/// both closures should return the same value.
/// 
/// This is useful for synchronize data which is not available in both sides,
/// for example the server cannot access the `url` through the `window.location` but can from the request,
/// and the same for the client.
#[hook]
pub fn use_sync_snapshot<T, S, C>(sync: SyncSnapshot<T, S, C>) -> T
where
    S: FnOnce(&RequestContext) -> T,
    C: FnOnce() -> T,
{
    let ctx = use_server_context();
    if crate::consts::IS_SERVER {
        let request_context = &*ctx.unwrap();
        (sync.server)(request_context)
    } else {
        (sync.client)()
    }
}
