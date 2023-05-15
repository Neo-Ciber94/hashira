use proc_macro2::{Ident, Span, TokenStream};
use syn::{parse::Parse, ItemFn, LitStr, Result};

// #[page_component("/route")]
// #[page_component("/route", render = "path::to::function")]
// #[page_component(None, render = "path::to::function")]

#[derive(Clone)]
pub struct PageComponentAttr {
    route: Option<LitStr>,
    render: Option<Ident>,
}

impl Parse for PageComponentAttr {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        const EXPECTED_ROUTE: &str =
            "`PageComponent` require a route, provide a string literal or `None`";

        let route = {
            if input.peek(syn::Ident) {
                let none: Ident = input.parse()?;
                if none == "None" {
                    None
                } else {
                    return Err(syn::Error::new(input.span(), EXPECTED_ROUTE));
                }
            } else {
                let lit: LitStr = input
                    .parse()
                    .map_err(|_| syn::Error::new(input.span(), EXPECTED_ROUTE))?;
                Some(lit)
            }
        };

        let _comma: Option<syn::Token![,]> = input.parse()?;

        if _comma.is_none() {
            if !input.is_empty() {
                return Err(input.error(
                    "expected #[page_component(\"/route\", render = \"path::to::render\")]",
                ));
            }

            return Ok(PageComponentAttr {
                route,
                render: None,
            });
        }

        let ident_span = input.span();
        let ident: syn::Path = input.parse()?;

        if !ident.is_ident("render") {
            return Err(syn::Error::new(
                ident_span,
                "invalid signature, expected: #[page_component(render = \"path::to::render\")]",
            ));
        }

        let _equals: syn::Token![=] = input.parse()?;
        let render_str: LitStr = input.parse()?;
        let render = Ident::new(&render_str.value(), Span::call_site());

        Ok(PageComponentAttr {
            route,
            render: Some(render),
        })
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

    let render = match attr.render {
        Some(render_fn) => {
            quote::quote! {
                let fut = ::hashira::components::handler::call_render(ctx, body, #render_fn);
                std::boxed::Box::pin(fut)
            }
        }
        None => {
            // TODO: Use FutureExt::map
            quote::quote! {
                std::boxed::Box::pin(async move {
                    let res = ctx.render::<Self, BASE>().await;
                    Ok(res)
                })
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

            fn render<BASE>(ctx: ::hashira::app::RenderContext, body: ::hashira::web::Body)
                -> ::hashira::types::BoxFuture<std::result::Result<::hashira::web::Response, ::hashira::error::BoxError>>
                where
                    BASE: yew::BaseComponent<Properties = yew::html::ChildrenProps>,{
                #render
            }
        }

        #[yew::function_component]
        #[allow(non_camel_case_types)]
        #item_fn
    })
}
