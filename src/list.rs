use config::Config;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use ratatui::widgets::block::Position;
use ratatui::widgets::{Block, Paragraph};
use std::collections::HashMap;
use std::fmt::Display;
use std::path::PathBuf;
use std::str::FromStr;
use style::Styled;

use ratatui::widgets::block::Title;

use crate::app::{App, CurrentFrame};
use crate::file_reader::{get_notes, get_tags_links};
use crate::note::Tag;
use crate::utils::{rc_rc, RcRc};
use crate::{note::Note, traits::ThisFrame};

#[derive(Debug, Default, Clone)]
pub struct MyList {
    pub notes: Vec<RcRc<Note>>,
    pub index: usize,
    pub path: PathBuf,
    pub is_active: bool,
    pub is_search: bool,
    pub search: Option<String>,
    pub tag_all: bool,
    pub local_list: bool,
    pub local_path: PathBuf,
}

impl Display for MyList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.notes)
    }
}

impl ThisFrame for MyList {
    fn new() -> Self {
        let home = std::env::home_dir().unwrap();
        let path = home.to_str().unwrap().to_string() + "/.config/noter/config";

        let config_build = Config::builder()
            .add_source(config::File::with_name(&path))
            .build()
            .unwrap();

        let path = config_build
            .try_deserialize::<HashMap<String, String>>()
            .unwrap()
            .get("path")
            .unwrap()
            .to_owned();
        let found_notes = get_notes(path.as_str());
        let cwd = std::env::current_dir().unwrap();
        MyList {
            notes: found_notes,
            index: 0,
            path: PathBuf::from_str(path.as_str()).unwrap(),
            is_active: true,
            is_search: false,
            search: None,
            tag_all: false,
            local_list: false,
            local_path: cwd,
        }
    }
    fn get_instructions(&self) -> ratatui::widgets::block::Title {
        Title::from(text::Line::from(vec![
            " Quit ".into(),
            "<q>".bold().red(),
            " Search tags ".into(),
            "<s>".bold().blue(),
            " Scroll Up ".into(),
            "<UP>".bold().blue(),
            " Scroll Down".into(),
            "<DOWN>".bold().blue(),
            " Select Note ".into(),
            "<ENTER>".bold().blue(),
            " Toggle Dir ".into(),
            "<TAB>".bold().blue(),
            " New Note ".into(),
            "<n>".bold().blue(),
        ]))
    }

    fn get_type(&self) -> String {
        "List".to_string()
    }

    fn handle_key_event(&mut self, app: &mut App, key_event: KeyEvent) {
        // let list = &app.note_list;
        match (key_event.code, self.is_search) {
            (KeyCode::Char('q'), false) => app.exit(),
            (KeyCode::Char('n'), false) => {
                app.note_list.is_active = false;
                app.note = rc_rc(Note::create_note());
                let mut note = app.note.borrow_mut();
                note.is_active = true;
                app.current_frame = CurrentFrame::Note;
                app.cursor_column = 0;
            }
            (KeyCode::Tab, false) => {
                if !self.local_list {
                    let builder = std::fs::DirBuilder::new();
                    app.note_list.local_list = true;
                    let path = PathBuf::from_str(&(self.local_path.to_string_lossy() + "/notes"));
                    builder.create(path.clone().unwrap()).unwrap_or(());
                    let mut notes = get_notes(path.unwrap_or(self.path.clone()).to_str().unwrap());
                    if notes.is_empty() {
                        notes.push(rc_rc(Note::create_note()))
                    }
                    app.note_list.notes = notes;
                    app.note_list.index = 0
                } else {
                    app.note_list.local_list = false;
                    app.note_list.notes = get_notes(self.path.to_str().unwrap());
                    app.note_list.index = 0
                }
            }
            (KeyCode::Backspace, true) => {
                if app.cursor_column == 0 {
                } else {
                    let search = &app.note_list.search.to_owned().unwrap();
                    let (first, second) = search.split_at(app.cursor_column);
                    app.cursor_column = app.cursor_column.saturating_sub(1);
                    app.note_list
                        .search
                        .replace(first.split_at(app.cursor_column).0.to_string() + second);
                }
            }
            (KeyCode::Up, false) => {
                if app.note_list.index == 0 {
                    app.note_list.index = self.notes.len() - 1;
                } else {
                    app.note_list.index = app.note_list.index.saturating_sub(1);
                }
            }
            (KeyCode::Left, true) => {
                if app.cursor_column > 0 {
                    app.cursor_column = app.cursor_column.saturating_sub(1);
                }
            }
            (KeyCode::Right, true) => {
                if app
                    .note_list
                    .search
                    .as_ref()
                    .is_some_and(|search| search.len() > app.cursor_column)
                {
                    app.cursor_column = app.cursor_column.saturating_add(1);
                }
            }
            (KeyCode::Char('s'), false) => {
                app.note_list.is_search = true;
                app.cursor_column = 0;
                app.note_list.index = 0;
            }
            (KeyCode::Esc, true) => {
                app.note_list.is_search = false;
            }
            (KeyCode::Char(c), true) => {
                if app.note_list.search.is_none() {
                    let search = &mut app.note_list.search;
                    search.replace(c.to_string());
                    app.cursor_column = app.cursor_column.saturating_add(1);
                } else {
                    let search = app.note_list.search.clone();
                    let (first, second) = search
                        .as_ref()
                        .unwrap()
                        .split_at(app.cursor_column)
                        .to_owned();
                    app.note_list
                        .search
                        .replace(first.to_owned().to_string() + &c.to_string() + second);
                    app.cursor_column = app.cursor_column.saturating_add(1);
                }
                app.note_list.index = 0;
            }
            (KeyCode::Down, false) => {
                if app.note_list.index == self.notes.len() - 1 {
                    app.note_list.index = 0;
                } else {
                    app.note_list.index = app.note_list.index.saturating_add(1);
                }
            }
            (KeyCode::Up, true) => {
                if app.note_list.index == 0 {
                    app.note_list.index = self.filter_list(self.search.clone()).unwrap().len() - 1;
                } else {
                    app.note_list.index = app.note_list.index.saturating_sub(1);
                }
            }
            (KeyCode::Down, true) => {
                if self
                    .filter_list(self.search.clone())
                    .is_some_and(|notes| notes.len() == app.note_list.index + 1)
                {
                    app.note_list.index = 0;
                } else {
                    app.note_list.index = app.note_list.index.saturating_add(1);
                }
            }
            (KeyCode::Enter, false) => {
                app.note = app
                    .note_list
                    .notes
                    .get(app.note_list.index)
                    .unwrap()
                    .to_owned();
                app.note_list.is_active = false;
                app.note.borrow_mut().is_active = true;
                app.current_frame = CurrentFrame::Note;
                app.cursor_column = 0;
            }
            (KeyCode::Tab, true) => {
                app.note_list.tag_all = !self.tag_all;
                app.note_list.index = 0;
            }
            (KeyCode::Enter, true) => {
                app.note = app
                    .note_list
                    .filter_list(self.search.clone())
                    .unwrap()
                    .get(app.note_list.index)
                    .unwrap()
                    .to_owned();
                app.note_list.is_active = false;
                app.note.borrow_mut().is_active = true;
                app.current_frame = CurrentFrame::Note;
                app.cursor_column = 0;
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
            .border_set(symbols::border::ROUNDED);
        if self.is_active {
            block = block.set_style(Color::White);
        } else {
            block = block.set_style(Color::Green)
        }
        let list: Vec<String>;
        if self.is_search {
            list = self
                .filter_list(self.search.clone())
                .unwrap()
                .iter()
                .map(|note| note.borrow().title.to_string())
                .collect();
        } else {
            list = state
                .1
                .iter()
                .map(|note| note.borrow_mut().title.to_string())
                .collect();
        };
        let mut count = 0;
        let text: Vec<text::Line> = list
            .iter()
            .map(|title| {
                let colour = if state.0 == count {
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

impl MyList {
    pub fn filter_list(&self, search: Option<String>) -> Option<Vec<RcRc<Note>>> {
        let search_tags: Vec<Tag> = match search {
            Some(tags) => tags
                .split_whitespace()
                .map(|tag| Tag("#".to_string() + tag))
                .collect(),
            None => return Some(self.notes.clone()),
        };
        if search_tags.is_empty() {
            return Some(self.notes.clone());
        }

        let mut notes = self.notes.clone();

        notes = match !self.tag_all {
            true => notes
                .iter()
                .filter(|note| {
                    note.borrow()
                        .tags
                        .clone()
                        .is_some_and(|tags| tags.iter().any(|tag| search_tags.contains(tag)))
                })
                .map(|note| note.to_owned())
                .collect(),
            false => notes
                .iter()
                .filter(|note| {
                    note.borrow()
                        .tags
                        .clone()
                        .is_some_and(|tags| search_tags.iter().all(|search| tags.contains(search)))
                })
                .map(|note| note.to_owned())
                .collect(),
        };
        if notes.is_empty() {
            return Some(self.notes.clone());
        }
        Some(notes)
    }
}
