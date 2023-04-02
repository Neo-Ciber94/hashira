use yew::{Html, function_component};
use crate::components::{Content, Meta, Links, Scripts, HASHIRA_ROOT};

/// Default root component.
#[function_component]
pub fn RootLayout() -> Html {
    yew::html! {
        <html lang={"en"}>
            <head>
                <Meta/>
                <Links/>
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