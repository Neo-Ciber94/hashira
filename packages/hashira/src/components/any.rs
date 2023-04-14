use std::sync::Arc;
use yew::Html;

/// A wrapper to render any component.
pub struct AnyComponent<Props>(Arc<dyn Fn(Props) -> Html + Send + Sync>);

impl<Props> AnyComponent<Props> {
    /// Use a function to render the html.
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(Props) -> yew::Html + Send + Sync + 'static,
    {
        AnyComponent(Arc::new(f))
    }
}

impl<Props> AnyComponent<Props>
where
    Props: Default,
{
    /// Renders the component.
    pub fn render(&self) -> Html {
        let props = Props::default();
        (self.0)(props)
    }
}

impl<Props> AnyComponent<Props> {
    /// Renders the component with the given props.
    pub fn render_with_props(&self, props: Props) -> Html {
        (self.0)(props)
    }
}

impl<Props> PartialEq for AnyComponent<Props> {
    #[allow(clippy::vtable_address_comparisons)]
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl<Props> Clone for AnyComponent<Props> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
