use crate::components::{Content, Links, Main, Meta, Scripts, Title};
use yew::{function_component, Html};

/// Default root component.
///
/// This element only is rendered on the server side and defines the overall
/// structure of the html where the page will be rendered.
#[function_component]
pub fn RootLayout() -> Html {
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
            </body>
        </html>
    }
}
