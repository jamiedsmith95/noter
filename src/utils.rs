use std::{cell::RefCell, rc::Rc};


pub type RcRc<T> = Rc<RefCell<T>>;
pub fn rc_rc<T>(t: T) -> RcRc<T> {
    Rc::new(RefCell::new(t))
}
