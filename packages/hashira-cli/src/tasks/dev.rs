use crate::emojis;
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
use std::sync::atomic::AtomicBool;
use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tokio::sync::{
    broadcast::{channel, Receiver, Sender},
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
        let (interrupt_signal, _) = channel(1);

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
        let (build_done_tx, build_done_rx) = channel::<()>(1);
        let (tx_live_reload, _) = channel::<LiveReloadAction>(16);

        // Starts the watcher
        self.start_watcher(build_done_tx, tx_live_reload.clone())?;

        // Wait until shutdown signal is received
        {
            let tx_shutdown = tx_shutdown.clone();

            tokio::spawn({
                async move {
                    tokio::signal::ctrl_c().await.ok();
                    tracing::info!("{}Exiting...", emojis::BYE);
                    let _ = tx_shutdown.send(());

                    // FIXME: Maybe is redundant to send a shutdown signal if we are exiting the process
                    std::process::exit(0);
                }
            });
        }

        // Spawn on reload/loading notifiers
        self.spawn_live_reload_notifiers(tx_live_reload.clone(), build_done_rx);

        // Start live-reload web-socket server
        let host = self.reload_host.as_str();
        let port = self.reload_port;
        let state = State {
            tx_shutdown,
            tx_live_reload,
        };

        start_server(state, host, port).await?;

        Ok(())
    }

    fn spawn_live_reload_notifiers(
        &self,
        tx_live_reload: Sender<LiveReloadAction>,
        mut build_done_rx: Receiver<()>,
    ) {
        // Wait until we receive an interrupt signal, we sent a loading notification to the client
        {
            let mut interrupt = self.interrupt_signal.subscribe();
            let tx_live_reload = tx_live_reload.clone();
            tokio::spawn(async move {
                loop {
                    if let Err(err) = interrupt.recv().await {
                        tracing::error!("{err}");
                    }

                    tracing::debug!("Received interrupt signal");
                    let _ = tx_live_reload.send(LiveReloadAction::Loading);
                }
            });
        }

        // We wait until the build is done, we sent a load notification to the client
        tokio::spawn(async move {
            loop {
                if let Err(err) = build_done_rx.recv().await {
                    tracing::error!("{err}");
                }

                tracing::debug!("Received build done signal");
                let _ = tx_live_reload.send(LiveReloadAction::Reload);
            }
        });
    }

    fn start_watcher(
        &self,
        build_done_tx: Sender<()>,
        tx_live_reload: Sender<LiveReloadAction>,
    ) -> anyhow::Result<()> {
        tracing::info!("{}Starting application in watch mode", emojis::SEMAPHORE);

        let build_options = &self.options;
        let interrupt_signal = self.interrupt_signal.clone();
        let (tx_watch, mut rx_watch) = channel::<Vec<DebouncedEvent>>(8);

        let opts = Arc::new(RestartOptions {
            build_options: build_options.clone(),
            ignore: self.ignore.clone(),
            host: self.host.clone(),
            port: self.port,
            reload_host: self.reload_host.clone(),
            reload_port: self.reload_port,
            static_dir: self.static_dir.clone(),
            build_done_signal: build_done_tx,
            interrupt_signal,
            tx_live_reload,
        });

        // Starts the file system watcher
        self.build_watcher(tx_watch)?;

        // Start
        tracing::debug!("Starting watch...");
        tokio::spawn(restart(opts.clone(), vec![], true));

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

                tracing::info!("{}Restarting...", emojis::RESTART);
                tokio::spawn(restart(opts, events, false));
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
                .watch(watch, RecursiveMode::Recursive)
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

fn remove_ignored_paths(opts: &RestartOptions, events: &mut Vec<DebouncedEvent>) {
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

struct RestartOptions {
    build_options: Arc<BuildOptions>,
    ignore: Vec<PathBuf>,
    host: String,
    port: u16,
    reload_host: String,
    reload_port: u16,
    static_dir: String,
    build_done_signal: Sender<()>,
    interrupt_signal: Sender<()>,
    tx_live_reload: Sender<LiveReloadAction>,
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
        if file_path.strip_prefix(&src_dir).is_ok() {
            return true;
        }
    }

    false
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LiveReloadAction {
    Reload,
    Loading,
}

#[allow(clippy::bool_comparison)]
async fn restart(opts: Arc<RestartOptions>, mut events: Vec<DebouncedEvent>, is_first_run: bool) {
    static BUILD_LOCK: Mutex<()> = Mutex::const_new(());

    // Remove any ignored path
    remove_ignored_paths(&opts, &mut events);

    if !is_first_run {
        // Only assets changed, reload
        if !change_inside_src(&events) {
            tracing::debug!("assets changed");

            // Notify clients is loading
            opts.tx_live_reload.send(LiveReloadAction::Loading).ok();

            // We rebuild the assets
            let build_task = BuildTask::new(opts.build_options.as_ref().clone());
            if let Err(err) = build_task.build_assets().await {
                tracing::error!("failed to build assets: {}", err);
            }

            // Notify clients to reload
            opts.tx_live_reload.send(LiveReloadAction::Reload).ok();

            // No reason to continue
            return;
        }
    }

    // // A guard to prevent concurrent access to build the app
    let lock = match BUILD_LOCK.try_lock() {
        Ok(x) => {
            // Interrupt the current build task, if any
            tracing::debug!("src changed, sending interrupt signal...");
            let _ = opts.interrupt_signal.send(());
            x
        }
        Err(_) => {
            return;
        }
    };

    if events.is_empty() && !is_first_run {
        return;
    }

    let paths = events.iter().map(|e| &e.path).cloned().collect::<Vec<_>>();
    if !paths.is_empty() {
        tracing::info!("Change detected on: {:#?}", paths);
    }

    // Run task
    let mut run_task = RunTask {
        envs: Default::default(),
        host: opts.host.clone(),
        port: opts.port,
        is_dev: true,
        static_dir: opts.static_dir.clone(),
        options: opts.build_options.clone(),
        build_done_signal: Some(opts.build_done_signal.clone()),
        interrupt_signal: Some(opts.interrupt_signal.clone()),
    };

    let host = opts.reload_host.clone();
    let port = opts.reload_port.to_string();

    run_task.env(crate::env::HASHIRA_LIVE_RELOAD_HOST, host);
    run_task.env(crate::env::HASHIRA_LIVE_RELOAD_PORT, port);
    run_task.env(crate::env::HASHIRA_LIVE_RELOAD, String::from("1"));

    // Build client and server
    match run_task.build().await {
        Ok(true) => {
            tracing::debug!("Build completed successfully");
        }
        Ok(false) => {
            tracing::debug!("Build interrupted");
            // The build was interrupted, so we return
            return;
        }
        Err(err) => {
            tracing::error!("Failed: {err}");
        }
    }

    // Release the lock
    drop(lock);

    // Execute binary
    if let Err(err) = run_task.exec().await {
        tracing::error!("Failed: {err}");
    }
}

struct State {
    tx_shutdown: Sender<()>,
    tx_live_reload: Sender<LiveReloadAction>,
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
    use std::sync::atomic::Ordering;

    static LOADING: AtomicBool = AtomicBool::new(false);

    #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
    #[serde(untagged)]
    enum LiveReloadEvent {
        Loading { loading: bool },
        Reload { reload: bool },
    }

    upgrade.on_upgrade(|ws| async move {
        tracing::debug!("Livereload web socket opened");

        // split the websocket into a sender and a receiver
        let (mut sender, _) = ws.split();
        let mut shutdown = state.tx_shutdown.subscribe();
        let mut event_stream = BroadcastStream::new(state.tx_live_reload.subscribe());

        if LOADING.load(Ordering::Acquire) {
            let json = serde_json::to_string(&LiveReloadEvent::Loading { loading: true })
                .expect("Failed to serialize message");

            if let Err(err) = sender.send(Message::Text(json)).await {
                tracing::debug!("Failed to send message to new web socket: {err}")
            };
        }

        loop {
            tokio::select! {
                event = event_stream.next() => {
                    tracing::debug!("Sending `{event:?}` message");

                    if let Some(Ok(event)) = event {
                        let json = match event {
                            LiveReloadAction::Reload => {
                                LOADING.store(false, Ordering::Relaxed);

                                serde_json::to_string(&LiveReloadEvent::Reload{ reload: true })
                                       .expect("Failed to serialize message")
                            },
                            LiveReloadAction::Loading => {
                                LOADING.store(true, Ordering::Relaxed);

                                serde_json::to_string(&LiveReloadEvent::Loading { loading: true })
                                         .expect("Failed to serialize message")
                            }
                        };

                        if sender.send(Message::Text(json)).await.is_err() {
                            break;
                        }
                    }
                }
                _ = shutdown.recv() => {
                    tracing::debug!("Shuting down livereload web socket");
                    return;
                }
            }
        }

        tracing::debug!("Livereload web socket closed");
    })
}
