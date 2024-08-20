use std::io::{stdout, Result, Stdout};

use crossterm::{terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}, ExecutableCommand};
use ratatui::prelude::*;


pub type Tui = Terminal<CrosstermBackend<Stdout>>;

pub fn init() -> Result<Tui> {
    stdout().execute(EnterAlternateScreen).unwrap();
    enable_raw_mode().unwrap();
    Terminal::new(CrosstermBackend::new(stdout()))
}

pub fn restore() -> Result<()> {
    stdout().execute(LeaveAlternateScreen).unwrap();
    disable_raw_mode().unwrap();
    Ok(())
}
