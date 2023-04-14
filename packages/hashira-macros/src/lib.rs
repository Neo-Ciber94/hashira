mod impls;
use proc_macro::TokenStream;

// FIXME: rename to `#[page]` ?
#[proc_macro_attribute]
pub fn page_component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemFn);
    impls::page_component_impl(input).into()
}
