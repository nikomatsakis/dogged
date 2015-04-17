use std::cell::RefCell;

pub struct DCell<T> {
    data: RefCell<Option<T>>
}

impl<T> DCell<T> {
    pub fn new() -> DCell<T> {
        DCell { data: RefCell::new(None) }
    }

    pub fn is_empty(&self) -> bool {
        self.data.borrow().is_none()
    }

    pub fn take(&self) -> T {
        let mut data = self.data.borrow_mut();
        assert!(data.is_some());
        data.take().unwrap()
    }

    pub fn put(&self, value: T) {
        let mut data = self.data.borrow_mut();
        assert!(data.is_none());
        *data = Some(value);
    }

    pub fn read<F,R>(&self, f: F) -> R
        where F: FnOnce(&T) -> R
    {
        let data = self.data.borrow();
        f(data.as_ref().unwrap())
    }
}
