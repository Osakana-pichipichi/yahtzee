mod app;
mod events;
mod hand;
mod score_table;
mod scoring;
mod ui;

use crate::app::{App, AppReturn};
use crate::events::{Events, InputEvent};
use crate::ui::draw_play_ui;
use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{cell::RefCell, io, rc::Rc, time::Duration};
use tui::{backend::CrosstermBackend, Terminal};

pub fn start_ui(app: Rc<RefCell<App>>) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = Duration::from_millis(30);
    let events = Events::new(tick_rate);

    loop {
        let mut app = app.borrow_mut();

        terminal.draw(|f| draw_play_ui(f, &app))?;

        let result = match events.next()? {
            InputEvent::Input(key) => app.do_action(key),
            _ => AppReturn::Continue,
        };

        if result == AppReturn::Exit {
            break;
        }
    }

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn main() -> Result<()> {
    let app = Rc::new(RefCell::new(App::new(2)));
    start_ui(app)?;
    Ok(())
}
