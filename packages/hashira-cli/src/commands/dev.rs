use super::RunOptions;
use crate::utils::interrupt::RUN_INTERRUPT;
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
use std::time::Duration;
use tokio::sync::broadcast::{channel, Sender};
use tokio_stream::wrappers::BroadcastStream;

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

#[derive(Clone, Debug)]
enum Notification {
    Reload,
    Close,
}

async fn start_server(opts: &DevOptions) -> anyhow::Result<()> {
    let (tx_notify, _rx_notify) = channel::<Notification>(16);

    tokio::spawn({
        let tx = tx_notify.clone();
        async move {
            tokio::signal::ctrl_c().await.ok();
            log::info!("Exiting...");
            tx.send(Notification::Close)
                .unwrap_or_else(|_| panic!("failed to send close event"));

            std::process::exit(0);
        }
    });

    // Starts the watcher
    start_watcher(tx_notify.clone(), opts)?;

    // create a router with a websocket handler
    let app = Router::new()
        .route("/ws", get(websocket_handler))
        .layer(Extension(tx_notify));

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
    observable: Extension<Sender<Notification>>,
) -> impl IntoResponse {
    upgrade.on_upgrade(|ws| async move {
        log::debug!("Web socket upgrade");

        // split the websocket into a sender and a receiver
        let (mut sender, _receiver) = ws.split();
        let receiver = observable.subscribe();
        let mut stream = BroadcastStream::new(receiver);

        loop {
            if let Some(Ok(event)) = stream.next().await {
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

async fn build_and_run(
    build_done_tx: tokio::sync::broadcast::Sender<()>,
    events: Vec<DebouncedEvent>,
    opts: DevOptions,
) {
    log::info!("Starting application watch mode");

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

    if let Err(err) = crate::commands::run_with_envs(run_opts, envs, Some(build_done_tx)).await {
        log::error!("Watch run failed: {err}");
    }
}

fn start_watcher(
    tx_notification: tokio::sync::broadcast::Sender<Notification>,
    opts: &DevOptions,
) -> anyhow::Result<()> {
    let (build_done_tx, mut build_done_rx) = tokio::sync::broadcast::channel(8);
    let (tx_watch, mut rx_watch) = tokio::sync::broadcast::channel::<Vec<DebouncedEvent>>(8);

    let opts = opts.clone();
    let run_interrupt = RUN_INTERRUPT.clone();

    {
        tokio::spawn(async move {
            loop {
                match build_done_rx.recv().await {
                    Ok(_) => {
                        log::debug!("Received build done signal");
                        if let Err(err) = tx_notification.send(Notification::Reload) {
                            log::error!("Error sending change event: {err}");
                        }
                    }
                    Err(err) => {
                        log::error!("Failed when receiving build event: {err}");
                    }
                }
            }
        });
    }

    // Listen for the changes
    build_watcher(tx_watch)?;

    // Starts
    {
        let opts = opts.clone();
        let build_done_tx = build_done_tx.clone();
        tokio::spawn(async move {
            log::debug!("Starting dev...");
            build_and_run(build_done_tx, vec![], opts).await;
        });
    }

    // Start notifier loop
    tokio::task::spawn(async move {
        loop {
            // Wait for change event
            let events = rx_watch
                .recv()
                .await
                .expect("failed to read debounce event");

            // Interrupt the current running task
            run_interrupt.interrupt();

            // Rerun
            let build_done_tx = build_done_tx.clone();
            let opts = opts.clone();
            log::debug!("Interrupted, now go to restart");
            tokio::spawn(async move {
                log::info!("Restarting dev...");
                build_and_run(build_done_tx, events, opts).await;
            });
        }
    });

    Ok(())
}

fn build_watcher(tx_watch: Sender<Vec<DebouncedEvent>>) -> anyhow::Result<()> {
    let (tx_debounced, rx_debounced) = std::sync::mpsc::channel();
    let mut debouncer = new_debouncer(Duration::from_secs(1), None, tx_debounced)
        .with_context(|| "failed to start watcher")?;

    let watch_path = Path::new(".").canonicalize()?;
    log::info!("Starting watcher at: {}", watch_path.display());

    debouncer
        .watcher()
        .watch(&watch_path, RecursiveMode::Recursive)
        .unwrap();

    std::thread::spawn(move || {
        let _debouncer = debouncer;

        loop {
            match rx_debounced.recv() {
                Ok(event) => {
                    if let Ok(evt) = event {
                        if let Err(err) = tx_watch.send(evt) {
                            log::error!("Failed to send debounced event: {err}");
                        }
                    }
                }
                Err(err) => {
                    log::error!("Failed to received debounce event: {err}");
                    break;
                }
            }
        }
    });

    Ok(())
}
