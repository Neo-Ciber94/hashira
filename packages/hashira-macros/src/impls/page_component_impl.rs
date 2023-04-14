use proc_macro2::TokenStream;
use syn::ItemFn;

/// Implementation of `#[page_component]`
pub fn page_component_impl(item_fn: ItemFn) -> TokenStream {
    let component = item_fn.sig.ident.clone();
    let name = component.to_string();

    // TODO: the id should always produce the same result across server and client
    // if the code is the same. Currently we are exposing information 
    // about the module where the component is declared, which may or not be relevant.
    // this implementation may return a similar result than `std::any::typename` 
    // which the documentation says the return may not be stable.
    quote::quote! {
        #[automatically_derived]
        impl hashira::components::PageComponent for #component {
            fn id() -> &'static str {
                std::concat!(std::module_path!(), "::", #name)
            }
        }

        #[yew::function_component]
        #[allow(non_camel_case_types)]
        #item_fn
    }
}
