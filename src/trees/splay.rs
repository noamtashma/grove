// an implementation of a splay tree
use super::*;
use super::basic_tree::*;



pub struct SplayTree<D : Data> {
    tree : BasicTree<D>,
}

impl<D : Data> SplayTree<D> {

    pub fn root_data(&self) -> Option<&D> {
        self.tree.data()
    }

    pub fn root_data_mut(&mut self) -> Option<&mut D> {
        self.tree.data_mut()
    }

    // note: using this directly may cause the tree to lose its properties as a splay tree
    pub fn basic_walker(&mut self) -> BasicWalker<'_, D> {
        BasicWalker::new(&mut self.tree)
    }

}

impl<D : Data> std::default::Default for SplayTree<D> {
    fn default() -> Self {
        SplayTree::new()
    }
}

impl<D : crate::data::example_data::Keyed > SplayTree<D> {
    // moves the wanted node to the root, if found
    // returns an error if the node was not found
    // in that case, another node will be splayed to the root
    // TODO: make into a general tree method
    
}

#[derive(destructure)]
pub struct SplayWalker<'a, D : Data> {
    walker : BasicWalker<'a, D>,
}

impl<'a, D : Data> SplayWalker<'a, D> {

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
            std::mem::swap(self.inner_mut(), &mut tree);
            return Some(SplayTree{ tree });
        } else {
            let tree = std::mem::replace(&mut node.right, BasicTree::Empty);
            return Some(SplayTree { tree });
        }
    }
}

impl<'a, D : Data> Drop for SplayWalker<'a, D> {
    fn drop(&mut self) {
        self.splay();
    }
}

impl<D : Data> SomeTree<D> for SplayTree<D> {
    fn into_inner(self) -> BasicTree<D> {
        self.tree
    }

    fn new() -> Self {
        SplayTree { tree : BasicTree::Empty }
    }

    fn from_inner(tree : BasicTree<D>) -> Self {
        SplayTree { tree }
    }
}

impl<'a, D : Data> SomeTreeRef<D> for &'a mut SplayTree<D> {
    type Walker = SplayWalker<'a, D>;
    fn walker(self : &'a mut SplayTree<D>) -> SplayWalker<'a, D> {
        SplayWalker { walker : self.basic_walker() }
    }
}

impl<'a, D : Data> SomeWalker<D> for SplayWalker<'a, D> {
    fn go_left(&mut self) -> Result<(), ()> {
        self.walker.go_left()
    }

    fn go_right(&mut self) -> Result<(), ()> {
        self.walker.go_right()
    }

    // you shouldn't use this too much, or you would lose the SplayTree's complexity properties.
    // basically, when you are going down the tree,
    // you should only stray from your path by a constant amount,
    // and you should remember to splay if you want to re-use your walker, instead of
    // using this fuctionn to get back up.
    // (when dropped the walker will splay by itself)
    fn go_up(&mut self) -> Result<bool, ()> {
        self.walker.go_up()
    }

    fn inner_mut(&mut self) -> &mut BasicTree<D> {
        self.walker.inner_mut()
    }

    fn inner(&self) -> &BasicTree<D> {
        self.walker.inner()
    }
}

impl<'a, D : Data> SomeEntry<D> for SplayWalker<'a, D> {
    fn data_mut(&mut self) -> Option<&mut D> {
        self.walker.data_mut()
    }

    fn data(&self) -> Option<&D> {
        self.walker.data()
    }

    /*
    fn write(&mut self, data : D) -> Option<D> {
        self.walker.write(data)
    }
    */

    fn insert_new(&mut self, data : D) -> Result<(), ()> {
        self.walker.insert_new(data)
    }
}