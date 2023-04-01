use super::{error::RenderError, Metadata, PageLinks, PageScripts};
use crate::components::{
    Content, Links, Meta, Page, PageData, PageProps, RenderFn, Scripts, HASHIRA_CONTENT_MARKER,
    HASHIRA_LINKS_MARKER, HASHIRA_META_MARKER, HASHIRA_PAGE_DATA, HASHIRA_ROOT,
    HASHIRA_SCRIPTS_MARKER,
};
use serde::Serialize;
use yew::{
    function_component,
    html::{ChildrenProps, ChildrenRenderer},
    BaseComponent, Html, LocalServerRenderer, ServerRenderer,
};

pub struct RenderPageOptions {
    // Represents the shell where the page will be rendered
    pub(crate) layout: String,

    // The `<meta>` tags of the page to render
    pub(crate) metadata: Metadata,

    // the <link> tags of the page to render
    pub(crate) links: PageLinks,

    // the <script> tags of the page to render
    pub(crate) scripts: PageScripts,
}

pub async fn render_page_to_html<COMP, ROOT>(
    props: COMP::Properties,
    options: RenderPageOptions,
) -> Result<String, RenderError>
where
    COMP: BaseComponent,
    COMP::Properties: Serialize + Send + Clone,
    ROOT: BaseComponent<Properties = ChildrenProps>,
{
    let RenderPageOptions {
        layout,
        metadata,
        links,
        scripts,
    } = options;

    // The base layout
    let mut result_html = layout;

    if !result_html.contains(HASHIRA_ROOT) {
        return Err(RenderError::NoRoot);
    }

    // Render the page

    let render = RenderFn::new({
        let props = props.clone();
        move || {
            let props = props.clone();
            yew::html! {
                <ROOT>
                    <COMP ..props/>
                </ROOT>
            }
        }
    });

    let renderer = ServerRenderer::<Page>::with_props(move || PageProps { render });
    let page_html = renderer.render().await;

    // Build the root html
    result_html = result_html.replace(HASHIRA_CONTENT_MARKER, &page_html);

    // Insert the <meta> elements from `struct Metadata`
    insert_metadata(&mut result_html, metadata);

    // Insert the <link> elements from `struct PageLinks`
    insert_links(&mut result_html, links);

    // Insert the <script> elements from `struct PageScripts`
    insert_scripts::<COMP>(&mut result_html, scripts, props)?;

    Ok(result_html)
}

fn insert_metadata(html: &mut String, metadata: Metadata) {
    let mut tags_html = metadata
        .meta_tags()
        .map(|meta| meta.to_string())
        .collect::<Vec<_>>();

    // Add <title> from <meta name="title" ...>
    if let Some(meta) = metadata.meta_tags().find(|x| x.name() == "title") {
        let content = meta.attrs().get("content").unwrap();
        tags_html.push(format!("<title>{}</title>", content));
    }

    let tags = tags_html.join("\n");

    *html = html.replace(HASHIRA_META_MARKER, &tags);
}

fn insert_links(html: &mut String, links: PageLinks) {
    let mut tags_html = links.iter().map(|x| x.to_string()).collect::<Vec<_>>();

    // We do this to prevent adding the bundle when building the layout,
    // TODO: Find a cleaner way to do this
    if crate::is_initialized() {
        // Add wasm bundle
        let crate_name = std::env::var("CARGO_PKG_NAME").unwrap();
        tags_html.push(format!(
            r#"
            <link rel="preload" href="/static/{crate_name}_bg.wasm" as="fetch" type="application/wasm" crossorigin=""/>
            <link rel="modulepreload" href="/static/{crate_name}.js" />
        "#));

        tags_html.push(format!(
            r#"<script type="module">
                    import init from "/static/{crate_name}.js";
                    init("/static/{crate_name}_bg.wasm");
                </script>
            "#
        ));
    }

    let links = tags_html.join("\n");
    *html = html.replace(HASHIRA_LINKS_MARKER, &links);
}

fn insert_scripts<COMP>(
    html: &mut String,
    scripts: PageScripts,
    props: COMP::Properties,
) -> Result<(), RenderError>
where
    COMP: BaseComponent,
    COMP::Properties: Serialize,
{
    let mut tags_html = scripts.iter().map(|x| x.to_string()).collect::<Vec<_>>();

    // Add the component data to the page
    if crate::is_initialized() {
        let props = serde_json::to_value(props).map_err(RenderError::InvalidProps)?;

        let page_data = PageData {
            component_name: std::any::type_name::<COMP>().to_string(),
            props,
        };

        let json_data = serde_json::to_string(&page_data).map_err(RenderError::InvalidProps)?;
        let page_data_script = format!(
            "<script type=\"application/json\" id={HASHIRA_PAGE_DATA}>{json_data}</script>"
        );
        tags_html.push(page_data_script);
    }

    let links = tags_html.join("\n");
    *html = html.replace(HASHIRA_SCRIPTS_MARKER, &links);
    Ok(())
}

#[function_component]
pub fn DefaultLayout() -> Html {
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

pub async fn render_to_static_html<F>(f: F) -> String
where
    F: FnOnce() -> Html + 'static,
{
    #[function_component]
    fn Dummy(props: &ChildrenProps) -> Html {
        yew::html! {
            <>{for props.children.iter()}</>
        }
    }

    // FIXME: Found a  way to use the `ServerRenderer` instead?
    let renderer = LocalServerRenderer::<Dummy>::with_props(ChildrenProps {
        children: ChildrenRenderer::new(vec![f()]),
    });

    renderer.hydratable(false).render().await
}
