// #[action] <-- autogenerated route
// #[action("/route")]

use proc_macro2::TokenStream;
use syn::{parse::Parse, spanned::Spanned, ItemFn, LitStr};

#[derive(Clone)]
pub struct ActionAttr {
    route: Option<String>,
}

impl Parse for ActionAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let route: Option<LitStr> = input
            .parse()
            .map_err(|_| syn::Error::new(input.span(), "server actions should declare a route"))?;

        Ok(ActionAttr {
            route: route.map(|x| x.value()),
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

    let route = match attr.route {
        Some(route) => quote::quote! { #route },
        None => {
            // If not route was set, we generate one
            // `/_hashira/action/name_of_action_{random}`
            let name_snake_case = to_snake_case(&name.to_string());
            let generated_route = format!("/_hashira/action/{name_snake_case}");
            quote::quote! {
                #generated_route
            }
        }
    };

    // We rename the function to `_{name}`
    let mut new_item_fn = item_fn.clone();
    let new_item_fn_ident = syn::Ident::new(&format!("_{name}"), name.span());
    new_item_fn.sig.ident = new_item_fn_ident.clone();
    let placeholder = placeholder_item_fn(&new_item_fn);

    Ok(quote::quote! {
        #[allow(non_snake_case)]
        #[allow(non_camel_case_types)]
        #vis struct #name {
            _marker: ::std::marker::PhantomData<()>
        }

        const _: () = {
            #[allow(non_snake_case)]
            #[allow(non_camel_case_types)]
            #[cfg(not(feature = "client"))]
            #[allow(dead_code, unused_variables)]
            #new_item_fn

            #[allow(non_snake_case)]
            #[allow(non_camel_case_types)]
            #[cfg(feature = "client")]
            #[allow(dead_code)]
            #placeholder

            #[automatically_derived]
            impl ::hashira::actions::Action for #name {
                type Response = #ret;

                fn route() -> &'static str {
                   #route
                }

                fn call(ctx: ::hashira::app::RequestContext, body: &mut ::hashira::web::Body) -> ::hashira::types::BoxFuture<::hashira::Result<Self::Response>> {
                    let fut = ::hashira::actions::call_action(ctx, body, #new_item_fn_ident);
                    ::std::boxed::Box::pin(fut)
                }
            }
        };
    })
}

fn placeholder_item_fn(item_fn: &ItemFn) -> ItemFn {
    /*
    We replace the body of the function and remove all the parameters:
        #[action]
        fn SomeAction(form: From<MyStruct>, pool: Inject<MySqlPool>, method: Method) -> Result<Response> {
            // logic
        }

    This is replaced for:
        #[action]
        fn SomeAction() -> Result<Response> {
            unreachable!()
        }
    */

    let mut item_fn = item_fn.clone();
    item_fn.block = syn::parse_str("{ ::std::unreachable!() }").unwrap();
    item_fn.sig.inputs.clear();
    item_fn
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_upper = false;
    for c in s.chars() {
        if c.is_uppercase() {
            if !prev_upper {
                result.push('_');
            }
            prev_upper = true;
        } else {
            prev_upper = false;
        }
        result.push(c.to_ascii_lowercase());
    }
    result
}