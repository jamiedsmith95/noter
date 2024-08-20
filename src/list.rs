use std::fmt::Display;

use crate::note::Note;


#[derive(Debug,Default,Clone)]
pub struct List {
    notes: Vec<Note>,
}

impl Display for List {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.notes)
    }
}


impl List {
}
