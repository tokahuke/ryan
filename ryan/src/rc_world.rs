use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

#[derive(Debug, Default, Clone)]
struct RcWorld {
    strings: Rc<RefCell<HashSet<Rc<str>>>>,
}

impl RcWorld {
    fn str_to_rc(&self, s: &str) -> Rc<str> {
        let mut strings = self.strings.borrow_mut();

        if let Some(rc) = strings.get(s) {
            Rc::clone(rc)
        } else {
            let new: Rc<str> = Rc::from(s.to_string());
            strings.insert(new.clone());
            new
        }
    }

    fn string_to_rc(&self, s: String) -> Rc<str> {
        let mut strings = self.strings.borrow_mut();

        if let Some(rc) = strings.get(&*s) {
            Rc::clone(rc)
        } else {
            let new: Rc<str> = Rc::from(s);
            strings.insert(new.clone());
            new
        }
    }
}

thread_local! {
    static RC_WORLD: RcWorld = RcWorld::default();
}

pub fn str_to_rc(s: &str) -> Rc<str> {
    RC_WORLD.with(|world| world.str_to_rc(s))
}

pub fn string_to_rc(s: String) -> Rc<str> {
    RC_WORLD.with(|world| world.string_to_rc(s))
}
