use std::{
    borrow::Borrow, cell::RefCell, fmt::{self, Display}, io::{self, Result}, rc::Rc
};

use crossterm::{cursor::SetCursorStyle, event::{self, Event, KeyCode, KeyEvent, KeyEventKind}};
use ratatui::{prelude::*, widgets::block::Title};

use crate::{list::MyList, note::Note, traits::ThisFrame, tui::Tui};

pub type RcRc<T> = Rc<RefCell<T>>;
pub fn rc_rc<T>(t: T) -> RcRc<T> {
    Rc::new(RefCell::new(t))
}

#[derive(Debug, Clone, Default)]
pub enum InputMode {
    #[default]
    Normal,
    Insert,
    EditTitle,
}

#[derive(Debug, Default, Clone)]
pub struct Splash;

impl ThisFrame for Splash {
    fn get_instructions(&self) -> Title {
        Title::from(Line::from(vec![
            " List Notes ".into(),
            "<l>".blue().bold(),
            " New Note ".into(),
            "<n>".blue().bold(),
            " Quit ".into(),
            "<q>".red().bold(),
        ]))
    }
    fn new() -> Self {
        Splash {}
    }
    fn handle_key_event(&mut self, app: &mut App, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => app.exit(),
            KeyCode::Char('l') => {
                app.current_frame = CurrentFrame::List;
                app.note_list.is_active = true;
            }
            KeyCode::Char('n') => {
                app.current_frame = CurrentFrame::Note;
                app.note.is_active = true;
                app.note = Note::create_note();
            }
            _ => {}
        };
    }
    fn get_type(self) -> String {
        "splash".to_string()
    }
}

#[derive(Debug, Clone)]
pub enum CurrentFrame {
    Note,
    Splash,
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
    pub note: Note,
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
        let layout = Layout::horizontal(Constraint::from_percentages([30, 70]));
        let [list_area, note_area] = layout.areas(frame.area());


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
                    .to_owned(),
                note_area,
            )
        } else {
            frame.render_widget(&self.note, note_area);
            frame.set_cursor_position(layout::Position::new(
                self.cursor_column as u16 + note_area.x + 1,
                self.cursor_row as u16 + 1,
            ))
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
            CurrentFrame::Note => &self.note.clone().handle_key_event(self, key_event),
            CurrentFrame::Splash => &ThisFrame::handle_key_event(&mut Splash {}, self, key_event),
            CurrentFrame::List => &self.note_list.clone().handle_key_event(self, key_event),
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

        let note_ref = &self.note;
        let note_instruction = note_ref.get_instructions();
        let note_list_ref = &self.note_list;
        let note_list_instruction = note_list_ref.get_instructions();

        let _instructions = match self.current_frame {
            CurrentFrame::Note => note_instruction,
            CurrentFrame::Splash => Splash::get_instructions(&Splash {}),
            CurrentFrame::List => note_list_instruction,
        };
    }
}
