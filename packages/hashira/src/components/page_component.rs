use yew::{html::ChildrenProps, BaseComponent};

use crate::{app::RenderContext, error::BoxError, types::BoxFuture, web::{Response, Body}};

/// Represents a page of a web app.
pub trait PageComponent: BaseComponent {
    /// Returns an unique identifier for this component.
    fn id() -> &'static str {
        std::any::type_name::<Self>()
    }

    /// The route of this page.
    fn route() -> Option<&'static str>;

    /// A function that renders this page component.
    fn render<BASE>(ctx: RenderContext, body: Body) -> BoxFuture<Result<Response, BoxError>>
    where
        BASE: BaseComponent<Properties = ChildrenProps>;
}

// A handler that renders a page component.
pub mod handler {
    use crate::{
        app::RenderContext,
        web::{Body, FromRequest, IntoResponse, Response},
    };
    use futures::Future;

    /// Calls the render function of a handler.
    pub async fn call_render<H, Args>(
        ctx: RenderContext,
        mut body: Body,
        handler: H,
    ) -> crate::Result<Response>
    where
        H: RenderHandler<Args>,
        Args: FromRequest,
    {
        let args = match Args::from_request(&ctx, &mut body).await {
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
            Func: Fn(RenderContext, $($param),*) -> Fut + Clone + 'static,
            Fut::Output: IntoResponse,
            Fut: Future,
        {
            type Output = Fut::Output;
            type Future = Fut;

            #[inline]
            #[allow(non_snake_case)]
            fn call(&self, ctx: RenderContext, ($($param,)*): ($($param,)*)) -> Self::Future {
                (self)(ctx, $($param,)*)
            }
        }
    });

    impl_render_handler_tuple! {}
    impl_render_handler_tuple! { T1 }
    impl_render_handler_tuple! { T1 T2 }
    impl_render_handler_tuple! { T1 T2 T3 }
    impl_render_handler_tuple! { T1 T2 T3 T4 }
    impl_render_handler_tuple! { T1 T2 T3 T4 T5 }
    impl_render_handler_tuple! { T1 T2 T3 T4 T5 T6 }
    impl_render_handler_tuple! { T1 T2 T3 T4 T5 T6 T7 }
    impl_render_handler_tuple! { T1 T2 T3 T4 T5 T6 T7 T8 }
    impl_render_handler_tuple! { T1 T2 T3 T4 T5 T6 T7 T8 T9 }
    impl_render_handler_tuple! { T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 }
    impl_render_handler_tuple! { T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 }
    impl_render_handler_tuple! { T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 }
}

pub(crate) mod macros {
    /// Helper to implement `PageComponent`
    #[macro_export]
    macro_rules! impl_page_component {
        ($component:ty) => {
            impl $crate::components::PageComponent for $component {
                fn route() -> Option<&'static str> {
                    None
                }

                fn render<BASE>(
                    ctx: $crate::app::RenderContext,
                    _body: $crate::web::Body
                ) -> $crate::types::BoxFuture<Result<$crate::web::Response, $crate::error::BoxError>>
                where
                    BASE: ::yew::BaseComponent<Properties = ChildrenProps>,
                {
                    std::boxed::Box::pin(async move {
                        let res = ctx.render::<Self, BASE>().await;
                        Ok(res)
                    })
                }
            }
        };

        ($component:ty, $path:literal) => {
            impl $crate::components::PageComponent for $component {
                fn route() -> Option<&'static str> {
                    Some($path)
                }

                fn render<BASE>(
                    ctx: $crate::app::RenderContext,
                    _body: $crate::web::Body
                ) -> $crate::types::BoxFuture<Result<$crate::web::Response, $crate::error::BoxError>>
                where
                    BASE: ::yew::BaseComponent<Properties = ChildrenProps>,
                {
                    std::boxed::Box::pin(async move {
                        let res = ctx.render::<Self, BASE>().await;
                        Ok(res)
                    })
                }
            }
        };
    }
}
