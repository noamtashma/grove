// this module contains implementations of specific balances search trees.
use super::basic_tree;

pub mod splay;

pub trait SomeTree<D> where
    for<'a> &'a mut Self : SomeTreeRef<D> {

    fn into_inner(self) -> basic_tree::Tree<D>;
    fn new() -> Self;
    fn from_inner(tree : basic_tree::Tree<D>) -> Self;

}

pub trait SomeTreeRef<D> {
    type Walker : SomeWalker<D>;
    fn walker(self) -> Self::Walker;
}


// walker should only hold a reference to the tree
// this kind of walker doesn't have a go_up method in order to include splay trees,
// which splay upwards.
pub trait SomeWalker<D> : SomeEntry<D> {
    // should be the samee as <&mut Self as SomeWalkerRef>::Entry
    // currently we can't writ that down
    // however, this shouldn't be much of a problem
    //type Entry : SomeEntry<D>;
    
    fn go_left(&mut self) -> Result<(), ()>;
    fn go_right(&mut self) -> Result<(), ()>;

    // these functions are here instead of Deref and DerefMut
    // using these functions directly might mess up the internal structure of the tree.
    // be warned!
    //fn inner_mut(&mut self) -> &mut basic_tree::Tree<D>;
    //fn inner(&self) -> &basic_tree::Tree<D>;
}
// things that act like entries - allow access to a maybe-missing value, as if it is an Option<D>
pub trait SomeEntry<D> {
    fn data_mut(&mut self) -> Option<&mut D>;
    fn data(&self) -> Option<&D>;

    fn is_empty(&self) -> bool {
        self.data().is_none()
    }

    // runs rebuild() after the write
    // returns the previous data value if there was any
    // if the place was empty, creates new empty nodes
    fn write(&mut self, data : D) -> Option<D>;

    // only writes if it is in an empty position. if the positions isn't empty,
    // return Err(()). runs rebuild() after the write.
    fn insert_new(&mut self, data : D) -> Result<(), ()>;
}

pub trait SomeWalkerUp<D> : SomeWalker<D> {
	// if successful, returns whether or not the previous current value was the left son.
    fn go_up(&mut self) -> Result<bool, ()>;
}

// this should become a method. should we push our general methods inside a trait so that they can be
// methods?

pub fn insert<D : crate::data::example_data::Keyed, TR>(tree : TR, data : D) where
    TR : SomeTreeRef<D>,
{
    let mut walker = tree.walker();
    let key = data.get_key();
    
    while let Some(node) = walker.data_mut() {
        if key < node.get_key() {
            walker.go_left().unwrap(); // the empty case is unreachable
        } else {
            walker.go_right().unwrap(); // the empty case is unreachable
        };
    }
    
    walker.insert_new(data).unwrap();
}
