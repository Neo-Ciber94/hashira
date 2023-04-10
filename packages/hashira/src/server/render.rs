use std::sync::Arc;

use super::{error::RenderError, Metadata, PageLinks, PageScripts};
use crate::app::error_router::ErrorRouter;
use crate::app::router::ClientRouter;
use crate::components::{
    Page, PageData, PageError, PageProps, HASHIRA_CONTENT_MARKER, HASHIRA_LINKS_MARKER,
    HASHIRA_META_MARKER, HASHIRA_PAGE_DATA, HASHIRA_ROOT, HASHIRA_SCRIPTS_MARKER,
    HASHIRA_TITLE_MARKER,
};
use crate::error::ResponseError;
use serde::Serialize;
use yew::{
    function_component,
    html::{ChildrenProps, ChildrenRenderer},
    BaseComponent, Html, LocalServerRenderer, ServerRenderer,
};

pub struct RenderPageOptions {
    // The title of the page
    pub(crate) title: Option<String>,

    // The current route path of the component to render
    pub(crate) path: String,

    // An error that occurred in the route
    pub(crate) error: Option<ResponseError>,

    // The router used to render the page
    pub(crate) client_router: ClientRouter,

    // The router used to render errors
    pub(crate) error_router: Arc<ErrorRouter>,

    // Represents the shell where the page will be rendered
    pub(crate) index_html: String,

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
        title,
        path,
        error,
        index_html,
        metadata,
        links,
        scripts,
        client_router,
        error_router: client_error_router,
    } = options;

    // The base layout
    let mut result_html = index_html;

    if !result_html.contains(HASHIRA_ROOT) {
        return Err(RenderError::NoRoot);
    }

    let props_json = serde_json::to_value(props).map_err(RenderError::InvalidProps)?;

    let page_error = error.map(|e| PageError {
        status: e.status(),
        message: e.message().map(|s| s.to_owned()),
    });

    let page_data = PageData {
        component_name: std::any::type_name::<COMP>().to_string(),
        props: props_json.clone(),
        path: path.clone(),
        error: page_error.clone(),
    };

    let page_props = PageProps {
        path: path.clone(),
        error: page_error,
        props_json: props_json.clone(),
        client_router,
        error_router: client_error_router,
    };

    let renderer = ServerRenderer::<Page<ROOT>>::with_props(move || page_props);
    let page_html = renderer.render().await;

    // Build the root html
    result_html = result_html.replace(HASHIRA_CONTENT_MARKER, &page_html);

    // Insert the <title> element
    insert_title(&mut result_html, title);

    // Insert the <meta> elements from `struct Metadata`
    insert_metadata(&mut result_html, metadata);

    // Insert the <link> elements from `struct PageLinks`
    insert_links(&mut result_html, links);

    // Insert the <script> elements from `struct PageScripts`
    insert_scripts::<COMP>(&mut result_html, scripts, page_data)?;

    // Prettify html
    #[cfg(not(target_arch = "wasm32"))]
    #[cfg(debug_assertions)]
    {
        prettify_html(&mut result_html);
    }

    Ok(result_html)
}

fn insert_title(html: &mut String, title: Option<String>) {
    if let Some(title) = title {
        let tag = format!("<title>{title}</title>");
        *html = html.replace(HASHIRA_TITLE_MARKER, &tag);
    }
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
    let links = tags_html.join("\n");
    *html = html.replace(HASHIRA_LINKS_MARKER, &links);
}

fn insert_scripts<COMP>(
    html: &mut String,
    scripts: PageScripts,
    page_data: PageData,
) -> Result<(), RenderError>
where
    COMP: BaseComponent,
    COMP::Properties: Serialize,
{
    let mut tags_html = scripts.iter().map(|x| x.to_string()).collect::<Vec<_>>();

    // Adds the page data
    let json_data = serde_json::to_string(&page_data).map_err(RenderError::InvalidProps)?;
    tags_html.push(format!(
        "<script type=\"application/json\" id={HASHIRA_PAGE_DATA}>{json_data}</script>"
    ));

    // Adds the wasm bundle
    if let Some(crate_name) = crate::env::get_wasm_name() {
        let static_dir = crate::env::get_static_dir();

        tags_html.push(format!(
            r#"
            <script type="module">
                import init, {{ hydrate }} from "{static_dir}/{crate_name}.js";
                init("{static_dir}/{crate_name}_bg.wasm").then(hydrate);
            </script>
        "#
        ));
    }

    let links = tags_html.join("\n");
    *html = html.replace(HASHIRA_SCRIPTS_MARKER, &links);
    Ok(())
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

#[cfg(not(target_arch = "wasm32"))]
#[cfg(debug_assertions)]
fn prettify_html(html: &mut String) {
    use lol_html::{HtmlRewriter, Settings};

    let mut output = vec![];
    let mut rewriter =
        HtmlRewriter::new(Settings::default(), |c: &[u8]| output.extend_from_slice(c));

    if let Err(err) = rewriter.write(html.as_bytes()) {
        log::warn!("Failed to write html for prettify: {err}");
        return;
    }

    if let Err(err) = rewriter.end() {
        log::warn!("Failed to close html writer for prettify: {err}");
        return;
    }

    match String::from_utf8(output) {
        Ok(pretty_html) => {
            *html = pretty_html;
        }
        Err(err) => {
            log::warn!("Failed to write pretty html: {err}");
        }
    }
}