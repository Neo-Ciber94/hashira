use http::Method;
use web_sys::FormData;
use yew::{function_component, Children, Properties};
use yew::{Classes, TargetCast};

use crate::actions::{Action, AnyForm, RequestOptions, UseActionHandle};
use crate::utils::refresh_window;

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
    pub class: Classes,

    /// Styles of the form.
    #[prop_or_default]
    pub style: Option<String>,

    /// The mime type to send the form, defaults to `application/x-www-form-urlencoded`.
    ///
    /// Checkout: <https://developer.mozilla.org/en-US/docs/Web/HTML/Element/form#attributes_for_form_submission>
    #[prop_or(String::from("application/x-www-form-urlencoded"))]
    pub enc_type: String,

    /// Action used to upload the form.
    pub action: UseActionHandle<A, AnyForm>,

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
    let loading = action.is_loading();
    let method = props.method.clone();
    let enc_type = props.enc_type.clone();
    let reload = props.reload;

    let on_submit = move |event: yew::html::onsubmit::Event| {
        event.prevent_default();

        if loading {
            return;
        }

        let form = event.target_dyn_into().unwrap();
        let form_data = FormData::new_with_form(&form).unwrap();
        let opts = RequestOptions::new().method(method.clone());

        // We are purposely ignoring `text/plain` here and maybe we shouldn't
        let form = match enc_type.as_str() {
            "application/x-www-form-urlencoded" => AnyForm::UrlEncoded(form_data),
            "multipart/form-data" => AnyForm::Multipart(form_data),
            s => {
                log::warn!("unsupported form enctype: {s}, only `application/x-www-form-urlencoded` and `multipart/form-data` are supported");
                AnyForm::Multipart(form_data)
            }
        };

        action
            .send_with_options(form, opts)
            .expect("failed to send form");

        if reload {
            refresh_window();
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
