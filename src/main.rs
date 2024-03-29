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
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::{cell::RefCell, io, panic, rc::Rc, time::Duration};

struct TuiYatzee<B>
where
    B: Backend,
{
    app: Rc<RefCell<App>>,
    events: Events,
    terminal: Terminal<B>,
}

impl<B> TuiYatzee<B>
where
    B: Backend,
{
    fn new(app: Rc<RefCell<App>>, terminal: Terminal<B>) -> Self {
        Self {
            app,
            events: Events::new(Duration::from_millis(30)),
            terminal,
        }
    }

    fn start(&mut self) -> Result<()> {
        loop {
            let mut app = self.app.borrow_mut();

            self.terminal.draw(|f| draw_ui(f, &app))?;

            match app.do_action(self.events.next()?) {
                Ok(AppReturn::Exit) => break Ok(()),
                Err(e) => break Err(e),
                _ => (),
            }
        }
    }
}

fn main() -> Result<()> {
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
    execute!(io::stdout(), EnterAlternateScreen)?;

    let terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    let app = Rc::new(RefCell::new(App::new()));
    let mut tui_yahtzee = TuiYatzee::new(app, terminal);

    let ret = tui_yahtzee.start();

    execute!(io::stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;

    ret
}
