use super::{error::RenderError, Metadata, PageLinks, PageScripts};
use crate::app::error_router::ErrorRouter;
use crate::app::page_head::PageHead;
use crate::app::router::PageRouterWrapper;
use crate::app::RequestContext;
use crate::components::id::PageId;
use crate::components::{
    Page, PageComponent, PageData, PageError, PageProps, HASHIRA_CONTENT_MARKER,
    HASHIRA_LINKS_MARKER, HASHIRA_META_MARKER, HASHIRA_PAGE_DATA, HASHIRA_ROOT,
    HASHIRA_SCRIPTS_MARKER, HASHIRA_TITLE_MARKER,
};
use crate::context::ServerContext;
use crate::error::{Error, ResponseError};
use crate::types::TryBoxStream;
use bytes::Bytes;
use futures::{stream, StreamExt, TryStreamExt};
use serde::Serialize;
use std::sync::Arc;
use yew::{
    function_component,
    html::{ChildrenProps, ChildrenRenderer},
    BaseComponent, Html, ServerRenderer,
};

pub(crate) struct RenderPageOptions {
    // Represents the shell where the page will be rendered
    pub index_html: String,

    // Contains the the `<head>` elements
    pub head: PageHead,

    // The context of the current request
    pub request_context: RequestContext,

    // An error that occurred in the route
    pub error: Option<ResponseError>,

    // The router used to render the page
    pub router: PageRouterWrapper,

    // The router used to render errors
    pub error_router: Arc<ErrorRouter>,
}

struct BeforeContentElements {
    title: Option<String>,
    metadata: Metadata,
    links: PageLinks,
}

struct AfterContentElements {
    scripts: PageScripts,
}

/// Renders the given component inside the given root as a stream of bytes.
pub(crate) async fn render_page_to_stream<COMP, ROOT>(
    props: COMP::Properties,
    options: RenderPageOptions,
) -> Result<TryBoxStream<Bytes>, RenderError>
where
    COMP: PageComponent,
    COMP::Properties: Serialize + Send + Clone,
    ROOT: BaseComponent<Properties = ChildrenProps>,
{
    let RenderPageOptions {
        head,
        error,
        index_html,
        router,
        error_router,
        request_context,
    } = options;

    // The base layout
    let result_html = index_html;

    if !result_html.contains(HASHIRA_ROOT) {
        return Err(RenderError::NoRoot);
    }

    let props_json = serde_json::to_value(props).map_err(RenderError::InvalidProps)?;
    let component_id = PageId::of::<COMP>();
    let page_error = error.map(|e| PageError {
        status: e.status(),
        message: e.message().map(|s| s.to_owned()),
    });

    // TODO: Include page Uri and params?
    // The data inserted in the html
    let page_data = PageData {
        id: component_id,
        props: props_json,
        uri: request_context.request().uri().clone(),
        error: page_error,
        params: request_context.params().clone(),
    };

    // The props passed to the container page
    let page_props = PageProps {
        page_data: page_data.clone(),
        router,
        error_router,

        // Unnecessary?
        server_context: ServerContext::new(Some(request_context)),
    };

    let (title, metadata, links, scripts) = head.into_parts();
    let before_content = BeforeContentElements {
        title,
        metadata,
        links,
    };
    let after_content = AfterContentElements { scripts };

    // We split the content to render
    let (before_content_html, after_content_html) = result_html
        .split_once(HASHIRA_CONTENT_MARKER)
        .map(|(a, b)| (a.to_owned(), b.to_owned()))
        .unwrap();

    // Render the page as a stream
    let renderer = ServerRenderer::<Page<ROOT>>::with_props(move || page_props);
    let page_html = renderer.render_stream().map(Result::<_, Error>::Ok);

    // We chain all the produced streams together
    let html_stream = stream::once(async move {
        // Before content
        render_before_content_markers(before_content_html, before_content).map_err(|e| e.into())
    })
    // content
    .chain(page_html)
    .chain(stream::once(async move {
        // After content
        render_after_content_markers::<COMP>(after_content_html, after_content, page_data)
            .map_err(|e| e.into())
    }))
    .map_ok(Bytes::from);

    Ok(Box::pin(html_stream))
}

/// Renders the given component inside the given root as a html string.
pub(crate) async fn render_page_to_html<COMP, ROOT>(
    props: COMP::Properties,
    options: RenderPageOptions,
) -> Result<String, RenderError>
where
    COMP: PageComponent,
    COMP::Properties: Serialize + Send + Clone,
    ROOT: BaseComponent<Properties = ChildrenProps>,
{
    let mut html_stream = render_page_to_stream::<COMP, ROOT>(props, options).await?;
    let mut result_html = String::new();

    while let Some(chunk) = html_stream.next().await {
        let chunk = chunk.map_err(RenderError::ChunkError)?.to_vec();
        let next_str = String::from_utf8(chunk).map_err(|e| RenderError::ChunkError(e.into()))?;
        result_html.push_str(&next_str);
    }

    Ok(result_html)
}

fn render_before_content_markers(
    mut html: String,
    elements: BeforeContentElements,
) -> Result<String, RenderError> {
    let BeforeContentElements {
        title,
        metadata,
        links,
    } = elements;

    // Insert the <title> element
    insert_title(&mut html, title);

    // Insert the <meta> elements from `struct Metadata`
    insert_metadata(&mut html, metadata);

    // Insert the <link> elements from `struct PageLinks`
    insert_links(&mut html, links);

    Ok(html)
}

fn render_after_content_markers<COMP>(
    mut html: String,
    elements: AfterContentElements,
    page_data: PageData,
) -> Result<String, RenderError>
where
    COMP: BaseComponent,
    COMP::Properties: Serialize,
{
    let AfterContentElements { scripts } = elements;

    // Insert the <script> elements from `struct PageScripts`
    insert_scripts::<COMP>(&mut html, scripts, page_data)?;

    Ok(html)
}

fn insert_title(html: &mut String, title: Option<String>) {
    if let Some(title) = title {
        let tag = format!("<title>{title}</title>");
        *html = html.replace(HASHIRA_TITLE_MARKER, &tag);
    }
}

fn insert_metadata(html: &mut String, metadata: Metadata) {
    let tags = metadata.to_string();
    *html = html.replace(HASHIRA_META_MARKER, &tags);
}

fn insert_links(html: &mut String, links: PageLinks) {
    let links = links.to_string();
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
    let mut tags_html = vec![scripts.to_string()];

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

    let scripts = tags_html.join("\n");
    *html = html.replace(HASHIRA_SCRIPTS_MARKER, &scripts);
    Ok(())
}

pub async fn render_to_static_html<F>(f: F) -> String
where
    F: FnOnce() -> Html + Send + Sync + 'static,
{
    #[function_component]
    fn Dummy(props: &ChildrenProps) -> Html {
        yew::html! {
            <>{for props.children.iter()}</>
        }
    }

    let (tx, rx) = tokio::sync::oneshot::channel::<String>();

    // FIXME: Not sure if may be a downgrade to block the thread.
    futures::executor::block_on(async move {
        let renderer = ServerRenderer::<Dummy>::with_props(move || ChildrenProps {
            children: ChildrenRenderer::new(vec![f()]),
        });
        let html = renderer.hydratable(false).render().await;
        tx.send(html).unwrap();
    });

    let html = rx.await.unwrap();
    html
}
