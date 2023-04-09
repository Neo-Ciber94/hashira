use anyhow::Context;
use axum::{
    extract::{ws::Message, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Extension, Router,
};
use futures::{SinkExt, StreamExt};
use notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebouncedEvent};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, path::Path, time::Duration};
use tokio::sync::broadcast::{channel, Sender};
use tokio_stream::wrappers::BroadcastStream;

use crate::{
    commands::{DevOptions, RunOptions},
    tasks::{build_task::BuildTask, run_task::RunTask},
};

#[derive(Clone, Debug)]
enum Notification {
    Reload,
    Close,
}

pub struct DevTask {
    // Options for running the application in watch mode
    options: DevOptions,

    // Signal used to shutdown the processes
    shutdown_signal: Sender<()>,
}

impl DevTask {
    pub fn new(options: DevOptions) -> Self {
        let (shutdown_signal, _) = channel(1);
        DevTask {
            options,
            shutdown_signal,
        }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        let opts = &self.options;
        let shutdown_signal = self.shutdown_signal.clone();
        let (build_done_tx, mut build_done_rx) = channel::<()>(1);
        let (tx_notify, _rx_notify) = channel::<Notification>(16);

        {
            let tx_notify = tx_notify.clone();
            tokio::spawn({
                async move {
                    tokio::signal::ctrl_c().await.ok();
                    log::info!("Exiting...");
                    let _ = shutdown_signal.send(());
                    tx_notify
                        .send(Notification::Close)
                        .unwrap_or_else(|_| panic!("failed to send close event"));

                    std::process::exit(0);
                }
            });
        }

        // When the system is rebuilding after a file change notification is sent,
        // we wait for the build done signal to notify the client
        {
            let tx_notify = tx_notify.clone();
            tokio::spawn(async move {
                loop {
                    let events = build_done_rx.recv().await.unwrap();
                    log::debug!("Received build done signal");
                    if let Err(err) = tx_notify.send(Notification::Reload) {
                        log::error!("Error sending change event: {err}");
                    }
                }
            });
        }

        // Starts the watcher
        self.start_watcher(build_done_tx)?;

        // Starts the server
        let host = opts.reload_host.as_str();
        let port = opts.reload_port;

        start_server(tx_notify, host, port).await?;
        Ok(())
    }

    fn start_watcher(&self, build_done_tx: Sender<()>) -> anyhow::Result<()> {
        let opts = &self.options;
        let shutdown_signal = self.shutdown_signal.clone();
        let (tx_watch, mut rx_watch) = channel::<Vec<DebouncedEvent>>(4);

        // Starts the file system watcher
        self.build_watcher(tx_watch)?;

        // Starts
        {
            let opts = opts.clone();
            let build_done_tx = build_done_tx.clone();
            let shutdown_signal = shutdown_signal.clone();

            tokio::spawn(async move {
                log::debug!("Starting dev...");
                build_and_run(opts, shutdown_signal, build_done_tx, vec![]).await;
            });
        }

        // Start notifier loop
        let opts = opts.clone();

        tokio::task::spawn(async move {
            loop {
                let shutdown_signal = shutdown_signal.clone();

                // Wait for change event
                let events = rx_watch
                    .recv()
                    .await
                    .expect("failed to read debounce event");

                // Interrupt the current running task
                let _ = shutdown_signal.send(());

                // Rerun
                let build_done_tx = build_done_tx.clone();
                let opts = opts.clone();

                tokio::spawn(async move {
                    log::info!("Restarting dev...");
                    build_and_run(opts, shutdown_signal, build_done_tx, events).await;
                });
            }
        });

        Ok(())
    }

    fn build_watcher(&self, tx_watch: Sender<Vec<DebouncedEvent>>) -> anyhow::Result<()> {
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
}

async fn build_and_run(
    opts: DevOptions,
    shutdown_signal: Sender<()>,
    build_done_tx: Sender<()>,
    events: Vec<DebouncedEvent>,
) {
    log::info!("Starting application watch mode");

    if !events.is_empty() {
        let paths = events.iter().map(|e| &e.path).cloned().collect::<Vec<_>>();
        log::info!("change detected on paths: {:?}", paths);
    }

    // Build task
    let mut run_task = RunTask::with_signal(
        RunOptions {
            quiet: opts.quiet,
            release: opts.release,
            public_dir: opts.public_dir.clone(),
            target_dir: opts.target_dir.clone(),
            include: opts.include.clone(),
            allow_include_external: opts.allow_include_external,
            allow_include_src: opts.allow_include_src,
            host: opts.host.clone(),
            port: opts.port,
            static_dir: opts.static_dir.clone(),
        },
        shutdown_signal.clone(),
        build_done_tx.clone(),
    );

    // TODO: We should decide what operation to perform depending on the files affected,
    // if only a `public_dir` file changed, maybe we don't need to rebuild the entire app

    let host = opts.reload_host.clone();
    let port = opts.reload_port.to_string();

    run_task.env(crate::env::HASHIRA_LIVE_RELOAD_HOST, host);
    run_task.env(crate::env::HASHIRA_LIVE_RELOAD_PORT, port);
    run_task.env(crate::env::HASHIRA_LIVE_RELOAD, String::from("1"));

    if let Err(err) = run_task.run().await {
        log::error!("Watch run failed: {err}");
    }
}

async fn start_server(
    tx_notify: Sender<Notification>,
    host: &str,
    port: u16,
) -> anyhow::Result<()> {
    // create a router with a websocket handler
    let app = Router::new()
        .route("/ws", get(websocket_handler))
        .layer(Extension(tx_notify));

    // parse address
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

#[derive(Debug, Serialize, Deserialize)]
struct ReloadMessage {
    reload: bool,
}

// this function handles websocket connections
async fn websocket_handler(
    upgrade: WebSocketUpgrade,
    tx_notify: Extension<Sender<Notification>>,
) -> impl IntoResponse {
    upgrade.on_upgrade(|ws| async move {
        log::debug!("Web socket upgrade");

        // split the websocket into a sender and a receiver
        let (mut sender, _receiver) = ws.split();
        let receiver = tx_notify.subscribe();
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
