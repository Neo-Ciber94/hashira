mod impls;
use impls::PageComponentAttr;
use proc_macro::TokenStream;

// FIXME: rename to `#[page]` ?

/// Provides an implementation of `PageComponent`.
/// 
/// # Usage
/// You need to pass the route of the page and an optional loader
/// which renders the page, if no loader is specified, a default loader
/// will be provided that renders the component providing `Default` of the props.
/// 
/// - `#[page_component("/route")]`
/// - `#[page_component("/route", loader = "path::to::function")]`
/// - `#[page_component(None, loader = "path::to::function")]`
/// 
/// # Example
/// 
/// ```rs,no_run
/// async fn render(ctx: RenderContext) -> Result<Response, Error> {
///     let res = ctx.render::<HelloPage, App>().await;
///     Ok(res) 
/// }
/// 
/// #[page_component("/hello", loader = "render")]
/// fn HelloPage() -> yew::Html {
///     yew::html! {
///         "Hello World!"
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn page_component(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_fn = syn::parse_macro_input!(item as syn::ItemFn);
    let attr = syn::parse_macro_input!(attr as PageComponentAttr);

    match impls::page_component_impl(attr, item_fn) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.into_compile_error().into(),
    }
}