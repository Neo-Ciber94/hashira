use std::sync::Arc;
use tokio::sync::broadcast::{channel, Receiver, Sender};

thread_local! {
    pub static RUN_INTERRUPT: Interrupt = Interrupt::new();
}

#[derive(Clone)]
pub struct Interrupt {
    signal: Arc<Sender<()>>,
}

impl Interrupt {
    pub fn new() -> Self {
        let (sender, _) = channel(8);
        Interrupt {
            signal: Arc::new(sender),
        }
    }

    pub fn subscribe(&self) -> Receiver<()> {
        self.signal.subscribe()
    }

    pub fn interrupt(&self) {
        if let Err(err) = self.signal.send(()) {
            log::error!("Failed to send interrupt signal: {err}");
        }
    }
}
