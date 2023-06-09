use crate::components::{id::PageId, AnyComponent};

// Represents a client-side page route, containing a component and a path pattern.
#[derive(Clone)]
pub struct ClientPageRoute {
    pub(crate) page_id: PageId,
    pub(crate) component: AnyComponent<serde_json::Value>, // The component for this page route.
    pub(crate) path: String,                               // The route of this component
}

impl ClientPageRoute {
    /// Returns this route with a new path.
    pub fn with_path(self, path: impl Into<String>) -> Self {
        ClientPageRoute {
            page_id: self.page_id,
            component: self.component,
            path: path.into(),
        }
    }

    /// Returns the id of the page of this route.
    pub fn id(&self) -> &PageId {
        &self.page_id
    }

    // Renders the component for this page route with the given props.
    pub fn render(&self, props: serde_json::Value) -> yew::Html {
        self.component.render_with_props(props)
    }

    // Returns a reference to the path pattern for this page route.
    pub fn path(&self) -> &str {
        self.path.as_str()
    }
}
