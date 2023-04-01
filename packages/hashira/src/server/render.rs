use super::{error::RenderError, Metadata, PageLinks, PageScripts};
use crate::components::{
    AppPage, AppPageProps, Content, Links, Meta, RenderFn, Scripts, HASHIRA_CONTENT_MARKER,
    HASHIRA_LINKS_MARKER, HASHIRA_META_MARKER, HASHIRA_ROOT, HASHIRA_SCRIPTS_MARKER,
};
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

pub async fn render_page_to_html<COMP>(
    props: COMP::Properties,
    options: RenderPageOptions,
) -> Result<String, RenderError>
where
    COMP: BaseComponent,
    COMP::Properties: Send + Clone,
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
    let render = RenderFn::new(move || {
        let props = props.clone();
        yew::html! {
            <COMP ..props/>
        }
    });

    let renderer = ServerRenderer::<AppPage>::with_props(move || AppPageProps { render });
    let page_html = renderer.render().await;

    // Build the root html
    result_html = result_html.replace(HASHIRA_CONTENT_MARKER, &page_html);

    // Insert the <meta> elements from `struct Metadata`
    insert_metadata(&mut result_html, metadata);

    // Insert the <link> elements from `struct PageLinks`
    insert_links(&mut result_html, links);

    // Insert the <script> elements from `struct PageScripts`
    insert_scripts(&mut result_html, scripts);

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

    let links = tags_html.join("\n");
    *html = html.replace(HASHIRA_LINKS_MARKER, &links);
}

fn insert_scripts(html: &mut String, scripts: PageScripts) {
    let tags_html = scripts.iter().map(|x| x.to_string()).collect::<Vec<_>>();
    let links = tags_html.join("\n");
    *html = html.replace(HASHIRA_SCRIPTS_MARKER, &links);
}

#[function_component]
pub fn DefaultLayout() -> Html {
    yew::html! {
        <html lang={"en"}>
            <head>
                <Meta/>
                <Links/>
            </head>
            <body id={HASHIRA_ROOT}>
                <Content/>
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
