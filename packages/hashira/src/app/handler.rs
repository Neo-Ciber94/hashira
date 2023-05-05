use futures::Future;

/// A request handler.
pub trait Handler<Args>: Clone + 'static {
    type Output;
    type Future: Future<Output = Self::Output>;

    fn call(&self, args: Args) -> Self::Future;
}

macro_rules! impl_handler_tuple ({ $($param:ident)* } => {
    impl<Func, Fut, $($param,)*> Handler<($($param,)*)> for Func
    where
        Func: Fn($($param),*) -> Fut + Clone + 'static,
        Fut: Future,
    {
        type Output = Fut::Output;
        type Future = Fut;

        #[inline]
        #[allow(non_snake_case)]
        fn call(&self, ($($param,)*): ($($param,)*)) -> Self::Future {
            (self)($($param,)*)
        }
    }
});

impl_handler_tuple! {}
impl_handler_tuple! { A }
impl_handler_tuple! { A B }
impl_handler_tuple! { A B C }
impl_handler_tuple! { A B C D }
impl_handler_tuple! { A B C D E }
impl_handler_tuple! { A B C D E F }
impl_handler_tuple! { A B C D E F G }
impl_handler_tuple! { A B C D E F G H }
impl_handler_tuple! { A B C D E F G H I }
impl_handler_tuple! { A B C D E F G H I J }
impl_handler_tuple! { A B C D E F G H I J K }
impl_handler_tuple! { A B C D E F G H I J K L }
