use hashira::{
    app::LayoutContext,
    components::{Content, Links, LiveReload, Main, Meta, Scripts, Title},
};
use yew::Html;

pub async fn root_layout(_: LayoutContext) -> Html {
    yew::html! {
        <html lang="en">
            <head>
                <Title/>
                <Meta/>
                <Links/>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1.0" />
            </head>
            <body class="dark">
                <Main>
                    <Content/>
                </Main>
                <Scripts/>
                <LiveReload/>
            </body>
        </html>
    }
}
