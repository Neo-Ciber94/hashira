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
    fn loader<BASE>(ctx: RenderContext) -> BoxFuture<Result<Response, Error>>
    where
        BASE: BaseComponent<Properties = ChildrenProps>;
}

mod x {

    use bytes::Bytes;
    use futures::Future;
    use http::{HeaderMap, Method};

    use yew::function_component;

    use crate::web::{FromRequest, IntoResponse, Response};

    use super::PageComponent;

    #[function_component]
    fn TestPage() -> yew::Html {
        yew::html! {}
    }

    impl PageComponent for TestPage {
        fn route() -> Option<&'static str> {
            Some("/test")
        }

        fn loader<BASE>(
            ctx: crate::app::RenderContext,
        ) -> crate::types::BoxFuture<Result<crate::web::Response, crate::error::Error>>
        where
            BASE: yew::BaseComponent<Properties = yew::html::ChildrenProps>,
        {
            let fut = call_handler(ctx, render);
            Box::pin(fut)
        }
    }

    async fn call_handler<H, Args>(
        ctx: crate::app::RenderContext,
        handler: H,
    ) -> crate::Result<Response>
    where
        H: RenderHandler<Args>,
        Args: FromRequest,
        H::Output: IntoResponse,
    {
        let args = match Args::from_request(&*ctx).await {
            Ok(x) => x,
            Err(err) => return Err(err.into()),
        };
        let ret = handler.call(ctx, args).await;
        let res = ret.into_response();
        Ok(res)
    }

    async fn render(
        ctx: crate::app::RenderContext,
        method: Method,
        headers: HeaderMap,
        body: Bytes,
    ) -> Response {
        todo!()
    }

    /// A request handler.
    pub trait RenderHandler<Args>: Clone + 'static {
        type Output;
        type Future: Future<Output = Self::Output>;

        fn call(&self, ctx: crate::app::RenderContext, args: Args) -> Self::Future;
    }

    macro_rules! impl_render_handler_tuple ({ $($param:ident)* } => {
        impl<Func, Fut, $($param,)*> RenderHandler<($($param,)*)> for Func
        where
            Func: Fn(crate::app::RenderContext, $($param),*) -> Fut + Clone + 'static,
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
