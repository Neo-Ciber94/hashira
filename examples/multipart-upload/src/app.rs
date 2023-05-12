use hashira::{actions::use_action_with_callback, components::ActionForm, page_component};
use multer_derive::{FormFile, FromMultipart};
use serde::{Deserialize, Serialize};
use web_sys::window;
use yew::{function_component, html::ChildrenProps, Properties};

// server only imports
cfg_if::cfg_if! {
    if #[cfg(not(feature = "client"))] {
        use std::path::PathBuf;
        use hashira::{
            app::RenderContext,
            error::Error,
            web::{Multipart, Response},
            server::Metadata,
            responses
        };

        static UPLOAD_DIR: &str = "uploads/";

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

    }
}

#[function_component]
pub fn App(props: &ChildrenProps) -> yew::Html {
    yew::html! {
        <>{for props.children.iter()}</>
    }
}

#[allow(dead_code)]
#[derive(FromMultipart)]
pub struct NewImage {
    image: FormFile,
}

#[hashira::action]
pub async fn UploadFileAction(input: Multipart<NewImage>) -> hashira::Result<()> {
    use std::io::Write;
    use std::time::SystemTime;

    let image = input.into_inner().image;
    log::info!(
        "Uploading file: {} ({} bytes)",
        image.file_name(),
        image.bytes().len()
    );

    let (_, ext) = image
        .file_name()
        .split_once(".")
        .ok_or(responses::bad_request("image do not contain extension"))?;

    let new_name = format!(
        "{timestamp}.{ext}",
        timestamp = SystemTime::UNIX_EPOCH.elapsed()?.as_millis()
    );

    let dir = uploads_dir();
    let dest_path = dir.join(new_name);
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
                            let url = format!("{UPLOAD_DIR}{file_name}");
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
            window.location().reload().unwrap();
        }
        Err(_) => {
            let window = window().unwrap();
            window.alert_with_message("failed to upload file").unwrap();
        }
    });

    yew::html! {
       <>
            <ActionForm<UploadFileAction> action={action.clone()} multipart={true} id="upload-form">
                <input type="file" name="image" accept="image/*" required={true} />
                <button>{"Upload"}</button>
            </ActionForm<UploadFileAction>>
            <div class="container">
                if props.files.is_empty() {
                    <strong id="empty">{"No images, upload something..."}</strong>
                } else {
                    {for props.files.iter().cloned().map(|file| {
                        yew::html_nested! {
                            <div class="image-file">
                                <img alt={file.clone()} src={file}/>
                            </div>
                        }
                    })}
                }
            </div>
       </>
    }
}
