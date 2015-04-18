use cell::DCell;
use std::clone::Clone;
use std::mem;
use std::rc::{self, Rc};
use self::DVecState::*;
use self::Action::*;

pub struct DVec<T> {
    state: Rc<DCell<DVecState<T>>>
}

enum DVecState<T> {
    Root(Vec<T>),
    Diff(DVec<T>, Action<T>),
}

enum Action<T> {
    Pop,
    Push(T),
    Set(usize, T),
}

impl<T> Clone for DVec<T> {
    fn clone(&self) -> DVec<T> {
        DVec { state: self.state.clone() }
    }
}

impl<T> DVec<T> {
    pub fn new() -> DVec<T> {
        DVec::with(vec![])
    }

    pub fn with(data: Vec<T>) -> DVec<T> {
        let placeholder = DVec::new_placeholder();
        placeholder.state.put(Root(data));
        placeholder
    }

    fn new_placeholder() -> DVec<T> {
        DVec { state: Rc::new(DCell::new()) }
    }

    /// Debugging fn that tells you how many levels of diff
    /// there between self and the root.
    fn depth(&self) -> usize {
        self.state.read(|state| {
            match *state {
                Root(..) => 0,
                Diff(ref base, _) => base.depth() + 1
            }
        })
    }

    /// Extracts the root vector as viewed from this node. At the end
    /// of this function, this node will be empty and all other nodes
    /// in the tree will point at this node in root. In other words,
    /// the state is *almost* consistent, except that this node is
    /// empty.
    fn extract_data(&self) -> Vec<T> {
        match self.state.take() {
            Root(data) => data,
            Diff(dvec, action) => dvec.make_diff(self, action),
        }
    }

    /// Converts self into a diff against `root`, where `root`
    /// is the same as self but that `index` is changed to `element`.
    /// Returns the data array (with `index` updated) but does not
    /// modify the state of `root`, which should be empty.
    fn make_diff(&self, root: &DVec<T>, action: Action<T>) -> Vec<T> {
        assert!(root.state.is_empty());
        let mut data = self.extract_data();
        let reverse = action.enact(&mut data);
        self.state.put(Diff(root.clone(), reverse));
        data
    }

    fn mutate(&mut self, action: Action<T>) {
        if rc::is_unique(&self.state) {
            // avoiding making a new diff if not needed
            let mut data = self.extract_data();
            let _ = action.enact(&mut data); // don't need the reverse
            self.state.put(Root(data));
        } else {
            let root = DVec::new_placeholder();
            let data = self.make_diff(&root, action);
            root.state.put(Root(data));
            *self = root;
        }
    }

    /// Converts this node into the root.  While the read is taking
    /// place, the vector is in an inconsistent state and recursive
    /// uses will panic. Similarly, if the read function panics, the
    /// vector remains in an inconsistent state.
    pub fn read<F,R>(&self, func: F) -> R
        where F: FnOnce(&[T]) -> R
    {
        let data = self.extract_data();
        let result = func(&data);
        self.state.put(Root(data));
        result
    }

    pub fn len(&self) -> usize {
        self.read(|data| data.len())
    }

    pub fn get(&self, index: usize) -> T
        where T: Clone
    {
        self.read(|data| data[index].clone())
    }

    pub fn push(&mut self, value: T) {
        self.mutate(Push(value));
    }

    pub fn set(&mut self, index: usize, element: T) {
        self.mutate(Set(index, element));
    }
}

impl<T> Action<T> {
    fn enact(self, data: &mut Vec<T>) -> Action<T> {
        match self {
            Pop => {
                let value = data.pop().unwrap();
                Push(value)
            }

            Push(v) => {
                data.push(v);
                Pop
            }

            Set(index, element) => {
                let old_element = mem::replace(&mut data[index], element);
                Set(index, old_element)
            }
        }
    }
}

#[cfg(test)]
mod test;

#[cfg(test)]
mod bench;
