use std::sync::Arc;
use tokio::sync::broadcast::{channel, Receiver, Sender};

#[derive(Clone)]
pub struct Interrupt {
    signal: Arc<Sender<()>>,
}

impl Interrupt {
    pub fn new() -> Self {
        let (sender, _) = channel(1);
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
