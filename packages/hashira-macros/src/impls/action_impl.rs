// #[action("/route")]

use proc_macro2::TokenStream;
use syn::{parse::Parse, spanned::Spanned, ItemFn, LitStr};

#[derive(Debug, Clone)]
pub struct ActionAttr {
    route: String,
}

impl Parse for ActionAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let route: LitStr = input
            .parse()
            .map_err(|_| syn::Error::new(input.span(), "server actions should declare a route"))?;

        // FIXME Autogenerate a route base on the function name?
        Ok(ActionAttr {
            route: route.value(),
        })
    }
}

/// Implementation of `#[action]`
#[allow(clippy::redundant_clone)]
pub fn action_impl(attr: ActionAttr, item_fn: ItemFn) -> syn::Result<TokenStream> {
    if item_fn.sig.asyncness.is_none() {
        return Err(syn::Error::new(
            item_fn.span(),
            "server actions should be async",
        ));
    }

    let name = item_fn.sig.ident.clone();
    let vis = item_fn.vis.clone();
    let ret = match &item_fn.sig.output {
        syn::ReturnType::Default => syn::parse_str("()").unwrap(),
        syn::ReturnType::Type(_, ty) => ty.clone(),
    };

    let route = attr.route;

    // We rename the function to `_{name}`
    let mut new_item_fn = item_fn.clone();
    let new_item_fn_ident = syn::Ident::new(&format!("_{name}"), name.span());
    new_item_fn.sig.ident = new_item_fn_ident.clone();

    Ok(quote::quote! {
        #[allow(non_snake_case)]
        #[allow(non_camel_case_types)]
        #vis struct #name {
            _marker: ::std::marker::PhantomData<()>
        }

        const _: () = {
            #[allow(non_snake_case)]
            #[allow(non_camel_case_types)]
            #new_item_fn

            #[automatically_derived]
            impl ::hashira::actions::Action for #name {
                type Response = <#ret as ::hashira::web::IntoJsonResponse>::Data;

                fn route() -> &'static str {
                   #route
                }

                fn call(ctx: ::hashira::app::RequestContext) -> ::hashira::types::BoxFuture<::hashira::Result<Self::Response>> {
                    let fut = ::hashira::actions::call_action(ctx, #new_item_fn_ident);
                    ::std::boxed::Box::pin(fut)
                }
            }
        };
    })
}
