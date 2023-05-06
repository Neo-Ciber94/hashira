use crate::{
    app::{Handler, RequestContext},
    web::{FromRequest, IntoJsonResponse, Response},
};

/// Calls an action handler.
pub async fn call_action<H, Args>(
    ctx: RequestContext,
    handler: H,
) -> crate::Result<Response<<H::Output as IntoJsonResponse>::Data>>
where
    Args: FromRequest,
    H: Handler<Args>,
    H::Output: IntoJsonResponse,
{
    let args = match Args::from_request(&ctx).await {
        Ok(x) => x,
        Err(err) => return Err(err.into()),
    };

    let res = handler.call(args).await.into_json_response()?;
    Ok(res)
}
