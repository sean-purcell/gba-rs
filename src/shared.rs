use std::cell::{RefCell, Ref, RefMut};
use std::fmt;
use std::ops::Deref;
use std::rc::Rc;

/// Class to help with the weak references between singletons in the GBA
/// structure
#[derive(Clone)]
struct Shared<T> {
    v: Weak<RefCell<T>>
}

impl <T> Shared<T> {
    fn new(t: T)-> Shared<T> {
        Shared{v: Rc::new(RefCell::new(t))}
    }
}

impl <T> Shared<T> {
    fn borrow(&self) -> Ref<T> {
        self.v.borrow()
    }

    fn borrow_mut(&self) -> RefMut<T> {
        self.v.borrow_mut()
    }

    fn as_ptr(&self) -> *mut T {
        self.v.as_ptr()
    }
}


impl <T: fmt::Display> fmt::Display for Shared<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.deref())
    }
}

impl <T: fmt::Debug> fmt::Debug for Shared<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.deref())
    }
}

impl <'a,T> Deref for Shared<T>{
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        unsafe {self.as_ptr().as_ref().unwrap()}
    }

}
