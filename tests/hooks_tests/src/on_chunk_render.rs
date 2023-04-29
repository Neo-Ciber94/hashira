use crate::utils::{create_app, start_server_in_random_port, ServerHandle};
use hashira::{app::LayoutContext, events::Hooks};

#[tokio::test]
async fn on_chunk_render_test() {
    let service = create_app()
        .layout(root_layout)
        .hooks(Hooks::new().on_chunk_render(|html: String, _|{
            Ok(html.replace("body-marker", "hashira-321"))
        }))
        .build();

    let ServerHandle {
        shutdown,
        host,
        port,
    } = start_server_in_random_port(service).await;

    let res = crate::utils::get(&format!("http://{host}:{port}/home")).await;
    crate::utils::assert_content_type(&res, "text/html");

    let html = res.text().await.unwrap();
    assert!(html.contains("hashira-321"));

    shutdown.send(()).unwrap();
}

async fn root_layout(_: LayoutContext) -> yew::Html {
    use hashira::components::*;

    yew::html! {
        <html lang="en">
            <head>
                <Title/>
                <Meta/>
                <Links/>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1.0" />
            </head>
            <body data="body-marker">
                <Main>
                    <Content/>
                </Main>
                <Scripts/>
                <LiveReload/>
            </body>
        </html>
    }
}
