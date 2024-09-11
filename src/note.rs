use std::path::PathBuf;
use std::str::FromStr;
use std::sync::mpsc::Receiver;
use std::{fmt::Display, fs, path::Path};

use crate::file_reader::parse_file;
use crate::{
    app::{App, CurrentFrame, InputMode},
    file_reader::write_file,
    traits::ThisFrame,
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Styled, Stylize},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{
        block::{Position, Title},
        Block, Paragraph, Widget, Wrap,
    },
};
use regex::Regex;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[repr(transparent)]
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
                " Search Tags ".into(),
                "<T>".blue().bold(),
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
    fn get_type(&self) -> String {
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
                    if app.note_list.local_list {
                        write_file(Some(app.note_list.local_path.clone()), &mut note.clone());
                    } else {
                        write_file(None, &mut note.clone());
                    }
                    note.edited = false;
                }
            }
            (KeyCode::Char('t'), InputMode::Normal) => {
                note.mode = InputMode::EditTitle;
                note.old_title = Some(self.title.clone());
                app.cursor_column = 0;
            }
            (KeyCode::Char('T'), InputMode::Normal) => {
                app.note_list.is_active = true;
                app.current_frame = CurrentFrame::List;
                app.note_list.is_search = true;
                app.note_list.search = self.tags_to_string();
                app.note_list.index = 0;
            }
            (KeyCode::Backspace, InputMode::EditTitle) => {
                if app.cursor_column == 0 {
                } else {
                    note.edited = true;
                    let (first, second) = note.title.as_mut_str().split_at(app.cursor_column);
                    app.cursor_column = app.cursor_column.saturating_sub(1);
                    note.title = first.split_at(app.cursor_column).0.to_string() + second;
                }
            }
            (KeyCode::Char(c), InputMode::EditTitle) => {
                note.edited = true;
                let (first, second) = note.title.as_mut_str().split_at(app.cursor_column);
                note.title = first.to_string() + &c.to_string() + second;
                app.cursor_column = app.cursor_column.saturating_add(1);
            }
            (KeyCode::Enter, InputMode::Insert) => {
                note.edited = true;
                let mut start_lines = note
                    .text
                    .lines()
                    .map(Line::raw)
                    .collect::<Vec<ratatui::prelude::Line>>();
                match start_lines.len() - app.cursor_row {
                    1.. => {
                        let mut end_lines = start_lines.split_off(app.cursor_row);
                        end_lines.reverse();
                        let current_line = end_lines.pop().unwrap().to_string();
                        end_lines.reverse();
                        // let current_line = end_lines.first().unwrap().to_string();
                        if app.cursor_column == 0 {
                            start_lines.push(Line::raw(""));
                            start_lines.push(Line::raw(current_line));
                            end_lines
                                .iter()
                                .for_each(|line| start_lines.push(line.to_owned()));
                        } else if app.cursor_column < current_line.len() + 1 {
                            let (first, second) = current_line.split_at(app.cursor_column);
                            start_lines.push(Line::raw(first));
                            start_lines.push(Line::raw(second));
                            end_lines
                                .iter()
                                .for_each(|line| start_lines.push(line.to_owned()));
                        } else {
                            start_lines.push(Line::raw(""));
                            end_lines
                                .iter()
                                .for_each(|line| start_lines.push(line.to_owned()));
                        }

                        note.text = Text::from(start_lines).to_string();
                    }
                    0 => {
                        if start_lines
                            .get(app.cursor_row)
                            .is_some_and(|line| line.to_string().len() < app.cursor_column - 1)
                        {
                            let line = &start_lines.last().unwrap();
                            let line = &line.to_string();
                            let (first, second) = line.split_at(app.cursor_column);
                            note.text = Text::from(vec![first.into(), second.into()]).to_string();
                        } else {
                            let line = Line::raw(" ".to_string());
                            start_lines.push(line);
                            note.text = Text::from(start_lines).to_string();
                        }
                    }
                }
                app.cursor_column = 0;
                app.cursor_row = app.cursor_row.saturating_add(1);
            }
            (KeyCode::Enter, InputMode::EditTitle) => {
                note.mode = InputMode::Normal;
                app.cursor_column = 0;
                let cur_path = match app.note_list.local_list {
                    true => PathBuf::from_str(
                        (app.note_list.local_path.to_string_lossy() + "/notes/")
                            .to_string()
                            .as_str(),
                    )
                    .unwrap(),
                    false => app.note_list.path.clone(),
                };
                note.edited = false;
                note.is_active = false;
                let path = if app.note_list.local_list {
                    Some(cur_path)
                } else {
                    None
                };
                if self.old_title.is_none() || self.old_title.as_mut().unwrap().is_empty() {
                    write_file(path, &mut note);
                    app.note_list
                        .notes
                        .push(parse_file(note.text.clone(), Path::new(&note.title)));
                } else {
                    let mut old_path = path.clone().unwrap();
                    old_path.push(note.old_title.as_ref().unwrap().to_owned() + ".md");
                    fs::remove_file(old_path).unwrap();
                    write_file(path,&mut note);
                }
                note.is_active = true;
            }
            (KeyCode::Left, InputMode::EditTitle) => {
                if app.cursor_column > 0 {
                    app.cursor_column = app.cursor_column.saturating_sub(1);
                }
            }
            (KeyCode::Right, InputMode::EditTitle) => {
                if app.cursor_column < self.title.len() {
                    app.cursor_column = app.cursor_column.saturating_add(1);
                }
            }
            (KeyCode::Up, InputMode::Normal | InputMode::Insert) => {
                if app.cursor_row > 0 {
                    app.cursor_row = app.cursor_row.saturating_sub(1);
                    app.cursor_column = 0;
                }
            }
            (KeyCode::Down, InputMode::Normal | InputMode::Insert) => {
                let lines: &[Line] = &Text::raw(&note.text).lines;
                if app.cursor_row < lines.len() - 1 {
                    let len = lines[app.cursor_row.saturating_add(1)].to_string().len();
                    if app.cursor_column >= len {
                        app.cursor_column = len;
                        app.cursor_row = app.cursor_row.saturating_add(1);
                    } else {
                        app.cursor_row = app.cursor_row.saturating_add(1);
                    }
                }
            }
            (KeyCode::Left, InputMode::Normal | InputMode::Insert) => {
                if app.cursor_column > 0 {
                    app.cursor_column = app.cursor_column.saturating_sub(1);
                }
            }
            (KeyCode::End, InputMode::Normal | InputMode::Insert) => {
                let lines: &[Line] = &Text::raw(&note.text).lines;
                app.cursor_column = lines[app.cursor_row].to_string().len();
            }
            (KeyCode::Home, InputMode::Normal | InputMode::Insert) => {
                app.cursor_column = 0;
            }
            (KeyCode::Char('w'), InputMode::Normal) => {
                let line = &Text::raw(&note.text).lines[app.cursor_row].to_string();
                if app.cursor_column < line.len() {
                    let split = line.split_at(app.cursor_column.saturating_add(1));
                    app.cursor_column = match split.1.find(" ") {
                        Some(idx) => app.cursor_column + idx + 1,
                        None => line.len(),
                    }
                }
            }
            (KeyCode::Char('b'), InputMode::Normal) => {
                let line = &Text::raw(&note.text).lines[app.cursor_row].to_string();
                if app.cursor_column > 0 {
                    let split = line.split_at(app.cursor_column.saturating_sub(1));
                    app.cursor_column = split.0.rfind(" ").unwrap_or(0);
                }
            }
            (KeyCode::Right, InputMode::Normal | InputMode::Insert) => {
                let lines = Text::raw(self.text.clone()).lines;
                if lines.len() <= app.cursor_row {
                } else if app.cursor_column
                    < lines.get(app.cursor_row).unwrap().to_string().len() - 1
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
                let lines = Text::raw(&note.text).lines;
                if app.cursor_column == 0 {
                    if app.cursor_row == 0 || app.cursor_row == lines.len() {
                        if app.cursor_row == lines.len() {
                            app.cursor_row = app.cursor_row.saturating_sub(1);
                            app.cursor_column = lines.last().unwrap().to_string().len();
                        }
                    } else {
                        let start = lines.split_at(app.cursor_row);
                        let last = start.1.split_first().unwrap();
                        let prev = start.0.split_last().unwrap();
                        let new_line = prev.0.to_string() + &last.0.to_string();
                        let mut text: Text = Text::default();
                        app.cursor_column = prev.0.to_string().len();

                        for line in prev.1.iter() {
                            text.push_line(line.to_owned());
                        }
                        text.push_line(new_line);
                        for line in last.1.iter() {
                            text.push_line(line.to_owned());
                        }
                        note.text = text.to_string();
                        note.edited = true;
                        app.cursor_row = app.cursor_row.saturating_sub(1);
                    }
                } else {
                    note.edited = true;
                    let mut lines = Text::raw(&note.text).lines;
                    let mut line = lines.get(app.cursor_row).unwrap().to_string();
                    let (first, second) = line.split_at(app.cursor_column);
                    app.cursor_column = app.cursor_column.saturating_sub(1);
                    line = first.split_at(app.cursor_column).0.to_string() + second;
                    lines[app.cursor_row] = Line::raw(line);
                    let text: Text = lines.into();
                    note.text = text.to_string();
                }
            }
            (KeyCode::Char(c), InputMode::Insert) => {
                note.edited = true;
                let mut lines = Text::raw(&note.text).lines;

                let mut line = if app.cursor_row == lines.len().saturating_add(1) {
                    "".to_string()
                } else {
                    lines
                        .get(app.cursor_row)
                        .unwrap_or(&Line::raw(""))
                        .to_string()
                };
                if line.is_empty() {
                    let line = c.to_string();
                    if lines.len() <= app.cursor_row {
                        lines.push(Line::raw(line));
                    } else {
                        lines[app.cursor_row] = Line::raw(line);
                    };
                } else {
                    let (first, second) = line.split_at(app.cursor_column);
                    line = first.to_string().to_owned() + &c.to_string() + second;
                    lines[app.cursor_row] = Line::raw(line);
                };
                let text: Text = lines.into();
                note.text = text.to_string();
                app.cursor_column = app.cursor_column.saturating_add(1);
            }
            _ => {}
        };
    }
}

impl Note {
    pub fn create_note() -> Self {
        Self::default()
    }
    pub fn tags_to_string(&self) -> Option<String> {
        let tags = self.tags.clone()?;
        let tag_str: Vec<String> = tags.iter().map(|tag| tag.0[1..].to_owned()).collect();
        Some(tag_str.join(" "))
    }
}

impl Widget for &Note {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let title_text = if self.is_active {
            self.title.clone().green().bold()
        } else {
            self.title.clone().green()
        };

        let title: Title = Title::from(title_text.bold());
        let mut my_border = border::ROUNDED;
        my_border.vertical_left = border::DOUBLE.vertical_left;
        my_border.horizontal_bottom = border::DOUBLE.horizontal_bottom;
        let mut block = Block::bordered()
            .title(title.alignment(Alignment::Center))
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

        let binding = self.text.clone();
        let lines: Vec<Line> = binding.split("\n").map(Line::raw).collect::<Vec<Line>>();
        let mut text_vec: Vec<Line> = vec![];
        for line in lines.clone() {
            let mut new_line: Vec<Span> = vec![];
            let vec_line = line
                .to_owned()
                .to_string()
                .split_inclusive(" ")
                .to_owned()
                .map(|token| token.to_string())
                .collect::<Vec<String>>();

            for token in vec_line {
                if token.starts_with("#") {
                    let spn = token.clone().magenta();
                    new_line.push(spn);
                } else {
                    let spn = token.clone().green();
                    new_line.push(spn);
                }
            }
            text_vec.push(Line::from(new_line));
        }

        let text = Text::from(text_vec);

        Paragraph::new(text)
            .left_aligned()
            .block(block)
            .render(area, buf);
    }
}
