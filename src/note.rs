use std::{fmt::Display, io::Result};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{style::Stylize, text::Line, widgets::block::Title};

use crate::{
    app::{App, CurrentFrame, InputMode},
    traits::ThisFrame,
};

#[derive(Debug, Clone)]
pub struct Tag(pub String);

#[derive(Debug, Clone)]
pub struct Link(pub String);

#[derive(Debug, Default, Clone)]
pub struct Note {
    pub title: String,
    pub text: String,
    pub links: Option<Vec<Link>>,
    pub tags: Option<Vec<Tag>>,
    pub mode: InputMode,
    pub edited: bool,
}
impl Display for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.title)
    }
}

impl ThisFrame for Note {
    type FrameType = Note;
    // get key bindings for this mode.
    fn get_instructions(&self) -> Title {
        match self.mode {
            InputMode::Normal => Title::from(Line::from(vec![
                " Back ".into(),
                "<esc>".blue().bold(),
                " Save Note ".into(),
                "<s>".blue().bold(),
                " Insert Mode  ".into(),
                "<i>".blue().bold(),
                " Quit ".into(),
                "<q>".blue().bold(),
            ])),
            InputMode::Insert => Title::from(Line::from(vec![
                " Normal Mode ".into(),
                "<esc>".blue().bold(),
            ])),
        }
    }
    fn new(&self) -> Note {
        Note {
            title: "test title".to_string(),
            text: "test text".to_string(),
            links: None,
            tags: None,
            mode: InputMode::Normal,
            edited: false,
        }
    }
    fn get_type(self) -> String {
        "Note".to_owned()
    }
    fn handle_key_event(&mut self, app: &mut App<impl ThisFrame + Display>, key_event: KeyEvent) {
        match (key_event.code, &self.mode) {
            (KeyCode::Char('q'), InputMode::Normal) => app.exit = true,
            (KeyCode::Left, _) => {}
            (KeyCode::Esc, InputMode::Normal) => {}
            (KeyCode::Esc, InputMode::Insert) => self.mode = InputMode::Normal,
            _ => {}
        };
    }
}

impl Note {
    pub fn create_note() -> Self {
        Self::default()
    }
}
