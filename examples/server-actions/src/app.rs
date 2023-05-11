#![allow(unused_imports)]
use crate::Messages;
use hashira::{
    actions::{use_action, use_action_with_callback},
    app::RenderContext,
    components::ActionForm,
    error::{Error, ServerError},
    page_component,
    server::Metadata,
    utils::show_alert,
    web::{status::StatusCode, Inject, Json, Response},
};
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use std::time::Duration;
use yew::{function_component, html::ChildrenProps, use_state, Properties};

#[function_component]
pub fn App(props: &ChildrenProps) -> yew::Html {
    yew::html! {
        <>{for props.children.iter()}</>
    }
}

#[derive(Serialize, Deserialize)]
pub struct NewMessage {
    text: String,
}

#[hashira::action]
pub async fn CreateMessageAction(
    Inject(messages): Inject<Messages>,
    input: hashira::web::Form<NewMessage>,
) -> hashira::Result<Json<String>> {
    let mut messages = messages.write().await;
    let text = input.into_inner().text;

    // Simulate server proceeding
    tokio::time::sleep(Duration::from_millis(700)).await;

    if text.trim().is_empty() {
        return Err(ServerError::new(StatusCode::BAD_REQUEST, "text cannot be empty").into());
    }

    messages.push(text.clone());
    Ok(Json(text))
}

#[hashira::render]
async fn render(
    mut ctx: RenderContext,
    Inject(messages): Inject<Messages>,
) -> Result<Response, Error> {
    ctx.metadata(Metadata::new().description("Hashira Server Actions"));
    let messages = messages.read().await.clone();

    let res = ctx
        .render_with_props::<MessagesPage, App>(MessagesPageProps { messages })
        .await;
    Ok(res)
}

#[derive(PartialEq, Properties, Serialize, Deserialize)]
pub struct MessagesPageProps {
    messages: Vec<String>,
}

#[page_component("/", render = "render")]
pub fn MessagesPage(props: &MessagesPageProps) -> yew::Html {
    let messages = use_state(|| props.messages.clone());
    let action = {
        let messages = messages.clone();
        use_action_with_callback(move |ret: &Result<String, _>| {
            if let Ok(msg) = ret {
                let mut new_messages = messages.deref().clone();
                new_messages.push(msg.clone());
                messages.set(new_messages);
            }
        })
    };

    yew::html! {
        <>
            <ActionForm<CreateMessageAction> action={action.clone()}>
                <input name="text" />
                <button>{"Send"}</button>
            </ActionForm<CreateMessageAction>>

            if action.is_loading() {
                <div>{"Creating..."}</div>
            }

            if let Some(err) = action.error().map(|e| e.to_string()) {
                <div style="color: red;">{err}</div>
            }

            <h4>{"Messages:"}</h4>
            <ul>
                {for messages.iter().map(|msg| {
                    yew::html_nested! {
                        <li>
                            {msg}
                        </li>
                    }
                })}
            </ul>
        </>
    }
}
