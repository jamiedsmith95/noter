use app::{App, CurrentFrame};
use list::MyList;
use note::Note;
use utils::rc_rc;
use std::
    io::{self, Result}
;
use traits::ThisFrame;
mod app;
mod file_reader;
mod list;
mod note;
mod traits;
mod tui;
mod utils;


fn main() -> io::Result<()> {
    let mut terminal = tui::init().unwrap();
    let mut app = App {
        current_frame: CurrentFrame::List,
        note: rc_rc(Note::new()),
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
