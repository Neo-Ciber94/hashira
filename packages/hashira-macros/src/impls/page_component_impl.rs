use proc_macro2::{Ident, Span, TokenStream};
use syn::{parse::Parse, ItemFn, LitStr, Result};

#[derive(Debug, Clone)]
pub struct PageComponentAttr {
    route: LitStr,
    loader: Option<Ident>,
}

impl Parse for PageComponentAttr {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let route: LitStr = input.parse()?;

        let _comma: Option<syn::Token![,]> = input.parse()?;

        if _comma.is_none() {
            if !input.is_empty() {
                return Err(input.error(
                    "expected #[page_component(\"/route\", loader = \"path::to::loader\")]",
                ));
            }

            return Ok(PageComponentAttr {
                route,
                loader: None,
            });
        }

        let ident_span = input.span();
        let ident: syn::Path = input.parse()?;

        if ident != syn::parse_str("loader").unwrap() {
            return Err(syn::Error::new(
                ident_span,
                "invalid signature, expected: #[page_component(loader = \"path::to::loader\")]",
            ));
        }

        let _equals: syn::Token![=] = input.parse()?;
        let loader_str: LitStr = input.parse()?;
        let loader = Ident::new(&loader_str.value(), Span::call_site());

        return Ok(PageComponentAttr {
            route,
            loader: Some(loader),
        });
    }
}

/// Implementation of `#[page_component]`
#[allow(clippy::redundant_clone)]
pub fn page_component_impl(attr: PageComponentAttr, item_fn: ItemFn) -> syn::Result<TokenStream> {
    let component = item_fn.sig.ident.clone();
    let name = component.to_string();

    let route = {
        let lit_str = attr.route;
        quote::quote! { Some(#lit_str) }
    };

    let loader = match attr.loader {
        Some(path) => {
            quote::quote! {
                let fut = #path (ctx);
                std::boxed::Box::pin(fut)
            }
        }
        None => {
            quote::quote! {
                let res = ctx.render::<Self, BASE>();
                std::boxed::Box::pin(fut)
            }
        }
    };

    // TODO: the id should always produce the same result across server and client
    // if the code is the same. Currently we are exposing information
    // about the module where the component is declared, which may or not be relevant.
    // this implementation may return a similar result than `std::any::typename`
    // which the documentation says the return may not be stable.
    Ok(quote::quote! {
        #[automatically_derived]
        impl ::hashira::components::PageComponent for #component {
            fn id() -> &'static str {
                std::concat!(std::module_path!(), "::", #name)
            }

            fn route() -> Option<&'static str> {
                #route
            }

            fn loader<BASE>(ctx: ::hashira::app::RenderContext)
                -> ::hashira::types::BoxFuture<std::result::Result<::hashira::web::Response, ::hashira::error::Error>>
                where
                    BASE: yew::BaseComponent<Properties = yew::html::ChildrenProps>,{
                #loader
            }
        }

        #[yew::function_component]
        #[allow(non_camel_case_types)]
        #item_fn
    })
}
