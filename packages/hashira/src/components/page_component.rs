use yew::{BaseComponent, html::ChildrenProps};

use crate::{app::RenderContext, error::Error, types::BoxFuture, web::Response};

/// Represents a page of a web app.
pub trait PageComponent: BaseComponent {
    /// Returns an unique identifier for this component.
    fn id() -> &'static str {
        std::any::type_name::<Self>()
    }

    /// The route of this page.
    fn route() -> Option<&'static str>;

    /// A function that renders this page component.
    fn loader<BASE>(ctx: RenderContext) -> BoxFuture<Result<Response, Error>>
    where
        BASE: BaseComponent<Properties = ChildrenProps>;
}
