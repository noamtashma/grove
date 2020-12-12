// an implementation of a splay tree
use super::super::tree_base;
use super::super::tree_base::*;
use super::super::tree_base::walker::*;


pub struct SplayTree<D : Data> {
    tree : tree_base::Tree<D>,
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

    // TODO: this would be better if we used &mut Box<Tree> instead of &mut Node in our data structures.
    // consider the change
    // this would require us to store trees in a box

    // note: using this directly may cause the tree to lose its properties as a splay tree
    pub fn basic_walker<'a>(&'a mut self) -> Option<TreeWalker<'a, D>> {
        match &mut self.tree {
            Tree::Empty => None,
            Tree::Root(node) => Some(TreeWalker::new(node)),
        }
    }

    pub fn walker<'a>(&'a mut self) -> Option<SplayWalker<'a, D>> {
        let basic_walker = self.basic_walker()?;
        return Some( SplayWalker { walker : basic_walker } );
    }
}

pub struct SplayWalker<'a, D : Data> {
    walker : TreeWalker<'a, D>,
}

impl <'a, D : Data> SplayWalker<'a, D> {
    pub fn go_left(&mut self) -> Result<(), ()> {
        self.walker.go_left()
    }

    pub fn go_right(&mut self) -> Result<(), ()> {
        self.walker.go_right()
    }

    // if at the root, do nothing.
    // otherwise, do a splay step upwards
    pub fn splay_step(&mut self) {
        unimplemented!()
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
    type Target = Node<D>;
    fn deref(&self) -> &Node<D> {
        &*self.walker
    }
}

impl<'a, D : Data> std::ops::DerefMut for SplayWalker<'a, D> {
    fn deref_mut(&mut self) -> &mut Node<D> {
        &mut *self.walker
    }
}