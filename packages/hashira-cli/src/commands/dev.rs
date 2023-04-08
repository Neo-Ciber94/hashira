use super::RunOptions;
use crate::utils::interruct::RUN_INTERRUPT;
use anyhow::Context;
use axum::extract::ws::Message;
use axum::extract::WebSocketUpgrade;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Extension;
use axum::Router;
use clap::Args;
use futures::SinkExt;
use futures::StreamExt;
use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use notify_debouncer_mini::DebouncedEvent;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::channel;
use tokio::sync::Mutex;
use tokio_stream::wrappers::ReceiverStream;

#[derive(Args, Debug, Clone)]
pub struct DevOptions {
    #[arg(short, long, help = "Base directory for the artifacts")]
    pub target_dir: Option<PathBuf>,

    #[arg(
        short,
        long,
        help = "Directory relative to the `target_dir` where the static files will be serve from",
        default_value = "public"
    )]
    pub public_dir: PathBuf,

    #[arg(
        short,
        long,
        help = "Build artifacts in release mode, with optimizations",
        default_value_t = false
    )]
    pub release: bool,

    #[arg(
        long,
        default_value_t = false,
        help = "Whether if output the commands output"
    )]
    pub quiet: bool,

    #[arg(
        short,
        long,
        help = "The server path where the static files will be serve",
        default_value = "/static"
    )]
    pub static_dir: String,

    #[arg(
        long,
        help = "A list of files to copy in the `public_dir` by default include the `public` and `assets` directories, if found"
    )]
    pub include: Vec<String>,

    #[arg(
        long,
        help = "Allow to include files outside the current directory",
        default_value_t = false
    )]
    pub allow_include_external: bool,

    #[arg(
        long,
        help = "Allow to include files inside src/ directory",
        default_value_t = false
    )]
    pub allow_include_src: bool,

    #[arg(
        long,
        help = "The host to run the application",
        default_value = "127.0.0.1"
    )]
    pub host: String,

    #[arg(long, help = "The port to run the application", default_value_t = 5000)]
    pub port: u16,

    #[arg(
        long,
        help = "The host to run the hot reload server",
        default_value = "127.0.0.1"
    )]
    pub reload_host: String,

    #[arg(
        long,
        help = "The port to run the hot reload server",
        default_value_t = 5002
    )]
    pub reload_port: u16,
}

pub async fn dev(opts: DevOptions) -> anyhow::Result<()> {
    start_server(&opts).await?;
    Ok(())
}

enum Notification {
    Reload,
    Close,
}

async fn start_server(opts: &DevOptions) -> anyhow::Result<()> {
    let (tx, rx) = channel::<Notification>(32);

    tokio::spawn({
        let tx = tx.clone();
        async move {
            tokio::signal::ctrl_c().await.ok();
            log::info!("Exiting...");
            tx.send(Notification::Close)
                .await
                .unwrap_or_else(|_| panic!("failed to send close event"));

            std::process::exit(0);
        }
    });

    // Starts the watcher
    start_watcher(tx, opts)?;

    let receiver = ReceiverStream::new(rx);

    // create a router with a websocket handler
    let app = Router::new()
        .route("/ws", get(websocket_handler))
        .layer(Extension(Arc::new(Mutex::new(receiver))));

    // parse address
    let host = opts.host.clone();
    let port = opts.reload_port;
    let addr = format!("{host}:{port}",)
        .parse::<SocketAddr>()
        .with_context(|| format!("invalid hot reload server address: {host}:{port}"))?;

    log::info!("Starting hot reload server on: http://{addr}");

    // Start server
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct ReloadMessage {
    reload: bool,
}

// this function handles websocket connections
async fn websocket_handler(
    upgrade: WebSocketUpgrade,
    stream: Extension<Arc<Mutex<ReceiverStream<Notification>>>>,
) -> impl IntoResponse {
    upgrade.on_upgrade(|ws| async move {
        log::debug!("Web socket upgrade");

        // split the websocket into a sender and a receiver
        let (mut sender, _receiver) = ws.split();

        loop {
            let mut receiver = stream.lock().await;
            if let Some(event) = receiver.next().await {
                match event {
                    Notification::Reload => {
                        log::debug!("Sending reload message...");

                        let json = serde_json::to_string(&ReloadMessage { reload: true })
                            .expect("Failed to serialize message");
                        let msg = Message::Text(json);
                        if let Err(err) = sender.send(msg).await {
                            log::error!("Failed to send web socket message: {err}");
                        }
                    }
                    Notification::Close => {
                        log::debug!("Closing web socket");
                        break;
                    }
                }
            }
        }
    })
}

async fn run_watch_mode(
    tx_notification: tokio::sync::mpsc::Sender<Notification>,
    events: Vec<DebouncedEvent>,
    opts: DevOptions,
) {
    log::info!("Starting application watch mode");

    if let Err(err) = tx_notification.send(Notification::Reload).await {
        log::error!("Error sending change event: {err}");
    }

    if !events.is_empty() {
        let paths = events.iter().map(|e| &e.path).cloned().collect::<Vec<_>>();
        log::info!("change detected on paths: {:?}", paths);
    }

    // Build options to send
    let run_opts = RunOptions {
        quiet: opts.quiet,
        release: opts.release,
        public_dir: opts.public_dir,
        target_dir: opts.target_dir,
        include: opts.include,
        allow_include_external: opts.allow_include_external,
        allow_include_src: opts.allow_include_src,
        host: opts.host,
        port: opts.port,
        static_dir: opts.static_dir,
    };

    // TODO: We should decide what operation to perform depending on the files affected,
    // if only a `public_dir` file changed, maybe we don't need to rebuild the entire app

    let host = opts.reload_host;
    let port = opts.reload_port.to_string();

    let envs = HashMap::from_iter([
        (crate::env::HASHIRA_LIVE_RELOAD_HOST, host),
        (crate::env::HASHIRA_LIVE_RELOAD_PORT, port),
        (crate::env::HASHIRA_LIVE_RELOAD, String::from("1")),
    ]);

    if let Err(err) = crate::commands::run_with_envs(run_opts, envs).await {
        log::error!("Watch run failed: {err}");
    }

    // let mut int = RUN_INTERRUPT.subscribe();

    // tokio::select! {
    //     ret = crate::commands::run_with_envs(run_opts, envs) => {
    //         if let Err(err) = ret {
    //             log::error!("Watch run failed: {err}");
    //         }
    //     },
    //     ret = int.recv() => {
    //         if let Err(err) = ret {
    //             log::error!("interruption error: {err}");
    //         }
    //     }
    // }

    // log::info!("Exiting dev execution...");
}

fn start_watcher(
    tx_notification: tokio::sync::mpsc::Sender<Notification>,
    opts: &DevOptions,
) -> anyhow::Result<()> {
    let (tx_debounced, rx_debounced) = std::sync::mpsc::channel();

    let mut debouncer = new_debouncer(Duration::from_secs(1), None, tx_debounced)
        .with_context(|| "failed to start watcher")?;

    let watch_path = Path::new(".").canonicalize()?;
    log::info!("Starting watcher at: {}", watch_path.display());

    debouncer
        .watcher()
        .watch(&watch_path, RecursiveMode::Recursive)
        .unwrap();

    let opts = opts.clone();
    let run_interrupt = RUN_INTERRUPT.clone();

    // Starts
    {
        let opts = opts.clone();
        let tx_notification = tx_notification.clone();
        tokio::spawn(async move {
            log::debug!("Starting dev...");
            run_watch_mode(tx_notification, vec![], opts).await;
        });
    }

    // Listen for the changes
    tokio::spawn(async move {
        let _debouncer = debouncer;

        loop {
            match rx_debounced.recv() {
                Ok(ret) => {
                    let events = ret.unwrap();
                    let tx_notification = tx_notification.clone();
                    let opts = opts.clone();

                    log::debug!("change detected!");
                    // Interrupt the current running task
                    run_interrupt.interrupt();

                    tokio::spawn(async move {
                        log::info!("Restarting dev...");
                        run_watch_mode(tx_notification, events, opts).await;
                    });
                }
                Err(err) => {
                    log::info!("Exit?: {err}");
                    break;
                }
            }
        }
    });

    Ok(())
}
