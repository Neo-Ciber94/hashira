use http::StatusCode;
use serde::{Deserialize, Serialize};
use yew::{function_component, Properties};

#[derive(Clone, Properties, PartialEq, Serialize, Deserialize)]
pub struct ErrorPageProps {
    #[serde(with = "crate::web::serde::status_code")]
    pub status: StatusCode,

    #[prop_or_default]
    pub message: Option<String>,
}

#[function_component]
pub fn ErrorPage(props: &ErrorPageProps) -> yew::Html {
    let status = &props.status;
    let message = &props.message;

    yew::html! {
        <>
            <style>
                {ERROR_PAGE_STYLES}
            </style>

            <div class="error-page-container">
                <div class="error-page">
                    <div class="error-details">
                    <h1 class="error-text">
                        <span class="error-status">{format!("{}", status.as_u16())}</span>
                        <span class="error-divider"></span>
                        <span class="error-status-message">{format!("{}", status.canonical_reason().unwrap_or("An error has occurred"))}</span>
                    </h1>
                        if let Some(message) = message {
                            <strong class="error-message">{message}</strong>
                        }
                    </div>
                </div>
            </div>
        </>
    }
}

#[derive(Clone, Default, Properties, PartialEq, Serialize, Deserialize)]
pub struct NotFoundPageProps {
    #[prop_or_default]
    pub message: Option<String>,
}

#[function_component]
pub fn NotFoundPage(props: &NotFoundPageProps) -> yew::Html {
    yew::html! {
        <ErrorPage status={StatusCode::NOT_FOUND} message={props.message.clone()}/>
    }
}

const ERROR_PAGE_STYLES: &str = r#"
.error-page-container {
    position: relative;
    height: 80vh;
}

.error-page {
    position: absolute;
    font-family: monospace;
    left: 50%;
    top: 50%;
    transform: translate(-50%, -50%);
    width: 100%;
}

.error-details {
    height: 100%;
    width: 100%;
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
}

.error-message {
    font-size: 16px;
    color: rgb(92, 92, 92);
}

.error-text {
    display: flex;
    flex-direction: row;
    justify-content: center;
    align-items: center;
    font-size: 28px;
    font-weight: 100;
    gap: 10px;
}

.error-divider {
    display: inline-block;
    height: 30px;
    width: 1.5px;
    background-color: rgb(200, 200, 200);
}

body.dark {
    background-color: black;
}

body.dark h1 {
    color: rgb(220, 220, 220);
}

body.dark .error-message {
    font-size: 16px;
    color: rgb(226, 226, 226);
}

@media (prefers-color-scheme: dark) {
    body {
        background-color: black;
    }
    
    h1 {
        color: rgb(220, 220, 220);
    }
    
    .error-message {
        font-size: 16px;
        color: rgb(226, 226, 226);
    }    
}
"#;
