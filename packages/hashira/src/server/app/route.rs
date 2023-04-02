use super::PageHandler;
use crate::components::RenderFn;

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
    pub(crate) render: RenderFn,
    pub(crate) match_pattern: String,
}

impl ClientPageRoute {
    pub fn render(&self) -> &RenderFn {
        &self.render
    }

    pub fn match_pattern(&self) -> &str {
        self.match_pattern.as_str()
    }
}
