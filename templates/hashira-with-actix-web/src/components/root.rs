use hashira::{
    app::LayoutContext,
    components::{Content, Links, LiveReload, Main, Meta, Scripts, Title}, web::RequestExt,
};
use serde::Deserialize;
use yew::Html;

#[derive(Debug, Deserialize)]
struct Theme {
    dark: bool,
}

pub async fn root_layout(ctx: LayoutContext) -> Html {
    let mut dark_class = None;
    if let Some(theme) = ctx.request().query_params::<Theme>().ok() {
        if theme.dark {
            dark_class = Some("dark");
        }
    }

    yew::html! {
        <html lang="en">
            <head>
                <Title/>
                <Meta/>
                <Links/>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1.0" />
            </head>
            <body class={yew::classes!(dark_class)}>
                <Main>
                    <Content/>
                </Main>
                <Scripts/>
                <LiveReload/>
            </body>
        </html>
    }
}
