use crate::components::{Content, Links, Meta, Scripts, HASHIRA_ROOT};
use yew::{function_component, Html};

/// Default root component.
#[function_component]
pub fn RootLayout() -> Html {
    yew::html! {
        <html lang={"en"}>
            <head>
                <Meta/>
                <Links/>
                <meta name="viewport" content="width=device-width, initial-scale=1.0" />
            </head>
            <body>
                <main id={HASHIRA_ROOT}>
                    <Content/>
                </main>
                <Scripts/>
            </body>
        </html>
    }
}
