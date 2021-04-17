/// this module contains:
/// * traits that all implementations of trees should implement
/// (both for the tree and the tree's walker)
/// * specific implementations of trees

pub mod basic_tree;
pub mod splay;

use crate::data::*;
pub trait SomeTree<D : Action> where
    for<'a> &'a mut Self : SomeTreeRef<D> {

    fn into_inner(self) -> basic_tree::BasicTree<D>;
    fn new() -> Self;
    fn from_inner(tree : basic_tree::BasicTree<D>) -> Self;

}

pub trait SomeTreeRef<D : Action> {
    type Walker : SomeWalker<D>;
    fn walker(self) -> Self::Walker;
}



/// A Walker of the tree is a type that allows you to walk up and down the tree while modifying it.
/// Under the hood, the walkers use the Telescope type to achieve this.
/// The walker should be able to walk on any of the existing nodes, or any empty position just near them.
/// i.e., the walker can also be in the position of a son of an existing node, where there isn't
/// a node yet.
/// The method is_empty() can tell whether you are at an empty position. Trying to move downward from an
/// empty position produces an error value.
pub trait SomeWalker<A : Action> : SomeEntry<A> {
    /// return `Err(())` if it is in an empty spot.
    fn go_left(&mut self) -> Result<(), ()>;
    /// returns `Err(())` if it is in an empty spot.
    fn go_right(&mut self) -> Result<(), ()>;

    /// If successful, returns whether or not the previous current value was the left son.
    /// If already at the root of the tree, returns `Err(())`.
    /// If you have a SplayTree, you shouldn't use this method too much, or you might lose the
    /// SplayTree's complexity properties - see documentation aboud splay tree.
    
    fn go_up(&mut self) -> Result<bool, ()>;

    /*
    fn segment_value(&self) -> A::Value {
        self.inner().segment_value()
    }
    */

    fn depth(&self) -> usize;

    fn far_left_value(&self) -> A::Value;
    fn far_right_value(&self) -> A::Value;
    

    // these functions are here instead of Deref and DerefMut. 
    fn inner_mut(&mut self) -> &mut basic_tree::BasicTree<A>;
    fn inner(&self) -> &basic_tree::BasicTree<A>;
}
/// Things that allow access to a maybe-missing value, as if it is an Option<D>.
/// Currently there are no actual Entry types, and the walkers themselves
/// act as the entries. However, the traits are still separated.
pub trait SomeEntry<A : Action> {
    fn value_mut(&mut self) -> Option<&mut A::Value>;
    fn value(&self) -> Option<&A::Value>;

    fn is_empty(&self) -> bool {
        self.value().is_none()
    }

    // there is no point of running access() or rebuild() qafter writing,
    // because the node can't access any other part of the tree except itself,
    // so the user could have just done the calls himself,
    // ergo, the user can be required to provide a value of data
    // that doesn't need access() or rebuild() to be called

    /// only writes if it is in an empty position. if the position isn't empty,
    /// return Err(()).
    fn insert_new(&mut self, value : A::Value) -> Result<(), ()>;
}

