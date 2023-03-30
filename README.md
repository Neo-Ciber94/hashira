# hashira

A server side rendering framework built on top of yew.

## A prototype of the expected structure

```rust

fn main() {
    let ssr = Ssr::new()
        .layout(|ctx| {
            ctx.add_metadata(Metadata::new()
                .title("My app")
                .description("A cool sample app")
                .viewport("width=device-width, initial-scale=1"))

            html! {
                <html lang="en">
                    <body>
                        <Outlet/>
                    </body>
                <html>
            }
        })
        .page("/users", async |ctx| {
            let users = user_service.get_users().await;
            ctx.render_with_props<UserListPage>(users)
        })
        .page("/users/{id}", async |ctx| {
            let id = ctx.params.get::<u32>("id");
            let user = user_service.get_user_by_id(id).await;
            match user {
                Some(user) => ctx.render_with_props<UserPage>(user),
                None => ctx.render_error(404)
            }
        })

    // Setup server
    HttpServer::new(||
        App::new()
            .route("/{...params}", ssr.router())
            .route("/hello", hello)
            .fallback(|ctx: Context| ctx.render_error(404))
        )
        .listen(5000)
        .run()
        .await
}
```
