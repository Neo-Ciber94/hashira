use crate::{
    app::{Handler, RequestContext},
    web::{Body, FromRequest},
};

// FIXME: This function is not constrained to only `Action` so could be use for any handler.
// FIXME: move to other place?
/// Calls an action handler.
pub async fn call_action<H, Args>(
    ctx: RequestContext,
    mut body: Body,
    handler: H,
) -> crate::Result<H::Output>
where
    Args: FromRequest,
    H: Handler<Args>,
{
    let args = match Args::from_request(&ctx, &mut body).await {
        Ok(x) => x,
        Err(err) => return Err(err.into()),
    };

    let res = handler.call(args).await;
    Ok(res)
}
