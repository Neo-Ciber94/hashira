use super::{error::RenderError, Metadata, PageLinks, PageScripts};
use crate::components::{
    AppPage, AppPageProps, Content, Links, Meta, Scripts, HASHIRA_CONTENT_MARKER, HASHIRA_LINKS_MARKER,
    HASHIRA_META_MARKER, HASHIRA_ROOT, HASHIRA_SCRIPTS_MARKER, RenderFn,
};
use yew::{function_component, BaseComponent, Html, ServerRenderer};

pub struct RenderPageOptions {
    // Represents the shell where the page will be rendered
    layout: String,

    // The `<meta>` tags of the page to render
    metadata: Metadata,

    // the <link> tags of the page to render
    links: PageLinks,

    // the <script> tags of the page to render
    scripts: PageScripts,
}

pub async fn render_page_to_html<Layout, COMP>(
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

    if result_html.contains(HASHIRA_ROOT) {
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
    let tags_html = metadata
        .meta_tags()
        .map(|meta| meta.to_string())
        .collect::<Vec<_>>();

    let tags = tags_html.join("\n");
    *html = html.replace(HASHIRA_META_MARKER, &tags);
}

fn insert_links(html: &mut String, links: PageLinks) {
    let tags_html = links.iter().map(|x| x.to_string()).collect::<Vec<_>>();

    let tags = tags_html.join("\n");
    *html = html.replace(HASHIRA_LINKS_MARKER, &tags);
}

fn insert_scripts(html: &mut String, scripts: PageScripts) {
    let tags_html = scripts.iter().map(|x| x.to_string()).collect::<Vec<_>>();

    let tags = tags_html.join("\n");
    *html = html.replace(HASHIRA_SCRIPTS_MARKER, &tags);

    // TODO: Insert the script for hydration
}

#[function_component]
fn DefaultLayout() -> Html {
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
