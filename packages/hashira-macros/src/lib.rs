mod impls;
use impls::PageComponentAttr;
use proc_macro::TokenStream;

// FIXME: rename to `#[page]` ?
/// Converts a function to a `PageComponent`.
#[proc_macro_attribute]
pub fn page_component(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_fn = syn::parse_macro_input!(item as syn::ItemFn);
    let attr = syn::parse_macro_input!(attr as PageComponentAttr);

    match impls::page_component_impl(attr, item_fn) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.into_compile_error().into(),
    }
}