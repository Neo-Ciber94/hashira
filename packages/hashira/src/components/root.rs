use crate::{
    app::LayoutContext,
    components::{Content, Links, LiveReload, Main, Meta, Scripts, Title, WasmLoading},
};

/// Renders the default root component.
pub async fn root_layout(_ctx: LayoutContext) -> yew::Html {
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
                // Show a loader while the wasm is not ready
                <WasmLoading/>

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
