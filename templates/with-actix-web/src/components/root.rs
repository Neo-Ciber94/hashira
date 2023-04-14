use hashira::{
    app::LayoutContext,
    components::{Content, Links, LiveReload, Main, Meta, Scripts, Title}, server::{PageLinks, LinkTag},
};
use yew::Html;

pub async fn root_layout(mut ctx: LayoutContext) -> Html {
    ctx.add_title("Hashira");
    ctx.add_links(PageLinks::new().insert(LinkTag::stylesheet("/static/global.css")));
    
    yew::html! {
        <html lang="en">
            <head>
                <Title/>
                <Meta/>
                <Links/>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1.0" />
            </head>
            <body>
                <Main>
                    <Content/>
                </Main>
                <Scripts/>
                <LiveReload/>
            </body>
        </html>
    }
}
