use std::{cell::RefCell, fmt::Display, fs, path::Path, rc::Rc, str::Lines};

use crate::{
    app::{App, CurrentFrame, InputMode},
    file_reader::write_file,
    traits::ThisFrame,
};
use crate::{
    file_reader::parse_file,
    utils::{rc_rc, RcRc},
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Styled, Stylize},
    symbols::border,
    text::{Line, Span, Text, ToSpan, ToText},
    widgets::{
        block::{Position, Title},
        Block, BorderType, Borders, Paragraph, Widget,
    },
};
use regex::Regex;

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
        match self.mode {
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
            title: "Enter Title".to_string(),
            text: "Text Here".to_string(),
            links: None,
            tags: None,
            mode: InputMode::Normal,
            edited: false,
            is_active: true,
            old_title: None,
        }
    }
    fn get_type(self) -> String {
        "Note".to_owned()
    }
    fn handle_key_event(&mut self, app: &mut App, key_event: KeyEvent) {
        let mut note = app.note.borrow_mut();
        match (key_event.code, &self.mode) {
            (KeyCode::Char('q'), InputMode::Normal) => app.exit = true,
            (KeyCode::Char('i'), InputMode::Normal) => {
                note.mode = InputMode::Insert;
            }
            (KeyCode::Char('s'), InputMode::Normal) => {
                if note.edited {
                    write_file(&mut note.clone());
                    note.edited = false;
                }
            }
            (KeyCode::Char('t'), InputMode::Normal) => {
                note.mode = InputMode::EditTitle;
                note.old_title = Some(self.title.clone());
                app.cursor_column = 0;
            }
            (KeyCode::Backspace, InputMode::EditTitle) => {
                if app.cursor_column == 0 {
                } else {
                    note.edited = true;
                    let (first, second) = note.title.as_mut_str().split_at(app.cursor_column);
                    app.cursor_column -= 1;
                    note.title = first.split_at(app.cursor_column).0.to_string() + second;
                }
            }
            (KeyCode::Char(c), InputMode::EditTitle) => {
                note.edited = true;
                let (first, second) = note.title.as_mut_str().split_at(app.cursor_column);
                note.title = first.to_string() + &c.to_string() + second;
                app.cursor_column += 1;
            }
            // (KeyCode::Enter, InputMode::Insert) => {
            //     note.edited = true;
            //     note.edited = true;
            //     let (first, second) = note.text.split_at(app.cursor_column);
            //     note.text = first.to_string().to_owned() + "\n".to_string().as_str() + second;
            //     let text = Text::raw(&note.text);
            //     note.text = text.to_string();
            //     app.cursor_column = 0;
            //     app.cursor_row += 1;
            // }
            (KeyCode::Enter, InputMode::EditTitle) => {
                note.mode = InputMode::Normal;
                app.cursor_column = 0;
                note.edited = false;
                note.is_active = false;
                if self.old_title.is_none() || self.old_title.as_mut().unwrap().is_empty() {
                    write_file(&mut note);
                    app.note_list
                        .notes
                        .push(parse_file(note.text.clone(), Path::new(&note.title)));
                } else {
                    let mut path = app.note_list.path.clone();
                    path.push(note.old_title.as_ref().unwrap().to_owned() + ".md");
                    fs::remove_file(path).unwrap();
                    write_file(&mut note);
                }
                note.is_active = true;
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
            (KeyCode::Up, InputMode::Normal | InputMode::Insert) => {
                if app.cursor_row > 0 {
                    app.cursor_row -= 1;
                    app.cursor_column = 0;
                }
            }
            (KeyCode::Down, InputMode::Normal | InputMode::Insert) => {
                if app.cursor_row < Text::raw(&note.text).lines.len() - 1 {
                    app.cursor_row += 1;
                }
            }
            (KeyCode::Left, InputMode::Normal | InputMode::Insert) => {
                if app.cursor_column > 0 {
                    app.cursor_column -= 1;
                }
            }
            (KeyCode::Right, InputMode::Normal | InputMode::Insert) => {
                if app.cursor_column
                    < Text::raw(self.text.clone())
                        .lines
                        .get(app.cursor_row)
                        .unwrap()
                        .to_string()
                        .len()
                {
                    app.cursor_column += 1;
                }
            }
            (KeyCode::Esc, InputMode::Normal) => {
                note.is_active = false;
                app.current_frame = CurrentFrame::List;
                app.note_list.is_active = true;
            }
            (KeyCode::Esc, InputMode::Insert) => note.mode = InputMode::Normal,
            (KeyCode::Backspace, InputMode::Insert) => {
                if app.cursor_column == 0 {
                } else {
                    note.edited = true;
                    let mut lines = Text::raw(&note.text).lines;
                    let mut line = lines.get(app.cursor_row).unwrap().to_string();
                    let (first, second) = line.split_at(app.cursor_column);
                    app.cursor_column -= 1;
                    line = first.split_at(app.cursor_column).0.to_string() + second;
                    lines[app.cursor_row] = Line::raw(line);
                    let text: Text = lines.into();
                    note.text = text.to_string();
                }
            }
            (KeyCode::Char(c), InputMode::Insert) => {
                note.edited = true;
                let mut lines = Text::raw(&note.text).lines;
                let mut line = lines.get(app.cursor_row).unwrap().to_string();
                let (first, second) = line.split_at(app.cursor_column);
                line = first.to_string().to_owned() + &c.to_string() + second;
                lines[app.cursor_row] = Line::raw(line);
                let text: Text = lines.into();
                note.text = text.to_string();
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
        let mut my_border = border::ROUNDED;
        my_border.horizontal_top = border::EMPTY.top_left;
        let mut block = Block::bordered()
            .title(title.alignment(Alignment::Center))
            .title(
                note_instruction
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .border_set(my_border)
            .style(Color::Black);
        if self.is_active {
            block = block.set_style(Color::White);
        } else {
            block = block.set_style(Color::Green)
        }

        let link_regex = Regex::new(r"(?:\(.+?\))").unwrap();
        let style_text: Vec<Span> = {
            self.text
                .split_inclusive(r" ")
                .map(|token| {
                    if token.starts_with("#") {
                        token.magenta()
                    } else {
                        token.green()
                    }
                })
                .collect()
        };
        let line_text = Line::from(style_text);
        let text = Text::from(line_text) ;

        Paragraph::new(text)
            .left_aligned()
            .block(block)
            .render(area, buf);
    }
}
