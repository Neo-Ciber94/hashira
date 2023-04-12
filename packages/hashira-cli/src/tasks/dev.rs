use crate::{
    cli::{BuildOptions, DevOptions},
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
    sync::Arc,
    time::Duration,
};
use tokio::sync::{
    broadcast::{channel, Sender},
    Mutex, Semaphore,
};
use tokio_stream::wrappers::BroadcastStream;

pub struct DevTask {
    // Options for running the application in watch mode
    options: Arc<BuildOptions>,

    pub static_dir: String,

    pub host: String,

    pub port: u16,

    pub reload_host: String,

    pub reload_port: u16,

    pub ignore: Vec<PathBuf>,

    // Signal used to shutdown the processes
    interrupt_signal: Sender<()>,
}

impl DevTask {
    pub fn new(options: DevOptions) -> Self {
        let (interrupt_signal, _) = channel(8);
        DevTask {
            options: Arc::new(BuildOptions::from(&options)),
            interrupt_signal,
            host: options.host,
            port: options.port,
            static_dir: options.static_dir,
            reload_host: options.reload_host,
            reload_port: options.reload_port,
            ignore: options.ignore,
        }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        let (tx_shutdown, _) = channel::<()>(1);
        let (build_done_tx, mut build_done_rx) = channel::<()>(1);
        let (tx_notify, _rx_notify) = channel::<()>(16);

        {
            let tx_notify = tx_notify.clone();
            let tx_shutdown = tx_shutdown.clone();
            tokio::spawn({
                async move {
                    tokio::signal::ctrl_c().await.ok();
                    log::info!("Exiting...");
                    let _ = tx_shutdown.send(());
                    tx_notify
                        .send(())
                        .unwrap_or_else(|_| panic!("failed to send shutdown signal"));

                    // FIXME: Maybe is redundant to send a shutdown signal if we are exiting the process
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

                    if let Err(err) = tx_notify.send(()) {
                        log::error!("Error sending change event: {err}");
                    }
                }
            });
        }

        // Starts the watcher
        self.start_watcher(build_done_tx)?;

        // Starts the server
        let host = self.reload_host.as_str();
        let port = self.reload_port;

        let state = State {
            tx_notify,
            tx_shutdown,
        };

        start_server(state, host, port).await?;
        Ok(())
    }

    fn start_watcher(&self, build_done_tx: Sender<()>) -> anyhow::Result<()> {
        log::info!("Starting application watch mode");

        let build_options = &self.options;
        let interrupt_signal = self.interrupt_signal.clone();
        let (tx_watch, mut rx_watch) = channel::<Vec<DebouncedEvent>>(8);

        let opts = Arc::new(BuildAndRunOptions {
            can_run: Arc::new(Mutex::new(true)),
            build_options: build_options.clone(),
            ignore: self.ignore.clone(),
            host: self.host.clone(),
            port: self.port,
            reload_host: self.reload_host.clone(),
            reload_port: self.reload_port.clone(),
            static_dir: self.static_dir.clone(),
            build_done_signal: build_done_tx.clone(),
            interrupt_signal: interrupt_signal.clone(),
        });

        // Starts the file system watcher
        self.build_watcher(tx_watch)?;

        // Starts
        log::debug!("Starting dev...");
        tokio::spawn(build_and_run(opts.clone(), vec![], true));

        // Start notifier loop
        tokio::task::spawn(async move {
            loop {
                let interrupt_signal = interrupt_signal.clone();

                // Wait for change event
                let events = rx_watch
                    .recv()
                    .await
                    .expect("failed to read debounce event");

                // Interrupt the current running task
                let _ = interrupt_signal.send(());

                // Rerun
                let opts = opts.clone();

                log::info!("Restarting dev...");
                tokio::spawn(build_and_run(opts, events, false));
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

fn remove_ignored_paths(opts: &BuildAndRunOptions, events: &mut Vec<DebouncedEvent>) {
    if events.is_empty() {
        return;
    }

    let target_dir = opts.build_options.target_dir.clone();
    let mut ignore_paths = opts.ignore.clone();
    ignore_paths.push(PathBuf::from(".git"));
    ignore_paths.push(PathBuf::from(".gitignore"));

    if let Some(target_dir) = target_dir.clone() {
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

struct BuildAndRunOptions {
    build_options: Arc<BuildOptions>,
    can_run: Arc<Mutex<bool>>,
    ignore: Vec<PathBuf>,
    host: String,
    port: u16,
    reload_host: String,
    reload_port: u16,
    static_dir: String,
    build_done_signal: Sender<()>,
    interrupt_signal: Sender<()>,
}

async fn build_and_run(
    opts: Arc<BuildAndRunOptions>,
    mut events: Vec<DebouncedEvent>,
    is_first_run: bool,
) {
    let mut lock = opts.can_run.lock().await;
    if *lock == false {
        return;
    }

    *lock = false;
    remove_ignored_paths(&opts, &mut events);

    if events.is_empty() && !is_first_run {
        return;
    }

    let paths = events.iter().map(|e| &e.path).cloned().collect::<Vec<_>>();
    if paths.len() > 0 {
        log::info!("change detected on paths: {:?}", paths);
    }

    // Build task
    let mut run_task = RunTask {
        envs: Default::default(),
        host: opts.host.clone(),
        port: opts.port.clone(),
        static_dir: opts.static_dir.clone(),
        options: opts.build_options.clone(),
        build_done_signal: Some(opts.build_done_signal.clone()),
        interrupt_signal: Some(opts.interrupt_signal.clone()),
    };

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

    *lock = true;
}

#[derive(Debug, Serialize, Deserialize)]
struct ReloadMessage {
    reload: bool,
}

struct State {
    tx_notify: Sender<()>,
    tx_shutdown: Sender<()>,
}

async fn start_server(state: State, host: &str, port: u16) -> anyhow::Result<()> {
    // create a router with a websocket handler
    let app = Router::new()
        .route("/ws", get(websocket_handler))
        .layer(Extension(Arc::new(state)));

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

// this function handles websocket connections
async fn websocket_handler(
    upgrade: WebSocketUpgrade,
    state: Extension<Arc<State>>,
) -> impl IntoResponse {
    upgrade.on_upgrade(|ws| async move {
        log::debug!("Web socket upgrade");

        let tx_notify = state.tx_notify.clone();
        let tx_shutdown = state.tx_shutdown.clone();

        // split the websocket into a sender and a receiver
        let (mut sender, _receiver) = ws.split();
        let receiver = tx_notify.subscribe();
        let mut shutdown = tx_shutdown.subscribe();
        let mut stream = BroadcastStream::new(receiver);

        loop {
            tokio::select! {
                _ = stream.next() => {
                    log::debug!("Sending reload message...");

                    let json = serde_json::to_string(&ReloadMessage { reload: true })
                        .expect("Failed to serialize message");
                    let msg = Message::Text(json);
                    if let Err(err) = sender.send(msg).await {
                        log::error!("Failed to send web socket message: {err}");
                    }
                }
                _ = shutdown.recv() => {
                    log::debug!("Closing web socket");
                    return;
                }
            }
        }
    })
}
