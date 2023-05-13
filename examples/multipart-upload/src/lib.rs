mod app;
mod components;

use app::{App, UploadFileAction, UploadsPage};
use hashira::{
    app::{App as HashiraApp, AppService, LayoutContext},
    routing::Route,
    server::{LinkTag, PageLinks},
};

cfg_if::cfg_if! {
    if #[cfg(not(feature = "client"))] {
        use std::path::PathBuf;
            
        pub static UPLOAD_DIR: &str = "uploads/";
    
        pub fn uploads_dir() -> PathBuf {
            let dir = std::env::current_exe()
                .expect("failed to get current dir")
                .parent()
                .unwrap()
                .to_path_buf()
                .join(UPLOAD_DIR);
    
            if !dir.exists() {
                log::info!("Creating upload directory: {}", dir.display());
                std::fs::create_dir_all(&dir).expect("failed to create upload directory");
            }
    
            dir
        }
    }
}

// Setup all the components
pub fn hashira() -> AppService {
    HashiraApp::<App>::new()
        .use_default_error_pages()
        .layout(root_layout)
        .page::<UploadsPage>()
        .action::<UploadFileAction>()
        .route(Route::post("/echo", |body: String| async move {
            let rev = body.chars().rev().collect::<String>();
            rev
        }))
        .build()
}

pub async fn root_layout(mut ctx: LayoutContext) -> yew::Html {
    use hashira::components::*;

    ctx.title("Hashira");
    ctx.links(PageLinks::new().insert(LinkTag::stylesheet("/static/global.css")));

    yew::html! {
        <html lang="en">
            <head>
                <Title/>
                <Meta/>
                <Links/>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1.0" />
            </head>
            <body>
                <Main>
                    <Content/>
                </Main>
                <Scripts/>
                <LiveReload/>
            </body>
        </html>
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    wasm_logger::init(wasm_logger::Config::default());
    log::debug!("Hydrating app...");
    let service = hashira();
    hashira::client::mount::<App>(service);
}
