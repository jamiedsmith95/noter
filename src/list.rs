use crossterm::cursor::SetCursorStyle;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use ratatui::widgets::block::Position;
use ratatui::widgets::{Block, Paragraph};
use style::Styled;
use std::fmt::Display;
use std::path::PathBuf;
use std::str::FromStr;

use ratatui::widgets::block::Title;

use crate::app::{App, CurrentFrame};
use crate::file_reader::get_notes;
use crate::{note::Note, traits::ThisFrame};

#[derive(Debug, Default, Clone)]
pub struct MyList {
    pub notes: Vec<Note>,
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
        let found_notes = get_notes("/mnt/g/My Drive/JamiesVault");
        MyList {
            notes: found_notes,
            index: 0,
            path: PathBuf::from_str("/mnt/g/My Drive/JamiesVault/").unwrap(),
            is_active: false,
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
            " Back ".into(),
            "<esc>".bold().blue(),
        ]))
    }

    fn get_type(self) -> String {
        "List".to_string()
    }

    fn handle_key_event(&mut self, app: &mut App, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => app.exit(),
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
                app.note.is_active = true;
                app.current_frame = CurrentFrame::Note;
            }
            KeyCode::Esc => {
                app.note_list.is_active = false;
                app.current_frame = CurrentFrame::Splash;
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
    type State = (usize, Vec<Note>);
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
            .map(|note| note.title.to_string())
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
