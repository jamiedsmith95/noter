use std::fmt::Display;

use crossterm::event::KeyEvent;
use ratatui::{widgets::block::Title, Frame};

use crate::app::App;



pub trait ThisFrame {
    type FrameType: ThisFrame + Display;
    fn get_instructions(&self) -> Title;
    fn handle_key_event(&mut self,app: &mut App<impl ThisFrame + Display>, key_event: KeyEvent);

    fn new(&self) -> Self;
    fn get_type(self) -> String ;
}
