use crossterm::event::{self, Event, KeyEvent, KeyEventKind};
use std::time::Duration;

pub enum AppEvent {
    Key(KeyEvent),
    Tick,
}

pub struct EventHandler {
    receiver: tokio::sync::mpsc::UnboundedReceiver<AppEvent>,
    _handle: tokio::task::JoinHandle<()>,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
        let _handle = tokio::spawn(async move {
            loop {
                if event::poll(tick_rate).unwrap_or(false) {
                    if let Ok(evt) = event::read() {
                        if let Event::Key(key) = evt {
                            if key.kind == KeyEventKind::Press {
                                if sender.send(AppEvent::Key(key)).is_err() {
                                    return;
                                }
                            }
                        }
                    }
                }
                if sender.send(AppEvent::Tick).is_err() {
                    return;
                }
            }
        });
        Self { receiver, _handle }
    }

    pub async fn next(&mut self) -> Option<AppEvent> {
        self.receiver.recv().await
    }
}
