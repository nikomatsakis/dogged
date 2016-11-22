#![cfg_attr(test, feature(test))]

#[cfg(test)]
use std::cmp;

#[cfg(test)]
extern crate test as test_crate;

#[cfg(test)]
extern crate rand;

use std::cmp::{PartialOrd, Ordering};
use std::fmt::Debug;
use std::mem;
use std::sync::Arc;

macro_rules! debug {
    ($($t:tt)*) => {
        // println!($($t)*);
    }
}

const VALIDATE: bool = false;

#[cfg(not(small_branch))]
const BRANCH_FACTOR: usize = 32;

#[cfg(not(small_branch))]
const BITS_PER_LEVEL: usize = 5;

#[cfg(not(small_branch))]
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

#[cfg(not(small_branch))]
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

#[cfg(small_branch)]
const BRANCH_FACTOR: usize = 4;

#[cfg(small_branch)]
const BITS_PER_LEVEL: usize = 2;

#[cfg(small_branch)]
macro_rules! no_children {
    () => {
        [None, None, None, None]
    }
}

#[cfg(small_branch)]
macro_rules! clone_arr {
    ($source:expr) => {
        {
            let s = $source;
            [
                s[0x00].clone(), s[0x01].clone(), s[0x02].clone(), s[0x03].clone(),
            ]
        }
    }
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct DVec<T> {
    root_len: Index, // number of things reachable from root (excluding tail)
    shift: Shift, // depth * BITS_PER_LEVEL
    root: Option<Arc<Node<T>>>,
    tail: Vec<T>, // incomplete set of BITS_PER_LEVEL items at end of list
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Shift(usize);

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Index(usize);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Node<T> {
    Branch {
        children: [Option<Arc<Node<T>>>; BRANCH_FACTOR],
    },
    Leaf {
        elements: Vec<T>,
    },
}

impl<T: Clone + Debug> DVec<T> {
    pub fn new() -> Self {
        DVec {
            root_len: Index(0),
            shift: Shift(0),
            root: None,
            tail: Vec::with_capacity(BRANCH_FACTOR),
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.root_len.0 {
            Some(self.root.as_ref().unwrap().get(self.shift, Index(index)))
        } else {
            self.tail.get(index - self.root_len.0)
        }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < self.root_len.0 {
            Some(Arc::make_mut(self.root.as_mut().unwrap()).get_mut(self.shift, Index(index)))
        } else {
            self.tail.get_mut(index - self.root_len.0)
        }
    }

    pub fn len(&self) -> usize {
        self.root_len.0 + self.tail.len()
    }

    pub fn push(&mut self, element: T) {
        self.tail.push(element);

        if self.tail.len() == BRANCH_FACTOR {
            let tail = mem::replace(&mut self.tail, Vec::with_capacity(BRANCH_FACTOR));
            self.push_tail(tail);
            self.root_len.0 += BRANCH_FACTOR;
        }

        self.validate();
    }

    #[cold]
    fn push_tail(&mut self, tail: Vec<T>) {
        // We just filled up the tail, therefore we should have an
        // even multiple of BRANCH_FACTOR elements.
        debug_assert!(self.root_len.0 % BRANCH_FACTOR == 0);
        debug!("---------------------------------------------------------------------------");
        debug!("DVec::push_tail(tail={:?})", tail);

        if let Some(root) = self.root.as_mut() {
            // Find out the total capacity in the "leaf" tree.
            let capacity = BRANCH_FACTOR << self.shift.0;

            // Still have room.
            debug!("push_tail: self.root_len={:?} capacity={:?}", self.root_len, capacity);
            if (self.root_len.0 + BRANCH_FACTOR) <= capacity {
                Arc::make_mut(root).push_tail(self.shift, self.root_len, tail);
                return;
            }

            // Going to need to add another level.
            let mut children = no_children!();
            children[0] = Some(root.clone());
            children[1] = Some(Node::branch_ladder(self.shift, tail));
            *root = Arc::new(Node::Branch { children: children });
            self.shift = self.shift.inc();
            return;
        }

        debug_assert!(self.root_len == 0);
        debug_assert!(self.shift == 0);
        self.root = Some(Arc::new(Node::Leaf { elements: tail }));
    }

    #[cfg(not(test))]
    fn validate(&self) {}

    #[cfg(test)]
    fn validate(&self) {
        if VALIDATE {
            if let Some(ref root) = self.root {
                if let Err(err) = root.validate(&mut vec![], self.shift, self.root_len) {
                    panic!("validation error {} with {:#?}", err, root);
                }
            }
            let tail_len = self.tail.len();
            assert!(tail_len < BRANCH_FACTOR,
                    "tail got too long: {:?}",
                    tail_len);
        }
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

    fn inc(self) -> Shift {
        Shift(self.0 + BITS_PER_LEVEL)
    }
}

impl<T: Clone + Debug> Node<T> {
    #[cfg(test)]
    pub fn validate(&self, path: &mut Vec<usize>, shift: Shift, len: Index) -> Result<(), String> {
        // This is called just after a `push_tail`. The tree should be
        // dense to the left.
        match *self {
            Node::Branch { ref children } => {
                if shift.0 == 0 {
                    return Err(format!("encountered branch at path {:?} but shift is {:?}",
                                       path,
                                       shift));
                }

                let mut children_iter = children.iter().enumerate();
                let mut walked = 0;
                while walked < len.0 {
                    if let Some((i, child)) = children_iter.next() {
                        match *child {
                            Some(ref c) => {
                                path.push(i);

                                let max_in_child = BRANCH_FACTOR << (shift.0 - BITS_PER_LEVEL);
                                let remaining = len.0 - walked;
                                let child_len = cmp::min(remaining, max_in_child);
                                if child_len == 0 {
                                    return Err(format!("at path {:?}, empty child", path));
                                }
                                c.validate(path, shift.dec(), Index(child_len))?;
                                walked += child_len;
                                assert!(i == path.pop().unwrap());
                            }
                            None => {
                                return Err(format!("at path {:?}, found unexpected none at {}",
                                                   path,
                                                   i));
                            }
                        }
                    } else {
                        return Err(format!("at path {:?}, iterator ended early", path));
                    }
                }

                if let Some(c) = children_iter.find(|c| c.1.is_some()) {
                    return Err(format!("node at path {:?} had unexected `some` ({})",
                                       path,
                                       c.0));
                }
            }

            Node::Leaf { ref elements } => {
                if shift.0 != 0 {
                    return Err(format!("encountered leaf at path {:?} but shift is {:?}",
                                       path,
                                       shift));
                }
                if elements.len() != BRANCH_FACTOR {
                    return Err(format!("encountered leaf at path {:?} with only {} elements",
                                       path,
                                       elements.len()));
                }
            }
        }
        Ok(())
    }

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
        debug!("push_tail(shift={:?}, index={:?})", shift, index);
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
        // We compute the index of the *start* of the chunk (96-32 ==
        // 64), which will be 2 -- that is the index where this set of
        // leaves will go.
        //
        // Example 2.
        //
        // Assume branch size is 4 and the vector has 16 things in it.
        // We are now pushing a new tail. In this case, we have an input
        // like so:
        //
        // 0 (shift = 4)
        // |
        // +- 00
        //     |
        //     +- 000 (Leaf: elements 0..4)
        //     +- 001 (Leaf: elements 4..8)
        //     +- 002 (Leaf: elements 8..12)
        //     +- 003 (Leaf: elements 12..16)
        // +- 01 (None)
        //
        // We want to expand `01` to a subtree like:
        //
        // +- 01 (None)
        //     |
        //     +- 000 (Leaf: elements 16..20)
        //     +- ... (None)
        //
        // This case is a bit different from the first, because we
        // encounter a `None` as we are walking down the tree, before
        // we get to the leaf.

        let mut p = self;
        let mut shift = shift;
        loop {
            debug!("shift={:?}", shift);
            debug_assert!(shift.0 >= BITS_PER_LEVEL);
            let mut q = p; // FIXME
            match *q {
                Node::Leaf { .. } => {
                    unreachable!("should not encounter a leaf w/ shift {:?}", shift)
                }
                Node::Branch { ref mut children } => {
                    let child = index.child(shift);
                    shift = shift.dec(); // represents the shift of children[child] now

                    if shift.0 == 0 {
                        // children[child] is the final level; this is example 1
                        debug_assert!(children[child].is_none());
                        debug!("Node::push_tail: storing with child={:?}", child);
                        children[child] = Some(Arc::new(Node::Leaf { elements: tail }));
                        return;
                    }

                    // Load up the child and descend to that level (if
                    // it is present). If not, we have example 2.
                    debug!("Node::push_tail: shift={:?} index={:?} child={:?}",
                           shift,
                           index,
                           child);
                    if children[child].is_some() {
                        let child = children[child].as_mut().unwrap();
                        p = Arc::make_mut(child);
                        continue;
                    }

                    // Example 2: have to construct multiple levels at once.
                    debug!("creating branch ladder at child {}", child);
                    children[child] = Some(Node::branch_ladder(shift, tail));
                    debug!("result: {:?}", children);
                    return;
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
                        None => panic!("missing child {} at shift {} (index={})",
                                       child, shift.0, index.0),
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

    pub fn get_mut(&mut self, shift: Shift, index: Index) -> &mut T {
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
                    return &mut elements[child];
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
            Node::Branch { ref children } => Node::Branch { children: clone_arr!(children) },
            Node::Leaf { ref elements } => Node::Leaf { elements: elements.clone() },
        }
    }
}

#[cfg(test)]
mod test;
