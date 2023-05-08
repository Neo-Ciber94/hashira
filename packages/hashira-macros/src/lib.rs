mod impls;
use impls::{ActionAttr, PageComponentAttr};
use proc_macro::TokenStream;
use syn::spanned::Spanned;

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

/// Mark a function as a server action.
#[proc_macro_attribute]
pub fn action(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_fn = syn::parse_macro_input!(item as syn::ItemFn);
    let attr = syn::parse_macro_input!(attr as ActionAttr);

    match impls::action_impl(attr, item_fn) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

/// Marks a method as a page component render function.
#[proc_macro_attribute]
pub fn render(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemFn);

    if input.sig.asyncness.is_none() {
        return syn::Error::new(input.span(), "render functions must be async")
            .into_compile_error()
            .into();
    }

    let vis = input.vis.clone();
    let name = input.sig.ident.clone();
    let attrs = input.attrs.clone();

    let result = quote::quote! {
        #[cfg(not(feature = "client"))]
        #[allow(dead_code, unused_variables)]
        #input
        
        #(#attrs)*
        #[cfg(feature = "client")]
        #[allow(dead_code, unused_variables)]
        #vis async fn #name(_ctx: ::hashira::app::RenderContext) -> ::hashira::Result<::hashira::web::Response> {
            std::unreachable!()
        }
    };

    result.into()
}
