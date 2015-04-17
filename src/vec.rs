use cell::DCell;

pub struct DVec<T> {
    state: Rc<DCell<DVecState<T>>>
}

enum DVecState<T> {
    Root(Vec<T>),
    Diff(DVec<T>, usize, T),
}

impl DVec<T> {
    /// Extracts the root vector as viewed from this node. At the end
    /// of this function, this node will be empty and all other nodes
    /// in the tree will point at this node in root. In other words,
    /// the state is *almost* consistent, except that this node is
    /// empty.
    fn extract_data(&self) -> Vec<T> {
        match self.state.take() {
            Root(data) => data,
            Diff(dvec, index, element) => dvec.update(index, element),
        }
    }

    fn update(&self, index: usize, element: T) -> Vec<T> {
        let data = self.extract_data();
        let old_element = data.swap(index, element);
        self.state.replace(Diff(self.clone(), index, old_element));
        data
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
        self.state.replace(Root(data));
        result
    }

    pub fn get(&self, index: usize) -> T
        where T: Clone
    {
        self.read(|data| data[index].clone())
    }

    pub fn with(&self, index: usize, element: T) -> DVec<T> {
        let data = self.update(index, element);
        DVec { state: Rc::new(RefCell::new(
    }
}
