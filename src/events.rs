use crossterm::event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers};
use std::{
    sync::mpsc::{channel, Receiver, RecvError, Sender},
    thread,
    time::Duration,
};

pub enum Actions {
    Select,
    Up,
    Down,
    Right,
    Left,
    Exit,
    Pass,
}

pub enum InputEvent {
    Input(KeyEvent),
    Resize(u16, u16),
    Tick,
}

impl InputEvent {
    pub fn action(&self) -> Actions {
        match self {
            InputEvent::Input(KeyEvent {
                code: KeyCode::Enter | KeyCode::Char(' '),
                ..
            }) => Actions::Select,

            InputEvent::Input(KeyEvent {
                code: KeyCode::Up | KeyCode::Char('w'),
                ..
            }) => Actions::Up,

            InputEvent::Input(KeyEvent {
                code: KeyCode::Down | KeyCode::Char('s'),
                ..
            }) => Actions::Down,

            InputEvent::Input(KeyEvent {
                code: KeyCode::Left | KeyCode::Char('a'),
                ..
            }) => Actions::Left,

            InputEvent::Input(KeyEvent {
                code: KeyCode::Right | KeyCode::Char('d'),
                ..
            }) => Actions::Right,

            InputEvent::Input(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            }) => Actions::Exit,

            _ => Actions::Pass,
        }
    }
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
