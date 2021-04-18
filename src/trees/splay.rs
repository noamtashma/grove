//! An implementation of a splay tree
//!
//! The tree implements the `splay` operation, that should be used after searching for a node.
//! See documentation for the splay function.
//!
//! Since the splay tree's complexity follows from splaying,
//! if you would apply `go_up()` too many times, the splay tree's complexity might suffer.
//! Every `go_up()` application costs a constant amount, but the depth of the tree might
//! as big as `O(n)`.
//! See documentation for the `go_up` function.
use super::*;
use super::basic_tree::*;



pub struct SplayTree<D : Action> {
    tree : BasicTree<D>,
}

impl<D : Action> SplayTree<D> {
    pub fn root_node_value(&self) -> Option<D::Value> {
        match &self.tree {
            BasicTree::Root(node) => Some(node.node_value()),
            _ => None,
        }
    }

    pub fn segment_value(&self) -> D::Value {
        self.tree.segment_value()
    }

    /// Note: using this directly may cause the tree to lose its properties as a splay tree
    pub fn basic_walker(&mut self) -> BasicWalker<'_, D> {
        BasicWalker::new(&mut self.tree)
    }

    // TODO: switch to a symmetric view, i.e.,
    // `tree3 = union(tree1, tree2)`, not
    // `tree1.concatenate(tree2)`.
    /// Concatenates the other tree into this tree.
    pub fn concatenate(&mut self, other : Self) {
        let mut walker = self.walker();
        while let Ok(_) = walker.go_right()
            {}
        match walker.go_up() {
            Err(()) => { // the tree is empty; just substiture the other tree.
                drop(walker);
                *self = other;
                return;
            },
            Ok(false) => (),
            Ok(true) => panic!(),
        };
        walker.splay();
        if let crate::basic_tree::BasicTree::Root(node) = walker.inner_mut() {
            node.right = other.into_inner();
            node.rebuild();
            return;
        }
        else {
            panic!();
        }
    }

    // TODO: implement
    /// Gets the tree into a state in which the locator's segment
    /// is a single subtree, and returns a walker at that subtree.
    pub fn isolate_segment<'a, L>(&mut self, locator : &L) -> SplayWalker<'a, D> where
        L : crate::methods::locator::Locator<D>
    {
        unimplemented!();
    }

    pub fn act_segment<'a, L>(&mut self, locator : &L, action : D) where
        L : crate::methods::locator::Locator<D>
    {
        let mut walker = self.isolate_segment(locator);
        match walker.inner_mut() {
            crate::basic_tree::BasicTree::Root(node) => {
                node.act(action);
                node.access();
            },
            _ => (),
        }
    }
}


impl<A : Action + Reverse> SplayTree<A> {
    pub fn reverse(&mut self) {
        self.tree.reverse();
    }
}


impl<D : Action> std::default::Default for SplayTree<D> {
    fn default() -> Self {
        SplayTree::new()
    }
}


#[derive(destructure)]
pub struct SplayWalker<'a, D : Action> {
    walker : BasicWalker<'a, D>,
}

impl<'a, D : Action> SplayWalker<'a, D> {

    pub fn inner(&self) -> &BasicTree<D> {
        &*self.walker
    }

    // using this function can really mess up the structure
    // use wisely
    // this function shouldn't really be public
    // TODO: should this function exist?
    pub fn inner_mut(&mut self) -> &mut BasicTree<D> {
        &mut *self.walker
    }

    pub fn into_inner(self) -> BasicWalker<'a, D> {
        // this is a workaround for the problem that, 
        // we can't move out of a type implementing Drop

        let (walker,) = self.destructure();
        walker
    }

    pub fn new(walker : BasicWalker<'a, D>) -> Self {
        SplayWalker { walker }
    }
    
    /// If at the root, do nothing.
    /// otherwise, do a single splay step upwards.

    /// Amortized complexity of splay steps:
    /// The amortized cost of any splay step, except the zig step near the root, is at most
    /// `log(new_node.size) - log(old_node.size) - 1`
    /// The -1 covers the complexity of going down the tree in the first place.

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

    /// Splay the current node to the top of the tree.
    ///
    /// If the walker is on an empty spot, it will splay some nearby node
    /// to the top of the tree instead. (current behavior is the father of the empty spot).
    ///
    /// Going down the tree, and then splaying,
    /// has an amortized cost of `O(log n)`.
    pub fn splay(&mut self) {
        while !self.walker.is_root() {
            self.splay_step();
        }
    }

    // TODO: make a trait for splittable trees
    /// Will only do anything if the current position is empty
    /// If it is empty, it will split the tree: the elements
    /// to the left will remain, and the elements to the right
    /// will be put in the new output tree.
    /// The walker will be at the root after this operation, if it succeeds.
    pub fn split(&mut self) -> Option<SplayTree<D>> {
        if !self.is_empty() { return None }
        
        // to know which side we should cut
        let b = match self.go_up() {
            Err(()) => { return Some(SplayTree::new()) }, // this is the empty tree
            Ok(b) => b,
        };
        self.splay();
        let node = match self.inner_mut() {
            BasicTree::Root(node) => node,
            _ => panic!(),
        };
        if b {
            let mut tree = std::mem::replace(&mut node.left, BasicTree::Empty);
            node.rebuild();
            std::mem::swap(self.inner_mut(), &mut tree);
            return Some(SplayTree{ tree });
        } else {
            let tree = std::mem::replace(&mut node.right, BasicTree::Empty);
            node.rebuild();
            return Some(SplayTree { tree });
        }
    }
}

impl<'a, D : Action> Drop for SplayWalker<'a, D> {
    fn drop(&mut self) {
        self.splay();
    }
}

impl<A : Action> SomeTree<A> for SplayTree<A> {
    fn into_inner(self) -> BasicTree<A> {
        self.tree
    }

    fn new() -> Self {
        SplayTree { tree : BasicTree::Empty }
    }

    fn from_inner(tree : BasicTree<A>) -> Self {
        SplayTree { tree }
    }
}

impl<'a, D : Action> SomeTreeRef<D> for &'a mut SplayTree<D> {
    type Walker = SplayWalker<'a, D>;
    fn walker(self : &'a mut SplayTree<D>) -> SplayWalker<'a, D> {
        SplayWalker { walker : self.basic_walker() }
    }
}

impl<'a, A : Action> SomeWalker<A> for SplayWalker<'a, A> {
    fn go_left(&mut self) -> Result<(), ()> {
        self.walker.go_left()
    }

    fn go_right(&mut self) -> Result<(), ()> {
        self.walker.go_right()
    }

    /// you shouldn't use this too much, or you would lose the SplayTree's complexity properties.
    /// basically, when you are going down the tree,
    /// you should only stray from your path by a constant amount,
    /// and you should remember to splay if you want to re-use your walker, instead of
    /// using this fuctionn to get back up.
    /// (when dropped the walker will splay by itself)
    fn go_up(&mut self) -> Result<bool, ()> {
        self.walker.go_up()
    }

    fn depth(&self) -> usize {
        self.walker.depth()
    }

    fn far_left_value(&self) -> A::Value {
        self.walker.far_left_value()
    }
    fn far_right_value(&self) -> A::Value {
        self.walker.far_right_value()
    }

    fn inner_mut(&mut self) -> &mut BasicTree<A> {
        self.walker.inner_mut()
    }

    fn inner(&self) -> &BasicTree<A> {
        self.walker.inner()
    }
}

impl<'a, A : Action> SomeEntry<A> for SplayWalker<'a, A> {
    fn value_mut(&mut self) -> Option<&mut A::Value> {
        self.walker.value_mut()
    }

    fn value(&self) -> Option<&A::Value> {
        self.walker.value()
    }

    /*
    fn write(&mut self, data : D) -> Option<D> {
        self.walker.write(data)
    }
    */

    fn insert_new(&mut self, value : A::Value) -> Result<(), ()> {
        self.walker.insert_new(value)
    }
}