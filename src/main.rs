mod app;
mod assets;
mod events;
mod game_data;
mod hand;
mod play;
mod score_table;
mod scoring;
mod ui;

use crate::app::{App, AppReturn};
use crate::events::Events;
use crate::ui::draw_ui;
use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{cell::RefCell, io, panic, rc::Rc, time::Duration};

pub fn start_ui(app: Rc<RefCell<App>>) -> Result<()> {
    let panic_hook = panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        if let Err(e) = execute!(io::stdout(), LeaveAlternateScreen) {
            println!("could not leave the alternate screen: {:?}", e);
        };
        if let Err(e) = disable_raw_mode() {
            println!("could not disable the raw mode: {:?}", e);
        };
        panic_hook(info);
    }));

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = Duration::from_millis(30);
    let events = Events::new(tick_rate);

    loop {
        let mut app = app.borrow_mut();

        terminal.draw(|f| draw_ui(f, &app))?;

        let result = app.do_action(events.next()?);

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
    let app = Rc::new(RefCell::new(App::new()));
    start_ui(app)?;
    Ok(())
}
