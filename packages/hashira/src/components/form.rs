use http::Method;
use yew::TargetCast;
use web_sys::FormData;
use yew::{function_component, Children, Properties};

use crate::app::{hooks::UseActionHandle, Action};

#[derive(Properties)]
pub struct FormProps<A>
where
    A: Action,
{
    /// Id of the form.
    #[prop_or_default]
    pub id: Option<String>,

    /// Children of the form.
    #[prop_or_default]
    pub children: Children,

    /// Classes of the form.
    #[prop_or_default]
    pub class: Option<String>,

    /// Styles of the form.
    #[prop_or_default]
    pub style: Option<String>,

    /// The mime type to send the form, defaults to `application/x-www-form-urlencoded`.
    #[prop_or(String::from("application/x-www-form-urlencoded"))]
    pub enc_type: String,

    /// Action used to upload the form.
    pub action: UseActionHandle<A, FormData>,

    /// The method used to send the form, default to `POST`.
    #[prop_or(Method::POST)]
    pub method: Method,
}

impl<A> PartialEq for FormProps<A>
where
    A: Action,
{
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.children == other.children
            && self.class == other.class
            && self.style == other.style
            && self.enc_type == other.enc_type
            && self.action == other.action
            && self.method == other.method
    }
}

#[function_component]
pub fn Form<A>(props: &FormProps<A>) -> yew::Html
where
    A: Action + 'static,
{
    let action = props.action.clone();
    let method = props.method.clone();
    let on_submit = move |event: yew::html::onsubmit::Event| {
        event.prevent_default();

        let form = event.target_dyn_into().unwrap();
        let form_data = FormData::new_with_form(&form).unwrap();
        action
            .send_with_method(method.clone(), form_data)
            .expect("failed to send form");
    };

    yew::html! {
        <form method={"POST"}
            onsubmit={on_submit}
            id={props.id.clone()}
            class={props.class.clone()}
            style={props.style.clone()}
            action={A::route()}
            enctype={props.enc_type.clone()}
        >
            {for props.children.iter()}
        </form>
    }
}
