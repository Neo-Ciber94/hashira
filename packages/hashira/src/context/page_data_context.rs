use crate::components::{PageData, PropsWithChildren};
use std::{ops::Deref, rc::Rc};
use yew::{function_component, hook, use_context, ContextProvider};

#[derive(Clone, Debug)]
pub struct PageDataHandle(Rc<PageData>);

impl PartialEq for PageDataHandle {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Deref for PageDataHandle {
    type Target = PageData;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[function_component]
pub fn PageDataContextProvider(props: &PropsWithChildren<PageData>) -> yew::Html {
    let ctx = PageDataHandle(Rc::new(props.cloned()));

    yew::html! {
        <ContextProvider<PageDataHandle> context={ctx}>
            {for props.children.clone().iter()}
        </ContextProvider<PageDataHandle>>
    }
}

#[hook]
pub fn use_page_data() -> PageDataHandle {
    use_context::<PageDataHandle>().expect("`PageDataContextProvider` should be a parent")
}
