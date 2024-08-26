use std::{cell::RefCell, fmt::Display, fs, path::Path, rc::Rc};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Styled, Stylize},
    symbols::border,
    text::Line,
    widgets::{
        block::{Position, Title},
        Block, Paragraph, Widget,
    },
};

use crate::{
    app::{rc_rc, App, CurrentFrame, InputMode, RcRc},
    file_reader::write_file,
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
    pub mode: RcRc<InputMode>,
    pub edited: bool,
    pub is_active: bool,
    pub old_title: Option<String>,
}
impl Display for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.title)
    }
}

impl ThisFrame for Note {
    // get key bindings for this mode.
    fn get_instructions(&self) -> Title {
        match self.mode.borrow().clone() {
            InputMode::Normal => Title::from(Line::from(vec![
                " Back ".into(),
                "<esc>".blue().bold(),
                " Edit Title ".into(),
                "<t>".blue().bold(),
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
            InputMode::EditTitle => Title::from(Line::from(vec![
                " Save Title ".into(),
                "<return>".blue().bold(),
            ])),
        }
    }
    fn new() -> Note {
        Note {
            title: "test title".to_string(),
            text: "test text".to_string(),
            links: None,
            tags: None,
            mode: Rc::new(RefCell::new(InputMode::Normal)),
            edited: false,
            is_active: true,
            old_title: None,
        }
    }
    fn get_type(self) -> String {
        "Note".to_owned()
    }
    fn handle_key_event(&mut self, app: &mut App, key_event: KeyEvent) {
        match (key_event.code, self.mode.clone().borrow().to_owned()) {
            (KeyCode::Char('q'), InputMode::Normal) => app.exit = true,
            (KeyCode::Char('i'), InputMode::Normal) => {
                app.note.mode = Rc::new(RefCell::new(InputMode::Insert))
            }
            (KeyCode::Char('s'), InputMode::Normal) => {
                write_file(&mut self.clone());
                app.note.edited = false;
            }
            (KeyCode::Char('t'), InputMode::Normal) => {
                app.note.mode = rc_rc(InputMode::EditTitle);
                app.note.old_title = Some(self.title.clone());
                app.cursor_column = 0;
            }
            (KeyCode::Backspace, InputMode::EditTitle) => {
                if app.cursor_column == 0 {
                } else {
                    app.note.edited = true;
                    let (first, second) = app.note.title.as_mut_str().split_at(app.cursor_column);
                    app.cursor_column -= 1;
                    app.note.title = first.split_at(app.cursor_column).0.to_string() + second;
                }
            }
            (KeyCode::Char(c), InputMode::EditTitle) => {
                app.note.edited = true;
                let (first, second) = app.note.title.as_mut_str().split_at(app.cursor_column);
                app.note.title = first.to_string() + &c.to_string() + second;
                app.cursor_column += 1;
            }
            (KeyCode::Enter, InputMode::EditTitle) => {
                if self.old_title.is_none() || self.old_title.as_mut().unwrap().is_empty() {
                    write_file(&mut app.note)
                } else {
                    let mut path = app.note_list.path.clone();
                    path.push(app.note.old_title.as_ref().unwrap().to_owned() + ".md");
                    fs::remove_file(path).unwrap();
                    write_file(&mut app.note);
                }

                app.note.mode = rc_rc(InputMode::Normal);
                app.cursor_column = 0;
                app.note.edited = false;
            }
            (KeyCode::Left, InputMode::EditTitle) => {
                if app.cursor_column > 0 {
                    app.cursor_column -= 1;
                }
            }
            (KeyCode::Right, InputMode::EditTitle) => {
                if app.cursor_column < self.title.len() {
                    app.cursor_column += 1;
                }
            }
            (KeyCode::Left, InputMode::Normal | InputMode::Insert) => {
                if app.cursor_column > 0 {
                    app.cursor_column -= 1;
                }
            }
            (KeyCode::Right, InputMode::Normal | InputMode::Insert) => {
                if app.cursor_column < self.text.len() {
                    app.cursor_column += 1;
                }
            }
            (KeyCode::Esc, InputMode::Normal) => {
                app.note.is_active = false;
                app.current_frame = CurrentFrame::Splash;
            }
            (KeyCode::Esc, InputMode::Insert) => app.note.mode = rc_rc(InputMode::Normal),
            (KeyCode::Backspace, InputMode::Insert) => {
                if app.cursor_column == 0 {
                } else {
                    app.note.edited = true;
                    let (first, second) = self.text.as_mut_str().split_at(app.cursor_column);
                    app.cursor_column -= 1;
                    app.note.text = first.split_at(app.cursor_column).0.to_string() + second;
                }
            }
            (KeyCode::Char(c), InputMode::Insert) => {
                app.note.edited = true;
                let (first, second) = self.text.as_mut_str().split_at(app.cursor_column);
                app.note.text = first.to_string() + &c.to_string() + second;
                app.cursor_column += 1;
            }
            _ => {}
        };
    }
}

impl Note {
    pub fn create_note() -> Self {
        Self::default()
    }
}

impl Widget for &Note {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let note_instruction = self.get_instructions();
        let title_text = if self.is_active {
            self.title.clone().green().bold()
        } else {
            self.title.clone().green()
        };

        let title: Title = Title::from(title_text);
        let mut block = Block::bordered()
            .title(title.alignment(Alignment::Center))
            .title(
                note_instruction
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .border_set(border::ROUNDED)
            .bg(Color::Black);
        if self.is_active {
            block = block.set_style(Color::Green);
        }
        Paragraph::new(self.text.clone())
            .left_aligned()
            .block(block)
            .render(area, buf);
    }
}
