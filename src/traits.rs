use std::fmt::Display;

use crossterm::event::KeyEvent;
use ratatui::{widgets::block::Title, Frame};

use crate::app::App;



pub trait ThisFrame {
    fn get_instructions(&self) -> Title;
    fn handle_key_event(&mut self,app: &mut App, key_event: KeyEvent);

    fn new() -> Self;
    fn get_type(self) -> String ;
}
