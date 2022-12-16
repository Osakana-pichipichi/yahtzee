use crossterm::event::{poll, read, Event, KeyEvent};
use std::{
    sync::mpsc::{channel, Receiver, RecvError, Sender},
    thread,
    time::Duration,
};

pub enum InputEvent {
    Input(KeyEvent),
    Resize(u16, u16),
    Tick,
}

pub struct Events {
    rx: Receiver<InputEvent>,
    // Need to be kept around to prevent disposing the sender side.
    _tx: Sender<InputEvent>,
}

impl Events {
    pub fn new(tick_rate: Duration) -> Events {
        let (tx, rx) = channel();

        let event_tx = tx.clone(); // the thread::spawn own event_tx
        thread::spawn(move || {
            loop {
                // poll for tick rate duration, if no event, sent tick event.
                if poll(tick_rate).unwrap() {
                    match read().unwrap() {
                        Event::Key(event) => event_tx.send(InputEvent::Input(event)).unwrap(),
                        Event::Resize(width, height) => {
                            event_tx.send(InputEvent::Resize(width, height)).unwrap()
                        }
                        _ => (),
                    }
                } else {
                    event_tx.send(InputEvent::Tick).unwrap();
                }
            }
        });

        Events { rx, _tx: tx }
    }

    /// Attempts to read an event.
    /// This function block the current thread.
    pub fn next(&self) -> Result<InputEvent, RecvError> {
        self.rx.recv()
    }
}
