use std::{
    fmt::{self, Display}, io::{self, Result}
};

use crossterm::event::{self, Event, KeyEvent, KeyEventKind};
use ratatui::{
    prelude::*,
    widgets::{
        block::{Position, Title},
        Block, BorderType, Borders, Paragraph,
    },
};
use symbols::border;
use text::{ToLine, ToSpan};

use crate::{
    list::MyList,
    note::{self, Note},
    traits::ThisFrame,
    tui::Tui,
    utils::RcRc,
};

#[derive(Debug, Clone, Default)]
pub enum InputMode {
    #[default]
    Normal,
    Insert,
    EditTitle,
}

#[derive(Debug, Clone)]
pub enum CurrentFrame {
    Note,
    List,
}
impl fmt::Display for CurrentFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct App {
    pub current_frame: CurrentFrame,
    pub note: RcRc<Note>,
    pub note_list: MyList,
    pub input_mode: bool,
    pub cursor_row: usize,
    pub cursor_column: usize,
    pub exit: bool,
}

impl App {
    pub fn run(&mut self, terminal: &mut Tui) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame)).unwrap();
            self.handle_events().unwrap();
        }
        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        let layout = Layout::horizontal(Constraint::from_percentages([15, 85]));
        let vertical = Layout::vertical(Constraint::from_percentages([98, 2]));
        let [main_area, instructions] = vertical.areas(frame.area());

        let [list_area, note_area] = layout.areas(main_area);
        let note = self.note.borrow_mut().clone();
        frame.render_widget(self, instructions);

        let index = self.note_list.index;
        frame.render_stateful_widget(
            &self.note_list.clone(),
            list_area,
            &mut (index, self.note_list.notes.clone()),
        );
        if self.note_list.is_active {
            if self.note_list.is_search {
                frame.set_cursor_position(layout::Position::new(
                    self.cursor_column as u16 + list_area.x + 8, // 8 for search:
                    instructions.y, // use instructions as has same vertical as search
                ));
                frame.render_widget(
                    &self
                        .note_list
                        .filter_list(self.note_list.search.clone())
                        .unwrap()
                        .get(self.note_list.index)
                        .unwrap()
                        .borrow_mut()
                        .clone(),
                    note_area,
                );
            } else {
                frame.render_widget(
                    &self
                        .note_list
                        .notes
                        .get(self.note_list.index)
                        .unwrap()
                        .borrow_mut()
                        .clone(),
                    note_area,
                )
            }
        } else {
            frame.render_widget(&self.note.borrow_mut().clone(), note_area);

            match self.note.borrow_mut().clone().mode {
                InputMode::EditTitle => {
                    frame.set_cursor_position(layout::Position::new(
                        self.cursor_column as u16
                            + note_area.x
                            + (note_area.width as f64 / 2.).floor() as u16
                            - (note.title.len() as f64 / 2.).ceil() as u16,
                        0,
                    ));
                }
                _ => frame.set_cursor_position(layout::Position::new(
                    self.cursor_column as u16 + note_area.x + 1,
                    self.cursor_row.saturating_add(1) as u16,
                )),
            }
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read().unwrap() {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match self.current_frame {
            CurrentFrame::Note => {
                let mut note = self.note.borrow_mut().clone();
                note.handle_key_event(self, key_event);
            }
            CurrentFrame::List => {
                let mut list = self.note_list.clone();
                list.handle_key_event(self, key_event);
            }
        };
    }

    pub fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let _title = Title::from(" Noter App".bold().green());

        let note_ref = &self.note.borrow_mut().clone();
        let note_list_ref = &self.note_list;

        let instructions = match self.current_frame {
            CurrentFrame::Note => note_ref.get_instructions(),
            CurrentFrame::List => note_list_ref.get_instructions(),
        };

        let pos = Title::from(vec![self.cursor_column.to_span()," ".to_span(),self.cursor_row.to_span()]);
        Paragraph::new(instructions.content)
            .alignment(Alignment::Center)
            .render(area, buf);
        if self.note_list.is_search {
        Paragraph::new("search: ".to_string() + self.note_list.search.as_ref().unwrap_or(&"".to_string()))
            .alignment(Alignment::Left)
            .render(area, buf);
        }
        Paragraph::new(pos.content).alignment(Alignment::Right).render(area,buf);
    }
}
