use std::cell::RefCell;

pub struct DCell<T> {
    data: RefCell<Option<T>>
}

impl<T> DCell<T> {
    pub fn take(&self) -> T {
        let data = self.data.borrow_mut();
        assert!(data.is_some());
        data.take().unwrap()
    }

    pub fn replace(&self, value: T) -> T {
        let data = self.data.borrow_mut();
        assert!(data.is_none());
        *data = Some(value);
    }

    pub fn read<F>(&self, f: F) -> F::Output
        where F: FnOnce(&T)
    {
        let data = self.data.borrow();
        f(data.as_ref().unwrap())
    }
}
