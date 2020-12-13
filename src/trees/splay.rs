// an implementation of a splay tree
use super::super::tree_base::*;
use super::super::tree_base::walker::*;


pub struct SplayTree<D : Data> {
    tree : Tree<D>,
}

impl<D : Data> SplayTree<D> {
    pub fn into_inner(self) -> Tree<D> {
        self.tree
    }

    pub fn new() -> Self {
        SplayTree { tree : Tree::Empty }
    }

    pub fn from_inner(tree : Tree<D>) -> Self {
        SplayTree { tree }
    }

    pub fn data(&self) -> Option<&D> {
        self.tree.data()
    }

    pub fn data_mut(&mut self) -> Option<&mut D> {
        self.tree.data_mut()
    }

    // note: using this directly may cause the tree to lose its properties as a splay tree
    pub fn basic_walker(&mut self) -> TreeWalker<'_, D> {
        TreeWalker::new(&mut self.tree)
    }

    pub fn walker(&mut self) -> SplayWalker<'_, D> {
        SplayWalker { walker : self.basic_walker() }
    }
}

impl<D : Data> std::default::Default for SplayTree<D> {
    fn default() -> Self {
        SplayTree::new()
    }
}

impl<D : crate::data::basic_data::Keyed > SplayTree<D> {
    // moves the wanted node to the root, if found
    // returns an error if the node was not found
    // in that case, another node will be splayed to the root
    pub fn search(&mut self, key : &<D as crate::data::basic_data::Keyed>::Key) -> Result<(), ()> {
        use std::cmp::Ordering::*;

        let mut walker = self.walker();
        // when we leave the function the walker's destructor will
        // automatically splay a node to the root for us.
        while let Tree::Root(node) = &mut *walker {
            let nodekey = node.get_key();
            match key.cmp(nodekey) {
                Equal   => return Ok(()),
                Less    => walker.go_left().unwrap(), // the empty case is unreachable
                Greater => walker.go_right().unwrap(), // the empty case is unreachable
            };
        }
        return Err(()); // splay some other node instead
    }

    pub fn insert(&mut self, data : D) {
        let mut walker = self.walker();

        let key = data.get_key();
        while let Tree::Root(node) = &mut *walker {
            if key < node.get_key() {
                walker.go_left().unwrap(); // the empty case is unreachable
            } else {
                walker.go_right().unwrap(); // the empty case is unreachable
            };
        }
        *walker = Tree::Root(Box::new(Node::new(data, Tree::Empty, Tree::Empty)));
    }
}

pub struct SplayWalker<'a, D : Data> {
    walker : TreeWalker<'a, D>,
}

impl <'a, D : Data> SplayWalker<'a, D> {
    // TODO - add an into_inner method
    // it was annoying because rust doesn't allow consuming values that implement Drop

    pub fn new(walker : TreeWalker<'a, D>) -> Self {
        SplayWalker { walker }
    }

    pub fn go_left(&mut self) -> Result<(), ()> {
        self.walker.go_left()
    }

    pub fn go_right(&mut self) -> Result<(), ()> {
        self.walker.go_right()
    }

    // if at the root, do nothing.
    // otherwise, do a splay step upwards.

    // about the amortized computational complexity of using splay steps:
    // the amortized cost of any splay step, except the zig step near the root, is at most
    // log(new_node.size) - log(old_node.size) - 1
    // the -1 covers the complexity of going down the tree in the first place,
    // and therefore you pay for at most log the size of the node where you stop splaying

    pub fn splay_step(&mut self) {

        // if the walker points to an empty position,
        // we can't splay it, just go upwards once.
        if self.walker.is_empty() {
            if let Err(()) = self.walker.go_up() { // if already the root, exit. otherwise, go up
                return
            };
        }

        let b1 = match self.walker.go_up() {
            Err(()) => return, // already the root
            Ok(b1) => b1,
        };

        let b2 = match self.walker.is_left_son() {
            None => { self.walker.rot_side(!b1).unwrap(); return }, // became the root - zig step
            Some(b2) => b2,
        };

        if b1 == b2 { // zig-zig case
            self.walker.rot_up().unwrap();
            self.walker.rot_side(!b1).unwrap();
        } else { // zig-zag case
            self.walker.rot_side(!b1).unwrap();
            self.walker.rot_up().unwrap();
        }
    }

    // splay the current node to the top of the tree
    pub fn splay(&mut self) {
        while !self.walker.is_root() {
            self.splay_step();
        }
    }
}

impl<'a, D : Data> Drop for SplayWalker<'a, D> {
    fn drop(&mut self) {
        self.splay();
    }
}

impl<'a, D : Data> std::ops::Deref for SplayWalker<'a, D> {
    type Target = Tree<D>;
    fn deref(&self) -> &Tree<D> {
        &*self.walker
    }
}

impl<'a, D : Data> std::ops::DerefMut for SplayWalker<'a, D> {
    fn deref_mut(&mut self) -> &mut Tree<D> {
        &mut *self.walker
    }
}