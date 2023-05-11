use std::path::PathBuf;

use hashira::{
    actions::use_action_with_callback,
    app::RenderContext,
    components::ActionForm,
    error::Error,
    page_component,
    server::Metadata,
    web::{Multipart, Response},
};
use multer_derive::{FormFile, FromMultipart};
use serde::{Deserialize, Serialize};
use web_sys::window;
use yew::{function_component, html::ChildrenProps, Properties};

static UPLOAD_DIR: &str = "uploads/";

#[function_component]
pub fn App(props: &ChildrenProps) -> yew::Html {
    yew::html! {
        <>{for props.children.iter()}</>
    }
}

#[derive(FromMultipart)]
pub struct NewImage {
    image: FormFile,
}

#[hashira::action]
pub async fn UploadFileAction(input: Multipart<NewImage>) -> hashira::Result<()> {
    use std::io::Write;

    let image = input.into_inner().image;
    log::info!("Uploading file: {}", image.file_name());

    let dir = uploads_dir();
    let dest_path = dir.join(image.file_name());
    let mut file = std::fs::File::create(dest_path)?;
    let mut writer = std::io::BufWriter::new(&mut file);
    writer.write_all(&image.bytes().to_vec())?;
    Ok(())
}

#[hashira::render]
async fn render(mut ctx: RenderContext) -> Result<Response, Error> {
    ctx.metadata(Metadata::new().title("Hashira Multipart Upload"));

    let dir = uploads_dir();
    let mut files = vec![];

    for entry in dir.read_dir()? {
        if let Ok(entry) = entry {
            let path = entry.path();
            if let Some(file_name) = path.file_name().and_then(|x| x.to_str()) {
                if let Some((name, ext)) = file_name.split_once(".") {
                    match ext {
                        "png" | "jpg" | "jpeg" | "svg" | "gif" | "tiff" | "webp" => {
                            let url = format!("{UPLOAD_DIR}/{file_name}");
                            files.push(url);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    let res = ctx
        .render_with_props::<UploadsPage, App>(UploadPageProps { files })
        .await;
    Ok(res)
}

#[derive(PartialEq, Properties, Serialize, Deserialize)]
pub struct UploadPageProps {
    files: Vec<String>,
}

#[page_component("/", render = "render")]
pub fn UploadsPage(props: &UploadPageProps) -> yew::Html {
    let action = use_action_with_callback(move |ret| match ret {
        Ok(_) => {
            let window = window().unwrap();
            window.alert_with_message("file uploaded!").unwrap();
        }
        Err(_) => {
            let window = window().unwrap();
            window.alert_with_message("failed to upload file").unwrap();
        }
    });

    yew::html! {
       <>
            <ActionForm<UploadFileAction> action={action.clone()} multipart={true}>
                <input type="file" name="image" accept="image/*"/>
                <button>{"Upload"}</button>
            </ActionForm<UploadFileAction>>
            <div class="container">
                {for props.files.iter().cloned().map(|file| {
                    yew::html_nested! {
                        <div class="image-file">
                            <img alt={file.clone()} src={file}/>
                        </div>
                    }
                })}
            </div>
       </>
    }
}

#[cfg(not(feature = "client"))]
fn uploads_dir() -> PathBuf {
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
