use std::{
    default,
    fmt::{self, Debug, Display},
    io::{self, Result},
    path::PathBuf,
};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    prelude::*,
    widgets::{
        block::{Position, Title},
        Block, Paragraph,
    },
};
use symbols::border;

use crate::{list::List, note::Note, traits::ThisFrame, tui::Tui};

#[derive(Debug, Clone, Default)]
pub enum InputMode {
    #[default]
    Normal,
    Insert,
}

#[derive(Debug, Default, Clone)]
pub struct Splash;

#[derive(Debug, Clone)]
pub enum CurrentFrame<T>
where
    T: ThisFrame + Display + Clone,
{
    Note(T),
    Splash(T),
    List(T),
}
impl<T> fmt::Display for CurrentFrame<T>
where
    T: ThisFrame + Debug + Display + Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

// impl<T> ThisFrame for CurrentFrame<T>
// where
//     T: ThisFrame + Display,
// {
//     type FrameType = T;
//     fn get_instructions(&self) -> Title {
//         match self {
//             CurrentFrame::Note(note) => note.get_instructions(),
//             CurrentFrame::Splash(splash) => splash.get_instructions(),
//             CurrentFrame::List(notes)
//         }
//     }

//     fn new(&self) -> Self {
//         match self {
//             CurrentFrame::Note(note) => CurrentFrame::Note(note.new()),
//             CurrentFrame::Splash(splash) => CurrentFrame::Splash(splash.new())
//         }
//     }
//     fn handle_key_event(&mut self,app: &mut App<impl ThisFrame + Display>, key_event: KeyEvent) {
//         match self {
//             CurrentFrame::Note(note) => note.handle_key_event(app, key_event),
//             CurrentFrame::Splash(splash) => splash.handle_key_event(app, key_event)

//         }

//     }

//     fn get_type(self) -> String {
//         match self {
//             CurrentFrame::Note(_) => "Note".to_owned(),
//             CurrentFrame::Splash(_) => "Splash".to_owned()
//         }
//     }
// }

#[derive(Debug)]
pub struct App<T>
where
    T: ThisFrame + Display + Clone,
{
    pub current_frame: CurrentFrame<T>,
    pub input_mode: bool,
    pub cursor_row: usize,
    pub cursor_column: usize,
    pub exit: bool,
}

impl<T> App<T>
where
    T: ThisFrame + Display + Clone,
{
    pub fn run(&mut self, terminal: &mut Tui) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame)).unwrap();
            self.handle_events().unwrap();
        }
        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
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
            CurrentFrame::Note(note) => ThisFrame::handle_key_event(&mut note, self, key_event),
            CurrentFrame::Splash(splash) => ThisFrame::handle_key_event(&mut splash,self, key_event),
            CurrentFrame::List(list) => ThisFrame::handle_key_event(&mut list, self, key_event),
        };
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl<T> Widget for &App<T>
where
    T: ThisFrame + Display + Clone,
{
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let title = Title::from(" Noter App".bold());
        // let instructions = Title::from(Line::from(vec![
        //     " Create New ".into(),
        //     "<Left>".blue().bold(),
        //     " Edit Existing ".into(),
        //     "<Right>".blue().bold(),
        //     " Quit ".into(),
        //     "<q>".blue().bold(),
        // ]));
        let instructions = self.get_instructions();

        let block = Block::bordered()
            .title(title.alignment(Alignment::Center))
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .border_set(border::THICK);

        let text = Text::from(vec![Line::from(vec![
            "Value ".into(),
            " Current Text ".to_string().red(),
        ])]);

        Paragraph::new(text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}
