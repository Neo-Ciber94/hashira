use http::{header, Method};
use web_sys::{window, FormData};
use yew::TargetCast;
use yew::{function_component, Children, Properties};

use crate::actions::{Action, RequestOptions, UseActionHandle};

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

    /// Whether if reload the page after sending the request.
    ///
    /// Defaults to `false`.
    #[prop_or(false)]
    pub reload: bool,

    // pub redirect: Uri?, // Redirect after send
    // pub persist_key: String? // Persist the form in session storage while is not sent
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

/// A form with extra functionality.
#[function_component]
pub fn Form<A>(props: &FormProps<A>) -> yew::Html
where
    A: Action + 'static,
{
    let action = props.action.clone();
    let method = props.method.clone();
    let enc_type = props.enc_type.clone();
    let reload = props.reload.clone();

    let on_submit = move |event: yew::html::onsubmit::Event| {
        event.prevent_default();

        let form = event.target_dyn_into().unwrap();
        let form_data = FormData::new_with_form(&form).unwrap();
        let mut opts = RequestOptions::new().method(method.clone());

        // By default FormData is set to `multipart/form-data` so we let the browser handle it
        if enc_type.as_str() != "multipart/form-data" {
            opts = opts.header(
                header::CONTENT_TYPE,
                header::HeaderValue::from_str(&enc_type).expect("invalid enc type"),
            );
        }

        action
            .send_with_options(form_data, opts)
            .expect("failed to send form");

        if reload {
            let window = window().unwrap();
            window.location().reload().unwrap();
        }
    };

    yew::html! {
        <form method={props.method.clone().to_string()}
            onsubmit={on_submit}
            id={props.id.clone()}
            class={props.class.clone()}
            style={props.style.clone()}
            action={A::route()}
            enctype={props.enc_type.clone()} // this is ignored if we had JS, we send the form manually
        >
            {for props.children.iter()}
        </form>
    }
}
