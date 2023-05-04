use yew::{html::ChildrenProps, BaseComponent};

use crate::{app::RenderContext, error::Error, types::BoxFuture, web::Response};

/// Represents a page of a web app.
pub trait PageComponent: BaseComponent {
    /// Returns an unique identifier for this component.
    fn id() -> &'static str {
        std::any::type_name::<Self>()
    }

    /// The route of this page.
    fn route() -> Option<&'static str>;

    /// A function that renders this page component.
    fn render<BASE>(ctx: RenderContext) -> BoxFuture<Result<Response, Error>>
    where
        BASE: BaseComponent<Properties = ChildrenProps>;
}

// A handler that renders a page component.
pub mod handler {
    use crate::{
        app::RenderContext,
        web::{FromRequest, IntoResponse, Response},
    };
    use futures::Future;

    /// Calls the render function of a handler.
    pub async fn call_render<H, Args>(ctx: RenderContext, handler: H) -> crate::Result<Response>
    where
        H: RenderHandler<Args>,
        Args: FromRequest,
    {
        let args = match Args::from_request(&*ctx).await {
            Ok(x) => x,
            Err(err) => return Err(err.into()),
        };
        let ret = handler.call(ctx, args).await;
        let res = ret.into_response();
        Ok(res)
    }

    /// A function that renders a page component.
    pub trait RenderHandler<Args>: Clone + 'static {
        type Output: IntoResponse;
        type Future: Future<Output = Self::Output>;

        fn call(&self, ctx: crate::app::RenderContext, args: Args) -> Self::Future;
    }

    macro_rules! impl_render_handler_tuple ({ $($param:ident)* } => {
        impl<Func, Fut, $($param,)*> RenderHandler<($($param,)*)> for Func
        where
            Func: Fn(crate::app::RenderContext, $($param),*) -> Fut + Clone + 'static,
            Fut::Output: IntoResponse,
            Fut: Future,
        {
            type Output = Fut::Output;
            type Future = Fut;

            #[inline]
            #[allow(non_snake_case)]
            fn call(&self, ctx: crate::app::RenderContext, ($($param,)*): ($($param,)*)) -> Self::Future {
                (self)(ctx, $($param,)*)
            }
        }
    });

    impl_render_handler_tuple! {}
    impl_render_handler_tuple! { A }
    impl_render_handler_tuple! { A B }
    impl_render_handler_tuple! { A B C }
    impl_render_handler_tuple! { A B C D }
    impl_render_handler_tuple! { A B C D E }
    impl_render_handler_tuple! { A B C D E F }
    impl_render_handler_tuple! { A B C D E F G }
    impl_render_handler_tuple! { A B C D E F G H }
    impl_render_handler_tuple! { A B C D E F G H I }
    impl_render_handler_tuple! { A B C D E F G H I J }
    impl_render_handler_tuple! { A B C D E F G H I J K }
    impl_render_handler_tuple! { A B C D E F G H I J K L }
}
