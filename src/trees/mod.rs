// this module contains implementations of specific balances search trees.
use super::tree_base;

pub mod splay;

pub trait SomeTree<D> where
    for<'a> &'a mut Self : SomeTreeRef<D> {

    fn into_inner(self) -> tree_base::Tree<D>;
    fn new() -> Self;
    fn from_inner(tree : tree_base::Tree<D>) -> Self;

}

pub trait SomeTreeRef<D> where
    for<'a>&'a mut <Self as SomeTreeRef<D>>::Walker : SomeWalkerRef<D> {
    type Walker : SomeWalker<D>;
    fn walker(self) -> Self::Walker;
}

// Walker should also implement DerefMut into a Tree<D>.
// however there is no way of specifying that directly

// walker should only hold a reference to the tree
pub trait SomeWalker<D> where 
    for<'a> &'a mut Self : SomeWalkerRef<D> {
    // should be the samee as <&mut Self as SomeWalkerRef>::Entry
    // currently we can't writ that down
    // however, this shouldn't be much of a problem
    type Entry : SomeEntry<D>;
    
    fn go_left(&mut self) -> Result<(), ()>;
    fn go_right(&mut self) -> Result<(), ()>;

    // these functions are here instead of Deref and DerefMut
    // using these functions might mess up the internal structure of the tree.
    // be warned!
    fn inner_mut(&mut self) -> &mut tree_base::Tree<D>;
    fn inner(&self) -> &tree_base::Tree<D>;

    fn into_entry(self) -> Self::Entry;
}

pub trait SomeWalkerRef<D> {
    type Entry : SomeEntry<D>;
    fn entry(self) -> Self::Entry;
}

pub trait SomeEntry<D> {
    // TODO - copy the entry methods
}