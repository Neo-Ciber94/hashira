use crate::{
    cli::{BuildOptions, DevOptions},
    tasks::{build::BuildTask, run::RunTask},
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
    Mutex,
};
use tokio_stream::wrappers::BroadcastStream;

pub struct DevTask {
    // Options for running the application in watch mode
    options: Arc<BuildOptions>,

    // Path in the server to serve the static files
    static_dir: String,

    // Host to run the server
    host: String,

    // Port to run the server
    port: u16,

    // Host of the reload server
    reload_host: String,

    // Port of the reload server
    reload_port: u16,

    // Additional paths to watch
    watch: Vec<PathBuf>,

    // Paths to ignore while waiting for changes
    ignore: Vec<PathBuf>,

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
            watch: options.watch,
            ignore: options.ignore,
        }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        let (tx_shutdown, _) = channel::<()>(1);
        let (build_done_tx, mut build_done_rx) = channel::<()>(1);
        let (tx_notify, _rx_notify) = channel::<()>(16);

        // Starts the watcher
        self.start_watcher(tx_notify.clone(), build_done_tx)?;

        // Wait until shutdown signal is received
        {
            let tx_notify = tx_notify.clone();
            let tx_shutdown = tx_shutdown.clone();

            tokio::spawn({
                async move {
                    tokio::signal::ctrl_c().await.ok();
                    tracing::info!("👋 Exiting...");
                    let _ = tx_shutdown.send(());
                    tx_notify
                        .send(())
                        .unwrap_or_else(|_| panic!("failed to send shutdown signal"));

                    // FIXME: Maybe is redundant to send a shutdown signal if we are exiting the process
                    std::process::exit(0);
                }
            });
        }

        // We wait until the build is done, we sent a notification to the client
        {
            let tx_notify = tx_notify.clone();

            tokio::spawn(async move {
                loop {
                    if let Err(err) = build_done_rx.recv().await {
                        tracing::error!("{err}");
                    }

                    tracing::debug!("Received build done signal");
                    if let Err(err) = tx_notify.send(()) {
                        tracing::error!("Error sending change event: {err}");
                    }
                }
            });
        }

        // Start live-reload web-socket server
        let host = self.reload_host.as_str();
        let port = self.reload_port;
        let state = State {
            tx_notify,
            tx_shutdown,
            tx_watch: self.interrupt_signal.clone(),
        };

        start_server(state, host, port).await?;

        Ok(())
    }

    fn start_watcher(
        &self,
        tx_notify: Sender<()>,
        build_done_tx: Sender<()>,
    ) -> anyhow::Result<()> {
        tracing::info!("🚦 Starting application in watch mode");

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
            reload_port: self.reload_port,
            static_dir: self.static_dir.clone(),
            tx_notify: tx_notify.clone(),
            build_done_signal: build_done_tx,
            interrupt_signal: interrupt_signal.clone(),
        });

        // Starts the file system watcher
        self.build_watcher(tx_watch)?;

        // Starts
        tracing::debug!("Starting watch...");
        tokio::spawn(build_and_run(opts.clone(), vec![], true));

        // Start notifier loop
        tokio::task::spawn(async move {
            loop {
                // Wait for change event
                let events = rx_watch
                    .recv()
                    .await
                    .expect("failed to read debounce event");

                // Rerun
                let opts = opts.clone();

                tracing::info!("🔃 Restarting...");
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
        tracing::info!("Watching: {}", watch_path.display());

        // Watch base path
        debouncer
            .watcher()
            .watch(&watch_path, RecursiveMode::Recursive)
            .expect("failed to watch directory");

        // Watch any additional path
        for watch in &self.watch {
            anyhow::ensure!(
                watch.exists(),
                "path to watch `{}` was not found",
                watch.display()
            );
            debouncer
                .watcher()
                .watch(&watch, RecursiveMode::Recursive)
                .unwrap();

            tracing::info!("Watching: {}", watch.display());
        }

        std::thread::spawn(move || {
            // We hold this otherwise the notify channel will be dropped
            let _debouncer = debouncer;

            loop {
                match rx_debounced.recv() {
                    Ok(event) => {
                        if let Ok(evt) = event {
                            if let Err(err) = tx_watch.send(evt) {
                                tracing::error!("Failed to send debounced event: {err}");
                            }
                        }
                    }
                    Err(err) => {
                        tracing::error!("Failed to received debounce event: {err}");
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
    ignore_paths.extend(target_dir);

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

            if let (Ok(ignore_path), Ok(event_path)) =
                (ignore_path.canonicalize(), event.path.canonicalize())
            {
                // If the ignore path contains the affected path, we remove the path from the event list
                if event_path.strip_prefix(ignore_path).is_ok() {
                    tracing::debug!("Ignoring path: {}", event.path.display());
                    events.remove(idx);
                    break 'outer;
                }
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
    tx_notify: Sender<()>,
    build_done_signal: Sender<()>,
    interrupt_signal: Sender<()>,
}

fn change_inside_src(events: &[DebouncedEvent]) -> bool {
    let cwd = std::env::current_dir().unwrap();
    let src_dir = dunce::canonicalize(cwd.join("src")).unwrap();

    let files = events
        .iter()
        .filter(|event| event.path.is_file())
        .map(|event| &event.path);

    for file in files {
        let file_path = dunce::canonicalize(file).unwrap();
        match file_path.strip_prefix(&src_dir) {
            Ok(_) => return true,
            Err(_) => {}
        }
    }

    false
}

#[allow(clippy::bool_comparison)]
async fn build_and_run(
    opts: Arc<BuildAndRunOptions>,
    mut events: Vec<DebouncedEvent>,
    is_first_run: bool,
) {
    // Remove any ignored path
    remove_ignored_paths(&opts, &mut events);

    if !is_first_run {
        // Only assets changed, reload
        if !change_inside_src(&events) {
            tracing::debug!("assets changed");

            // We rebuild the assets
            let build_task = BuildTask::new(opts.build_options.as_ref().clone());
            if let Err(err) = build_task.build_assets().await {
                tracing::error!("failed to build assets: {}", err);
            }

            let _ = opts.tx_notify.send(());
            return;
        } else {
            // Interrupt the current running task
            tracing::debug!("src changed, sending interrupt signal...");
            let _ = opts.interrupt_signal.send(());
        }
    }

    // A guard to prevent concurrent access to run the app
    let mut lock = opts.can_run.lock().await;
    if *lock == false {
        return;
    }

    *lock = false;

    if events.is_empty() && !is_first_run {
        return;
    }

    let paths = events.iter().map(|e| &e.path).cloned().collect::<Vec<_>>();
    if !paths.is_empty() {
        tracing::info!("Change detected on: {:#?}", paths);
    }

    // Build task
    let mut run_task = RunTask {
        envs: Default::default(),
        host: opts.host.clone(),
        port: opts.port,
        static_dir: opts.static_dir.clone(),
        options: opts.build_options.clone(),
        build_done_signal: Some(opts.build_done_signal.clone()),
        interrupt_signal: Some(opts.interrupt_signal.clone()),
    };

    // TODO: We should decide what operation to perform depending on the files affected,
    // if only public assets are changed, we don't need to rebuild the entire app

    let host = opts.reload_host.clone();
    let port = opts.reload_port.to_string();

    run_task.env(crate::env::HASHIRA_LIVE_RELOAD_HOST, host);
    run_task.env(crate::env::HASHIRA_LIVE_RELOAD_PORT, port);
    run_task.env(crate::env::HASHIRA_LIVE_RELOAD, String::from("1"));

    tracing::debug!("Starting building and running...");
    if let Err(err) = run_task.run().await {
        tracing::error!("Watch run failed: {err}");
    }

    *lock = true;
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum LiveReloadEvent {
    Loading { loading: bool },
    Reload { reload: bool },
}

struct State {
    tx_notify: Sender<()>,
    tx_shutdown: Sender<()>,
    tx_watch: Sender<()>,
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

    tracing::info!("Starting hot reload server on: http://{addr}");

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
        tracing::debug!("Livereload web socket opened");

        let tx_notify = state.tx_notify.clone();
        let tx_shutdown = state.tx_shutdown.clone();
        let mut tx_watch = state.tx_watch.subscribe();

        // split the websocket into a sender and a receiver
        let (mut sender, _) = ws.split();
        let notify = tx_notify.subscribe();
        let mut shutdown = tx_shutdown.subscribe();
        let mut notify_stream = BroadcastStream::new(notify);

        loop {
            tokio::select! {
                _ = notify_stream.next() => {
                    tracing::debug!("Sending reload message...");

                    let json = serde_json::to_string(&LiveReloadEvent::Reload{ reload: true })
                        .expect("Failed to serialize message");

                    if sender.send( Message::Text(json)).await.is_err() {
                        break;
                    }
                },
                _ = tx_watch.recv() => {
                    tracing::debug!("Sending loading message...");
                    let json = serde_json::to_string(&LiveReloadEvent::Loading { loading: true })
                        .expect("Failed to serialize message");

                    if sender.send( Message::Text(json)).await.is_err() {
                        break;
                    }
                },
                _ = shutdown.recv() => {
                    tracing::debug!("Shuting down livereload web socket");
                    return;
                }
            }
        }

        tracing::debug!("Livereload web socket closed");
    })
}
