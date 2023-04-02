use super::PageHandler;
use crate::components::any::AnyComponent;

pub struct ServerPageRoute<C> {
    pub(crate) handler: PageHandler<C>,
    pub(crate) match_pattern: String,
}

impl<C> ServerPageRoute<C> {
    pub fn handler(&self) -> &PageHandler<C> {
        &self.handler
    }

    pub fn match_pattern(&self) -> &str {
        self.match_pattern.as_str()
    }
}

pub struct ClientPageRoute {
    pub(crate) component: AnyComponent<serde_json::Value>,
    pub(crate) match_pattern: String,
}

impl ClientPageRoute {
    pub fn render(&self, props: serde_json::Value) -> yew::Html {
        self.component.render_with_props(props)
    }

    pub fn match_pattern(&self) -> &str {
        self.match_pattern.as_str()
    }
}
