use std::cmp::{PartialOrd, Ordering};
use std::mem;
use std::sync::Arc;

const BRANCH_FACTOR: usize = 32;
const BITS_PER_LEVEL: usize = 5;

macro_rules! no_children {
    () => {
        [None, None, None, None,
         None, None, None, None,
         None, None, None, None,
         None, None, None, None,
         None, None, None, None,
         None, None, None, None,
         None, None, None, None,
         None, None, None, None]
    }
}

macro_rules! clone_arr {
    ($source:expr) => {
        {
            let s = $source;
            [
                s[0x00].clone(), s[0x01].clone(), s[0x02].clone(), s[0x03].clone(),
                s[0x04].clone(), s[0x05].clone(), s[0x06].clone(), s[0x07].clone(),
                s[0x08].clone(), s[0x09].clone(), s[0x0A].clone(), s[0x0B].clone(),
                s[0x0C].clone(), s[0x0D].clone(), s[0x0E].clone(), s[0x0F].clone(),
                s[0x10].clone(), s[0x11].clone(), s[0x12].clone(), s[0x13].clone(),
                s[0x14].clone(), s[0x15].clone(), s[0x16].clone(), s[0x17].clone(),
                s[0x18].clone(), s[0x19].clone(), s[0x1A].clone(), s[0x1B].clone(),
                s[0x1C].clone(), s[0x1D].clone(), s[0x1E].clone(), s[0x1F].clone(),
            ]
        }
    }
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct PersistentVec<T> {
    len: Index,
    shift: Shift, // depth * BITS_PER_LEVEL
    root: Option<Arc<Node<T>>>,
    tail: Vec<T>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Shift(usize);

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Index(usize);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Node<T> {
    Branch { children: [Option<Arc<Node<T>>>; BRANCH_FACTOR] },
    Leaf { elements: Vec<T> },
}

impl<T: Clone> PersistentVec<T> {
    pub fn new() -> Self {
        PersistentVec {
            len: Index(0),
            shift: Shift(0),
            root: None,
            tail: Vec::with_capacity(BRANCH_FACTOR),
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        match self.root {
            None => {
                debug_assert!(self.len.0 == 0);
                None
            }
            Some(ref node) => {
                if index >= self.len.0 {
                    None
                } else {
                    Some(node.get(self.shift, self.len))
                }
            }
        }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&T> {
        match self.root {
            None => {
                debug_assert!(self.len.0 == 0);
                None
            }
            Some(ref mut node) => {
                if index >= self.len.0 {
                    None
                } else {
                    Some(Arc::make_mut(node).get_mut(self.shift, self.len))
                }
            }
        }
    }

    pub fn len(&self) -> usize {
        self.len.0
    }

    pub fn push(&mut self, element: T) {
        self.tail.push(element);
        self.len.0 += 1;

        if self.tail.len() == BRANCH_FACTOR {
            let tail = mem::replace(&mut self.tail, Vec::with_capacity(BRANCH_FACTOR));
            self.push_tail(tail);
        }
    }

    #[cold]
    fn push_tail(&mut self, tail: Vec<T>) {
        // We just filled up the tail, therefore we should have an
        // even multiple of BRANCH_FACTOR elements.
        debug_assert!(self.len.0 % BRANCH_FACTOR == 0);

        if let Some(root) = self.root.as_mut() {
            // Find out the total capacity in the "leaf" tree.
            let capacity = BRANCH_FACTOR << self.shift.0;

            // Still have room.
            if self.len < capacity {
                Arc::make_mut(root).push_tail(self.shift, self.len, tail);
                return;
            }

            // Going to need to add another level.
            let mut children = no_children!();
            children[0] = Some(root.clone());
            children[1] = Some(Node::branch_ladder(self.shift, tail));
            *root = Arc::new(Node::Branch { children: children });
            return;
        }

        debug_assert!(self.len == BRANCH_FACTOR);
        debug_assert!(self.shift == 0);
        self.root = Some(Arc::new(Node::Leaf { elements: tail }));
    }
}

impl Index {
    fn child(self, shift: Shift) -> usize {
        (self.0 >> shift.0) & (BRANCH_FACTOR - 1)
    }
    fn leaf_child(self) -> usize {
        self.0 & (BRANCH_FACTOR - 1)
    }
}

impl Shift {
    fn dec(self) -> Shift {
        Shift(self.0 - BITS_PER_LEVEL)
    }
}

impl<T: Clone> Node<T> {
    pub fn branch_ladder(shift: Shift, tail: Vec<T>) -> Arc<Node<T>> {
        if shift.0 > 0 {
            let mut children = no_children!();
            children[0] = Some(Node::branch_ladder(shift.dec(), tail));
            Arc::new(Node::Branch { children: children })
        } else {
            Arc::new(Node::Leaf { elements: tail })
        }
    }

    pub fn push_tail(&mut self, shift: Shift, index: Index, tail: Vec<T>) {
        // Example 1.
        //
        // The vector has 96 elements, 32 of which are in the tail that we
        // are now pushing.
        //
        // A (shift = 5)
        // |
        // +- B (Some(Leaf); elements 0..32)
        // +- C (Some(Leaf); elements 32..64)
        // +- D (None)
        // +- ... (None; repeats 29 times)
        //
        // We want to replace D with a new `Some(Leaf)`. Our inital
        // shift will be 5 and our index will be 96 (32*3). Since the
        // shift is equal to BITS_PER_LEVEL, we know that the
        // immediate children are leaves, so our iteration is done.
        // We compute the index, which will be 3 -- that is the index
        // where the *next* set of leaves will go. To find the index
        // for the current set, we subtract one. This gets us 2, and
        // we store. Note that we know that the index is never 0,
        // because that case corresponds to having to add a new level
        // to the tree, and we handle that elsewhere.
        let mut p = self;
        let mut shift = shift;
        loop {
            debug_assert!(shift.0 >= BITS_PER_LEVEL);
            let mut q = p; // FIXME
            match *q {
                Node::Leaf { .. } => {
                    unreachable!("should not encounter a leaf w/ shift {:?}", shift)
                }
                Node::Branch { ref mut children } => {
                    let child = index.child(shift);
                    if shift.0 == BITS_PER_LEVEL {
                        // We are on the final level; our children are leaves.
                        debug_assert!(child > 0); // this case is handled by `branch_ladder` above
                        let dest = child - 1;
                        debug_assert!(children[dest].is_none());
                        children[dest] = Some(Arc::new(Node::Leaf { elements: tail }));
                        return;
                    }

                    // Load up the child and descend to that
                    // level. There should be a child there or
                    // something went wrong.
                    p = Arc::make_mut(children[child].as_mut().unwrap());
                    shift = shift.dec();
                }
            }
        }
    }

    pub fn get(&self, shift: Shift, index: Index) -> &T {
        let mut p = self;
        let mut shift = shift;
        loop {
            match *p {
                Node::Branch { ref children } => {
                    debug_assert!(shift.0 > 0);
                    let child = index.child(shift);
                    p = match children[child] {
                        Some(ref c) => &*c,
                        None => panic!("missing child {} at shift {}", child, shift.0),
                    };
                    shift = shift.dec();
                }

                Node::Leaf { ref elements } => {
                    debug_assert!(shift.0 == 0);
                    debug_assert!(elements.len() == BRANCH_FACTOR);
                    let child = index.leaf_child();
                    return &elements[child];
                }
            }
        }
    }

    pub fn get_mut(&mut self, shift: Shift, index: Index) -> &T {
        let mut p = self;
        let mut shift = shift;
        loop {
            let mut q = p; // FIXME
            match *q {
                Node::Branch { ref mut children } => {
                    debug_assert!(shift.0 > 0);
                    let child = index.child(shift);
                    p = match children[child] {
                        Some(ref mut c) => Arc::make_mut(c),
                        None => panic!("missing child {} at shift {}", child, shift.0),
                    };
                    shift = shift.dec();
                }

                Node::Leaf { ref mut elements } => {
                    debug_assert!(shift.0 == 0);
                    debug_assert!(elements.len() == BRANCH_FACTOR);
                    let child = index.leaf_child();
                    return &elements[child];
                }
            }
        }
    }
}

impl PartialEq<usize> for Index {
    fn eq(&self, other: &usize) -> bool {
        self.0.eq(other)
    }
}

impl PartialOrd<usize> for Index {
    fn partial_cmp(&self, other: &usize) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}

impl PartialEq<usize> for Shift {
    fn eq(&self, other: &usize) -> bool {
        self.0.eq(other)
    }
}

impl PartialOrd<usize> for Shift {
    fn partial_cmp(&self, other: &usize) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}

impl<T: Clone> Clone for Node<T> {
    fn clone(&self) -> Self {
        match *self {
            Node::Branch { ref children } => {
                Node::Branch { children: clone_arr!(children) }
            }
            Node::Leaf { ref elements } => {
                Node::Leaf { elements: elements.clone() }
            }
        }
    }
}

#[cfg(test)]
mod test;
