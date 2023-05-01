use hashira::{
    app::LayoutContext,
    components::{Content, Links, LiveReload, Main, Meta, Scripts, Title}, server::{PageLinks, LinkTag},
};
use yew::Html;

pub async fn root_layout(mut ctx: LayoutContext) -> Html {
    ctx.title("Hashira e-Commerce");
    ctx.links(PageLinks::new().insert(LinkTag::stylesheet("/static/global.css")));
    
    yew::html! {
        <html lang="en">
            <head>
                <Title/>
                <Meta/>
                <Links/>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1.0" />
            </head>
            <body class="bg-slate-800">
                <Main>
                    <Content/>
                </Main>
                <Scripts/>
                <LiveReload/>
            </body>
        </html>
    }
}
