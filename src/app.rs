use core::borrow;
use std::borrow::Borrow;
use std::{
    fmt::{self, Display},
    io::{self, Result},
};

use crossterm::event::{self, Event, KeyEvent, KeyEventKind};
use ratatui::{prelude::*, widgets::block::Title};

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
        let [list_area, note_area] = layout.areas(frame.area());
        let note = self.note.borrow_mut().clone();

        let index = self.note_list.index;
        frame.render_stateful_widget(
            &self.note_list.clone(),
            list_area,
            &mut (index, self.note_list.notes.clone()),
        );
        if self.note_list.is_active {
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
        } else {
            frame.render_widget(&self.note.borrow_mut().clone(), note_area);

            match self.note.borrow_mut().clone().mode {
                InputMode::EditTitle => {
                    frame.set_cursor_position(layout::Position::new(
                        self.cursor_column as u16 + note_area.x + (note_area.width as f64 / 2.).ceil() as u16 - (note.title.len() as f64/2.).ceil() as u16,
                        self.cursor_row as u16,
                    ));
                }
                _ => frame.set_cursor_position(layout::Position::new(
                    self.cursor_column as u16 + note_area.x + 1,
                    self.cursor_row as u16 + 1,
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
        let note_instruction = note_ref.get_instructions();
        let note_list_ref = &self.note_list;
        let note_list_instruction = note_list_ref.get_instructions();

        let _instructions = match self.current_frame {
            CurrentFrame::Note => note_instruction,
            CurrentFrame::List => note_list_instruction,
        };
    }
}
