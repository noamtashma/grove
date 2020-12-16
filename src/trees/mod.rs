// this module contains implementations of specific balances search trees.

pub mod basic_tree;
pub mod splay;

pub trait SomeTree<D> where
    for<'a> &'a mut Self : SomeTreeRef<D> {

    fn into_inner(self) -> basic_tree::BasicTree<D>;
    fn new() -> Self;
    fn from_inner(tree : basic_tree::BasicTree<D>) -> Self;

}

pub trait SomeTreeRef<D> {
    type Walker : SomeWalker<D>;
    fn walker(self) -> Self::Walker;
}


// walker should only hold a reference to the tree
pub trait SomeWalker<D> : SomeEntry<D> {
    /// return `Err(())` if it is in an empty spot.
    fn go_left(&mut self) -> Result<(), ()>;
    /// returns `Err(())` if it is in an empty spot.
    fn go_right(&mut self) -> Result<(), ()>;

    /// if successful, returns whether or not the previous current value was the left son.
    /// if you have a SplayTree, you shouldn't use this too much, or you might lose the
    /// SplayTree's complexity properties.
    /// see splayTree
    fn go_up(&mut self) -> Result<bool, ()>;

    // these functions are here instead of Deref and DerefMut
    // using these functions directly might mess up the internal structure of the tree.
    // be warned!
    //fn inner_mut(&mut self) -> &mut basic_tree::Tree<D>;
    //fn inner(&self) -> &basic_tree::Tree<D>;
}
/// things that act like entries - allow access to a maybe-missing value, as if it is an Option<D>
pub trait SomeEntry<D> {
    fn data_mut(&mut self) -> Option<&mut D>;
    fn data(&self) -> Option<&D>;

    fn is_empty(&self) -> bool {
        self.data().is_none()
    }

    
    /// runs access() and rebuild() after writing the value.
    /// returns the previous data value if there was any
    /// if the place was empty, creates new empty nodes
    fn write(&mut self, data : D) -> Option<D>;

    // there is no point of running access() or rebuild() qafter writing,
    // because the node can't access any other part of the tree except itself,
    // so the user could have just done the calls himself,
    // ergo, the user can be required to provide a value of data
    // that doesn't need access() or rebuild() to be called

    /// only writes if it is in an empty position. if the positions isn't empty,
    /// return Err(()).
    fn insert_new(&mut self, data : D) -> Result<(), ()>;
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
