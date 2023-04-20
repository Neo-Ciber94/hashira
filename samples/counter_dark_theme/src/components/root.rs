use hashira::{
    app::LayoutContext,
    components::{Content, Links, LiveReload, Main, Meta, Scripts, Title},
    server::{LinkTag, PageLinks},
    web::RequestExt,
};
use yew::Html;

pub async fn root_layout(mut ctx: LayoutContext) -> Html {
    ctx.title("Hashira");
    ctx.links(PageLinks::new().insert(LinkTag::stylesheet("/static/global.css")));

    let mut dark_class = None;
    if ctx
        .request()
        .cookie("dark")
        .map(|c| c.value() == "true")
        .unwrap_or_default()
    {
        dark_class = Some("dark");
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
