use app::{App, CurrentFrame};
use file_reader::{list_files, parse_file, read_file};
use note::Note;
use std::io::{self, Result};
mod tui;
mod app;
mod list;
mod note;
mod traits;
mod file_reader;

fn main() -> io::Result<()> {
    let mut terminal = tui::init().unwrap();
    let note = Note::default();
    let mut app = App {
        current_frame: CurrentFrame::Note(note) ,
        input_mode: false,
        cursor_row: 0,
        cursor_column: 0,
        exit: false,
    };
    let app_result = app.run(&mut terminal);
    tui::restore().unwrap();
    app_result
}
