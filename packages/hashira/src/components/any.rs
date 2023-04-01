use std::sync::Arc;
use yew::{BaseComponent, Html};

pub struct AnyComponent<Props>(Arc<dyn Fn(Props) -> Html + Send + Send>);

impl AnyComponent<()> {
    pub fn new<COMP>() -> AnyComponent<COMP::Properties>
    where
        COMP: BaseComponent,
    {
        AnyComponent(Arc::new(|props| {
            yew::html! {
                <COMP ..props/>
            }
        }))
    }
}

impl<Props> AnyComponent<Props>
where
    Props: Default,
{
    pub fn render(&self) -> Html {
        let props = Props::default();
        (self.0)(props)
    }
}

impl<Props> AnyComponent<Props> {
    pub fn render_with_props(&self, props: Props) -> Html {
        (self.0)(props)
    }
}

impl<Props> PartialEq for AnyComponent<Props> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl<Props> Clone for AnyComponent<Props> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
