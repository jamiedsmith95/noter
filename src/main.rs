use app::{App, CurrentFrame};
use crossterm::cursor::SetCursorStyle;
use file_reader::{list_files, parse_file, read_file};
use list::MyList;
use note::Note;
use std::{
    cell::RefCell,
    io::{self, Result},
    rc::Rc,
    sync::mpsc::Receiver,
};
use traits::ThisFrame;
mod app;
mod file_reader;
mod list;
mod note;
mod traits;
mod tui;

fn main() -> io::Result<()> {
    let mut terminal = tui::init().unwrap();
    let mut app = App {
        current_frame: CurrentFrame::Splash,
        note: Note::new(),
        input_mode: false,
        cursor_row: 0,
        cursor_column: 0,
        exit: false,
        note_list: MyList::new(),
    };
    let app_result = app.run(&mut terminal);
    tui::restore().unwrap();
    app_result
}
