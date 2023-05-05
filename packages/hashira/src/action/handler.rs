use serde::{de::DeserializeOwned, Serialize};

use crate::{
    app::{Handler, RequestContext},
    web::{FromRequest, Response},
};

/// Calls an action handler.
pub async fn call_action<T, H, Args>(
    ctx: RequestContext,
    handler: H,
) -> crate::Result<Response<T>>
where
    Args: FromRequest,
    H: Handler<Args, Output = crate::Result<Response<T>>>,
    T: Serialize + DeserializeOwned,
{
    let args = match Args::from_request(&ctx).await {
        Ok(x) => x,
        Err(err) => return Err(err.into()),
    };

    let res = handler.call(args).await?;
    Ok(res)
}