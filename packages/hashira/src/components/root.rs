use crate::components::{Content, Links, Main, Meta, Scripts, Title, LiveReload};
use yew::{function_component, Html};

/// Default root component.
///
/// This element only is rendered on the server side and defines the overall
/// structure of the html where the page will be rendered.
#[function_component]
pub fn RootLayout() -> Html {
    yew::html! {
        // Base <html>
        <html lang="en">
            // All the <head> elements
            <head>
                // A marker for <title> of the page
                <Title/>

                // A marker for the <meta> elements of the page
                <Meta/>

                // A marker for the <link> elements of the page
                <Links/>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1.0" />
            </head>

            // The <body>
            <body>
                // Renders a <main> element where the page will be rendered and hydrated
                <Main>
                    // Were the page will be inserted after rendered
                    <Content/>
                </Main>

                // Adds the page <script> elements and also the script to hydrate the page
                // to provide interactivity
                <Scripts/>

                // Script used for reloading the client during development
                // in release mode if renders nothing
                <LiveReload/>
            </body>
        </html>
    }
}
