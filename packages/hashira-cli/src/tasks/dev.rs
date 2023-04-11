use crate::{
    cli::{DevOptions, RunOptions},
    tasks::run::RunTask,
};
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
use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::sync::broadcast::{channel, Sender};
use tokio_stream::wrappers::BroadcastStream;

#[derive(Clone, Debug)]
enum Notification {
    Reload,
    Close,
}

pub struct DevTask {
    // Options for running the application in watch mode
    options: DevOptions,

    // Signal used to shutdown the processes
    interrupt_signal: Sender<()>,
}

impl DevTask {
    pub fn new(options: DevOptions) -> Self {
        let (interrupt_signal, _) = channel(1);
        DevTask {
            options,
            interrupt_signal,
        }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        let opts = &self.options;
        let interrupt_signal = self.interrupt_signal.clone();
        let (build_done_tx, mut build_done_rx) = channel::<()>(1);
        let (tx_notify, _rx_notify) = channel::<Notification>(16);

        {
            let tx_notify = tx_notify.clone();
            tokio::spawn({
                async move {
                    tokio::signal::ctrl_c().await.ok();
                    log::info!("Exiting...");
                    let _ = interrupt_signal.send(());
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
                    if let Err(err) = build_done_rx.recv().await {
                        log::error!("{err}");
                    }
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
        log::info!("Starting application watch mode");

        let opts = &self.options;
        let interrupt_signal = self.interrupt_signal.clone();
        let (tx_watch, mut rx_watch) = channel::<Vec<DebouncedEvent>>(4);

        // Starts the file system watcher
        self.build_watcher(tx_watch)?;

        // Starts
        {
            let opts = opts.clone();
            let build_done_tx = build_done_tx.clone();
            let shutdown_signal = interrupt_signal.clone();

            tokio::spawn(async move {
                log::debug!("Starting dev...");
                build_and_run(opts, shutdown_signal, build_done_tx, vec![], true).await;
            });
        }

        // Start notifier loop
        let opts = opts.clone();

        tokio::task::spawn(async move {
            loop {
                let shutdown_signal = interrupt_signal.clone();

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
                    build_and_run(opts, shutdown_signal, build_done_tx, events, false).await;
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

fn remove_ignored_paths(opts: &DevOptions, events: &mut Vec<DebouncedEvent>) {
    if events.is_empty() {
        return;
    }

    let mut ignore_paths = opts.ignore.clone();
    ignore_paths.push(PathBuf::from(".git"));
    ignore_paths.push(PathBuf::from(".gitignore"));

    if let Some(target_dir) = opts.target_dir.clone() {
        ignore_paths.push(target_dir);
    }

    // Remove any path that is within the paths to ignore
    let mut idx = 0;

    'outer: loop {
        if idx >= events.len() {
            break;
        }

        let event = &events[idx];

        for ignore_path in &ignore_paths {
            if !ignore_path.exists() {
                continue;
            }

            match (ignore_path.canonicalize(), event.path.canonicalize()) {
                (Ok(ignore_path), Ok(event_path)) => {
                    // If the ignore path contains the affected path, we remove the path from the event list
                    if let Ok(_) = event_path.strip_prefix(ignore_path) {
                        log::debug!("Ignoring path: {}", event.path.display());
                        events.remove(idx);
                        break 'outer;
                    }
                }
                _ => {}
            }
        }

        idx += 1;
    }
}

async fn build_and_run(
    opts: DevOptions,
    shutdown_signal: Sender<()>,
    build_done_tx: Sender<()>,
    mut events: Vec<DebouncedEvent>,
    is_first_run: bool,
) {
    remove_ignored_paths(&opts, &mut events);

    if events.is_empty() && !is_first_run {
        return;
    }

    let paths = events.iter().map(|e| &e.path).cloned().collect::<Vec<_>>();
    log::info!("change detected on paths: {:?}", paths);

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
