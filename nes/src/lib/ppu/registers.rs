use std::cell::RefCell;
use std::rc::Rc;

pub(crate) struct Registers {
    register2000: Rc<RefCell<u8>>,
    register2001: Rc<RefCell<u8>>,
    register2002: Rc<RefCell<u8>>,
    register2003: Rc<RefCell<u8>>,
    register2004: Rc<RefCell<u8>>,
    register2005: Rc<RefCell<u8>>,
    register2006: Rc<RefCell<u8>>,
    register2007: Rc<RefCell<u8>>,
    register4014: Rc<RefCell<u8>>,
}