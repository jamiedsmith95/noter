use config::Config;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use ratatui::widgets::block::Position;
use ratatui::widgets::{Block, Paragraph};
use style::Styled;
use std::collections::HashMap;
use std::fmt::Display;
use std::path::PathBuf;
use std::rc::Rc;
use std::str::FromStr;

use ratatui::widgets::block::Title;

use crate::app::{App, CurrentFrame};
use crate::file_reader::get_notes;
use crate::utils::{rc_rc, RcRc};
use crate::{note::Note, traits::ThisFrame};

#[derive(Debug, Default, Clone)]
pub struct MyList {
    pub notes: Vec<RcRc<Note>>,
    pub index: usize,
    pub path: PathBuf,
    pub is_active: bool,
}

impl Display for MyList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.notes)
    }
}

impl ThisFrame for MyList {
    fn new() -> Self {
        let config = Config::builder().add_source(config::File::with_name("/home/jsmith49/.config/noter/config")).build().unwrap();
        let path = config.try_deserialize::<HashMap<String,String>>().unwrap().get("path").unwrap().to_owned();
        let found_notes = get_notes(path.as_str());
        MyList {
            notes: found_notes,
            index: 0,
            path: PathBuf::from_str(path.as_str()).unwrap(),
            is_active: true,
        }
    }
    fn get_instructions(&self) -> ratatui::widgets::block::Title {
        Title::from(text::Line::from(vec![
            " Quit ".into(),
            "<q>".bold().red(),
            " Scroll Up ".into(),
            "<UP>".bold().blue(),
            " Scroll Down".into(),
            "<DOWN".bold().blue(),
            " Select Note ".into(),
            "<ENTER>".bold().blue(),
            " New Note ".into(),
            "<n>".bold().blue()
        ]))
    }

    fn get_type(self) -> String {
        "List".to_string()
    }

    fn handle_key_event(&mut self, app: &mut App, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => app.exit(),
            KeyCode::Char('n') => {
                app.note_list.is_active = false;
                app.note = rc_rc(Note::create_note());
                let mut note = app.note.borrow_mut();
                note.is_active = true;
                app.current_frame = CurrentFrame::Note;
            }
            KeyCode::Up => {
                if app.note_list.index == 0 {
                    app.note_list.index = self.notes.len() - 1;
                } else {
                    app.note_list.index -= 1;
                }
            }
            KeyCode::Down => {
                if app.note_list.index == self.notes.len() - 1 {
                    app.note_list.index = 0;
                } else {
                    app.note_list.index += 1;
                }
            }
            KeyCode::Enter => {
                app.note = app.note_list.notes.get(app.note_list.index).unwrap().to_owned();
                app.note_list.is_active = false;
                app.note.borrow_mut().is_active = true;
                app.current_frame = CurrentFrame::Note;
            }
            _ => {}
        }
    }
}

impl Widget for &MyList {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        // let title = Title::from(" Note List ".bold().green());
        // let block = Block::bordered()
        //     .title(title.alignment(Alignment::Center))
        //     .title(
        //         self.get_instructions()
        //             .alignment(Alignment::Center)
        //             .position(Position::Bottom),
        //     )
        //     .border_set(symbols::border::ROUNDED)
        //     .green();
    }
}

impl StatefulWidget for &MyList {
    type State = (usize, Vec<RcRc<Note>>);
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let title_text = if self.is_active {
            " Note List".green().bold()
        } else {
            " Note List ".green().dim()
        };
        let title = Title::from(title_text);
        let mut block = Block::bordered()
            .title(title.alignment(Alignment::Center))
            .title(
                self.get_instructions()
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .border_set(symbols::border::ROUNDED);
        if self.is_active {
            block = block.set_style(Color::White);
        } else {
            block = block.set_style(Color::Green)
        }
        let list: Vec<String> = state.1
            .iter()
            .map(|note| note.borrow_mut().title.to_string())
            .collect();
        let mut count = 0;
        let text: Vec<text::Line> = list
            .iter()
            .map(|title| {
                let colour = if state.0  == count {
                    Color::Blue
                } else {
                    Color::Green
                };
                count += 1;
                text::Line::raw(title).style(colour)
            })
            .collect();
        Paragraph::new(text)
            .left_aligned()
            .block(block)
            .bg(Color::Black)
            .render(area, buf);
    }
}
