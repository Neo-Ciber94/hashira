# hashira

[![CI-badge]](ci) [![Latest Version]][crates.io] [![Docs][docs-badge]][docs-link]

[CI-badge]: https://github.com/Neo-Ciber94/hashira/actions/workflows/ci.yml/badge.svg
[ci]: https://github.com/Neo-Ciber94/hashira/actions/workflows/ci.yml

[latest version]: https://img.shields.io/crates/v/hashira.svg
[crates.io]: https://crates.io/crates/hashira

[docs-badge]: https://img.shields.io/badge/docs-hashira-blue.svg
[docs-link]: https://docs.rs/hashira/latest

A server side rendering framework built on top of yew.

## Getting started

To create a project with `hashira` you need to install the `CLI`.

```bash
cargo install --force hashira-cli
```

## Creating a new project

Using `hashira new` you can create a new project,
the CLI will prompt you with the template to use,
and the project name.

You can also use a shortcut to create a new project:

```bash
hashira new --{{template}} --name {{name}}
```

There are the templates available at the moment:

- actix-web
- axum
- rocket
- deno

More will be added in the future. Or if you want to create an adapter for your own, look at the code, most of the templates just use an adapter which starts the server, you can check the adapters in `/packages/adapters`.

## This project still on alpha

`hashira` still on alpha that means:

- Some features are incomplete
- Some features may be removed

## Features

### SSR (Server Side Rendering)

Allow to render your `yew` components on the server passing down the properties from server side.

```rs
async fn render_page(ctx: RenderContext) -> Result<Response> {
    let products = get_products().await?;
    let res = ctx.render_with_props(ProductsPageProps { products }).await;
    Ok(res)
}

#[page_component("/products", render = "render_page")]
fn ProductsPage(props: &ProductsPageProps) -> yew::Html {
    yew::html! {
        // ...
    }
}
```

### Server Actions

Execute code in your server from your components.

```rs
struct CreateProduct {
    name: String,
    price: i64,
    description: Option<String>
}

#[action("/api/products/create")]
async fn CreateProductAction(form: Form<CreateProduct>) -> Json<Product> {
    // server side logic
}

#[page_component("/products/add", render = "...")]
fn CreateProductPage() -> yew::Html {
    let action = use_action();

    if action.is_loading() {
        return yew::html! {
            "Creating product..."
        };
    }

    yew::html! {
        <Form<CreateProductAction> action={action.clone()}>
            <input name="name" required={true} />
            <input name="price" required={true} />
            <textarea name="description" rows={4}></textarea>
        </Form<CreateProductAction>>
    }
}
```

### Extractors

Render functions and Server actions allow to inject any parameter 
that implements `FromRequest`.

```rs
#[action("/api/products/create")]
async fn CreateProductAction(
    form: Form<CreateProduct>,
    headers: HeadersMap,
    method: Method,
    Inject(pool): Inject<MySqlPool>) -> Json<Product> {
    // server side logic
}

```
